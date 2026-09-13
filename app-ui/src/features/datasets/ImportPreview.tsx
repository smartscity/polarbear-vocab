import { Modal } from "../../design-system/primitives/Dialogs";
import { Button } from "../../design-system/primitives/Button";
import { useI18n } from "../../lib/i18n";
import type { PendingImport } from "./useDatasetController";

interface ImportPreviewProps {
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => void;
  pending: PendingImport;
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
        {hasIssues ? <div className="import-issues"><strong>{t("dataset.csvIssues")}</strong>{preview.issues.map((issue, index) => <p key={`${issue.row}-${index}`}>#{issue.row}: {issue.message}</p>)}</div> : null}
        <div className="pb-dialog-actions">
          <Button disabled={props.busy} onClick={props.onCancel}>{t("common.cancel")}</Button>
          <Button disabled={props.busy || hasIssues || preview.validRows === 0} onClick={props.onConfirm} variant="primary">{t("dataset.csvImport", { count: preview.validRows })}</Button>
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
