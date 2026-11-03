import { useState } from "react";
import { useI18n } from "./i18n";
import { Icon, type IconName } from "./components/Icon";
import { LanguageSelect, ThemeToggle } from "./components/ThemeLangControls";
import type { ReelRecord } from "./lib/api";
import { ContourPage } from "./features/contour/ContourPage";
import { GeneratorPage } from "./features/generator/GeneratorPage";
import { PropertiesPage } from "./features/properties/PropertiesPage";
import { MachinesPage } from "./features/machines/MachinesPage";
import { SettingsPage } from "./features/settings/SettingsPage";

type Route = "contour" | "generator" | "machines" | "properties" | "settings";

const nav: { route: Route; icon: IconName }[] = [
  { route: "contour", icon: "grid" },
  { route: "generator", icon: "dice" },
  { route: "machines", icon: "machines" },
  { route: "properties", icon: "properties" },
  { route: "settings", icon: "settings" },
];

export default function App() {
  const { t } = useI18n();
  const [route, setRoute] = useState<Route>("contour");
  /** Bobina aberta a partir do Gerador. */
  const [contourReel, setContourReel] = useState<ReelRecord | null>(null);
  const [menuOpen, setMenuOpen] = useState(false);

  const go = (r: Route) => {
    setRoute(r);
    setMenuOpen(false);
  };

  return (
    <div className="flex h-full">
      {menuOpen && <div className="fixed inset-0 z-30 bg-black/40 md:hidden" onClick={() => setMenuOpen(false)} aria-hidden="true" />}
      <aside
        className={`fixed inset-y-0 left-0 z-40 flex w-64 flex-col border-r border-border bg-sidebar transition-transform md:static md:translate-x-0 ${
          menuOpen ? "translate-x-0" : "-translate-x-full"
        }`}
      >
        <div className="flex items-center gap-3 px-4 py-5">
          <div className="flex size-9 items-center justify-center rounded-xl bg-accent text-on-accent">
            <Icon name="grid" className="size-5" />
          </div>
          <div className="min-w-0 flex-1">
            <p className="truncate text-sm font-semibold">{t("app.name")}</p>
            <p className="truncate text-xs text-muted">{t("app.subtitle")}</p>
          </div>
          <button type="button" className="rounded-lg p-1.5 text-muted hover:bg-surface-hover md:hidden" onClick={() => setMenuOpen(false)} aria-label={t("nav.close")}>
            <Icon name="close" />
          </button>
        </div>
        <nav className="flex flex-1 flex-col gap-0.5 px-2" aria-label="Main">
          {nav.map((n) => (
            <button
              key={n.route}
              type="button"
              onClick={() => go(n.route)}
              aria-current={route === n.route ? "page" : undefined}
              className={`flex items-center gap-3 rounded-lg px-3 py-2 text-left text-sm transition-colors ${
                route === n.route ? "bg-surface-hover font-medium text-fg" : "text-muted hover:bg-surface hover:text-fg"
              }`}
            >
              <Icon name={n.icon} className={`size-4 ${route === n.route ? "text-accent" : ""}`} />
              {t(`nav.${n.route}`)}
            </button>
          ))}
        </nav>
        <div className="flex flex-col gap-3 border-t border-border p-4">
          <ThemeToggle />
          <LanguageSelect />
        </div>
      </aside>

      <div className="flex min-w-0 flex-1 flex-col">
        <header className="flex items-center gap-3 border-b border-border px-4 py-3 md:hidden">
          <button type="button" className="rounded-lg p-1.5 hover:bg-surface-hover" onClick={() => setMenuOpen(true)} aria-label={t("nav.menu")}>
            <Icon name="menu" className="size-5" />
          </button>
          <span className="text-sm font-semibold">{t(`nav.${route}`)}</span>
        </header>
        <main className="flex-1 overflow-y-auto px-4 py-6 sm:px-8">
          <div className="mx-auto max-w-[1600px]">
            {route === "contour" && <ContourPage initialReel={contourReel} />}
            {route === "generator" && (
              <GeneratorPage
                onOpenContour={(reel) => {
                  setContourReel(reel);
                  go("contour");
                }}
              />
            )}
            {route === "machines" && <MachinesPage />}
            {route === "properties" && <PropertiesPage />}
            {route === "settings" && <SettingsPage />}
          </div>
        </main>
      </div>
    </div>
  );
}
