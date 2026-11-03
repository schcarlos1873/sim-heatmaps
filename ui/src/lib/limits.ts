import type { Limits } from "./api";

export type LimitStatus = "ok" | "warning" | "error";

/** Espelha `domain::property::Limits::classify`. */
export function classify(limits: Limits, value: number): LimitStatus {
  if (value < limits.errorLow || value > limits.errorHigh) return "error";
  if (value < limits.warningLow || value > limits.warningHigh) return "warning";
  return "ok";
}

export const statusClass: Record<LimitStatus, string> = {
  ok: "bg-ok-bg text-ok-fg",
  warning: "bg-warn-bg text-warn-fg",
  error: "bg-bad-bg text-bad-fg",
};

export const statusDot: Record<LimitStatus, string> = {
  ok: "bg-ok-dot",
  warning: "bg-warn-dot",
  error: "bg-bad-dot",
};

/** Ordem de exibição dos limites, do menor para o maior, com a cor da faixa. */
export const limitKeys: { key: keyof Limits; dot: string }[] = [
  { key: "errorLow", dot: "bg-bad-dot" },
  { key: "warningLow", dot: "bg-warn-dot" },
  { key: "target", dot: "bg-ok-dot" },
  { key: "warningHigh", dot: "bg-warn-dot" },
  { key: "errorHigh", dot: "bg-bad-dot" },
];

export function fmt(value: number, decimals: number): string {
  const s = value.toFixed(decimals);
  const trimmed = s.includes(".") ? s.replace(/0+$/, "").replace(/\.$/, "") : s;
  return trimmed === "-0" ? "0" : trimmed;
}

export function parseNumber(text: string): number | null {
  const t = text.trim().replace(",", ".");
  if (t === "" || !/^-?\d*\.?\d+$|^-?\d+\.?$/.test(t)) return null;
  const n = Number(t);
  return Number.isFinite(n) ? n : null;
}

/** `2026-11-02T08:00:00` → `2026-11-02 08:00:00`. */
export function fmtDateTime(iso: string): string {
  return iso.replace("T", " ").replace(/\.\d+$/, "");
}

/** Valor para `<input type="datetime-local">` com a hora local atual (sem segundos). */
export function nowLocalInput(): string {
  const d = new Date();
  d.setSeconds(0, 0);
  return new Date(d.getTime() - d.getTimezoneOffset() * 60000).toISOString().slice(0, 16);
}
