import { useEffect, useState, type FormEvent } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { api, defaultContourColors, isAppError, type AppInfo, type ContourColors, type Settings } from "../../lib/api";
import { useI18n } from "../../i18n";
import { useToast } from "../../components/Toast";
import { Button } from "../../components/Button";
import { Field, controlClass } from "../../components/Field";
import { Card, PageHeader, Spinner } from "../../components/Page";
import { LanguageSelect, ThemeToggle } from "../../components/ThemeLangControls";
import { buildPalette, COLOR_KEYS, gradientCss, type ColorKey } from "../contour/colors";

const HEX = /^#[0-9a-f]{6}$/i;

function ColorRow({ name, value, onChange }: { name: ColorKey; value: string; onChange: (v: string) => void }) {
  const { t } = useI18n();
  const valid = HEX.test(value);
  return (
    <div className="grid grid-cols-[8rem_auto_7rem] items-center gap-3">
      <label htmlFor={`color-${name}`} className="text-sm">
        {t(`limits.${name}`)}
      </label>
      <input
        type="color"
        aria-label={t(`limits.${name}`)}
        className="h-9 w-12 cursor-pointer rounded-md border border-border bg-surface p-0.5"
        value={valid ? value.toLowerCase() : "#000000"}
        onChange={(e) => onChange(e.target.value)}
      />
      <input
        id={`color-${name}`}
        aria-invalid={!valid}
        className={`${controlClass} font-mono text-xs`}
        value={value}
        maxLength={7}
        spellCheck={false}
        onChange={(e) => onChange(e.target.value)}
      />
    </div>
  );
}

export function SettingsPage() {
  const { t, errorText } = useI18n();
  const toast = useToast();
  const [settings, setSettings] = useState<Settings | null>(null);
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    Promise.all([api.getSettings(), api.appInfo()])
      .then(([s, i]) => {
        setSettings(s);
        setInfo(i);
      })
      .catch((e) => toast("error", errorText(e)));
  }, [errorText, toast]);

  const browse = async () => {
    const dir = await openDialog({ directory: true, defaultPath: settings?.exportDir });
    if (typeof dir === "string" && settings) setSettings({ ...settings, exportDir: dir });
  };

  const setColor = (key: ColorKey, value: string) =>
    settings && setSettings({ ...settings, contourColors: { ...settings.contourColors, [key]: value } });
  const colors: ContourColors = settings?.contourColors ?? defaultContourColors;
  const colorsValid = COLOR_KEYS.every((k) => HEX.test(colors[k]));

  const save = async (e: FormEvent) => {
    e.preventDefault();
    if (!settings) return;
    setSaving(true);
    setErrors({});
    try {
      setSettings(await api.updateSettings(settings));
      toast("success", t("common.saved"));
    } catch (err) {
      if (isAppError(err) && err.field) setErrors({ [err.field]: errorText(err) });
      else toast("error", errorText(err));
    } finally {
      setSaving(false);
    }
  };

  if (!settings) {
    return (
      <div className="flex justify-center py-12">
        <Spinner />
      </div>
    );
  }

  return (
    <div className="max-w-3xl">
      <PageHeader title={t("settings.title")} description={t("settings.description")} />
      <div className="flex flex-col gap-4">
        <Card>
          <h2 className="mb-4 text-base font-semibold">{t("settings.export")}</h2>
          <form onSubmit={save} className="flex flex-col gap-4">
            <Field label={t("settings.exportDir")} hint={t("settings.exportDirHint")} error={errors.exportDir}>
              {(id, describedBy) => (
                <div className="flex gap-2">
                  <input
                    id={id}
                    aria-describedby={describedBy}
                    aria-invalid={!!errors.exportDir}
                    className={`${controlClass} font-mono text-xs`}
                    value={settings.exportDir}
                    onChange={(e) => setSettings({ ...settings, exportDir: e.target.value })}
                  />
                  <Button icon="folder" onClick={browse}>
                    {t("settings.browse")}
                  </Button>
                </div>
              )}
            </Field>
            <div>
              <Button type="submit" variant="primary" loading={saving}>
                {t("common.save")}
              </Button>
            </div>
          </form>
        </Card>

        <Card>
          <h2 className="mb-1 text-base font-semibold">{t("settings.contourColors")}</h2>
          <p className="mb-4 text-sm text-muted">{t("settings.contourColorsHint")}</p>
          <form onSubmit={save} className="flex flex-col gap-4">
            <div className="flex flex-wrap items-stretch gap-8">
              <div className="flex flex-col gap-2">
                {COLOR_KEYS.map((k) => (
                  <ColorRow key={k} name={k} value={colors[k]} onChange={(v) => setColor(k, v)} />
                ))}
              </div>
              <div className="flex flex-col gap-1.5">
                <span className="text-sm font-medium">{t("settings.preview")}</span>
                <div
                  className="w-6 flex-1 rounded-sm border border-border"
                  style={{ background: colorsValid ? gradientCss(buildPalette(colors)) : undefined }}
                  aria-hidden="true"
                />
              </div>
            </div>
            {errors.contourColors && <p className="text-sm text-danger">{errors.contourColors}</p>}
            <div className="flex gap-2">
              <Button type="submit" variant="primary" loading={saving} disabled={!colorsValid}>
                {t("common.save")}
              </Button>
              <Button icon="refresh" onClick={() => settings && setSettings({ ...settings, contourColors: { ...defaultContourColors } })}>
                {t("settings.restoreDefaults")}
              </Button>
            </div>
          </form>
        </Card>

        <Card>
          <h2 className="mb-4 text-base font-semibold">{t("settings.appearance")}</h2>
          <div className="flex flex-wrap items-end gap-6">
            <div className="flex flex-col gap-1.5">
              <span className="text-sm font-medium">{t("theme.label")}</span>
              <ThemeToggle />
            </div>
            <div className="flex w-60 flex-col gap-1.5">
              <span className="text-sm font-medium">{t("language.label")}</span>
              <LanguageSelect />
            </div>
          </div>
        </Card>

        {info && (
          <Card>
            <h2 className="mb-4 text-base font-semibold">{t("settings.about")}</h2>
            <dl className="grid grid-cols-[8rem_1fr] gap-x-4 gap-y-2 text-sm">
              <dt className="text-muted">{t("settings.version")}</dt>
              <dd>{info.version}</dd>
              <dt className="text-muted">{t("settings.database")}</dt>
              <dd className="font-mono text-xs break-all">{info.dbPath}</dd>
              <dt className="text-muted">{t("settings.logs")}</dt>
              <dd className="font-mono text-xs break-all">{info.logDir}</dd>
            </dl>
          </Card>
        )}
      </div>
    </div>
  );
}
