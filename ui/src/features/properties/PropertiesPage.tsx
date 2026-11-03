import { useCallback, useEffect, useMemo, useState } from "react";
import { api, type Property } from "../../lib/api";
import { fmt, limitKeys } from "../../lib/limits";
import { useI18n } from "../../i18n";
import { useToast } from "../../components/Toast";
import { Button } from "../../components/Button";
import { TextField } from "../../components/Field";
import { ConfirmDialog } from "../../components/ConfirmDialog";
import { Card, EmptyState, PageHeader, Spinner, td, th } from "../../components/Page";
import { useMachines } from "../useMachines";
import { MachineSelect } from "../MachineSelect";
import { PropertyForm } from "./PropertyForm";
import { LimitBar } from "./LimitBar";

export function PropertiesPage() {
  const { t, errorText } = useI18n();
  const toast = useToast();
  const { machines, machineId, setMachineId, machine } = useMachines();
  const [items, setItems] = useState<Property[]>([]);
  const [loading, setLoading] = useState(false);
  const [query, setQuery] = useState("");
  const [editing, setEditing] = useState<Property | null | undefined>(undefined);
  const [deleting, setDeleting] = useState<Property | null>(null);

  const load = useCallback(async () => {
    if (!machineId) return setItems([]);
    setLoading(true);
    try {
      setItems(await api.listProperties(machineId));
    } catch (e) {
      toast("error", errorText(e));
    } finally {
      setLoading(false);
    }
  }, [machineId, errorText, toast]);

  useEffect(() => {
    void load();
  }, [load]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return q ? items.filter((p) => p.name.toLowerCase().includes(q) || p.code.toLowerCase().includes(q)) : items;
  }, [items, query]);

  const limitHead = (key: string, dot: string) => (
    <th key={key} className={`${th} text-right`}>
      <span className="inline-flex items-center gap-1.5">
        <span className={`size-2 rounded-full ${dot}`} aria-hidden="true" />
        {t(`limits.${key}`)}
      </span>
    </th>
  );

  return (
    <>
      <PageHeader
        title={t("properties.title")}
        description={t("properties.description")}
        actions={
          <Button variant="primary" icon="plus" onClick={() => setEditing(null)} disabled={!machineId}>
            {t("properties.new")}
          </Button>
        }
      />
      <Card className="mb-4">
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-[14rem_1fr]">
          <MachineSelect machines={machines} value={machineId} onChange={setMachineId} />
          <TextField label={t("common.search")} type="search" value={query} onChange={(e) => setQuery(e.target.value)} placeholder={`${t("properties.name")} / ${t("properties.code")}`} />
        </div>
      </Card>

      {loading ? (
        <div className="flex justify-center py-12">
          <Spinner />
        </div>
      ) : items.length === 0 ? (
        <EmptyState>{t("properties.empty")}</EmptyState>
      ) : filtered.length === 0 ? (
        <EmptyState>{t("properties.noMatch")}</EmptyState>
      ) : (
        <>
          <p className="mb-2 text-xs text-muted">{t("properties.count", { count: filtered.length })}</p>
          <div className="overflow-auto rounded-2xl border border-border" style={{ maxHeight: "calc(100vh - 19rem)" }}>
            <table className="w-full border-separate border-spacing-0 text-sm">
              <thead>
                <tr>
                  <th className={th}>#</th>
                  <th className={th}>{t("properties.code")}</th>
                  <th className={`${th} min-w-52`}>{t("properties.name")}</th>
                  {limitKeys.map(({ key, dot }) => limitHead(key, dot))}
                  <th className={`${th} min-w-28`} aria-label={t("limits.title")} />
                  <th className={th}>{t("properties.active")}</th>
                  <th className={`${th} text-right`}>{t("common.actions")}</th>
                </tr>
              </thead>
              <tbody>
                {filtered.map((p) => {
                  const d = Math.max(p.decimals, 2);
                  return (
                    <tr key={p.id} className={`hover:bg-surface/60 ${p.active ? "" : "opacity-60"}`}>
                      <td className={`${td} text-muted tabular-nums`}>{p.sortOrder}</td>
                      <td className={`${td} font-mono text-xs`}>{p.code}</td>
                      <td className={td}>
                        {p.name}
                        {p.unit && <span className="ml-1 text-xs text-muted">({p.unit})</span>}
                      </td>
                      {limitKeys.map(({ key: k }) => (
                        <td key={k} className={`${td} text-right tabular-nums ${k === "target" ? "font-medium" : ""}`}>
                          {fmt(p.limits[k], d)}
                        </td>
                      ))}
                      <td className={td}>
                        <LimitBar limits={p.limits} />
                      </td>
                      <td className={`${td} text-xs`}>{p.active ? t("common.yes") : t("common.no")}</td>
                      <td className={`${td} text-right whitespace-nowrap`}>
                        <Button variant="ghost" iconOnly icon="edit" label={t("common.edit")} onClick={() => setEditing(p)} />
                        <Button variant="ghost" iconOnly icon="trash" label={t("common.delete")} onClick={() => setDeleting(p)} className="text-danger" />
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </>
      )}

      {editing !== undefined && machine && (
        <PropertyForm
          key={editing?.id ?? "new"}
          open
          machine={machine}
          property={editing}
          onClose={() => setEditing(undefined)}
          onSaved={() => {
            setEditing(undefined);
            toast("success", t("common.saved"));
            void load();
          }}
        />
      )}
      <ConfirmDialog
        open={!!deleting}
        title={t("properties.deleteTitle")}
        message={deleting ? t("properties.deleteMessage", { name: deleting.name, code: deleting.code }) : ""}
        confirmLabel={t("common.delete")}
        onClose={() => setDeleting(null)}
        onConfirm={async () => {
          if (!deleting) return;
          try {
            await api.deleteProperty(deleting.id);
            toast("success", t("common.deleted"));
            setDeleting(null);
            void load();
          } catch (e) {
            toast("error", errorText(e));
          }
        }}
      />
    </>
  );
}
