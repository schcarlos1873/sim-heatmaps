import { useCallback, useEffect, useState } from "react";
import { api, type Machine } from "../lib/api";
import { useI18n } from "../i18n";
import { useToast } from "../components/Toast";

/** Lista de máquinas + máquina selecionada (persistida entre telas). */
export function useMachines() {
  const { errorText } = useI18n();
  const toast = useToast();
  const [machines, setMachines] = useState<Machine[]>([]);
  const [loading, setLoading] = useState(true);
  const [machineId, setMachineIdState] = useState<number | null>(() => Number(localStorage.getItem("machineId")) || null);

  const setMachineId = useCallback((id: number | null) => {
    localStorage.setItem("machineId", id ? String(id) : "");
    setMachineIdState(id);
  }, []);

  const reload = useCallback(async () => {
    setLoading(true);
    try {
      const list = await api.listMachines();
      setMachines(list);
      setMachineIdState((cur) => (cur && list.some((m) => m.id === cur) ? cur : (list[0]?.id ?? null)));
    } catch (e) {
      toast("error", errorText(e));
    } finally {
      setLoading(false);
    }
  }, [errorText, toast]);

  useEffect(() => {
    void reload();
  }, [reload]);

  return { machines, loading, machineId, setMachineId, machine: machines.find((m) => m.id === machineId) ?? null, reload };
}
