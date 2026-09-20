import { open, save } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";

import { ProgressStatus } from "../../design-system/components/ProgressStatus";
import { Button } from "../../design-system/primitives/Button";
import {
  exportSync,
  getSyncStatus,
  importSync,
  type SyncExportResult,
  type SyncImportResult,
  type SyncStatus,
} from "../../lib/commands";
import { useI18n } from "../../lib/i18n";

export function SyncPanel({ onError }: { onError: (error: unknown) => void }) {
  const [status, setStatus] = useState<SyncStatus>();
  const [phase, setPhase] = useState<"exporting" | "importing" | null>(null);
  const [message, setMessage] = useState("");
  const { t } = useI18n();

  useEffect(() => {
    if (!("__TAURI_INTERNALS__" in window)) return;
    void getSyncStatus().then(setStatus).catch(onError);
  }, [onError]);

  const exportPackage = async () => {
    const path = await save({
      defaultPath: `polarbear-vocab-sync-${new Date().toISOString().slice(0, 10)}.polarbear-vocab-sync`,
      filters: [{ name: t("settings.syncFile"), extensions: ["polarbear-vocab-sync"] }],
    });
    if (!path) return;
    await run("exporting", () => exportSync(path), formatExportResult);
  };

  const importPackage = async () => {
    const path = await open({
      directory: false,
      fileAccessMode: "copy",
      filters: [{ name: t("settings.syncFile"), extensions: ["polarbear-vocab-sync"] }],
      multiple: false,
      pickerMode: "document",
    });
    if (!path) return;
    await run("importing", () => importSync(path), formatImportResult);
  };

  const run = async <T,>(next: "exporting" | "importing", action: () => Promise<T>, format: (value: T) => string) => {
    setPhase(next);
    setMessage("");
    try {
      setMessage(format(await action()));
      setStatus(await getSyncStatus());
    } catch (error) {
      onError(error);
    } finally {
      setPhase(null);
    }
  };

  const formatExportResult = (result: SyncExportResult) => t("settings.syncExported", {
    articles: result.articleCount,
    datasets: result.datasetCount,
    reviews: result.reviewEventCount,
  });
  const formatImportResult = (result: SyncImportResult) => result.alreadyApplied
    ? t("settings.syncAlreadyApplied")
    : t("settings.syncImported", {
      articles: result.articleCount,
      conflicts: result.conflictCount,
      datasets: result.datasetCount,
      reviews: result.reviewEventCount,
    });

  return (
    <div className="sync-panel">
      <div className="backup-version-heading">
        <strong>{t("settings.syncTitle")}</strong>
        <span className="pb-muted">{t("settings.syncHint")}</span>
      </div>
      <div className="settings-data-actions">
        <Button disabled={phase !== null} onClick={() => void exportPackage()}>{t("settings.exportSync")}</Button>
        <Button disabled={phase !== null} onClick={() => void importPackage()}>{t("settings.importSync")}</Button>
      </div>
      {phase ? <ProgressStatus label={t(`settings.syncStatus.${phase}`)} /> : null}
      {status ? (
        <p className="pb-muted">
          {t("settings.syncSummary", {
            changes: status.pendingChangeCount,
            devices: status.peerCount,
            time: formatDate(status.lastSyncAt, t("settings.never")),
          })}
        </p>
      ) : null}
      {message ? <p role="status">{message}</p> : null}
    </div>
  );
}

function formatDate(value: string | undefined, fallback: string): string {
  if (!value) return fallback;
  const date = new Date(value);
  return Number.isNaN(date.valueOf()) ? fallback : date.toLocaleString();
}
