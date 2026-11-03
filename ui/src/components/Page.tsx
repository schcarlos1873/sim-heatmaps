import type { ReactNode } from "react";

export function PageHeader({ title, description, actions }: { title: string; description?: string; actions?: ReactNode }) {
  return (
    <div className="mb-6 flex flex-wrap items-end justify-between gap-4">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">{title}</h1>
        {description && <p className="mt-1 text-sm text-muted">{description}</p>}
      </div>
      {actions && <div className="flex flex-wrap gap-2">{actions}</div>}
    </div>
  );
}

export function Card({ children, className = "" }: { children: ReactNode; className?: string }) {
  return <section className={`rounded-2xl border border-border bg-bg p-4 sm:p-5 ${className}`}>{children}</section>;
}

export function EmptyState({ children }: { children: ReactNode }) {
  return <div className="rounded-2xl border border-dashed border-border px-6 py-14 text-center text-sm text-muted">{children}</div>;
}

export function Spinner() {
  return <span className="inline-block size-5 animate-spin rounded-full border-2 border-muted border-t-transparent" role="progressbar" />;
}

export const th = "sticky top-0 z-10 bg-surface px-3 py-2 text-left text-xs font-semibold tracking-wide text-muted uppercase whitespace-nowrap";
export const td = "border-t border-border px-3 py-2 align-middle";
