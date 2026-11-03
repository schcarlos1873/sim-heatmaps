import { useEffect, useState } from "react";
import type { ContourColors, Limits } from "../../lib/api";

/**
 * Escala de cor do Contour Map (eixo Z), ancorada nos limites da propriedade.
 * - `gradient`: a cor é exata em cada limite (Error Low, Warning Low, Target, Warning High, Error High)
 *   e muda gradualmente entre dois limites vizinhos; além dos Errors fica fixa.
 * - `steps`: uma cor por faixa (as mesmas 5 cores, sem transição).
 * As cores vêm de Configurações (tabela `settings`).
 */
export type ColorMode = "gradient" | "steps";
export const colorModes: ColorMode[] = ["gradient", "steps"];

export function readColorMode(): ColorMode {
  const saved = localStorage.getItem("contourColorMode") as ColorMode;
  return colorModes.includes(saved) ? saved : "gradient";
}

export type Rgb = [number, number, number];

/** Ordem de cima (maior valor) para baixo. */
export const COLOR_KEYS = ["errorHigh", "warningHigh", "target", "warningLow", "errorLow"] as const;
export type ColorKey = (typeof COLOR_KEYS)[number];
export type Palette = Record<ColorKey, Rgb> & { nodata: Rgb };

function parseHex(hex: string): Rgb {
  const h = hex.trim().replace("#", "");
  return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16)];
}

export const rgbCss = (c: Rgb) => `rgb(${c[0]} ${c[1]} ${c[2]})`;

/** Cores configuradas + hachura "Sem dado" do tema atual (token `--viz-nodata`). */
export function buildPalette(colors: ContourColors): Palette {
  const nodata = getComputedStyle(document.documentElement).getPropertyValue("--viz-nodata") || "#808080";
  const p = Object.fromEntries(COLOR_KEYS.map((k) => [k, parseHex(colors[k])])) as Record<ColorKey, Rgb>;
  return { ...p, nodata: parseHex(nodata) };
}

/** Retorna `true` quando o tema escuro está ativo (acompanha a troca de tema). */
export function useIsDark(): boolean {
  const [dark, setDark] = useState(() => document.documentElement.classList.contains("dark"));
  useEffect(() => {
    const el = document.documentElement;
    const obs = new MutationObserver(() => setDark(el.classList.contains("dark")));
    obs.observe(el, { attributes: true, attributeFilter: ["class"] });
    return () => obs.disconnect();
  }, []);
  return dark;
}

function mix(a: Rgb, b: Rgb, t: number): Rgb {
  return [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
}

/** Função de cor do mapa para o modo escolhido. */
export function colorizer(mode: ColorMode, limits: Limits, p: Palette): (v: number) => Rgb {
  const { errorLow, warningLow, target, warningHigh, errorHigh } = limits;
  if (mode === "steps") {
    return (v) => {
      if (v < errorLow) return p.errorLow;
      if (v < warningLow) return p.warningLow;
      if (v <= warningHigh) return p.target;
      if (v <= errorHigh) return p.warningHigh;
      return p.errorHigh;
    };
  }
  // Segmentos [limite inferior, limite superior, cor inferior, cor superior], do menor para o maior.
  const segments: [number, number, Rgb, Rgb][] = [
    [errorLow, warningLow, p.errorLow, p.warningLow],
    [warningLow, target, p.warningLow, p.target],
    [target, warningHigh, p.target, p.warningHigh],
    [warningHigh, errorHigh, p.warningHigh, p.errorHigh],
  ];
  return (v) => {
    if (v <= errorLow) return p.errorLow;
    if (v >= errorHigh) return p.errorHigh;
    for (const [lo, hi, a, b] of segments) {
      if (v <= hi) return hi > lo ? mix(a, b, (v - lo) / (hi - lo)) : b;
    }
    return p.errorHigh;
  };
}

/**
 * Legenda do degradê (CSS linear-gradient, de cima = Error High para baixo = Error Low).
 * Os 5 limites ficam igualmente espaçados; cada trecho é linear em valor.
 */
export function gradientCss(p: Palette): string {
  const stops = COLOR_KEYS.map((k, i) => `${rgbCss(p[k])} ${i * 25}%`);
  return `linear-gradient(to bottom, ${stops.join(", ")})`;
}
