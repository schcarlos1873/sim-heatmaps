import { useState, type FormEvent } from "react";
import { api, isAppError, type Limits, type Machine, type Property, type PropertyInput } from "../../lib/api";
import { limitKeys, parseNumber } from "../../lib/limits";
import { useI18n } from "../../i18n";
import { Modal } from "../../components/Modal";
import { Button } from "../../components/Button";
import { Checkbox, TextField } from "../../components/Field";
import { LimitBar } from "./LimitBar";

type FormState = Record<"code" | "name" | "unit" | "decimals" | "sortOrder" | keyof Limits, string> & { active: boolean };

function toForm(p: Property | null): FormState {
  const s = (v: number | undefined | null) => (v === undefined || v === null ? "" : String(v));
  return {
    code: p?.code ?? "",
    name: p?.name ?? "",
    unit: p?.unit ?? "",
    decimals: s(p?.decimals ?? 2),
    sortOrder: s(p?.sortOrder),
    active: p?.active ?? true,
    errorLow: s(p?.limits.errorLow),
    warningLow: s(p?.limits.warningLow),
    target: s(p?.limits.target),
    warningHigh: s(p?.limits.warningHigh),
    errorHigh: s(p?.limits.errorHigh),
  };
}

interface Props {
  machine: Machine;
  property: Property | null;
  open: boolean;
  onClose: () => void;
  onSaved: (p: Property) => void;
}

export function PropertyForm({ machine, property, open, onClose, onSaved }: Props) {
  const { t, errorText } = useI18n();
  const [form, setForm] = useState<FormState>(() => toForm(property));
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [general, setGeneral] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const set = <K extends keyof FormState>(k: K, v: FormState[K]) => {
    setForm((f) => ({ ...f, [k]: v }));
    setErrors((e) => ({ ...e, [k]: "" }));
  };

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    const errs: Record<string, string> = {};
    const required = (k: keyof FormState) => {
      const v = parseNumber(form[k] as string);
      if (v === null) errs[k] = t("errors.validation.number", { field: t(`fields.${k}`) });
      return v ?? 0;
    };
    const limits: Limits = {
      errorLow: required("errorLow"),
      warningLow: required("warningLow"),
      target: required("target"),
      warningHigh: required("warningHigh"),
      errorHigh: required("errorHigh"),
    };
    const decimals = required("decimals");
    if (!form.code.trim()) errs.code = t("errors.validation.required", { field: t("fields.code") });
    if (!form.name.trim()) errs.name = t("errors.validation.required", { field: t("fields.name") });
    setErrors(errs);
    if (Object.keys(errs).length) return;

    const input: PropertyInput = {
      machineId: machine.id,
      code: form.code,
      name: form.name,
      unit: form.unit,
      decimals,
      sortOrder: parseNumber(form.sortOrder),
      active: form.active,
      limits,
    };
    setSaving(true);
    setGeneral(null);
    try {
      const saved = property ? await api.updateProperty(property.id, input) : await api.createProperty(input);
      onSaved(saved);
    } catch (err) {
      if (isAppError(err) && err.field && err.field in form) setErrors({ [err.field]: errorText(err) });
      else setGeneral(errorText(err));
    } finally {
      setSaving(false);
    }
  };

  const preview = Object.fromEntries(limitKeys.map(({ key }) => [key, parseNumber(form[key])])) as Record<keyof Limits, number | null>;

  return (
    <Modal
      open={open}
      onClose={onClose}
      size="lg"
      title={property ? t("properties.editTitle") : t("properties.newTitle")}
      footer={
        <>
          <Button onClick={onClose}>{t("common.cancel")}</Button>
          <Button variant="primary" type="submit" form="property-form" loading={saving}>
            {t("common.save")}
          </Button>
        </>
      }
    >
      <form id="property-form" onSubmit={submit} className="flex flex-col gap-5" noValidate>
        {general && (
          <p className="rounded-lg bg-bad-bg px-3 py-2 text-sm text-bad-fg" role="alert">
            {general}
          </p>
        )}
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
          <TextField
            label={t("properties.code")}
            value={form.code}
            onChange={(e) => set("code", e.target.value.toUpperCase())}
            error={errors.code}
            hint={t("properties.codeHint")}
            className="font-mono"
            maxLength={16}
            autoFocus
            required
          />
          <TextField label={t("properties.name")} value={form.name} onChange={(e) => set("name", e.target.value)} error={errors.name} wrapperClass="sm:col-span-2" maxLength={64} required />
          <TextField label={t("properties.unit")} value={form.unit} onChange={(e) => set("unit", e.target.value)} error={errors.unit} maxLength={16} />
          <TextField label={t("properties.decimals")} type="number" min={0} max={6} value={form.decimals} onChange={(e) => set("decimals", e.target.value)} error={errors.decimals} />
          <TextField label={t("properties.sortOrder")} inputMode="numeric" value={form.sortOrder} onChange={(e) => set("sortOrder", e.target.value)} hint={t("properties.sortOrderHint")} />
          <div className="sm:col-span-3">
            <Checkbox label={t("properties.active")} checked={form.active} onChange={(e) => set("active", e.target.checked)} />
          </div>
        </div>

        <fieldset className="rounded-xl border border-border p-4">
          <legend className="px-1 text-sm font-semibold">{t("limits.title")}</legend>
          <p className="mb-3 text-xs text-muted">{t("limits.order")}</p>
          <div className="grid grid-cols-2 gap-3 sm:grid-cols-5">
            {limitKeys.map(({ key, dot }) => (
              <TextField
                key={key}
                label={
                  <span className="inline-flex items-center gap-1.5">
                    <span className={`size-2.5 rounded-full ${dot}`} aria-hidden="true" />
                    {t(`limits.${key}`)}
                  </span>
                }
                inputMode="decimal"
                value={form[key]}
                onChange={(e) => set(key, e.target.value)}
                error={errors[key]}
                className="text-right tabular-nums"
              />
            ))}
          </div>
          <div className="mt-4">
            <LimitBar limits={preview} />
          </div>
        </fieldset>
      </form>
    </Modal>
  );
}
