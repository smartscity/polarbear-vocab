import { useState, type FormEvent } from "react";
import { open } from "@tauri-apps/plugin-dialog";

import {
  createDataset,
  deleteDataset,
  importDatasetCsv,
  previewDatasetCsv,
  renameDataset,
  type CsvImportPreview,
  type DatasetSummary,
} from "../../lib/commands";
import { useI18n } from "../../lib/i18n";

interface DatasetsViewProps {
  datasets: DatasetSummary[];
  selectedDatasetId: string;
  onSelect: (datasetId: string) => void;
  onChanged: (preferredDatasetId?: string) => Promise<void>;
  onStart: (datasetId: string) => void;
  onError: (error: unknown) => void;
}

interface PendingImport {
  path: string;
  preview: CsvImportPreview;
}

export function DatasetsView(props: DatasetsViewProps) {
  const { t } = useI18n();
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);
  const [pendingImport, setPendingImport] = useState<PendingImport | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const selected = props.datasets.find((dataset) => dataset.id === props.selectedDatasetId);
  const run = (action: () => Promise<void>) => runDatasetAction(setBusy, setNotice, props.onError, action);
  const submitCreate = (event: FormEvent) => {
    event.preventDefault();
    const trimmed = name.trim();
    if (!trimmed) return;
    void run(async () => {
      const created = await createDataset(trimmed);
      setName("");
      await props.onChanged(created.id);
    });
  };

  const chooseCsv = () => {
    if (!selected) return;
    void run(async () => {
      const path = await open({
        multiple: false,
        directory: false,
        filters: [{ name: t("dataset.csvFormat"), extensions: ["csv"] }],
      });
      if (typeof path !== "string") return;
      setPendingImport({ path, preview: await previewDatasetCsv(path) });
    });
  };

  const confirmImport = () => {
    if (!selected || !pendingImport || pendingImport.preview.issues.length > 0) return;
    void run(async () => {
      const result = await importDatasetCsv(selected.id, pendingImport.path);
      setPendingImport(null);
      setNotice(t("dataset.csvSuccess", { count: result.importedItems }));
      await props.onChanged(selected.id);
    });
  };

  const requestRename = () => {
    if (!selected) return;
    const nextName = window.prompt(t("dataset.renamePrompt"), selected.name)?.trim();
    if (!nextName || nextName === selected.name) return;
    void run(async () => {
      await renameDataset(selected.id, nextName);
      await props.onChanged(selected.id);
    });
  };

  const requestDelete = () => {
    if (!selected || !window.confirm(t("dataset.deleteConfirm", { name: selected.name }))) return;
    void run(async () => {
      await deleteDataset(selected.id);
      await props.onChanged();
    });
  };

  return <DatasetsPage
    busy={busy}
    chooseCsv={chooseCsv}
    confirmImport={confirmImport}
    datasets={props.datasets}
    name={name}
    notice={notice}
    onDelete={requestDelete}
    onNameChange={setName}
    onRename={requestRename}
    onSelect={props.onSelect}
    onStart={props.onStart}
    pendingImport={pendingImport}
    selected={selected}
    selectedDatasetId={props.selectedDatasetId}
    setPendingImport={setPendingImport}
    submitCreate={submitCreate}
  />;
}

async function runDatasetAction(
  setBusy: (busy: boolean) => void,
  setNotice: (notice: string | null) => void,
  onError: (error: unknown) => void,
  action: () => Promise<void>,
) {
  setBusy(true);
  setNotice(null);
  try {
    await action();
  } catch (error) {
    onError(error);
  } finally {
    setBusy(false);
  }
}

interface DatasetsPageProps {
  busy: boolean;
  chooseCsv: () => void;
  confirmImport: () => void;
  datasets: DatasetSummary[];
  name: string;
  notice: string | null;
  onDelete: () => void;
  onNameChange: (name: string) => void;
  onRename: () => void;
  onSelect: (datasetId: string) => void;
  onStart: (datasetId: string) => void;
  pendingImport: PendingImport | null;
  selected?: DatasetSummary;
  selectedDatasetId: string;
  setPendingImport: (pending: PendingImport | null) => void;
  submitCreate: (event: FormEvent) => void;
}

function DatasetsPage(props: DatasetsPageProps) {
  const { t } = useI18n();
  return (
    <section className="datasets-page">
      <div className="page-heading">
        <div><p className="eyebrow">{t("app.name")}</p><h2>{t("dataset.title")}</h2></div>
        <form className="create-dataset" onSubmit={props.submitCreate}>
          <input
            aria-label={t("dataset.name")}
            disabled={props.busy}
            maxLength={80}
            onChange={(event) => props.onNameChange(event.target.value)}
            placeholder={t("dataset.name")}
            value={props.name}
          />
          <button type="submit" disabled={props.busy || !props.name.trim()}>{t("dataset.create")}</button>
        </form>
      </div>

      <div className="dataset-layout">
        <div className="dataset-list" role="list">
          {props.datasets.map((dataset) => (
            <button
              className={dataset.id === props.selectedDatasetId ? "active" : ""}
              key={dataset.id}
              onClick={() => props.onSelect(dataset.id)}
              role="listitem"
              type="button"
            >
              <span>{dataset.name}</span>
              <small>{t("dataset.words", { count: dataset.wordCount })}</small>
            </button>
          ))}
        </div>

        {props.selected ? (
          <article className="dataset-detail">
            <div>
              <span className="dataset-kind">{t(props.selected.preloaded ? "dataset.preloaded" : "dataset.custom")}</span>
              <h3>{props.selected.name}</h3>
              <p>{t("dataset.words", { count: props.selected.wordCount })}</p>
            </div>
            <p className="csv-hint">{t("dataset.csvColumns")}</p>
            {props.notice ? <p className="success-notice">{props.notice}</p> : null}
            <div className="dataset-actions">
              <button className="primary-button" disabled={props.busy || props.selected.wordCount === 0} onClick={() => props.onStart(props.selected!.id)} type="button">{t("dataset.start")}</button>
              <button disabled={props.busy} onClick={props.chooseCsv} type="button">{t("dataset.importCsv")}</button>
              <button disabled={props.busy} onClick={props.onRename} type="button">{t("dataset.rename")}</button>
              <button className="danger-button" disabled={props.busy} onClick={props.onDelete} type="button">{t("dataset.delete")}</button>
            </div>
          </article>
        ) : <div className="dataset-detail empty-state">{t("dataset.select")}</div>}
      </div>

      {props.pendingImport ? (
        <ImportPreview
          pending={props.pendingImport}
          busy={props.busy}
          onCancel={() => props.setPendingImport(null)}
          onConfirm={props.confirmImport}
        />
      ) : null}
    </section>
  );
}

function ImportPreview(props: {
  pending: PendingImport;
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  const { t } = useI18n();
  const { preview } = props.pending;
  const hasIssues = preview.issues.length > 0;
  return (
    <div className="modal-backdrop" role="presentation">
      <section aria-modal="true" className="import-modal" role="dialog">
        <h3>{t("dataset.csvTitle")}</h3>
        <p>{t("dataset.csvFile", { name: preview.fileName })}</p>
        <strong>{t("dataset.csvRows", { valid: preview.validRows, total: preview.totalRows })}</strong>
        {preview.sample.length > 0 ? (
          <div className="preview-table">
            {preview.sample.map((row) => (
              <div key={row.senseUid}><span>{row.lemma}</span><small>{row.partOfSpeech}</small><span lang="zh-CN">{row.quizPromptZh}</span></div>
            ))}
          </div>
        ) : null}
        {hasIssues ? (
          <div className="import-issues">
            <strong>{t("dataset.csvIssues")}</strong>
            {preview.issues.map((issue, index) => <p key={`${issue.row}-${index}`}>#{issue.row}: {issue.message}</p>)}
          </div>
        ) : null}
        <div className="modal-actions">
          <button disabled={props.busy} onClick={props.onCancel} type="button">{t("common.cancel")}</button>
          <button className="primary-button" disabled={props.busy || hasIssues || preview.validRows === 0} onClick={props.onConfirm} type="button">
            {t("dataset.csvImport", { count: preview.validRows })}
          </button>
        </div>
      </section>
    </div>
  );
}
