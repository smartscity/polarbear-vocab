import { open, save } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";

import { PageHeader } from "../../design-system/components/PageHeader";
import { ProgressStatus } from "../../design-system/components/ProgressStatus";
import { SettingsSection } from "../../design-system/components/SettingsSection";
import { Button } from "../../design-system/primitives/Button";
import { SelectControl } from "../../design-system/primitives/SelectControl";
import {
  exportBackup,
  getBackupStatus,
  importBackup,
  SPEECH_VOICES,
  type BackupStatus,
  type SpeechLocale,
  type UiLanguage,
  type UiTheme,
} from "../../lib/commands";
import { useI18n } from "../../lib/i18n";

export function SettingsView({ onError }: { onError: (error: unknown) => void }) {
  const [backupStatus, setBackupStatus] = useState<BackupStatus>({});
  const [backupBusy, setBackupBusy] = useState(false);
  const [backupMessage, setBackupMessage] = useState("");
  const [backupImportPhase, setBackupImportPhase] = useState<"choosing" | "restoring" | null>(null);
  const {
    language,
    setLanguage,
    setSpeechLocale,
    setSpeechRatePercent,
    setSpeechVoice,
    setTheme,
    speechLocale,
    speechRatePercent,
    speechVoice,
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
  const voiceOptions = SPEECH_VOICES.map((value) => ({
    label: t(`settings.voice.${value}`),
    value,
  }));
  useEffect(() => {
    if (!("__TAURI_INTERNALS__" in window)) return;
    void getBackupStatus().then(setBackupStatus).catch(onError);
  }, [onError]);
  const exportData = async () => {
    const path = await save({
      defaultPath: `polarbear-vocab-${new Date().toISOString().slice(0, 10)}.polarbear-vocab-backup`,
      filters: [{ name: t("settings.backupFile"), extensions: ["polarbear-vocab-backup"] }],
    });
    if (!path) return;
    setBackupBusy(true);
    setBackupMessage("");
    try {
      setBackupStatus(await exportBackup(path));
      setBackupMessage(t("settings.backupExported"));
    } catch (error) {
      onError(error);
    } finally {
      setBackupBusy(false);
    }
  };
  const importData = async () => {
    if (backupBusy) return;
    setBackupBusy(true);
    setBackupImportPhase("choosing");
    setBackupMessage("");
    try {
      const path = await open({
        directory: false,
        fileAccessMode: "copy",
        filters: [{ name: t("settings.backupFile"), extensions: ["polarbear-vocab-backup"] }],
        multiple: false,
        pickerMode: "document",
      });
      if (!path || !window.confirm(t("settings.restoreConfirm"))) return;
      setBackupImportPhase("restoring");
      const result = await importBackup(path);
      setBackupStatus(await getBackupStatus());
      window.alert(t("settings.restoreComplete", { path: result.automaticBackupPath }));
      window.location.reload();
    } catch (error) {
      onError(error);
    } finally {
      setBackupBusy(false);
      setBackupImportPhase(null);
    }
  };
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
        <SettingsSection description={t("settings.voiceHint")} label={t("settings.voice")}>
          <SelectControl ariaLabel={t("settings.voice")} onValueChange={(value) => void setSpeechVoice(value).catch(onError)} options={voiceOptions} value={speechVoice} />
        </SettingsSection>
        <SettingsSection description={t("settings.speechRateHint")} label={t("settings.speechRate")}>
          <div className="pb-speech-rate">
            <input
              aria-label={t("settings.speechRate")}
              max="200"
              min="50"
              onChange={(event) => void setSpeechRatePercent(Number(event.currentTarget.value)).catch(onError)}
              step="50"
              type="range"
              value={speechRatePercent}
            />
            <output>{speechRatePercent}%</output>
          </div>
        </SettingsSection>
        <SettingsSection description={t("settings.dataHint")} label={t("settings.data")}>
          <div className="settings-data-actions">
            <Button disabled={backupBusy} onClick={() => void exportData()}>{t("settings.exportBackup")}</Button>
            <Button aria-busy={backupImportPhase !== null} disabled={backupBusy} onClick={() => void importData()}>
              {backupImportPhase ? t(`settings.importStatus.${backupImportPhase}`) : t("settings.importBackup")}
            </Button>
          </div>
          {backupImportPhase ? <ProgressStatus label={t(`settings.importStatus.${backupImportPhase}`)} /> : null}
          <p className="pb-muted">
            {t("settings.lastBackup")}: {formatBackupDate(backupStatus.lastBackupAt, t("settings.never"))}
          </p>
          {backupMessage ? <p role="status">{backupMessage}</p> : null}
        </SettingsSection>
      </div>
    </section>
  );
}

function formatBackupDate(value: string | undefined, fallback: string): string {
  if (!value) return fallback;
  const date = new Date(value);
  return Number.isNaN(date.valueOf()) ? fallback : date.toLocaleDateString();
}
