import { useEffect, useRef, useState } from "react";
import type { Limits } from "../../lib/api";

/** Tamanho do elemento em pixels CSS, atualizado por ResizeObserver. */
export function useElementSize<T extends HTMLElement>() {
  const ref = useRef<T>(null);
  const [size, setSize] = useState({ width: 0, height: 0 });
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const obs = new ResizeObserver(([entry]) => {
      const { width, height } = entry.contentRect;
      setSize((s) => (s.width === width && s.height === height ? s : { width, height }));
    });
    obs.observe(el);
    return () => obs.disconnect();
  }, []);
  return [ref, size] as const;
}

/** Ticks "redondos" (1, 2, 2.5, 5 × 10^n) cobrindo [min, max]. */
export function niceTicks(min: number, max: number, count = 5): number[] {
  if (!Number.isFinite(min) || !Number.isFinite(max) || min === max) return [min];
  const raw = (max - min) / count;
  const mag = 10 ** Math.floor(Math.log10(raw));
  const step = [1, 2, 2.5, 5, 10].map((m) => m * mag).find((s) => s >= raw) ?? raw;
  const ticks: number[] = [];
  for (let v = Math.ceil(min / step) * step; v <= max + step * 1e-9; v += step) ticks.push(+v.toFixed(10));
  return ticks;
}

/** Domínio do eixo de valores: limites Error e extremos dos dados, com folga de 8%. */
export function valueDomain(limits: Limits, values: (number | null)[][]): [number, number] {
  let lo = limits.errorLow;
  let hi = limits.errorHigh;
  for (const series of values) {
    for (const v of series) {
      if (v === null) continue;
      if (v < lo) lo = v;
      if (v > hi) hi = v;
    }
  }
  const pad = (hi - lo) * 0.08 || 1;
  return [lo - pad, hi + pad];
}

/** Caminho SVG de uma série com lacunas (null interrompe a linha). */
export function linePath(points: { a: number; b: number | null }[], map: (a: number, b: number) => [number, number]): string {
  let d = "";
  let pen = false;
  for (const { a, b } of points) {
    if (b === null) {
      pen = false;
      continue;
    }
    const [x, y] = map(a, b);
    d += `${pen ? "L" : "M"}${x.toFixed(1)},${y.toFixed(1)}`;
    pen = true;
  }
  return d;
}

/** Linhas de limite: rótulo curto e token de cor de status (sempre com texto). */
export const limitLines: { key: keyof Limits; label: string; color: string; dash?: string }[] = [
  { key: "errorHigh", label: "EH", color: "var(--viz-critical)", dash: "4 3" },
  { key: "warningHigh", label: "WH", color: "var(--viz-warning)", dash: "4 3" },
  { key: "target", label: "T", color: "var(--viz-good)" },
  { key: "warningLow", label: "WL", color: "var(--viz-warning)", dash: "4 3" },
  { key: "errorLow", label: "EL", color: "var(--viz-critical)", dash: "4 3" },
];
