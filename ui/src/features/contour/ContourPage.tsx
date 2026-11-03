import { useCallback, useEffect, useMemo, useState } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { api, defaultContourColors, type ContourColors, type ContourData, type ReelRecord, type ScanInfo } from "../../lib/api";
import { classify, fmt, fmtDateTime, statusDot, type LimitStatus } from "../../lib/limits";
import { useI18n } from "../../i18n";
import { useToast } from "../../components/Toast";
import { Button } from "../../components/Button";
import { SelectField } from "../../components/Field";
import { Card, EmptyState, PageHeader, Spinner } from "../../components/Page";
import { useMachines } from "../useMachines";
import { MachineSelect } from "../MachineSelect";
import { ContourCanvas, type Cursor } from "./ContourCanvas";
import { CdChart, MdChart, SeriesLegend, type SeriesSpec } from "./ProfileCharts";
import { buildPalette, colorizer, colorModes, gradientCss, readColorMode, rgbCss, useIsDark, type ColorMode, type Palette } from "./colors";
import { valueDomain } from "./chartUtils";

/** Distância da borda esquerda até o mapa (coluna da legenda + espaço); o eixo Y do perfil CD usa a mesma medida. */
const GUTTER = 96;
const MD_WIDTH = 220;
const FOOTER = 32;
const statusSymbol: Record<LimitStatus, string> = { ok: "✓", warning: "!", error: "✕" };

function useMapHeight() {
  const calc = () => Math.round(Math.max(300, Math.min(680, window.innerHeight - 440)));
  const [h, setH] = useState(calc);
  useEffect(() => {
    const on = () => setH(calc());
    window.addEventListener("resize", on);
    return () => window.removeEventListener("resize", on);
  }, []);
  return h;
}

function StatusBadge({ status }: { status: LimitStatus }) {
  const { t } = useI18n();
  return (
    <span className="inline-flex items-center gap-1">
      <span className={`inline-flex size-3.5 items-center justify-center rounded-full text-[9px] font-bold text-white ${statusDot[status]}`} aria-hidden="true">
        {statusSymbol[status]}
      </span>
      {t(`status.${status}`)}
    </span>
  );
}

function Legend({ data, mode, palette, first, last }: { data: ContourData; mode: ColorMode; palette: Palette; first?: ScanInfo; last?: ScanInfo }) {
  const { t } = useI18n();
  const { limits, decimals, unit } = data.property;
  const d = Math.max(decimals, 1);
  const steps: { color: string; label: string }[] = [
    { color: rgbCss(palette.errorHigh), label: `> ${fmt(limits.errorHigh, d)}` },
    { color: rgbCss(palette.warningHigh), label: `${fmt(limits.warningHigh, d)}–${fmt(limits.errorHigh, d)}` },
    { color: rgbCss(palette.target), label: `${fmt(limits.warningLow, d)}–${fmt(limits.warningHigh, d)}` },
    { color: rgbCss(palette.warningLow), label: `${fmt(limits.errorLow, d)}–${fmt(limits.warningLow, d)}` },
    { color: rgbCss(palette.errorLow), label: `< ${fmt(limits.errorLow, d)}` },
  ];
  const anchors: [string, number][] = [
    ["EH", limits.errorHigh],
    ["WH", limits.warningHigh],
    ["T", limits.target],
    ["WL", limits.warningLow],
    ["EL", limits.errorLow],
  ];
  return (
    <div className="flex h-full flex-col justify-between gap-2 pr-2 text-[11px] leading-tight text-muted">
      <div className="tabular-nums">
        <p>{first ? fmtDateTime(first.startedAt).slice(11) : ""}</p>
        <p className="text-[10px]">{first ? fmtDateTime(first.startedAt).slice(0, 10) : ""}</p>
      </div>
      <div className="flex flex-col gap-1">
        <p className="font-medium text-fg">{unit || data.property.code}</p>
        {mode === "steps" ? (
          <ul className="flex flex-col gap-0.5">
            {steps.map((s) => (
              <li key={s.label} className="flex items-center gap-1.5 tabular-nums">
                <span className="size-3 shrink-0 rounded-sm border border-border" style={{ background: s.color }} aria-hidden="true" />
                <span className="truncate">{s.label}</span>
              </li>
            ))}
          </ul>
        ) : (
          <div className="flex gap-1.5">
            <div className="h-36 w-3 shrink-0 rounded-sm border border-border" style={{ background: gradientCss(palette) }} aria-hidden="true" />
            <div className="flex flex-col justify-between tabular-nums">
              {anchors.map(([k, v]) => (
                <span key={k} className={k === "T" ? "font-medium text-fg" : undefined}>
                  {k} {fmt(v, d)}
                </span>
              ))}
            </div>
          </div>
        )}
        <p className="mt-1 flex items-center gap-1.5">
          <span className="size-3 shrink-0 rounded-sm border border-border bg-[repeating-linear-gradient(135deg,var(--viz-nodata)_0_1px,transparent_1px_4px)]" aria-hidden="true" />
          {t("contour.noData")}
        </p>
      </div>
      <div className="tabular-nums">
        <p>{last ? fmtDateTime(last.endedAt).slice(11) : ""}</p>
        <p className="text-[10px]">{last ? `${fmt(last.mdPositionM / 1000, 1)} km` : ""}</p>
      </div>
    </div>
  );
}

function StatsPanel({ data }: { data: ContourData }) {
  const { t } = useI18n();
  const { limits, decimals, unit } = data.property;
  const counts = useMemo(() => {
    const c: Record<LimitStatus, number> = { ok: 0, warning: 0, error: 0 };
    for (const row of data.values) for (const v of row) if (v !== null) c[classify(limits, v)]++;
    return c;
  }, [data, limits]);
  const total = counts.ok + counts.warning + counts.error;
  const s = data.summary;
  const d = decimals + 1;
  const rows: [string, string][] = s
    ? [
        [t("contour.stats.mean"), fmt(s.mean, d)],
        [t("contour.stats.stdDev"), fmt(s.stdDev, d)],
        [t("contour.stats.twoSigma"), fmt(2 * s.stdDev, d)],
        [t("contour.stats.min"), fmt(s.min, decimals)],
        [t("contour.stats.max"), fmt(s.max, decimals)],
        [t("contour.stats.count"), s.count.toLocaleString()],
      ]
    : [];
  return (
    <div className="flex h-full flex-col gap-3 rounded-xl border border-border p-3 text-xs">
      <p className="font-semibold">
        {t("contour.stats.title")} {unit && <span className="font-normal text-muted">({unit})</span>}
      </p>
      <dl className="grid grid-cols-[1fr_auto] gap-x-3 gap-y-1 tabular-nums">
        {rows.map(([k, v]) => (
          <div key={k} className="contents">
            <dt className="text-muted">{k}</dt>
            <dd className="text-right font-medium">{v}</dd>
          </div>
        ))}
      </dl>
      {total > 0 && (
        <ul className="flex flex-col gap-1 border-t border-border pt-2">
          {(["ok", "warning", "error"] as const).map((k) => (
            <li key={k} className="flex items-center justify-between gap-2">
              <StatusBadge status={k} />
              <span className="font-medium tabular-nums">{fmt((counts[k] / total) * 100, 1)}%</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export function ContourPage({ initialReel }: { initialReel: ReelRecord | null }) {
  const { t, errorText } = useI18n();
  const toast = useToast();
  const { machines, machineId, setMachineId } = useMachines();
  const [reels, setReels] = useState<ReelRecord[]>([]);
  const [reelPk, setReelPk] = useState<number | null>(initialReel?.id ?? null);
  const [propertyCode, setPropertyCode] = useState<string>(() => localStorage.getItem("contourProperty") ?? "");
  const [colorMode, setColorMode] = useState<ColorMode>(readColorMode);
  const [colors, setColors] = useState<ContourColors>(defaultContourColors);
  const [data, setData] = useState<ContourData | null>(null);
  const [loading, setLoading] = useState(false);
  const [loadMs, setLoadMs] = useState<number | null>(null);
  const [cursor, setCursor] = useState<Cursor | null>(null);
  const dark = useIsDark();
  const mapHeight = useMapHeight();

  useEffect(() => {
    api
      .getSettings()
      .then((s) => setColors(s.contourColors))
      .catch((e) => toast("error", errorText(e)));
  }, [errorText, toast]);

  useEffect(() => {
    if (initialReel?.machineId) setMachineId(initialReel.machineId);
  }, [initialReel, setMachineId]);

  useEffect(() => {
    if (!machineId) return;
    api
      .listReels(machineId, 500)
      .then((list) => {
        setReels(list);
        setReelPk((cur) => (cur && list.some((r) => r.id === cur) ? cur : (list[0]?.id ?? null)));
      })
      .catch((e) => toast("error", errorText(e)));
  }, [machineId, errorText, toast]);

  const reel = reels.find((r) => r.id === reelPk) ?? null;
  const code = reel && (reel.files.some((f) => f.propertyCode === propertyCode) ? propertyCode : reel.files[0]?.propertyCode);

  const load = useCallback(
    async (fn: () => Promise<ContourData>) => {
      setLoading(true);
      setCursor(null);
      const t0 = performance.now();
      try {
        setData(await fn());
        setLoadMs(Math.round(performance.now() - t0));
      } catch (e) {
        setData(null);
        toast("error", errorText(e));
      } finally {
        setLoading(false);
      }
    },
    [errorText, toast],
  );

  useEffect(() => {
    if (reelPk && code) void load(() => api.loadContour(reelPk, code));
  }, [reelPk, code, load]);

  const openFile = async () => {
    const path = await openDialog({ multiple: false, filters: [{ name: "CSV", extensions: ["csv", "CSV"] }] });
    if (typeof path === "string") {
      setReelPk(null);
      void load(() => api.loadContourFile(path));
    }
  };

  // Cores de Configurações; a hachura "Sem dado" é relida a cada troca de tema (token --viz-nodata).
  const palette = useMemo(() => buildPalette(colors), [colors, dark]); // eslint-disable-line react-hooks/exhaustive-deps
  const colorize = useMemo(() => (data ? colorizer(colorMode, data.property.limits, palette) : null), [data, colorMode, palette]);

  const domain = useMemo(() => (data ? valueDomain(data.property.limits, [data.cd.min, data.cd.max, data.md]) : ([0, 1] as [number, number])), [data]);
  const binColumn = useMemo(() => (data && cursor ? data.values.map((row) => row[cursor.bin] ?? null) : null), [data, cursor?.bin]); // eslint-disable-line react-hooks/exhaustive-deps

  const scanText = (s: ScanInfo) => `${t("contour.scan")} ${s.number} ${s.direction === "forward" ? "→" : "←"} · ${fmtDateTime(s.startedAt).slice(11)}`;

  const cdSeries: SeriesSpec[] = data
    ? [
        { label: t("contour.cdAvg"), color: "var(--viz-series-1)", values: data.cd.avg },
        ...(cursor ? [{ label: `${t("contour.scan")} ${data.scans[cursor.scan].number}`, color: "var(--viz-series-2)", values: data.values[cursor.scan] }] : []),
      ]
    : [];
  const mdSeries: SeriesSpec[] = data
    ? [
        { label: t("contour.mdMean"), color: "var(--viz-series-1)", values: data.md },
        ...(cursor && binColumn ? [{ label: `Bin ${cursor.bin + 1}`, color: "var(--viz-series-2)", values: binColumn }] : []),
      ]
    : [];

  const cell = data && cursor ? { scan: data.scans[cursor.scan], value: data.values[cursor.scan][cursor.bin], mm: data.binCenters[cursor.bin] } : null;
  const dec = data?.property.decimals ?? 2;
  const unit = data?.property.unit ?? "";
  const cellNote = (s: ScanInfo) =>
    s.sheetBreak ? t("contour.sheetBreak") : s.sensorStatus !== "ok" ? t(`contour.sensor.${s.sensorStatus}`) : t("contour.outsideEdges");

  return (
    <>
      <PageHeader
        title={t("contour.title")}
        description={t("contour.description")}
        actions={
          <Button icon="folder" onClick={openFile}>
            {t("contour.openFile")}
          </Button>
        }
      />
      <Card className="mb-4">
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-[12rem_minmax(0,1fr)_minmax(0,1fr)_auto]">
          <MachineSelect machines={machines} value={machineId} onChange={setMachineId} />
          <SelectField label={t("contour.reel")} value={reelPk ?? ""} onChange={(e) => setReelPk(Number(e.target.value) || null)} disabled={reels.length === 0}>
            {reelPk === null && <option value="">{data ? t("contour.fromFile") : t("contour.selectReel")}</option>}
            {reels.map((r) => (
              <option key={r.id} value={r.id}>
                {r.reelId} · {fmtDateTime(r.startedAt)} · {r.scanCount} scans
              </option>
            ))}
          </SelectField>
          <SelectField
            label={t("contour.property")}
            value={code ?? ""}
            onChange={(e) => {
              setPropertyCode(e.target.value);
              localStorage.setItem("contourProperty", e.target.value);
            }}
            disabled={!reel}
          >
            {!reel && <option value="">{data ? `${data.property.code} · ${data.property.name}` : "–"}</option>}
            {reel?.files.map((f) => (
              <option key={f.propertyCode} value={f.propertyCode}>
                {f.propertyCode} · {f.propertyName}
              </option>
            ))}
          </SelectField>
          <div className="flex flex-col gap-1.5">
            <span className="text-sm font-medium" id="color-mode-label">
              {t("contour.colorScale")}
            </span>
            <div role="radiogroup" aria-labelledby="color-mode-label" className="inline-flex h-9 rounded-lg border border-border bg-surface p-0.5">
              {colorModes.map((m) => (
                <button
                  key={m}
                  type="button"
                  role="radio"
                  aria-checked={colorMode === m}
                  onClick={() => {
                    setColorMode(m);
                    localStorage.setItem("contourColorMode", m);
                  }}
                  className={`rounded-md px-2.5 text-xs whitespace-nowrap transition-colors ${colorMode === m ? "bg-bg font-medium text-fg shadow-sm" : "text-muted hover:text-fg"}`}
                >
                  {t(`contour.colorModes.${m}`)}
                </button>
              ))}
            </div>
          </div>
        </div>
      </Card>

      {loading && !data ? (
        <div className="flex justify-center py-16">
          <Spinner />
        </div>
      ) : !data || !colorize ? (
        <EmptyState>{t("contour.empty")}</EmptyState>
      ) : (
        <section className={`rounded-2xl border border-border bg-[var(--viz-surface)] p-4 ${loading ? "opacity-60" : ""}`} aria-busy={loading}>
          <div className="mb-3 flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
            <h2 className="text-base font-semibold">
              {data.property.name}
              {unit && <span className="font-normal text-muted"> ({unit})</span>}
              <span className="font-normal text-muted">
                {" "}
                · {data.machine.name} · {data.reelId}
              </span>
            </h2>
            <p className="max-w-full truncate font-mono text-[11px] text-muted" title={data.filePath}>
              {t("contour.meta", { scans: data.scans.length, bins: data.machine.binCount, width: fmt(data.machine.widthMm, 0), ms: loadMs ?? 0 })} · {data.filePath}
            </p>
          </div>

          <div className="grid gap-x-3" style={{ gridTemplateColumns: `${GUTTER - 12}px minmax(0,1fr) ${MD_WIDTH}px` }}>
            <div />
            <div className="mb-1 text-xs font-medium">{t("contour.map")}</div>
            <div className="mb-1">
              <p className="text-xs font-medium">{t("contour.mdProfile")}</p>
              <SeriesLegend series={mdSeries} />
            </div>

            <div style={{ height: mapHeight }}>
              <Legend data={data} mode={colorMode} palette={palette} first={data.scans[0]} last={data.scans[data.scans.length - 1]} />
            </div>
            <div className="flex flex-col">
              <div style={{ height: mapHeight }}>
                <ContourCanvas
                  values={data.values}
                  binCount={data.machine.binCount}
                  colorize={colorize}
                  palette={palette}
                  cursor={cursor}
                  onCursor={setCursor}
                  label={t("contour.mapLabel", { property: data.property.name, scans: data.scans.length, bins: data.machine.binCount })}
                  tooltip={
                    cell && (
                      <>
                        <p className="font-medium tabular-nums">
                          {cell.value === null ? "–" : `${fmt(cell.value, dec)} ${unit}`}
                        </p>
                        <p className="text-muted">{cell.value === null ? cellNote(cell.scan) : <StatusBadge status={classify(data.property.limits, cell.value)} />}</p>
                      </>
                    )
                  }
                />
              </div>
              <div className="flex items-center gap-x-4 overflow-hidden text-xs whitespace-nowrap tabular-nums text-muted" style={{ height: FOOTER }} aria-live="polite">
                {cell && cursor ? (
                  <>
                    <span className="text-fg">{scanText(cell.scan)}</span>
                    <span>
                      Bin {cursor.bin + 1} · {fmt(cell.mm, 1)} mm
                    </span>
                    <span className="font-medium text-fg">{cell.value === null ? cellNote(cell.scan) : `${fmt(cell.value, dec)} ${unit}`}</span>
                    <span>MD {fmt(cell.scan.mdPositionM / 1000, 2)} km</span>
                    <span>
                      {fmt(cell.scan.speed, 0)} m/min · {fmt(cell.scan.sensorTempC, 1)} °C
                    </span>
                  </>
                ) : (
                  <span>{t("contour.hint")}</span>
                )}
              </div>
            </div>
            <div>
              <MdChart
                series={mdSeries}
                limits={data.property.limits}
                domain={domain}
                decimals={dec}
                plotHeight={mapHeight}
                axisHeight={FOOTER}
                scanLabel={(i) => scanText(data.scans[i])}
                label={t("contour.mdProfile")}
              />
            </div>

            <div className="col-span-2 mt-3">
              <div className="mb-1 flex flex-wrap items-baseline gap-x-4" style={{ paddingLeft: GUTTER }}>
                <p className="text-xs font-medium">{t("contour.cdProfile")}</p>
                <SeriesLegend
                  series={cdSeries}
                  extra={
                    <span className="inline-flex items-center gap-1.5">
                      <span className="h-2.5 w-4 rounded-sm bg-[var(--viz-series-1)] opacity-20" aria-hidden="true" />
                      {t("contour.cdBand")}
                    </span>
                  }
                />
              </div>
              <div>
                <CdChart
                  binCenters={data.binCenters}
                  widthMm={data.machine.widthMm}
                  band={{ min: data.cd.min, max: data.cd.max }}
                  series={cdSeries}
                  limits={data.property.limits}
                  domain={domain}
                  decimals={dec}
                  gutter={GUTTER}
                  height={200}
                  label={t("contour.cdProfile")}
                />
              </div>
            </div>
            <div className="mt-3">
              <StatsPanel data={data} />
            </div>
          </div>
        </section>
      )}
    </>
  );
}
