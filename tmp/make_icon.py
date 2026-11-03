#!/usr/bin/env python3
"""Gera o ícone base (PNG 1024x1024) do Sim Heatmaps usando apenas a stdlib.

Uso:
    python3 tmp/make_icon.py                       # grava src-tauri/icons/app-icon.png
    python3 tmp/make_icon.py --dry-run             # mostra o que seria feito
    cargo tauri icon src-tauri/icons/app-icon.png  # gera os tamanhos/plataformas
"""
import argparse
import struct
import sys
import zlib
from pathlib import Path

GREEN, YELLOW, CYAN, RESET = "\033[32m", "\033[33m", "\033[36m", "\033[0m"
ROOT = Path(__file__).resolve().parent.parent
DEFAULT_OUT = ROOT / "src-tauri" / "icons" / "app-icon.png"

BG = (29, 78, 216)       # azul accent (#1D4ED8)
FG = (255, 255, 255)
BARS = [0.42, 0.62, 0.50, 0.78, 0.58, 0.70, 0.46]  # perfil CD estilizado


def pixel(x, y, size):
    r = size * 0.22  # cantos arredondados
    cx = min(max(x, r), size - r)
    cy = min(max(y, r), size - r)
    if (x - cx) ** 2 + (y - cy) ** 2 > r * r:
        return (0, 0, 0, 0)
    pad, gap = size * 0.2, size * 0.035
    width = (size - 2 * pad - gap * (len(BARS) - 1)) / len(BARS)
    base = size * 0.78
    for i, h in enumerate(BARS):
        x0 = pad + i * (width + gap)
        if x0 <= x < x0 + width and base - h * size * 0.56 <= y < base:
            return (*FG, 255)
    if base + size * 0.03 <= y < base + size * 0.06 and pad <= x < size - pad:
        return (*FG, 200)
    return (*BG, 255)


def png(size):
    rows = bytearray()
    for y in range(size):
        rows.append(0)
        for x in range(size):
            rows.extend(pixel(x + 0.5, y + 0.5, size))

    def chunk(tag, data):
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    ihdr = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"IDAT", zlib.compress(bytes(rows), 9)) + chunk(b"IEND", b"")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", type=Path, default=DEFAULT_OUT, help="arquivo PNG de saída")
    ap.add_argument("--size", type=int, default=1024, help="tamanho em pixels (padrão 1024)")
    ap.add_argument("--dry-run", action="store_true", help="não grava; apenas mostra o que seria feito")
    args = ap.parse_args()

    if args.dry_run:
        print(f"{YELLOW}[dry-run]{RESET} geraria {args.size}x{args.size} em {CYAN}{args.out}{RESET}")
        return 0
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_bytes(png(args.size))
    print(f"{GREEN}ok{RESET} {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
