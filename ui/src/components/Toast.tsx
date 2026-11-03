import { createContext, useCallback, useContext, useState, type ReactNode } from "react";
import { Icon } from "./Icon";

type Kind = "success" | "error" | "info";
interface Toast {
  id: number;
  kind: Kind;
  text: string;
}

const Ctx = createContext<(kind: Kind, text: string) => void>(() => {});
let seq = 0;

const styles: Record<Kind, string> = {
  success: "border-l-ok-dot",
  error: "border-l-bad-dot",
  info: "border-l-accent",
};

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const dismiss = (id: number) => setToasts((l) => l.filter((x) => x.id !== id));
  const push = useCallback((kind: Kind, text: string) => {
    const id = ++seq;
    setToasts((l) => [...l.slice(-3), { id, kind, text }]);
    setTimeout(() => dismiss(id), kind === "error" ? 9000 : 5000);
  }, []);

  return (
    <Ctx.Provider value={push}>
      {children}
      <div className="pointer-events-none fixed right-4 bottom-4 z-50 flex w-[min(28rem,calc(100%-2rem))] flex-col gap-2" role="status" aria-live="polite">
        {toasts.map((t) => (
          <div key={t.id} className={`pointer-events-auto flex items-start gap-3 rounded-xl border border-border border-l-4 bg-bg px-4 py-3 text-sm shadow-lg ${styles[t.kind]}`}>
            <p className="flex-1 break-all">{t.text}</p>
            <button type="button" onClick={() => dismiss(t.id)} className="text-muted hover:text-fg" aria-label="×">
              <Icon name="close" />
            </button>
          </div>
        ))}
      </div>
    </Ctx.Provider>
  );
}

export const useToast = () => useContext(Ctx);
