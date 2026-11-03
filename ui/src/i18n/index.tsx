import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import ptBR from "./pt-BR.json";
import enUS from "./en-US.json";
import { isAppError } from "../lib/api";

export type Lang = "pt-BR" | "en-US";
type Dict = typeof ptBR;
type Vars = Record<string, string | number>;

const dictionaries: Record<Lang, Dict> = { "pt-BR": ptBR, "en-US": enUS };
export const languages: { value: Lang; label: string }[] = [
  { value: "pt-BR", label: "Português (Brasil)" },
  { value: "en-US", label: "English (US)" },
];

function lookup(dict: unknown, key: string): string | undefined {
  const v = key.split(".").reduce<unknown>((o, k) => (o && typeof o === "object" ? (o as Record<string, unknown>)[k] : undefined), dict);
  return typeof v === "string" ? v : undefined;
}

function interpolate(text: string, vars?: Vars) {
  return vars ? text.replace(/\{(\w+)\}/g, (_, k) => String(vars[k] ?? `{${k}}`)) : text;
}

interface I18n {
  lang: Lang;
  setLang: (l: Lang) => void;
  t: (key: string, vars?: Vars) => string;
  /** Traduz um `AppError` do backend (ou qualquer erro) para uma mensagem legível. */
  errorText: (e: unknown) => string;
}

const Ctx = createContext<I18n | null>(null);

function initialLang(): Lang {
  const saved = localStorage.getItem("lang");
  if (saved === "pt-BR" || saved === "en-US") return saved;
  return navigator.language.toLowerCase().startsWith("pt") ? "pt-BR" : "en-US";
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [lang, setLangState] = useState<Lang>(initialLang);

  useEffect(() => {
    document.documentElement.lang = lang;
  }, [lang]);

  const setLang = useCallback((l: Lang) => {
    localStorage.setItem("lang", l);
    setLangState(l);
  }, []);

  const t = useCallback(
    (key: string, vars?: Vars) => interpolate(lookup(dictionaries[lang], key) ?? lookup(ptBR, key) ?? key, vars),
    [lang],
  );

  const errorText = useCallback(
    (e: unknown) => {
      if (isAppError(e)) {
        const field = e.field ? t(`fields.${e.field}`) : "";
        const key = `errors.${e.code}`;
        const text = lookup(dictionaries[lang], key);
        return text ? interpolate(text, { field, message: e.message }) : t("errors.unknown", { message: e.message });
      }
      return t("errors.unknown", { message: String(e) });
    },
    [lang, t],
  );

  const value = useMemo(() => ({ lang, setLang, t, errorText }), [lang, setLang, t, errorText]);
  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useI18n() {
  const ctx = useContext(Ctx);
  if (!ctx) throw new Error("useI18n outside I18nProvider");
  return ctx;
}
