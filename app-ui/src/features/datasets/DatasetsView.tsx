import type { FormEvent } from "react";

import { EmptyState } from "../../design-system/components/EmptyState";
import { PageHeader } from "../../design-system/components/PageHeader";
import { Button } from "../../design-system/primitives/Button";
import { ConfirmDialog, Modal } from "../../design-system/primitives/Dialogs";
import type { DatasetSummary } from "../../lib/commands";
import { useI18n } from "../../lib/i18n";
import { ImportPreview } from "./ImportPreview";
import { useDatasetController } from "./useDatasetController";

interface DatasetsViewProps {
  datasets: DatasetSummary[];
  selectedDatasetId: string;
  onSelect: (datasetId: string) => void;
  onChanged: (preferredDatasetId?: string) => Promise<void>;
  onStart: (datasetId: string) => void;
  onError: (error: unknown) => void;
}

export function DatasetsView(props: DatasetsViewProps) {
  const { t } = useI18n();
  const controller = useDatasetController({
    ...props,
    csvLabel: t("dataset.csvFormat"),
    successMessage: (count) => t("dataset.csvSuccess", { count }),
  });
  const submitCreate = (event: FormEvent) => {
    event.preventDefault();
    if (controller.name.trim()) controller.create();
  };
  return (
    <section className="datasets-page">
      <PageHeader actions={<CreateDatasetForm busy={controller.busy} name={controller.name} onChange={controller.setName} onSubmit={submitCreate} />} eyebrow={t("app.name")} title={t("dataset.title")} />
      <div className="dataset-layout">
        <DatasetList datasets={props.datasets} onSelect={props.onSelect} selectedId={props.selectedDatasetId} />
        {controller.selected ? <DatasetDetail busy={controller.busy} dataset={controller.selected} notice={controller.notice} onDelete={() => controller.setDeleteOpen(true)} onImport={controller.chooseCsv} onRename={controller.beginRename} onStart={props.onStart} /> : <EmptyState>{t("dataset.select")}</EmptyState>}
      </div>
      {controller.pendingImport ? <ImportPreview busy={controller.busy} onCancel={() => controller.setPendingImport(null)} onConfirm={controller.confirmImport} pending={controller.pendingImport} /> : null}
      <RenameDialog controller={controller} />
      {controller.selected ? <ConfirmDialog cancelLabel={t("common.cancel")} confirmLabel={t("common.delete")} description={t("dataset.deleteConfirm", { name: controller.selected.name })} onConfirm={controller.confirmDelete} onOpenChange={controller.setDeleteOpen} open={controller.deleteOpen} title={t("dataset.delete")} /> : null}
    </section>
  );
}

function CreateDatasetForm(props: { busy: boolean; name: string; onChange: (name: string) => void; onSubmit: (event: FormEvent) => void }) {
  const { t } = useI18n();
  return (
    <form className="create-dataset" onSubmit={props.onSubmit}>
      <input aria-label={t("dataset.name")} className="pb-input" disabled={props.busy} maxLength={80} onChange={(event) => props.onChange(event.target.value)} placeholder={t("dataset.name")} value={props.name} />
      <Button disabled={props.busy || !props.name.trim()} type="submit" variant="primary">{t("dataset.new")}</Button>
    </form>
  );
}

function DatasetList(props: { datasets: DatasetSummary[]; onSelect: (id: string) => void; selectedId: string }) {
  const { t } = useI18n();
  return (
    <div className="dataset-list">
      {props.datasets.map((dataset) => (
        <button data-active={dataset.id === props.selectedId} key={dataset.id} onClick={() => props.onSelect(dataset.id)} type="button">
          <span>{dataset.name}</span><small>{t("dataset.words", { count: dataset.wordCount })}</small>
        </button>
      ))}
    </div>
  );
}

function DatasetDetail(props: { busy: boolean; dataset: DatasetSummary; notice: string | null; onDelete: () => void; onImport: () => void; onRename: () => void; onStart: (id: string) => void }) {
  const { t } = useI18n();
  return (
    <article className="dataset-detail">
      <span className="dataset-kind">{t(props.dataset.preloaded ? "dataset.preloaded" : "dataset.custom")}</span>
      <h2 className="pb-display">{props.dataset.name}</h2><p>{t("dataset.words", { count: props.dataset.wordCount })}</p>
      <p className="csv-hint">{t("dataset.csvColumns")}</p>{props.notice ? <p className="success-notice">{props.notice}</p> : null}
      <div className="dataset-actions">
        <Button disabled={props.busy || props.dataset.wordCount === 0} onClick={() => props.onStart(props.dataset.id)} variant="primary">{t("dataset.start")}</Button>
        <Button disabled={props.busy} onClick={props.onImport}>{t("dataset.importCsv")}</Button>
        <Button disabled={props.busy} onClick={props.onRename}>{t("dataset.rename")}</Button>
        <Button disabled={props.busy} onClick={props.onDelete} variant="danger">{t("dataset.delete")}</Button>
      </div>
    </article>
  );
}

type DatasetController = ReturnType<typeof useDatasetController>;

function RenameDialog({ controller }: { controller: DatasetController }) {
  const { t } = useI18n();
  const submit = (event: FormEvent) => {
    event.preventDefault();
    controller.confirmRename();
  };
  return (
    <Modal description={t("dataset.renameHint")} onOpenChange={controller.setRenameOpen} open={controller.renameOpen} title={t("dataset.rename")}>
      <form onSubmit={submit}>
        <input aria-label={t("dataset.renamePrompt")} autoFocus className="pb-input w-full" maxLength={80} onChange={(event) => controller.setRenameName(event.target.value)} value={controller.renameName} />
        <div className="pb-dialog-actions"><Button onClick={() => controller.setRenameOpen(false)} type="button">{t("common.cancel")}</Button><Button disabled={!controller.renameName.trim()} type="submit" variant="primary">{t("common.rename")}</Button></div>
      </form>
    </Modal>
  );
}
