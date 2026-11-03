import { useState, type MouseEvent, type ReactNode } from "react";
import type { Limits } from "../../lib/api";
import { fmt } from "../../lib/limits";
import { limitLines, linePath, niceTicks, useElementSize } from "./chartUtils";

const AXIS_TEXT = "fill-[var(--viz-ink-muted)] text-[10px] tabular-nums";

export interface SeriesSpec {
  label: string;
  color: string;
  values: (number | null)[];
}

/** Legenda de séries (sempre presente com 2+ séries; texto na cor de texto, não na da série). */
export function SeriesLegend({ series, extra }: { series: SeriesSpec[]; extra?: ReactNode }) {
  return (
    <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-muted">
      {series.map((s) => (
        <span key={s.label} className="inline-flex items-center gap-1.5">
          <span className="h-0.5 w-4 rounded-full" style={{ background: s.color }} aria-hidden="true" />
          {s.label}
        </span>
      ))}
      {extra}
    </div>
  );
}

/** Faixa mín–máx (área) com lacunas onde não há dado. */
function bandPath(xs: number[], lo: (number | null)[], hi: (number | null)[], x: (v: number) => number, y: (v: number) => number): string {
  let d = "";
  let seg: number[] = [];
  const flush = () => {
    if (seg.length > 1) {
      d += seg.map((i, k) => `${k ? "L" : "M"}${x(xs[i]).toFixed(1)},${y(hi[i]!).toFixed(1)}`).join("");
      d += [...seg].reverse().map((i) => `L${x(xs[i]).toFixed(1)},${y(lo[i]!).toFixed(1)}`).join("") + "Z";
    }
    seg = [];
  };
  xs.forEach((_, i) => (lo[i] === null || hi[i] === null ? flush() : seg.push(i)));
  flush();
  return d;
}

interface CdProps {
  binCenters: number[];
  widthMm: number;
  band: { min: (number | null)[]; max: (number | null)[] };
  series: SeriesSpec[];
  limits: Limits;
  domain: [number, number];
  decimals: number;
  /** Largura (px) à esquerda reservada ao eixo Y, igual à coluna da legenda do mapa. */
  gutter: number;
  height: number;
  label: string;
}

/** Perfil CD: x = posição na largura (mm), alinhado com o mapa acima. */
export function CdChart({ binCenters, widthMm, band, series, limits, domain, decimals, gutter, height, label }: CdProps) {
  const [ref, { width }] = useElementSize<HTMLDivElement>();
  const [hover, setHover] = useState<number | null>(null);
  const top = 8;
  const bottom = height - 20;
  const right = width;
  const x = (mm: number) => gutter + (mm / widthMm) * (right - gutter);
  const y = (v: number) => bottom - ((v - domain[0]) / (domain[1] - domain[0])) * (bottom - top);
  const yTicks = niceTicks(domain[0], domain[1], 4);
  const xTicks = niceTicks(0, widthMm, 8).filter((v) => v <= widthMm);

  const onMove = (e: MouseEvent<SVGSVGElement>) => {
    const px = e.clientX - e.currentTarget.getBoundingClientRect().left;
    if (px < gutter) return setHover(null);
    const bin = Math.floor(((px - gutter) / (right - gutter)) * binCenters.length);
    setHover(bin >= 0 && bin < binCenters.length ? bin : null);
  };

  return (
    <div ref={ref} className="relative w-full" style={{ height }}>
      {width > 0 && (
        <svg width={width} height={height} role="img" aria-label={label} onMouseMove={onMove} onMouseLeave={() => setHover(null)}>
          {yTicks.map((v) => (
            <g key={v}>
              <line x1={gutter} x2={right} y1={y(v)} y2={y(v)} stroke="var(--viz-grid)" strokeWidth={1} />
              <text x={gutter - 6} y={y(v)} dy="0.32em" textAnchor="end" className={AXIS_TEXT}>
                {fmt(v, decimals)}
              </text>
            </g>
          ))}
          <line x1={gutter} x2={right} y1={bottom} y2={bottom} stroke="var(--viz-axis)" />
          {xTicks.map((v) => (
            <text key={v} x={x(v)} y={height - 6} textAnchor={v === 0 ? "start" : v >= widthMm ? "end" : "middle"} className={AXIS_TEXT}>
              {fmt(v, 0)}
            </text>
          ))}
          {limitLines.map((l) => (
            <g key={l.key}>
              <line x1={gutter} x2={right} y1={y(limits[l.key])} y2={y(limits[l.key])} stroke={l.color} strokeWidth={1} strokeDasharray={l.dash} />
              <text x={right - 2} y={y(limits[l.key]) - 3} textAnchor="end" className="fill-[var(--viz-ink-muted)] text-[9px] font-semibold">
                {l.label}
              </text>
            </g>
          ))}
          <path d={bandPath(binCenters, band.min, band.max, x, y)} fill="var(--viz-series-1)" fillOpacity={0.14} />
          {series.map((s) => (
            <path key={s.label} d={linePath(binCenters.map((a, i) => ({ a, b: s.values[i] })), (a, b) => [x(a), y(b)])} fill="none" stroke={s.color} strokeWidth={s === series[0] ? 1.5 : 1.25} strokeLinejoin="round" />
          ))}
          {hover !== null && <line x1={x(binCenters[hover])} x2={x(binCenters[hover])} y1={top} y2={bottom} stroke="var(--text-primary)" strokeOpacity={0.5} />}
        </svg>
      )}
      {hover !== null && (
        <div className="pointer-events-none absolute top-1 z-10 rounded-lg border border-border bg-bg px-2.5 py-1.5 text-xs shadow-lg" style={{ left: Math.min(x(binCenters[hover]) + 10, width - 180) }}>
          <p className="font-medium tabular-nums">
            Bin {hover + 1} · {fmt(binCenters[hover], 1)} mm
          </p>
          {series.map((s) => (
            <p key={s.label} className="flex items-center gap-1.5 tabular-nums">
              <span className="h-0.5 w-3 rounded-full" style={{ background: s.color }} aria-hidden="true" />
              {s.label}: {s.values[hover] === null ? "–" : fmt(s.values[hover]!, decimals)}
            </p>
          ))}
          <p className="text-muted tabular-nums">
            min/max: {band.min[hover] === null ? "–" : `${fmt(band.min[hover]!, decimals)} / ${fmt(band.max[hover]!, decimals)}`}
          </p>
        </div>
      )}
    </div>
  );
}

interface MdProps {
  series: SeriesSpec[];
  limits: Limits;
  domain: [number, number];
  decimals: number;
  /** Altura do mapa (px): a área de plotagem coincide com as linhas dos scans. */
  plotHeight: number;
  axisHeight: number;
  scanLabel: (scan: number) => string;
  label: string;
}

/** Perfil MD: y = scans (de cima para baixo, alinhado com o mapa), x = valor. */
export function MdChart({ series, limits, domain, decimals, plotHeight, axisHeight, scanLabel, label }: MdProps) {
  const [ref, { width }] = useElementSize<HTMLDivElement>();
  const [hover, setHover] = useState<number | null>(null);
  const rows = series[0]?.values.length ?? 0;
  const left = 4;
  const right = width - 4;
  const x = (v: number) => left + ((v - domain[0]) / (domain[1] - domain[0])) * (right - left);
  const y = (i: number) => ((i + 0.5) / Math.max(1, rows)) * plotHeight;
  const xTicks = niceTicks(domain[0], domain[1], 3);

  const onMove = (e: MouseEvent<SVGSVGElement>) => {
    const py = e.clientY - e.currentTarget.getBoundingClientRect().top;
    const i = Math.floor((py / plotHeight) * rows);
    setHover(i >= 0 && i < rows ? i : null);
  };

  return (
    <div ref={ref} className="relative h-full w-full">
      {width > 0 && (
        <svg width={width} height={plotHeight + axisHeight} role="img" aria-label={label} onMouseMove={onMove} onMouseLeave={() => setHover(null)}>
          {xTicks.map((v) => (
            <g key={v}>
              <line x1={x(v)} x2={x(v)} y1={0} y2={plotHeight} stroke="var(--viz-grid)" />
              <text x={x(v)} y={plotHeight + 14} textAnchor="middle" className={AXIS_TEXT}>
                {fmt(v, decimals)}
              </text>
            </g>
          ))}
          <line x1={left} x2={right} y1={plotHeight} y2={plotHeight} stroke="var(--viz-axis)" />
          {limitLines.map((l) => (
            <g key={l.key}>
              <line x1={x(limits[l.key])} x2={x(limits[l.key])} y1={0} y2={plotHeight} stroke={l.color} strokeWidth={1} strokeDasharray={l.dash} />
              <text x={x(limits[l.key])} y={plotHeight + 26} textAnchor="middle" className="fill-[var(--viz-ink-muted)] text-[9px] font-semibold">
                {l.label}
              </text>
            </g>
          ))}
          {series.map((s, k) => (
            <path key={s.label} d={linePath(s.values.map((b, a) => ({ a, b })), (a, b) => [x(b), y(a)])} fill="none" stroke={s.color} strokeWidth={k === 0 ? 1.5 : 1.25} strokeLinejoin="round" />
          ))}
          {hover !== null && <line x1={left} x2={right} y1={y(hover)} y2={y(hover)} stroke="var(--text-primary)" strokeOpacity={0.5} />}
        </svg>
      )}
      {hover !== null && (
        <div className="pointer-events-none absolute right-1 z-10 rounded-lg border border-border bg-bg px-2.5 py-1.5 text-xs shadow-lg" style={{ top: Math.min(y(hover) + 10, plotHeight - 60) }}>
          <p className="font-medium tabular-nums">{scanLabel(hover)}</p>
          {series.map((s) => (
            <p key={s.label} className="flex items-center gap-1.5 tabular-nums">
              <span className="h-0.5 w-3 rounded-full" style={{ background: s.color }} aria-hidden="true" />
              {s.label}: {s.values[hover] === null ? "–" : fmt(s.values[hover]!, decimals)}
            </p>
          ))}
        </div>
      )}
    </div>
  );
}
