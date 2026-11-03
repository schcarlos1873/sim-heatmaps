import type { Limits } from "../../lib/api";

/** Visualização das faixas: vermelho | amarelo | verde | amarelo | vermelho. */
export function LimitBar({ limits }: { limits: Partial<Record<keyof Limits, number | null>> }) {
  const { errorLow, warningLow, target, warningHigh, errorHigh } = limits;
  const vals = [errorLow, warningLow, target, warningHigh, errorHigh];
  const ordered = vals.every((v) => typeof v === "number" && Number.isFinite(v)) && vals.every((v, i) => i === 0 || (v as number) > (vals[i - 1] as number));
  if (!ordered) return <div className="h-3 rounded-full bg-surface" aria-hidden="true" />;
  const [el, wl, tg, wh, eh] = vals as number[];
  const span = eh - el;
  const margin = span * 0.15;
  const min = el - margin;
  const total = span + 2 * margin;
  const pct = (v: number) => `${((v - min) / total) * 100}%`;
  const w = (a: number, b: number) => `${((b - a) / total) * 100}%`;
  return (
    <div className="relative h-3 w-full overflow-hidden rounded-full" aria-hidden="true">
      <div className="absolute inset-0 bg-bad-dot/70" />
      <div className="absolute inset-y-0 bg-warn-dot/80" style={{ left: pct(el), width: w(el, eh) }} />
      <div className="absolute inset-y-0 bg-ok-dot/80" style={{ left: pct(wl), width: w(wl, wh) }} />
      <div className="absolute inset-y-0 w-0.5 bg-fg" style={{ left: pct(tg) }} />
    </div>
  );
}
