import type { ButtonHTMLAttributes } from "react";
import { Icon, type IconName } from "./Icon";

type Variant = "primary" | "secondary" | "ghost" | "danger";

const variants: Record<Variant, string> = {
  primary: "bg-accent text-on-accent hover:bg-accent-hover",
  secondary: "bg-surface text-fg border border-border hover:bg-surface-hover",
  ghost: "text-fg hover:bg-surface-hover",
  danger: "bg-danger text-on-danger hover:opacity-90",
};

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  icon?: IconName;
  loading?: boolean;
  /** Botão apenas com ícone: `label` vira aria-label/title. */
  iconOnly?: boolean;
  label?: string;
}

export function Button({ variant = "secondary", icon, loading, iconOnly, label, className = "", children, disabled, ...rest }: Props) {
  return (
    <button
      type="button"
      {...rest}
      disabled={disabled || loading}
      aria-label={iconOnly ? label : undefined}
      title={iconOnly ? label : rest.title}
      aria-busy={loading || undefined}
      className={`inline-flex items-center justify-center gap-2 rounded-lg text-sm font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
        iconOnly ? "size-9" : "h-9 px-3.5"
      } ${variants[variant]} ${className}`}
    >
      {loading ? <span className="size-4 animate-spin rounded-full border-2 border-current border-t-transparent" /> : icon && <Icon name={icon} />}
      {!iconOnly && (children ?? label)}
    </button>
  );
}
