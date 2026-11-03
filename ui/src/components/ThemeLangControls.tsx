import { useTheme, type ThemeMode } from "../theme/ThemeProvider";
import { languages, useI18n, type Lang } from "../i18n";
import { Icon, type IconName } from "./Icon";
import { controlClass } from "./Field";

const modes: { value: ThemeMode; icon: IconName }[] = [
  { value: "light", icon: "sun" },
  { value: "dark", icon: "moon" },
  { value: "system", icon: "monitor" },
];

export function ThemeToggle() {
  const { mode, setMode } = useTheme();
  const { t } = useI18n();
  return (
    <div role="radiogroup" aria-label={t("theme.label")} className="inline-flex w-fit rounded-lg border border-border bg-surface p-0.5">
      {modes.map((m) => (
        <button
          key={m.value}
          type="button"
          role="radio"
          aria-checked={mode === m.value}
          title={t(`theme.${m.value}`)}
          aria-label={t(`theme.${m.value}`)}
          onClick={() => setMode(m.value)}
          className={`rounded-md p-1.5 ${mode === m.value ? "bg-bg text-fg shadow-sm" : "text-muted hover:text-fg"}`}
        >
          <Icon name={m.icon} />
        </button>
      ))}
    </div>
  );
}

export function LanguageSelect({ className = "" }: { className?: string }) {
  const { lang, setLang, t } = useI18n();
  return (
    <select aria-label={t("language.label")} value={lang} onChange={(e) => setLang(e.target.value as Lang)} className={`${controlClass} ${className}`}>
      {languages.map((l) => (
        <option key={l.value} value={l.value}>
          {l.label}
        </option>
      ))}
    </select>
  );
}
