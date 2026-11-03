import { useEffect, useRef, type ReactNode } from "react";
import { Icon } from "./Icon";
import { useI18n } from "../i18n";

interface Props {
  open: boolean;
  title: string;
  onClose: () => void;
  children: ReactNode;
  footer?: ReactNode;
  size?: "sm" | "md" | "lg";
}

const widths = { sm: "max-w-md", md: "max-w-xl", lg: "max-w-3xl" };

/** Modal baseado em <dialog> nativo: foco preso, Esc fecha, backdrop. */
export function Modal({ open, title, onClose, children, footer, size = "md" }: Props) {
  const ref = useRef<HTMLDialogElement>(null);
  const { t } = useI18n();

  useEffect(() => {
    const d = ref.current;
    if (!d) return;
    if (open && !d.open) {
      d.showModal();
      // <dialog> foca o primeiro elemento (o botão fechar) e ignora o autoFocus do React:
      // foca o elemento marcado com data-autofocus ou o primeiro campo do conteúdo.
      d.querySelector<HTMLElement>("[data-autofocus], .modal-body input, .modal-body select, .modal-body textarea")?.focus();
    }
    if (!open && d.open) d.close();
  }, [open]);

  return (
    <dialog
      ref={ref}
      onCancel={(e) => {
        e.preventDefault();
        onClose();
      }}
      aria-labelledby="modal-title"
      className={`m-auto w-[calc(100%-2rem)] ${widths[size]} rounded-2xl border border-border bg-bg p-0 text-fg shadow-2xl`}
    >
      {open && (
        <div className="flex max-h-[85vh] flex-col">
          <header className="flex items-center justify-between gap-4 border-b border-border px-5 py-4">
            <h2 id="modal-title" className="text-base font-semibold">
              {title}
            </h2>
            <button type="button" onClick={onClose} aria-label={t("common.close")} className="rounded-lg p-1.5 text-muted hover:bg-surface-hover hover:text-fg">
              <Icon name="close" />
            </button>
          </header>
          <div className="modal-body overflow-y-auto px-5 py-4">{children}</div>
          {footer && <footer className="flex justify-end gap-2 border-t border-border px-5 py-3">{footer}</footer>}
        </div>
      )}
    </dialog>
  );
}
