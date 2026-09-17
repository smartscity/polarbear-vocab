import { Modal } from "../../design-system/primitives/Dialogs";
import { Button } from "../../design-system/primitives/Button";
import { ProgressStatus } from "../../design-system/components/ProgressStatus";
import { useI18n } from "../../lib/i18n";
import type { PendingImport } from "./useDatasetController";
import type { DatasetImportStrategy } from "../../lib/commands";

interface ImportPreviewProps {
  busy: boolean;
  importing: boolean;
  onCancel: () => void;
  onConfirm: () => void;
  pending: PendingImport;
  strategy: DatasetImportStrategy;
  onStrategyChange: (strategy: DatasetImportStrategy) => void;
}

export function ImportPreview(props: ImportPreviewProps) {
  const { t } = useI18n();
  const preview = props.pending.preview;
  const hasIssues = preview.issues.length > 0;
  return (
    <Modal description={t("dataset.csvFile", { name: preview.fileName })} onOpenChange={(open) => { if (!open) props.onCancel(); }} open title={t("dataset.csvTitle")}>
      <div className="import-modal">
        <strong>{t("dataset.csvRows", { valid: preview.validRows, total: preview.totalRows })}</strong>
        {preview.sample.length > 0 ? <PreviewRows pending={props.pending} /> : null}
        <label className="import-strategy">
          <span>{t("dataset.importStrategy")}</span>
          <select onChange={(event) => props.onStrategyChange(event.target.value as DatasetImportStrategy)} value={props.strategy}>
            <option value="addOnly">{t("dataset.strategy.addOnly")}</option>
            <option value="updateExisting">{t("dataset.strategy.updateExisting")}</option>
            <option value="replaceDataset">{t("dataset.strategy.replaceDataset")}</option>
          </select>
        </label>
        {hasIssues ? <div className="import-issues"><strong>{t("dataset.csvIssues")}</strong>{preview.issues.map((issue, index) => <p key={`${issue.row}-${index}`}>#{issue.row}: {issue.message}</p>)}</div> : null}
        {props.importing ? <ProgressStatus label={t("dataset.importStatus.importing")} /> : null}
        <div className="pb-dialog-actions">
          <Button disabled={props.busy} onClick={props.onCancel}>{t("common.cancel")}</Button>
          <Button aria-busy={props.importing} disabled={props.busy || hasIssues || preview.validRows === 0} onClick={props.onConfirm} variant="primary">
            {props.importing ? t("dataset.importStatus.importing") : t("dataset.csvImport", { count: preview.validRows })}
          </Button>
        </div>
      </div>
    </Modal>
  );
}

function PreviewRows({ pending }: { pending: PendingImport }) {
  return (
    <div className="preview-table">
      {pending.preview.sample.map((row) => (
        <div key={row.senseUid}><span>{row.lemma}</span><small>{row.partOfSpeech}</small><span lang="zh-CN">{row.quizPromptZh}</span></div>
      ))}
    </div>
  );
}
