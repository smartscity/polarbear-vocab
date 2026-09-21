import { useState } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";

import {
  createDataset,
  createVocabularyDataset,
  deleteDataset,
  exportDatasetCsv,
  importDatasetCsv,
  previewDatasetCsv,
  reorderDatasets,
  renameDataset,
  type CsvImportPreview,
  type DatasetImportStrategy,
  type DatasetSummary,
} from "../../lib/commands";

export interface DatasetControllerOptions {
  datasets: DatasetSummary[];
  onChanged: (preferredDatasetId?: string) => Promise<void>;
  onError: (error: unknown) => void;
  selectedDatasetId: string;
  csvLabel: string;
  successMessage: (count: number) => string;
  exportMessage: (count: number) => string;
  vocabularyDatasetName: string;
  vocabularySuccessMessage: (count: number) => string;
}

export interface PendingImport {
  path: string;
  preview: CsvImportPreview;
}

export type DatasetImportPhase = "choosing" | "previewing" | "importing";

export function useDatasetController(options: DatasetControllerOptions) {
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);
  const [pendingImport, setPendingImport] = useState<PendingImport | null>(null);
  const [importPhase, setImportPhase] = useState<DatasetImportPhase | null>(null);
  const [importStrategy, setImportStrategy] = useState<DatasetImportStrategy>("updateExisting");
  const [notice, setNotice] = useState<string | null>(null);
  const [renameOpen, setRenameOpen] = useState(false);
  const [renameName, setRenameName] = useState("");
  const [deleteOpen, setDeleteOpen] = useState(false);
  const selected = options.datasets.find((dataset) => dataset.id === options.selectedDatasetId);
  const run = (action: () => Promise<void>) => runAction(setBusy, setNotice, options.onError, action);
  const create = () => void run(async () => {
    const created = await createDataset(name.trim());
    setName("");
    await options.onChanged(created.id);
  });
  const createFromVocabulary = () => void run(async () => {
    const created = await createVocabularyDataset(options.vocabularyDatasetName);
    setNotice(options.vocabularySuccessMessage(created.wordCount));
    await options.onChanged(created.id);
  });
  const chooseCsv = () => void run(async () => {
    setImportPhase("choosing");
    try {
      const path = await open({
        directory: false,
        fileAccessMode: "copy",
        filters: [{ name: options.csvLabel, extensions: ["csv"] }],
        multiple: false,
        pickerMode: "document",
      });
      if (typeof path !== "string") return;
      setImportPhase("previewing");
      setPendingImport({ path, preview: await previewDatasetCsv(path) });
    } finally {
      setImportPhase(null);
    }
  });
  const confirmImport = () => void run(async () => {
    if (!selected || !pendingImport) return;
    setImportPhase("importing");
    try {
      const result = await importDatasetCsv(selected.id, pendingImport.path, importStrategy);
      setPendingImport(null);
      setNotice(options.successMessage(result.importedItems));
      await options.onChanged(selected.id);
    } finally {
      setImportPhase(null);
    }
  });
  const exportCsv = () => void run(async () => {
    if (!selected) return;
    const path = await save({
      defaultPath: `${safeFileName(selected.name)}.csv`,
      filters: [{ name: options.csvLabel, extensions: ["csv"] }],
    });
    if (!path) return;
    const count = await exportDatasetCsv(selected.id, path);
    setNotice(options.exportMessage(count));
  });
  const beginRename = () => {
    if (!selected) return;
    setRenameName(selected.name);
    setRenameOpen(true);
  };
  const confirmRename = () => void run(async () => {
    if (!selected || !renameName.trim()) return;
    await renameDataset(selected.id, renameName.trim());
    setRenameOpen(false);
    await options.onChanged(selected.id);
  });
  const reorder = (datasetIds: string[]) => run(async () => {
    await reorderDatasets(datasetIds);
    await options.onChanged(options.selectedDatasetId);
  });
  const confirmDelete = () => void run(async () => {
    if (!selected) return;
    await deleteDataset(selected.id);
    setDeleteOpen(false);
    await options.onChanged();
  });
  return { beginRename, busy, chooseCsv, confirmDelete, confirmImport, confirmRename, create, createFromVocabulary, deleteOpen, exportCsv, importPhase, importStrategy, name, notice, pendingImport, renameName, renameOpen, reorder, selected, setDeleteOpen, setImportStrategy, setName, setPendingImport, setRenameName, setRenameOpen };
}

function safeFileName(name: string): string {
  return name.trim().replace(/[^\p{L}\p{N}._-]+/gu, "-").replace(/^-+|-+$/g, "") || "dataset";
}

async function runAction(
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
