import { useId, type InputHTMLAttributes, type ReactNode, type SelectHTMLAttributes } from "react";

export const controlClass =
  "h-9 w-full rounded-lg border border-control/60 bg-bg px-3 text-sm text-fg placeholder:text-muted hover:border-control focus:border-accent focus:outline-none focus-visible:outline-2 focus-visible:outline-accent aria-[invalid=true]:border-danger disabled:opacity-50";

interface FieldProps {
  label: ReactNode;
  hint?: ReactNode;
  error?: string | null;
  className?: string;
  children: (id: string, describedBy?: string) => ReactNode;
}

export function Field({ label, hint, error, className = "", children }: FieldProps) {
  const id = useId();
  const describedBy = error ? `${id}-err` : hint ? `${id}-hint` : undefined;
  return (
    <div className={`flex flex-col gap-1.5 ${className}`}>
      <label htmlFor={id} className="text-sm font-medium text-fg">
        {label}
      </label>
      {children(id, describedBy)}
      {error ? (
        <p id={`${id}-err`} className="text-xs text-danger" role="alert">
          {error}
        </p>
      ) : (
        hint && (
          <p id={`${id}-hint`} className="text-xs text-muted">
            {hint}
          </p>
        )
      )}
    </div>
  );
}

type InputProps = InputHTMLAttributes<HTMLInputElement> & { label: ReactNode; hint?: ReactNode; error?: string | null; wrapperClass?: string };

export function TextField({ label, hint, error, wrapperClass, className = "", ...rest }: InputProps) {
  return (
    <Field label={label} hint={hint} error={error} className={wrapperClass}>
      {(id, describedBy) => (
        <input id={id} aria-describedby={describedBy} aria-invalid={!!error} className={`${controlClass} ${className}`} {...rest} />
      )}
    </Field>
  );
}

type SelectProps = SelectHTMLAttributes<HTMLSelectElement> & { label: ReactNode; wrapperClass?: string };

export function SelectField({ label, wrapperClass, className = "", children, ...rest }: SelectProps) {
  return (
    <Field label={label} className={wrapperClass}>
      {(id) => (
        <select id={id} className={`${controlClass} ${className}`} {...rest}>
          {children}
        </select>
      )}
    </Field>
  );
}

export function Checkbox({ label, ...rest }: InputHTMLAttributes<HTMLInputElement> & { label: ReactNode }) {
  return (
    <label className="inline-flex cursor-pointer items-center gap-2 text-sm">
      <input type="checkbox" className="size-4 accent-[var(--accent)]" {...rest} />
      {label}
    </label>
  );
}
