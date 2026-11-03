import type { Machine } from "../lib/api";
import { SelectField } from "../components/Field";
import { useI18n } from "../i18n";

export function MachineSelect({ machines, value, onChange, className = "" }: { machines: Machine[]; value: number | null; onChange: (id: number) => void; className?: string }) {
  const { t } = useI18n();
  return (
    <SelectField label={t("common.machine")} value={value ?? ""} onChange={(e) => onChange(Number(e.target.value))} wrapperClass={className} disabled={machines.length === 0}>
      {machines.length === 0 && <option value="">{t("common.selectMachine")}</option>}
      {machines.map((m) => (
        <option key={m.id} value={m.id}>
          {m.name}
        </option>
      ))}
    </SelectField>
  );
}
