import type { UiLanguage } from "../../lib/commands";
import { useI18n } from "../../lib/i18n";

export function SettingsView({ onError }: { onError: (error: unknown) => void }) {
  const { language, setLanguage, t } = useI18n();
  const updateLanguage = (nextLanguage: UiLanguage) => {
    void setLanguage(nextLanguage).catch(onError);
  };

  return (
    <section className="settings-page">
      <div className="page-heading"><div><p className="eyebrow">{t("app.name")}</p><h2>{t("settings.title")}</h2></div></div>
      <div className="settings-card">
        <label htmlFor="ui-language">
          <strong>{t("settings.language")}</strong>
          <span>{t("settings.languageHint")}</span>
        </label>
        <select
          id="ui-language"
          onChange={(event) => updateLanguage(event.target.value as UiLanguage)}
          value={language}
        >
          <option value="system">{t("settings.language.system")}</option>
          <option value="en">{t("settings.language.en")}</option>
          <option value="zh-CN">{t("settings.language.zh-CN")}</option>
        </select>
      </div>
    </section>
  );
}
