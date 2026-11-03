#!/usr/bin/env python3
"""Valida arquivos .al3 gerados pelo simulador contra o layout dos originais do Autoline.

Verificações:
  * UTF-8 sem BOM, quebras de linha CRLF, termina com "end AL300 SAMPLE\\r\\n"
  * cabeçalho de 26 linhas (AL300 SAMPLE, máquina, datas, Sample ID, NA, contagem)
  * nº de blocos PROP-SUMMARY == contagem declarada na linha 26
  * PROP-SUMMARY com 14 campos; n coerente com PROP-VALUES
  * média/desvio(populacional)/mín/máx coerentes com os valores (tolerância de arredondamento)
  * posições numéricas e crescentes (cada módulo tem sua grade) ou "Position 1"

Uso:
    python3 tmp/validate_al3.py ~/Documents/sim-heatmaps/export/*.al3
    python3 tmp/validate_al3.py --dry-run export-files/*.al3
    python3 tmp/validate_al3.py --strict-stats export-files/*.al3   # estatística divergente vira erro
"""
import argparse
import math
import re
import sys
from pathlib import Path

RED, GREEN, YELLOW, CYAN, DIM, RESET = "\033[31m", "\033[32m", "\033[33m", "\033[36m", "\033[2m", "\033[0m"
DATE = re.compile(r"^\d{2}/\d{2}/\d{4}$")
TIME = re.compile(r"^\d{2}:\d{2}:\d{2}$")
HEADER = re.compile(r"^AL300 SAMPLE \d{2}/\d{2}/\d{4} \d{2}:\d{2}:\d{2},.*\.AL1$")


def close(a, b, decimals):
    return abs(a - b) <= 0.5 * 10 ** -decimals + 1e-9


def validate(path: Path, strict_stats: bool):
    errors, warnings = [], []
    raw = path.read_bytes()
    if raw.startswith(b"\xef\xbb\xbf"):
        errors.append("arquivo possui BOM UTF-8")
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError as e:
        return [f"não é UTF-8 válido: {e}"], warnings, 0, 0
    if re.search(r"(?<!\r)\n", text):
        errors.append("há quebras de linha LF sem CR")
    if not text.endswith("end AL300 SAMPLE\r\n"):
        errors.append('não termina com "end AL300 SAMPLE\\r\\n"')

    lines = text.split("\r\n")
    if len(lines) < 27:
        return errors + ["cabeçalho truncado"], warnings, 0, 0
    expected = {2: "", 3: "0", 4: "", 5: "0", 6: "0", 11: "", 23: "", 24: "NA"}
    if not HEADER.match(lines[0]):
        errors.append(f"linha 1 inválida: {lines[0]!r}")
    if not lines[1].strip():
        errors.append("linha 2: máquina vazia")
    for idx, val in expected.items():
        if lines[idx] != val:
            errors.append(f"linha {idx + 1}: esperado {val!r}, obtido {lines[idx]!r}")
    for idx in (7, 9):
        if not DATE.match(lines[idx]):
            errors.append(f"linha {idx + 1}: data inválida {lines[idx]!r}")
    for idx in (8, 10):
        if not TIME.match(lines[idx]):
            errors.append(f"linha {idx + 1}: hora inválida {lines[idx]!r}")
    for idx in list(range(12, 17)) + list(range(18, 23)):
        if lines[idx] != "NA":
            errors.append(f"linha {idx + 1}: esperado 'NA'")
    sample_id = lines[17]
    if lines[0].rsplit("\\", 1)[-1] != f"{sample_id}.AL1":
        warnings.append("Sample ID da linha 1 difere da linha 18")
    if path.stem.upper() != sample_id.upper():
        warnings.append(f"nome do arquivo ({path.stem}) difere do Sample ID ({sample_id})")
    try:
        declared = int(lines[25])
    except ValueError:
        return errors + [f"linha 26: contagem inválida {lines[25]!r}"], warnings, 0, 0

    count, i, grids = 0, 26, set()
    while i < len(lines) and lines[i] != "end AL300 SAMPLE":
        if lines[i] != "PROP-SUMMARY":
            errors.append(f"linha {i + 1}: esperado PROP-SUMMARY, obtido {lines[i]!r}")
            break
        count += 1
        fields = lines[i + 1].split(";")
        if len(fields) != 14:
            errors.append(f"linha {i + 2}: PROP-SUMMARY com {len(fields)} campos (esperado 14)")
            break
        if lines[i + 2] != "PROP-VALUES":
            errors.append(f"linha {i + 3}: esperado PROP-VALUES")
            break
        j, vals, pos = i + 3, [], []
        while j < len(lines) and lines[j] not in ("PROP-SUMMARY", "end AL300 SAMPLE"):
            p, _, v = lines[j].partition(";")
            pos.append(p)
            vals.append(float(v))
            j += 1
        name = fields[2]
        n = int(fields[7])
        if n != len(vals) or fields[13] != fields[7]:
            errors.append(f"{name}: n={n} mas há {len(vals)} valores")
        if pos != ["Position 1"]:
            try:
                nums = [float(p) for p in pos]
                if any(b <= a for a, b in zip(nums, nums[1:])):
                    errors.append(f"{name}: posições não crescentes")
                grids.add(tuple(pos))
            except ValueError:
                errors.append(f"{name}: posição inválida em {pos}")
        if vals:
            mean = sum(vals) / len(vals)
            std = math.sqrt(sum((v - mean) ** 2 for v in vals) / len(vals))
            checks = [("média", float(fields[5]), mean, 2), ("desvio", float(fields[6]), std, 2),
                      ("mín", float(fields[8]), min(vals), 6), ("máx", float(fields[9]), max(vals), 6)]
            for label, got, exp, dec in checks:
                if not close(got, exp, dec):
                    msg = f"{name}: {label} {got} ≠ {round(exp, 4)}"
                    (errors if strict_stats else warnings).append(msg)
        i = j
    if count != declared:
        errors.append(f"linha 26 declara {declared} propriedades, encontradas {count}")
    return errors, warnings, count, len(grids)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("files", nargs="+", type=Path, help="arquivos .al3")
    ap.add_argument("--strict-stats", action="store_true", help="estatística divergente conta como erro")
    ap.add_argument("--dry-run", action="store_true", help="lista os arquivos sem validar")
    args = ap.parse_args()

    if args.dry_run:
        for f in args.files:
            print(f"{YELLOW}[dry-run]{RESET} validaria {CYAN}{f}{RESET}")
        return 0

    failed = 0
    for f in args.files:
        if not f.is_file():
            print(f"{RED}✕{RESET} {f}: arquivo não encontrado")
            failed += 1
            continue
        errors, warnings, count, grids = validate(f, args.strict_stats)
        mark = f"{RED}✕{RESET}" if errors else f"{GREEN}✓{RESET}"
        print(f"{mark} {f} {DIM}({count} propriedades, {grids} grades de posição){RESET}")
        for e in errors:
            print(f"    {RED}erro{RESET}  {e}")
        for w in warnings:
            print(f"    {YELLOW}aviso{RESET} {w}")
        failed += bool(errors)
    print(f"\n{len(args.files) - failed}/{len(args.files)} válidos")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
