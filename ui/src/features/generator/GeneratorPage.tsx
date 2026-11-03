import { useCallback, useEffect, useState, type FormEvent } from "react";
import { api, defaultGeneratorParams, isAppError, type GenerateRequest, type GeneratorParams, type Machine, type Property, type ReelRecord } from "../../lib/api";
import { fmt, fmtDateTime, nowLocalInput, parseNumber } from "../../lib/limits";
import { useI18n } from "../../i18n";
import { useToast } from "../../components/Toast";
import { Button } from "../../components/Button";
import { Checkbox, TextField } from "../../components/Field";
import { Modal } from "../../components/Modal";
import { Card, EmptyState, PageHeader, Spinner, td, th } from "../../components/Page";
import { useMachines } from "../useMachines";
import { MachineSelect } from "../MachineSelect";

const MAX_SCANS = 2880;

/** Parâmetros exibidos no formulário: amplitudes em fração da meia faixa de Warning, probabilidades em %. */
const paramFields: { key: keyof GeneratorParams; percent?: boolean; integer?: boolean }[] = [
  { key: "cdAmplitude" },
  { key: "mdAmplitude" },
  { key: "noise" },
  { key: "edgeAmplitude" },
  { key: "streakCount", integer: true },
  { key: "sheetBreakProbability", percent: true },
  { key: "sensorFaultProbability", percent: true },
];

function paramsToForm(p: GeneratorParams): Record<keyof GeneratorParams, string> {
  return Object.fromEntries(paramFields.map(({ key, percent }) => [key, String(percent ? +(p[key] * 100).toFixed(4) : p[key])])) as Record<keyof GeneratorParams, string>;
}

function suggestReelId(machine: Machine | null, startedAt: string): string {
  const stamp = startedAt.replace(/[-:]/g, "").replace("T", "-").slice(0, 13);
  return machine ? `${machine.name}-${stamp}` : "";
}

export function GeneratorPage({ onOpenContour }: { onOpenContour: (reel: ReelRecord) => void }) {
  const { t, errorText } = useI18n();
  const toast = useToast();
  const { machines, machineId, setMachineId, machine } = useMachines();
  const [properties, setProperties] = useState<Property[]>([]);
  const [selected, setSelected] = useState<Set<number>>(new Set());
  const [startedAt, setStartedAt] = useState(nowLocalInput);
  const [reelId, setReelId] = useState("");
  const [reelIdTouched, setReelIdTouched] = useState(false);
  const [duration, setDuration] = useState("60");
  const [params, setParams] = useState(() => paramsToForm(defaultGeneratorParams));
  const [seed, setSeed] = useState("");
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [generating, setGenerating] = useState(false);
  const [confirmOverwrite, setConfirmOverwrite] = useState<GenerateRequest | null>(null);
  const [reels, setReels] = useState<ReelRecord[]>([]);
  const [loadingReels, setLoadingReels] = useState(true);
  const [deleting, setDeleting] = useState<ReelRecord | null>(null);
  const [deleteFiles, setDeleteFiles] = useState(false);

  useEffect(() => {
    if (!reelIdTouched) setReelId(suggestReelId(machine, startedAt));
  }, [machine, startedAt, reelIdTouched]);

  useEffect(() => {
    if (!machineId) return setProperties([]);
    api
      .listProperties(machineId)
      .then((list) => {
        setProperties(list);
        setSelected(new Set(list.filter((p) => p.active).map((p) => p.id)));
      })
      .catch((e) => toast("error", errorText(e)));
  }, [machineId, errorText, toast]);

  const loadReels = useCallback(async () => {
    setLoadingReels(true);
    try {
      setReels(await api.listReels(null, 500));
    } catch (e) {
      toast("error", errorText(e));
    } finally {
      setLoadingReels(false);
    }
  }, [errorText, toast]);

  useEffect(() => {
    void loadReels();
  }, [loadReels]);

  const durationMin = parseNumber(duration);
  const scans = machine && durationMin && durationMin > 0 ? Math.ceil((durationMin * 60) / machine.scanTimeS) : null;

  const run = async (request: GenerateRequest) => {
    setGenerating(true);
    try {
      const reel = await api.generateReel(request);
      toast("success", t("generator.generated", { files: reel.files.length, scans: reel.scanCount, path: reel.directory }));
      setConfirmOverwrite(null);
      setReelIdTouched(false);
      void loadReels();
    } catch (err) {
      if (isAppError(err) && err.code === "conflict.reelExists" && !request.overwrite) setConfirmOverwrite(request);
      else if (isAppError(err) && err.field && err.field in errorsFields) setErrors({ [errorsFields[err.field]]: errorText(err) });
      else toast("error", errorText(err));
    } finally {
      setGenerating(false);
    }
  };

  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (!machine) return;
    const errs: Record<string, string> = {};
    if (!reelId.trim()) errs.reelId = t("errors.validation.required", { field: t("fields.reelId") });
    if (scans === null) errs.duration = t("errors.validation.positive", { field: t("fields.durationMin") });
    else if (scans > MAX_SCANS) errs.duration = t("generator.tooManyScans", { max: MAX_SCANS });
    if (selected.size === 0) errs.properties = t("errors.validation.required", { field: t("fields.properties") });
    const parsed = {} as GeneratorParams;
    for (const { key, percent, integer } of paramFields) {
      const v = parseNumber(params[key]);
      if (v === null || v < 0 || (integer && !Number.isInteger(v))) errs[key] = t("errors.validation.number", { field: t(`fields.${key}`) });
      else parsed[key] = percent ? v / 100 : v;
    }
    const seedValue = seed.trim() === "" ? null : parseNumber(seed);
    if (seed.trim() !== "" && (seedValue === null || !Number.isSafeInteger(seedValue) || seedValue < 0)) errs.seed = t("errors.validation.number", { field: t("fields.seed") });
    setErrors(errs);
    if (Object.keys(errs).length) return;
    void run({
      machineId: machine.id,
      reelId: reelId.trim(),
      startedAt,
      scanCount: scans,
      durationMin: null,
      propertyIds: [...selected],
      params: parsed,
      seed: seedValue,
      overwrite: false,
    });
  };

  const toggle = (id: number) =>
    setSelected((s) => {
      const n = new Set(s);
      if (n.has(id)) n.delete(id);
      else n.add(id);
      return n;
    });

  return (
    <>
      <PageHeader title={t("generator.title")} description={t("generator.description")} />
      <form onSubmit={submit} noValidate className="mb-6 flex flex-col gap-4">
        <Card>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <MachineSelect machines={machines} value={machineId} onChange={setMachineId} />
            <TextField
              label={t("generator.reelId")}
              value={reelId}
              onChange={(e) => {
                setReelId(e.target.value);
                setReelIdTouched(true);
                setErrors((x) => ({ ...x, reelId: "" }));
              }}
              error={errors.reelId}
              hint={t("generator.reelIdHint")}
              className="font-mono"
              maxLength={64}
            />
            <TextField label={t("generator.startedAt")} type="datetime-local" value={startedAt} onChange={(e) => setStartedAt(e.target.value)} />
            <TextField
              label={t("generator.durationMin")}
              inputMode="decimal"
              value={duration}
              onChange={(e) => {
                setDuration(e.target.value);
                setErrors((x) => ({ ...x, duration: "" }));
              }}
              error={errors.duration}
              hint={machine && scans ? t("generator.scansHint", { scans, length: fmt((scans * machine.speedMMin * machine.scanTimeS) / 60 / 1000, 1) }) : undefined}
            />
          </div>
          {machine && <p className="mt-3 text-xs text-muted">{t("generator.machineInfo", { model: machine.scannerModel, width: fmt(machine.widthMm, 0), bins: machine.binCount, binWidth: fmt(machine.widthMm / machine.binCount, 2), speed: fmt(machine.speedMMin, 0), scanTime: fmt(machine.scanTimeS, 1) })}</p>}
        </Card>

        <Card>
          <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
            <h2 className="text-base font-semibold">{t("generator.properties")}</h2>
            <div className="flex gap-2">
              <Button variant="ghost" onClick={() => setSelected(new Set(properties.map((p) => p.id)))}>
                {t("generator.selectAll")}
              </Button>
              <Button variant="ghost" onClick={() => setSelected(new Set())}>
                {t("generator.selectNone")}
              </Button>
            </div>
          </div>
          {properties.length === 0 ? (
            <p className="text-sm text-muted">{t("generator.noProperties")}</p>
          ) : (
            <div className="grid grid-cols-1 gap-2 sm:grid-cols-2 lg:grid-cols-3">
              {properties.map((p) => (
                <Checkbox
                  key={p.id}
                  checked={selected.has(p.id)}
                  onChange={() => toggle(p.id)}
                  label={
                    <span>
                      <span className="font-mono text-xs text-muted">{p.code}</span> {p.name}
                      {p.unit && <span className="text-xs text-muted"> ({p.unit})</span>}
                    </span>
                  }
                />
              ))}
            </div>
          )}
          {errors.properties && (
            <p className="mt-2 text-xs text-danger" role="alert">
              {errors.properties}
            </p>
          )}
        </Card>

        <Card>
          <details>
            <summary className="cursor-pointer text-base font-semibold">{t("generator.advanced")}</summary>
            <p className="mt-2 mb-4 text-xs text-muted">{t("generator.advancedHint")}</p>
            <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
              {paramFields.map(({ key }) => (
                <TextField
                  key={key}
                  label={t(`generator.params.${key}`)}
                  inputMode="decimal"
                  value={params[key]}
                  onChange={(e) => setParams((p) => ({ ...p, [key]: e.target.value }))}
                  error={errors[key]}
                  className="text-right tabular-nums"
                />
              ))}
              <TextField label={t("generator.seed")} inputMode="numeric" value={seed} onChange={(e) => setSeed(e.target.value)} error={errors.seed} hint={t("generator.seedHint")} />
            </div>
            <div className="mt-4">
              <Button variant="ghost" icon="refresh" onClick={() => setParams(paramsToForm(defaultGeneratorParams))}>
                {t("generator.resetParams")}
              </Button>
            </div>
          </details>
        </Card>

        <div>
          <Button type="submit" variant="primary" icon="dice" loading={generating} disabled={!machine}>
            {generating ? t("generator.generating") : t("generator.generate")}
          </Button>
        </div>
      </form>

      <div className="mb-3 flex items-center justify-between gap-2">
        <h2 className="text-lg font-semibold">{t("generator.reels")}</h2>
        <Button icon="refresh" onClick={loadReels}>
          {t("common.refresh")}
        </Button>
      </div>
      {loadingReels ? (
        <div className="flex justify-center py-12">
          <Spinner />
        </div>
      ) : reels.length === 0 ? (
        <EmptyState>{t("generator.noReels")}</EmptyState>
      ) : (
        <div className="overflow-auto rounded-2xl border border-border">
          <table className="w-full border-separate border-spacing-0 text-sm">
            <thead>
              <tr>
                <th className={th}>{t("generator.reelId")}</th>
                <th className={th}>{t("common.machine")}</th>
                <th className={th}>{t("generator.start")}</th>
                <th className={th}>{t("generator.end")}</th>
                <th className={`${th} text-right`}>{t("generator.scans")}</th>
                <th className={`${th} text-right`}>{t("generator.bins")}</th>
                <th className={`${th} text-right`}>{t("generator.files")}</th>
                <th className={th}>{t("generator.directory")}</th>
                <th className={`${th} text-right`}>{t("common.actions")}</th>
              </tr>
            </thead>
            <tbody>
              {reels.map((r) => (
                <tr key={r.id} className="hover:bg-surface/60">
                  <td className={`${td} font-mono text-xs font-medium`}>{r.reelId}</td>
                  <td className={td}>{r.machineName}</td>
                  <td className={`${td} tabular-nums whitespace-nowrap`}>{fmtDateTime(r.startedAt)}</td>
                  <td className={`${td} tabular-nums whitespace-nowrap text-muted`}>{fmtDateTime(r.endedAt)}</td>
                  <td className={`${td} text-right tabular-nums`}>{r.scanCount}</td>
                  <td className={`${td} text-right tabular-nums`}>{r.binCount}</td>
                  <td className={`${td} text-right tabular-nums`} title={r.files.map((f) => f.propertyCode).join(", ")}>
                    {r.files.length}
                  </td>
                  <td className={`${td} max-w-72 truncate font-mono text-xs text-muted`} title={r.directory}>
                    {r.directory}
                  </td>
                  <td className={`${td} text-right whitespace-nowrap`}>
                    <Button variant="ghost" iconOnly icon="grid" label={t("generator.openContour")} onClick={() => onOpenContour(r)} />
                    <Button
                      variant="ghost"
                      iconOnly
                      icon="trash"
                      label={t("common.delete")}
                      onClick={() => {
                        setDeleteFiles(false);
                        setDeleting(r);
                      }}
                      className="text-danger"
                    />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <Modal
        open={!!confirmOverwrite}
        size="sm"
        title={t("generator.overwriteTitle")}
        onClose={() => setConfirmOverwrite(null)}
        footer={
          <>
            <Button onClick={() => setConfirmOverwrite(null)}>{t("common.cancel")}</Button>
            <Button variant="danger" data-autofocus loading={generating} onClick={() => confirmOverwrite && run({ ...confirmOverwrite, overwrite: true })}>
              {t("generator.overwrite")}
            </Button>
          </>
        }
      >
        <p className="text-sm text-muted">{t("generator.overwriteMessage", { id: confirmOverwrite?.reelId ?? "" })}</p>
      </Modal>

      <Modal
        open={!!deleting}
        size="sm"
        title={t("generator.deleteTitle")}
        onClose={() => setDeleting(null)}
        footer={
          <>
            <Button onClick={() => setDeleting(null)}>{t("common.cancel")}</Button>
            <Button
              variant="danger"
              data-autofocus
              onClick={async () => {
                if (!deleting) return;
                try {
                  await api.deleteReel(deleting.id, deleteFiles);
                  toast("success", t("common.deleted"));
                  setDeleting(null);
                  void loadReels();
                } catch (e) {
                  toast("error", errorText(e));
                }
              }}
            >
              {t("common.delete")}
            </Button>
          </>
        }
      >
        <div className="flex flex-col gap-3">
          <p className="text-sm text-muted">{t("generator.deleteMessage", { id: deleting?.reelId ?? "" })}</p>
          <Checkbox label={t("generator.deleteFiles", { count: deleting?.files.length ?? 0 })} checked={deleteFiles} onChange={(e) => setDeleteFiles(e.target.checked)} />
        </div>
      </Modal>
    </>
  );
}

/** Campos do backend → campos do formulário. */
const errorsFields: Record<string, string> = {
  reelId: "reelId",
  startedAt: "duration",
  scanCount: "duration",
  durationMin: "duration",
  cdAmplitude: "cdAmplitude",
  mdAmplitude: "mdAmplitude",
  noise: "noise",
  edgeAmplitude: "edgeAmplitude",
  streakCount: "streakCount",
  sheetBreakProbability: "sheetBreakProbability",
  sensorFaultProbability: "sensorFaultProbability",
};
