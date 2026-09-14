import { PageHeader } from "../../design-system/components/PageHeader";
import { SettingsSection } from "../../design-system/components/SettingsSection";
import { SelectControl } from "../../design-system/primitives/SelectControl";
import type { SpeechLocale, UiLanguage, UiTheme } from "../../lib/commands";
import { useI18n } from "../../lib/i18n";

export function SettingsView({ onError }: { onError: (error: unknown) => void }) {
  const {
    language,
    setLanguage,
    setSpeechLocale,
    setSpeechRatePercent,
    setTheme,
    speechLocale,
    speechRatePercent,
    t,
    theme,
  } = useI18n();
  const languageOptions: Array<{ label: string; value: UiLanguage }> = [
    { label: t("settings.language.system"), value: "system" },
    { label: t("settings.language.en"), value: "en" },
    { label: t("settings.language.zh-CN"), value: "zh-CN" },
  ];
  const themeOptions: Array<{ label: string; value: UiTheme }> = [
    { label: t("settings.theme.system"), value: "system" },
    { label: t("settings.theme.light"), value: "light" },
    { label: t("settings.theme.dark"), value: "dark" },
  ];
  const speechOptions: Array<{ label: string; value: SpeechLocale }> = [
    { label: t("settings.speech.en-US"), value: "en-US" },
    { label: t("settings.speech.en-GB"), value: "en-GB" },
  ];
  return (
    <section className="settings-page">
      <PageHeader eyebrow={t("app.name")} title={t("settings.title")} />
      <div className="grid gap-3">
        <SettingsSection description={t("settings.languageHint")} label={t("settings.language")}>
          <SelectControl ariaLabel={t("settings.language")} onValueChange={(value) => void setLanguage(value).catch(onError)} options={languageOptions} value={language} />
        </SettingsSection>
        <SettingsSection description={t("settings.themeHint")} label={t("settings.theme")}>
          <SelectControl ariaLabel={t("settings.theme")} onValueChange={(value) => void setTheme(value).catch(onError)} options={themeOptions} value={theme} />
        </SettingsSection>
        <SettingsSection description={t("settings.speechHint")} label={t("settings.speech")}>
          <SelectControl ariaLabel={t("settings.speech")} onValueChange={(value) => void setSpeechLocale(value).catch(onError)} options={speechOptions} value={speechLocale} />
        </SettingsSection>
        <SettingsSection description={t("settings.speechRateHint")} label={t("settings.speechRate")}>
          <div className="pb-speech-rate">
            <input
              aria-label={t("settings.speechRate")}
              max="200"
              min="50"
              onChange={(event) => void setSpeechRatePercent(Number(event.currentTarget.value)).catch(onError)}
              step="25"
              type="range"
              value={speechRatePercent}
            />
            <output>{speechRatePercent}%</output>
          </div>
        </SettingsSection>
      </div>
    </section>
  );
}
