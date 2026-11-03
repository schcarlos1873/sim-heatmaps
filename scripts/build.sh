#!/usr/bin/env bash
# Compila e empacota o Sim Heatmaps para Linux e macOS.
#
# Etapas: pré-requisitos → dependências npm → testes → tauri build → cópia dos
# instaladores para release/<versão>/<plataforma>/ com SHA256SUMS.txt.
#
# Compatível com o bash 3.2 do macOS (sem arrays associativos nem ${var,,}).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# ----------------------------------------------------------------- cores ANSI
if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
  RED=$'\033[31m'; GREEN=$'\033[32m'; YELLOW=$'\033[33m'; BLUE=$'\033[34m'; DIM=$'\033[2m'; BOLD=$'\033[1m'; RESET=$'\033[0m'
else
  RED=""; GREEN=""; YELLOW=""; BLUE=""; DIM=""; BOLD=""; RESET=""
fi

step() { printf '\n%s==>%s %s%s%s\n' "$BLUE" "$RESET" "$BOLD" "$*" "$RESET"; }
ok()   { printf '  %s✓%s %s\n' "$GREEN" "$RESET" "$*"; }
warn() { printf '  %s!%s %s\n' "$YELLOW" "$RESET" "$*"; }
die()  { printf '%serro:%s %s\n' "$RED" "$RESET" "$*" >&2; exit 1; }

# ------------------------------------------------------------------ argumentos
DRY_RUN=0; SKIP_TESTS=0; SKIP_INSTALL=0; CLEAN=0; DEBUG=0; UNIVERSAL=0
TARGET=""; BUNDLES=""; OUT_DIR="$ROOT/release"

usage() {
  cat <<EOF
Uso: scripts/build.sh [opções]

Compila o Sim Heatmaps e copia os instaladores para release/<versão>/<plataforma>/.

Opções:
  --dry-run          mostra os comandos sem executá-los
  --skip-tests       não roda cargo test / typecheck
  --skip-install     não roda npm ci
  --clean            remove builds anteriores (cargo clean, dist/) antes de compilar
  --debug            build de depuração (mais rápido, sem otimizações)
  --target TRIPLE    alvo Rust (ex.: aarch64-apple-darwin, x86_64-unknown-linux-gnu)
  --universal        macOS: binário universal (Intel + Apple Silicon)
  --bundles LISTA    formatos separados por vírgula
                       Linux: deb,rpm,appimage   macOS: app,dmg
                       (padrão: todos os da plataforma)
  --out DIR          pasta de saída (padrão: release/)
  -h, --help         mostra esta ajuda

Assinatura (opcional, lida pelo Tauri via variáveis de ambiente):
  macOS: APPLE_SIGNING_IDENTITY, APPLE_ID, APPLE_PASSWORD, APPLE_TEAM_ID
  Atualizador: TAURI_SIGNING_PRIVATE_KEY, TAURI_SIGNING_PRIVATE_KEY_PASSWORD

Exemplos:
  scripts/build.sh --dry-run
  scripts/build.sh --bundles rpm,appimage
  scripts/build.sh --universal --bundles dmg
EOF
}

need_value() { [ $# -ge 2 ] && [ -n "$2" ] || die "opção $1 requer um valor"; }

while [ $# -gt 0 ]; do
  case "$1" in
    --dry-run)      DRY_RUN=1 ;;
    --skip-tests)   SKIP_TESTS=1 ;;
    --skip-install) SKIP_INSTALL=1 ;;
    --clean)        CLEAN=1 ;;
    --debug)        DEBUG=1 ;;
    --universal)    UNIVERSAL=1 ;;
    --target)       need_value "$@"; TARGET="$2"; shift ;;
    --bundles)      need_value "$@"; BUNDLES="$2"; shift ;;
    --out)          need_value "$@"; OUT_DIR="$2"; shift ;;
    -h|--help)      usage; exit 0 ;;
    *)              usage >&2; die "opção desconhecida: $1" ;;
  esac
  shift
done

# Executa (ou apenas mostra, em --dry-run) um comando.
run() {
  printf '  %s$ %s%s\n' "$DIM" "$*" "$RESET"
  [ "$DRY_RUN" -eq 1 ] || "$@"
}

# ------------------------------------------------------------------ plataforma
case "$(uname -s)" in
  Linux)  OS=linux ;;
  Darwin) OS=macos ;;
  *)      die "plataforma não suportada: $(uname -s) — no Windows use scripts\\build.ps1" ;;
esac

if [ "$UNIVERSAL" -eq 1 ]; then
  [ "$OS" = macos ] || die "--universal só é suportado no macOS"
  [ -z "$TARGET" ] || die "use --universal ou --target, não ambos"
  TARGET="universal-apple-darwin"
fi

case "$TARGET" in
  "")       ARCH="$(uname -m)" ;;
  universal-apple-darwin) ARCH=universal ;;
  *)        ARCH="${TARGET%%-*}" ;;
esac
[ "$ARCH" = arm64 ] && ARCH=aarch64

if [ -n "$BUNDLES" ]; then
  for b in $(printf '%s' "$BUNDLES" | tr ',' ' '); do
    case "$OS:$b" in
      linux:deb|linux:rpm|linux:appimage|macos:app|macos:dmg) ;;
      *) die "formato '$b' inválido para $OS" ;;
    esac
  done
fi

[ "$DRY_RUN" -eq 1 ] && warn "modo --dry-run: nenhum comando será executado"

# ------------------------------------------------------------- pré-requisitos
step "Verificando pré-requisitos ($OS/$ARCH)"
for cmd in node npm cargo rustc; do
  command -v "$cmd" >/dev/null 2>&1 || die "'$cmd' não encontrado no PATH"
  ok "$cmd $("$cmd" --version 2>/dev/null | head -n1)"
done

NODE_MAJOR="$(node -p 'process.versions.node.split(".")[0]')"
[ "$NODE_MAJOR" -ge 20 ] || die "Node 20+ é necessário (encontrado $(node --version))"

if [ "$OS" = linux ]; then
  command -v pkg-config >/dev/null 2>&1 || die "pkg-config não encontrado"
  missing=""
  for lib in webkit2gtk-4.1 javascriptcoregtk-4.1 libsoup-3.0 gtk+-3.0; do
    if pkg-config --exists "$lib"; then ok "$lib $(pkg-config --modversion "$lib")"; else missing="$missing $lib"; fi
  done
  if [ -n "$missing" ]; then
    die "bibliotecas ausentes:$missing
  Fedora:        sudo dnf install webkit2gtk4.1-devel libsoup3-devel gtk3-devel openssl-devel librsvg2-devel
  Debian/Ubuntu: sudo apt install libwebkit2gtk-4.1-dev libsoup-3.0-dev libgtk-3-dev libssl-dev librsvg2-dev"
  fi
else
  xcode-select -p >/dev/null 2>&1 || die "Xcode Command Line Tools ausentes: xcode-select --install"
  ok "Xcode Command Line Tools"
fi

if [ -n "$TARGET" ]; then
  if [ "$TARGET" = universal-apple-darwin ]; then
    wanted="aarch64-apple-darwin x86_64-apple-darwin"
  else
    wanted="$TARGET"
  fi
  for t in $wanted; do
    if rustup target list --installed 2>/dev/null | grep -qx "$t"; then
      ok "alvo Rust $t"
    else
      warn "alvo Rust $t não instalado"
      run rustup target add "$t"
    fi
  done
fi

VERSION="$(node -p "require('./src-tauri/tauri.conf.json').version")"
PRODUCT="$(node -p "require('./src-tauri/tauri.conf.json').productName")"
ok "$PRODUCT $VERSION"

# --------------------------------------------------------------------- limpeza
if [ "$CLEAN" -eq 1 ]; then
  step "Limpando builds anteriores"
  run rm -rf dist
  run cargo clean --manifest-path src-tauri/Cargo.toml
fi

# ---------------------------------------------------------------- dependências
if [ "$SKIP_INSTALL" -eq 0 ]; then
  step "Instalando dependências npm"
  if [ -f package-lock.json ]; then run npm ci; else run npm install; fi
fi

# ---------------------------------------------------------------------- testes
if [ "$SKIP_TESTS" -eq 0 ]; then
  step "Rodando testes"
  run cargo test --workspace --manifest-path src-tauri/Cargo.toml
  run npm run typecheck
fi

# ----------------------------------------------------------------------- build
step "Compilando e empacotando"
BUILD_ARGS="build"
[ "$DEBUG" -eq 1 ] && BUILD_ARGS="$BUILD_ARGS --debug"
[ -n "$TARGET" ] && BUILD_ARGS="$BUILD_ARGS --target $TARGET"
[ -n "$BUNDLES" ] && BUILD_ARGS="$BUILD_ARGS --bundles $BUNDLES"
# shellcheck disable=SC2086 # divisão de palavras intencional
run npx tauri $BUILD_ARGS

# ---------------------------------------------------------------- distribuição
PROFILE=release; [ "$DEBUG" -eq 1 ] && PROFILE=debug
if [ -n "$TARGET" ]; then BUNDLE_DIR="src-tauri/target/$TARGET/$PROFILE/bundle"; else BUNDLE_DIR="src-tauri/target/$PROFILE/bundle"; fi
DEST="$OUT_DIR/$VERSION/$OS-$ARCH"

step "Copiando instaladores para ${DEST#"$ROOT"/}"
if [ "$DRY_RUN" -eq 1 ]; then
  printf '  %s(dry-run) copiaria os arquivos de %s e geraria SHA256SUMS.txt%s\n' "$DIM" "$BUNDLE_DIR" "$RESET"
  exit 0
fi
[ -d "$BUNDLE_DIR" ] || die "pasta de bundles não encontrada: $BUNDLE_DIR"

rm -rf "$DEST"
mkdir -p "$DEST"
count=0
# Instaladores (arquivos) e o pacote .app (diretório, compactado em zip para distribuição).
for f in "$BUNDLE_DIR"/deb/*.deb "$BUNDLE_DIR"/rpm/*.rpm "$BUNDLE_DIR"/appimage/*.AppImage "$BUNDLE_DIR"/dmg/*.dmg; do
  [ -f "$f" ] || continue
  cp "$f" "$DEST/"
  ok "$(basename "$f")"
  count=$((count + 1))
done
for app in "$BUNDLE_DIR"/macos/*.app; do
  [ -d "$app" ] || continue
  zip_name="$(basename "$app" .app)_${VERSION}_${ARCH}.app.zip"
  ditto -c -k --keepParent "$app" "$DEST/$zip_name"
  ok "$zip_name"
  count=$((count + 1))
done
[ "$count" -gt 0 ] || die "nenhum instalador encontrado em $BUNDLE_DIR"

(
  cd "$DEST"
  if command -v sha256sum >/dev/null 2>&1; then sha256sum -- * > SHA256SUMS.txt; else shasum -a 256 -- * > SHA256SUMS.txt; fi
)
ok "SHA256SUMS.txt"

printf '\n%s%s concluído:%s %d arquivo(s) em %s\n' "$GREEN" "$BOLD" "$RESET" "$count" "$DEST"
