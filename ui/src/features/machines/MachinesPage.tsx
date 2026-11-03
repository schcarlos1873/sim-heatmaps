import { useState, type FormEvent } from "react";
import { api, isAppError, scannerModels, type Machine, type MachineInput, type ScannerModel } from "../../lib/api";
import { fmt, parseNumber } from "../../lib/limits";
import { useI18n } from "../../i18n";
import { useToast } from "../../components/Toast";
import { Button } from "../../components/Button";
import { SelectField, TextField } from "../../components/Field";
import { Modal } from "../../components/Modal";
import { ConfirmDialog } from "../../components/ConfirmDialog";
import { EmptyState, PageHeader, Spinner, td, th } from "../../components/Page";
import { useMachines } from "../useMachines";

const DEFAULT_BINS = 800;

function MachineForm({ machine, onClose, onSaved }: { machine: Machine | null; onClose: () => void; onSaved: () => void }) {
  const { t, errorText } = useI18n();
  const [form, setForm] = useState({
    name: machine?.name ?? "",
    scannerModel: machine?.scannerModel ?? ("NP800" as ScannerModel),
    widthMm: String(machine?.widthMm ?? 8000),
    binCount: String(machine?.binCount ?? DEFAULT_BINS),
    speedMMin: String(machine?.speedMMin ?? 900),
    scanTimeS: String(machine?.scanTimeS ?? 30),
  });
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [saving, setSaving] = useState(false);
  const set = (k: keyof typeof form, v: string) => {
    setForm((f) => ({ ...f, [k]: v }));
    setErrors((e) => ({ ...e, [k]: "" }));
  };

  const width = parseNumber(form.widthMm);
  const bins = parseNumber(form.binCount);
  const speed = parseNumber(form.speedMMin);
  const scanTime = parseNumber(form.scanTimeS);
  const binWidth = width && bins ? width / bins : null;
  const mdPerScan = speed && scanTime ? (speed * scanTime) / 60 : null;

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    const errs: Record<string, string> = {};
    for (const [k, v] of [["widthMm", width], ["speedMMin", speed], ["scanTimeS", scanTime]] as const) {
      if (v === null) errs[k] = t("errors.validation.number", { field: t(`fields.${k}`) });
    }
    setErrors(errs);
    if (Object.keys(errs).length || width === null || speed === null || scanTime === null) return;
    setSaving(true);
    try {
      const input: MachineInput = {
        name: form.name,
        scannerModel: form.scannerModel,
        widthMm: width,
        binCount: bins === null ? null : Math.trunc(bins),
        speedMMin: speed,
        scanTimeS: scanTime,
      };
      if (machine) await api.updateMachine(machine.id, input);
      else await api.createMachine(input);
      onSaved();
    } catch (err) {
      const key = isAppError(err) && err.field && err.field in form ? err.field : "name";
      setErrors({ [key]: errorText(err) });
    } finally {
      setSaving(false);
    }
  };

  return (
    <Modal
      open
      onClose={onClose}
      size="md"
      title={machine ? t("machines.editTitle") : t("machines.newTitle")}
      footer={
        <>
          <Button onClick={onClose}>{t("common.cancel")}</Button>
          <Button variant="primary" type="submit" form="machine-form" loading={saving}>
            {t("common.save")}
          </Button>
        </>
      }
    >
      <form id="machine-form" onSubmit={submit} className="grid grid-cols-1 gap-4 sm:grid-cols-2" noValidate>
        <TextField label={t("machines.name")} value={form.name} onChange={(e) => set("name", e.target.value)} error={errors.name} hint={t("machines.nameHint")} autoFocus maxLength={32} required />
        <SelectField label={t("machines.scannerModel")} value={form.scannerModel} onChange={(e) => set("scannerModel", e.target.value)}>
          {scannerModels.map((m) => (
            <option key={m} value={m}>
              {m}
            </option>
          ))}
        </SelectField>
        <TextField label={t("machines.widthMm")} inputMode="decimal" value={form.widthMm} onChange={(e) => set("widthMm", e.target.value)} error={errors.widthMm} />
        <TextField
          label={t("machines.binCount")}
          type="number"
          min={10}
          max={2000}
          value={form.binCount}
          onChange={(e) => set("binCount", e.target.value)}
          error={errors.binCount}
          hint={binWidth ? t("machines.binWidthHint", { width: fmt(binWidth, 2) }) : t("machines.binCountHint")}
        />
        <TextField label={t("machines.speedMMin")} inputMode="decimal" value={form.speedMMin} onChange={(e) => set("speedMMin", e.target.value)} error={errors.speedMMin} />
        <TextField
          label={t("machines.scanTimeS")}
          inputMode="decimal"
          value={form.scanTimeS}
          onChange={(e) => set("scanTimeS", e.target.value)}
          error={errors.scanTimeS}
          hint={mdPerScan ? t("machines.mdPerScanHint", { length: fmt(mdPerScan, 1) }) : undefined}
        />
        {!machine && <p className="text-xs text-muted sm:col-span-2">{t("machines.defaultPropertiesHint")}</p>}
        {machine && <p className="text-xs text-muted sm:col-span-2">{t("machines.editHint")}</p>}
      </form>
    </Modal>
  );
}

export function MachinesPage() {
  const { t, errorText } = useI18n();
  const toast = useToast();
  const { machines, loading, reload } = useMachines();
  const [editing, setEditing] = useState<Machine | null | undefined>(undefined);
  const [deleting, setDeleting] = useState<Machine | null>(null);

  return (
    <>
      <PageHeader
        title={t("machines.title")}
        description={t("machines.description")}
        actions={
          <Button variant="primary" icon="plus" onClick={() => setEditing(null)}>
            {t("machines.new")}
          </Button>
        }
      />
      {loading ? (
        <div className="flex justify-center py-12">
          <Spinner />
        </div>
      ) : machines.length === 0 ? (
        <EmptyState>{t("machines.empty")}</EmptyState>
      ) : (
        <div className="overflow-auto rounded-2xl border border-border">
          <table className="w-full border-separate border-spacing-0 text-sm">
            <thead>
              <tr>
                <th className={th}>{t("machines.name")}</th>
                <th className={th}>{t("machines.scannerModel")}</th>
                <th className={`${th} text-right`}>{t("machines.widthMm")}</th>
                <th className={`${th} text-right`}>{t("machines.binCount")}</th>
                <th className={`${th} text-right`}>{t("machines.binWidth")}</th>
                <th className={`${th} text-right`}>{t("machines.speedMMin")}</th>
                <th className={`${th} text-right`}>{t("machines.scanTimeS")}</th>
                <th className={`${th} text-right`}>{t("common.actions")}</th>
              </tr>
            </thead>
            <tbody>
              {machines.map((m) => (
                <tr key={m.id} className="hover:bg-surface/60">
                  <td className={`${td} font-medium`}>{m.name}</td>
                  <td className={`${td} font-mono text-xs`}>{m.scannerModel}</td>
                  <td className={`${td} text-right tabular-nums`}>{fmt(m.widthMm, 1)}</td>
                  <td className={`${td} text-right tabular-nums`}>{m.binCount}</td>
                  <td className={`${td} text-right text-muted tabular-nums`}>{fmt(m.widthMm / m.binCount, 2)}</td>
                  <td className={`${td} text-right tabular-nums`}>{fmt(m.speedMMin, 1)}</td>
                  <td className={`${td} text-right tabular-nums`}>{fmt(m.scanTimeS, 1)}</td>
                  <td className={`${td} text-right whitespace-nowrap`}>
                    <Button variant="ghost" iconOnly icon="edit" label={t("common.edit")} onClick={() => setEditing(m)} />
                    <Button variant="ghost" iconOnly icon="trash" label={t("common.delete")} onClick={() => setDeleting(m)} className="text-danger" />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {editing !== undefined && (
        <MachineForm
          machine={editing}
          onClose={() => setEditing(undefined)}
          onSaved={() => {
            setEditing(undefined);
            toast("success", t("common.saved"));
            void reload();
          }}
        />
      )}
      <ConfirmDialog
        open={!!deleting}
        title={t("machines.deleteTitle")}
        message={deleting ? t("machines.deleteMessage", { name: deleting.name }) : ""}
        confirmLabel={t("common.delete")}
        onClose={() => setDeleting(null)}
        onConfirm={async () => {
          if (!deleting) return;
          try {
            await api.deleteMachine(deleting.id);
            toast("success", t("common.deleted"));
            setDeleting(null);
            void reload();
          } catch (e) {
            toast("error", errorText(e));
          }
        }}
      />
    </>
  );
}
