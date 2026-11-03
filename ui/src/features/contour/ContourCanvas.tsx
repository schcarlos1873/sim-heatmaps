import { useEffect, useMemo, useRef, type KeyboardEvent, type MouseEvent, type ReactNode } from "react";
import { rgbCss, type Palette, type Rgb } from "./colors";
import { useElementSize } from "./chartUtils";

export interface Cursor {
  scan: number;
  bin: number;
}

interface Props {
  values: (number | null)[][];
  binCount: number;
  colorize: (v: number) => Rgb;
  palette: Palette;
  cursor: Cursor | null;
  onCursor: (c: Cursor | null) => void;
  label: string;
  /** Conteúdo do tooltip flutuante para a célula sob o cursor. */
  tooltip?: ReactNode;
}

/** Imagem 1 px por célula (bin × scan); o canvas visível a amplia sem suavização. */
function renderImage(values: (number | null)[][], bins: number, colorize: (v: number) => Rgb): HTMLCanvasElement {
  const rows = values.length;
  const canvas = document.createElement("canvas");
  canvas.width = Math.max(1, bins);
  canvas.height = Math.max(1, rows);
  const ctx = canvas.getContext("2d")!;
  const img = ctx.createImageData(canvas.width, canvas.height);
  const px = img.data;
  for (let r = 0; r < rows; r++) {
    const row = values[r];
    for (let c = 0; c < bins; c++) {
      const v = row[c];
      if (v === null || v === undefined) continue; // transparente: aparece a hachura de "sem dado"
      const [red, green, blue] = colorize(v);
      const o = (r * bins + c) * 4;
      px[o] = red;
      px[o + 1] = green;
      px[o + 2] = blue;
      px[o + 3] = 255;
    }
  }
  ctx.putImageData(img, 0, 0);
  return canvas;
}

function hatch(ctx: CanvasRenderingContext2D, p: Palette, surface: string): CanvasPattern | null {
  const tile = document.createElement("canvas");
  tile.width = tile.height = 8;
  const t = tile.getContext("2d")!;
  t.fillStyle = surface;
  t.fillRect(0, 0, 8, 8);
  t.strokeStyle = rgbCss(p.nodata);
  t.lineWidth = 1;
  t.beginPath();
  t.moveTo(0, 8);
  t.lineTo(8, 0);
  t.stroke();
  return ctx.createPattern(tile, "repeat");
}

export function ContourCanvas({ values, binCount, colorize, palette, cursor, onCursor, label, tooltip }: Props) {
  const [box, size] = useElementSize<HTMLDivElement>();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rows = values.length;
  const image = useMemo(() => renderImage(values, binCount, colorize), [values, binCount, colorize]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || size.width === 0 || size.height === 0) return;
    const dpr = window.devicePixelRatio || 1;
    canvas.width = Math.round(size.width * dpr);
    canvas.height = Math.round(size.height * dpr);
    const ctx = canvas.getContext("2d")!;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    const surface = getComputedStyle(document.documentElement).getPropertyValue("--viz-surface") || "#fff";
    ctx.fillStyle = hatch(ctx, palette, surface) ?? surface;
    ctx.fillRect(0, 0, size.width, size.height);
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(image, 0, 0, size.width, size.height);
  }, [image, size, palette]);

  const cellAt = (e: MouseEvent<HTMLDivElement>): Cursor | null => {
    const rect = e.currentTarget.getBoundingClientRect();
    const x = (e.clientX - rect.left) / rect.width;
    const y = (e.clientY - rect.top) / rect.height;
    if (x < 0 || x >= 1 || y < 0 || y >= 1) return null;
    return { bin: Math.floor(x * binCount), scan: Math.floor(y * rows) };
  };

  const onKey = (e: KeyboardEvent<HTMLDivElement>) => {
    const step = e.shiftKey ? 10 : 1;
    const cur = cursor ?? { scan: 0, bin: Math.floor(binCount / 2) };
    const moves: Record<string, Cursor> = {
      ArrowLeft: { ...cur, bin: Math.max(0, cur.bin - step) },
      ArrowRight: { ...cur, bin: Math.min(binCount - 1, cur.bin + step) },
      ArrowUp: { ...cur, scan: Math.max(0, cur.scan - step) },
      ArrowDown: { ...cur, scan: Math.min(rows - 1, cur.scan + step) },
    };
    if (e.key in moves) {
      e.preventDefault();
      onCursor(moves[e.key]);
    } else if (e.key === "Escape") onCursor(null);
  };

  const cx = cursor ? ((cursor.bin + 0.5) / binCount) * size.width : 0;
  const cy = cursor ? ((cursor.scan + 0.5) / rows) * size.height : 0;
  const tipLeft = cx > size.width - 220;
  const tipUp = cy > size.height - 90;

  return (
    <div
      ref={box}
      className="relative h-full w-full cursor-crosshair overflow-hidden rounded-md outline-offset-2"
      tabIndex={0}
      role="img"
      aria-label={label}
      onMouseMove={(e) => onCursor(cellAt(e))}
      onMouseLeave={() => onCursor(null)}
      onKeyDown={onKey}
    >
      <canvas ref={canvasRef} className="absolute inset-0 h-full w-full" />
      {cursor && (
        <>
          <svg className="pointer-events-none absolute inset-0 h-full w-full" aria-hidden="true">
            <line x1={cx} x2={cx} y1={0} y2={size.height} stroke="var(--text-primary)" strokeOpacity={0.55} strokeWidth={1} />
            <line x1={0} x2={size.width} y1={cy} y2={cy} stroke="var(--text-primary)" strokeOpacity={0.55} strokeWidth={1} />
          </svg>
          {tooltip && (
            <div
              className="pointer-events-none absolute z-10 rounded-lg border border-border bg-bg px-2.5 py-1.5 text-xs shadow-lg"
              style={{ left: tipLeft ? cx - 12 : cx + 12, top: tipUp ? cy - 12 : cy + 12, transform: `translate(${tipLeft ? "-100%" : "0"}, ${tipUp ? "-100%" : "0"})` }}
            >
              {tooltip}
            </div>
          )}
        </>
      )}
    </div>
  );
}
