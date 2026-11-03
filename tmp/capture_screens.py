#!/usr/bin/env python3
"""Captura screenshots das telas do Sim Heatmaps (README e verificação visual).

Como funciona:
  * serve o frontend compilado (dist/) num servidor HTTP local, injetando um "mock" de
    window.__TAURI_INTERNALS__ que responde aos commands do backend;
  * os dados do mock vêm do backend Rust real: o exemplo `mock_data` cria a máquina PM1,
    gera duas bobinas (CSV numa pasta temporária fora do projeto) e imprime em JSON as
    respostas dos commands (máquinas, propriedades, bobinas e Contour Map de cada propriedade);
  * abre o Firefox em modo headless e o controla pelo protocolo Marionette (TCP + JSON,
    somente stdlib), navegando pelas telas e salvando PNGs.

Uso:
    python3 tmp/capture_screens.py                         # compila o frontend e captura
    python3 tmp/capture_screens.py --skip-build            # usa o dist/ existente
    python3 tmp/capture_screens.py --out /tmp/telas        # outra pasta de saída
    python3 tmp/capture_screens.py --dry-run               # mostra o que seria feito
"""
import argparse
import base64
import functools
import http.server
import json
import os
import shutil
import socket
import socketserver
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path

RED, GREEN, YELLOW, CYAN, DIM, RESET = "\033[31m", "\033[32m", "\033[33m", "\033[36m", "\033[2m", "\033[0m"
ROOT = Path(__file__).resolve().parent.parent
DIST = ROOT / "dist"
OUT = ROOT / "assets" / "screenshots"
MAC_FIREFOX = Path("/Applications/Firefox.app/Contents/MacOS/firefox")


def info(msg):
    print(f"{CYAN}==>{RESET} {msg}")


def ok(msg):
    print(f"  {GREEN}✓{RESET} {msg}")


def die(msg):
    print(f"{RED}erro:{RESET} {msg}", file=sys.stderr)
    sys.exit(1)


# ------------------------------------------------------------------ mock do backend
MOCK_JS = r"""
(() => {
  const DATA = __DATA__;
  const handlers = {
    app_info: () => DATA.info,
    list_machines: () => DATA.machines,
    create_machine: ({ input }) => ({ ...input, id: 99, binCount: input.binCount ?? 800 }),
    update_machine: ({ id, input }) => ({ ...input, id, binCount: input.binCount ?? 800 }),
    delete_machine: () => null,
    list_properties: () => DATA.properties,
    create_property: ({ input }) => ({ ...input, id: 999, sortOrder: input.sortOrder ?? 99 }),
    update_property: ({ id, input }) => ({ ...input, id, sortOrder: input.sortOrder ?? 1 }),
    delete_property: () => null,
    generate_reel: () => DATA.reels[0],
    list_reels: () => DATA.reels,
    delete_reel: () => null,
    load_contour: ({ propertyCode }) => DATA.contours[propertyCode],
    load_contour_file: () => Object.values(DATA.contours)[0],
    get_settings: () => DATA.settings,
    update_settings: ({ settings }) => settings,
  };
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main" } },
    transformCallback: (cb) => { const id = Math.floor(Math.random() * 1e9); window["_" + id] = cb; return id; },
    invoke: async (cmd, args = {}) => {
      const h = handlers[cmd];
      if (!h) throw { code: "unknown", field: null, message: "mock: " + cmd };
      return h(args);
    },
  };
})();
"""


def build_mock(data: dict) -> str:
    data = dict(data)
    data["info"] = {"version": "0.1.0", "dbPath": "<app data>/sim-heatmaps.db", "logDir": str(ROOT / "log")}
    return MOCK_JS.replace("__DATA__", json.dumps(data, ensure_ascii=False))


# ------------------------------------------------------------- servidor HTTP
def start_server(mock_js: str):
    index = (DIST / "index.html").read_text(encoding="utf-8")
    index = index.replace("<script", '<script src="/__mock_tauri.js"></script><script', 1)

    class Handler(http.server.SimpleHTTPRequestHandler):
        def log_message(self, *a):
            pass

        def do_GET(self):
            path = self.path.split("?")[0]
            if path in ("/", "/index.html"):
                return self._send(index, "text/html")
            if path == "/__mock_tauri.js":
                return self._send(mock_js, "text/javascript")
            return super().do_GET()

        def _send(self, text, ctype):
            body = text.encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", f"{ctype}; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

    server = socketserver.TCPServer(("127.0.0.1", 0), functools.partial(Handler, directory=str(DIST)))
    threading.Thread(target=server.serve_forever, daemon=True).start()
    return server, server.server_address[1]


# ------------------------------------------------------------- Marionette
class Marionette:
    """Cliente mínimo do protocolo Marionette do Firefox (mensagens "<tamanho>:<json>")."""

    def __init__(self, port, timeout=30):
        deadline = time.time() + timeout
        while True:
            try:
                self.sock = socket.create_connection(("127.0.0.1", port), timeout=10)
                break
            except OSError:
                if time.time() > deadline:
                    die("não foi possível conectar ao Marionette do Firefox")
                time.sleep(0.3)
        self.buf = b""
        self.msg_id = 0
        self._read()  # saudação

    def _read(self):
        while b":" not in self.buf:
            self.buf += self.sock.recv(65536)
        size, _, rest = self.buf.partition(b":")
        size = int(size)
        while len(rest) < size:
            rest += self.sock.recv(65536)
        self.buf = rest[size:]
        return json.loads(rest[:size])

    def cmd(self, name, params=None):
        self.msg_id += 1
        payload = json.dumps([0, self.msg_id, name, params or {}]).encode()
        self.sock.sendall(str(len(payload)).encode() + b":" + payload)
        _, _, error, result = self._read()
        if error:
            raise RuntimeError(f"{name}: {error.get('message', error)}")
        return result

    def js(self, script, *args):
        return self.cmd("WebDriver:ExecuteScript", {"script": script, "args": list(args)}).get("value")

    def wait(self, script, timeout=15):
        deadline = time.time() + timeout
        while time.time() < deadline:
            if self.js(script):
                return
            time.sleep(0.2)
        raise RuntimeError(f"timeout aguardando: {script}")

    def click_text(self, text, scope="button"):
        found = self.js(
            "const [scope, text] = arguments;"
            "const el = [...document.querySelectorAll(scope)].find(e => e.textContent.trim().includes(text));"
            "if (el) el.click(); return !!el;", scope, text)
        if not found:
            raise RuntimeError(f"elemento não encontrado: {scope} '{text}'")

    def screenshot(self, path: Path):
        data = self.cmd("WebDriver:TakeScreenshot", {"full": False})["value"]
        path.write_bytes(base64.b64decode(data))


def free_port():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


# ------------------------------------------------------------- roteiro
SHOTS = [
    ("01-contour.png", "Contour Map (tema claro, autoescala) com cursor sobre o mapa"),
    ("02-contour-faixas-escuro.png", "Contour Map (tema escuro, faixas de limite) de Moisture"),
    ("03-gerador.png", "Gerador com parâmetros e lista de bobinas"),
    ("04-maquinas.png", "Cadastro de máquinas"),
    ("05-maquina-editar.png", "Formulário da máquina (bins ajustados à largura)"),
    ("06-propriedades.png", "Propriedades e limites"),
    ("07-configuracoes.png", "Configurações"),
]

MAP = 'document.querySelector(\'[role="img"][tabindex="0"]\')'


def capture(m: Marionette, url: str, out: Path):
    def load(theme, extra=None):
        m.cmd("WebDriver:Navigate", {"url": url})
        m.js("const [t, extra] = arguments; localStorage.clear();"
             "localStorage.setItem('theme', t); localStorage.setItem('lang', 'pt-BR');"
             "for (const [k, v] of Object.entries(extra)) localStorage.setItem(k, v);", theme, extra or {})
        m.cmd("WebDriver:Navigate", {"url": url})
        m.wait("return document.querySelectorAll('nav button').length === 5")

    def nav(label, ready):
        m.click_text(label, "nav button")
        m.wait(ready)
        time.sleep(0.5)

    def shot(name):
        m.screenshot(out / name)
        ok(f"{name}")

    def hover_map(fx, fy):
        m.wait(f"return !!{MAP} && {MAP}.getBoundingClientRect().width > 0")
        time.sleep(0.6)
        m.js(f"const [fx, fy] = arguments; const el = {MAP}; const r = el.getBoundingClientRect();"
             "el.dispatchEvent(new MouseEvent('mousemove', { bubbles: true, clientX: r.left + r.width * fx, clientY: r.top + r.height * fy }));"
             "return true;", fx, fy)
        time.sleep(0.5)

    load("light", {"contourColorMode": "gradient", "contourProperty": "BW"})
    hover_map(0.62, 0.45)
    m.js("const s = document.querySelector('section[aria-busy]'); document.querySelector('main').scrollTop = s.offsetTop - 8; return true;")
    time.sleep(0.3)
    shot("01-contour.png")

    load("dark", {"contourColorMode": "steps", "contourProperty": "MOIST"})
    hover_map(0.3, 0.7)
    m.js("const s = document.querySelector('section[aria-busy]'); document.querySelector('main').scrollTop = s.offsetTop - 8; return true;")
    time.sleep(0.3)
    shot("02-contour-faixas-escuro.png")

    load("light")
    nav("Gerador", "return document.querySelectorAll('tbody tr').length >= 1")
    m.js("document.querySelector('details').open = true; return true;")
    time.sleep(0.3)
    shot("03-gerador.png")

    nav("Máquinas", "return document.querySelectorAll('tbody tr').length >= 1")
    shot("04-maquinas.png")
    m.js("document.querySelector('button[aria-label=\"Editar\"]').click(); return true;")
    m.wait("return document.querySelector('dialog[open] form')")
    time.sleep(0.6)
    shot("05-maquina-editar.png")
    m.js("document.querySelector('dialog[open] button[aria-label=\"Fechar\"]').click(); return true;")

    nav("Propriedades", "return document.querySelectorAll('tbody tr').length > 5")
    shot("06-propriedades.png")

    nav("Configurações", "return !!document.querySelector('dl')")
    shot("07-configuracoes.png")


def find_firefox():
    found = shutil.which("firefox")
    if found:
        return found
    return str(MAC_FIREFOX) if MAC_FIREFOX.is_file() else None


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", type=Path, default=OUT, help="pasta de saída (padrão: assets/screenshots)")
    ap.add_argument("--scans", type=int, default=240, help="scans da bobina principal (padrão: 240 = 2 h)")
    ap.add_argument("--width", type=int, default=1440)
    ap.add_argument("--height", type=int, default=900)
    ap.add_argument("--skip-build", action="store_true", help="não roda npm run build")
    ap.add_argument("--dry-run", action="store_true", help="mostra o que seria feito, sem executar")
    args = ap.parse_args()

    if args.dry_run:
        print(f"{YELLOW}[dry-run]{RESET} dados: cargo run --example mock_data ({args.scans} scans, pasta temporária do sistema)")
        print(f"{YELLOW}[dry-run]{RESET} {'usaria dist/ existente' if args.skip_build else 'rodaria npm run build'}")
        print(f"{YELLOW}[dry-run]{RESET} Firefox headless {args.width}x{args.height}, saída em {args.out}:")
        for name, desc in SHOTS:
            print(f"    {CYAN}{name}{RESET} {DIM}{desc}{RESET}")
        return 0

    firefox = find_firefox()
    if not firefox:
        die("Firefox não encontrado (PATH ou /Applications/Firefox.app)")

    if not args.skip_build:
        info("Compilando o frontend")
        subprocess.run(["npm", "run", "build"], cwd=ROOT, check=True, stdout=subprocess.DEVNULL)
    if not (DIST / "index.html").is_file():
        die("dist/index.html não encontrado")

    # Pasta temporária do sistema (fora do projeto): CSV gerados e perfil do Firefox.
    work = Path(tempfile.mkdtemp(prefix="sim-heatmaps-screens-"))
    try:
        info("Gerando bobinas de demonstração com o backend Rust")
        raw = subprocess.run(
            ["cargo", "run", "-q", "-p", "infrastructure", "--example", "mock_data", "--", str(work / "export"), str(args.scans)],
            cwd=ROOT / "src-tauri", check=True, capture_output=True, text=True).stdout
        data = json.loads(raw)
        ok(f"{len(data['reels'])} bobina(s), {len(data['contours'])} propriedade(s), {len(raw) / 1e6:.1f} MB de JSON")

        server, http_port = start_server(build_mock(data))
        args.out.mkdir(parents=True, exist_ok=True)
        profile = work / "firefox-profile"
        profile.mkdir()
        mport = free_port()
        (profile / "user.js").write_text(
            f'user_pref("marionette.port", {mport});\n'
            'user_pref("browser.shell.checkDefaultBrowser", false);\n'
            'user_pref("datareporting.policy.dataSubmissionEnabled", false);\n'
            'user_pref("toolkit.telemetry.reportingpolicy.firstRun", false);\n'
            'user_pref("layout.css.devPixelsPerPx", "1.0");\n', encoding="utf-8")

        info(f"Abrindo Firefox headless ({args.width}x{args.height})")
        env = dict(os.environ, MOZ_HEADLESS="1", MOZ_HEADLESS_WIDTH=str(args.width), MOZ_HEADLESS_HEIGHT=str(args.height))
        proc = subprocess.Popen([firefox, "--headless", "--marionette", "--no-remote", "--profile", str(profile), "about:blank"],
                                env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        try:
            m = Marionette(mport)
            m.cmd("WebDriver:NewSession", {"capabilities": {}})
            m.cmd("WebDriver:SetWindowRect", {"width": args.width, "height": args.height})
            info(f"Capturando telas em {args.out}")
            capture(m, f"http://127.0.0.1:{http_port}/", args.out)
            try:
                m.cmd("WebDriver:DeleteSession")
            except RuntimeError:
                pass
        finally:
            proc.terminate()
            try:
                proc.wait(timeout=10)
            except subprocess.TimeoutExpired:
                proc.kill()
            server.shutdown()
    finally:
        shutil.rmtree(work, ignore_errors=True)

    print(f"\n{GREEN}concluído:{RESET} {len(SHOTS)} screenshot(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
