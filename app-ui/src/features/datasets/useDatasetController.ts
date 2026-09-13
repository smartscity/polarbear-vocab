import { useState } from "react";
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

export interface DatasetControllerOptions {
  datasets: DatasetSummary[];
  onChanged: (preferredDatasetId?: string) => Promise<void>;
  onError: (error: unknown) => void;
  selectedDatasetId: string;
  csvLabel: string;
  successMessage: (count: number) => string;
}

export interface PendingImport {
  path: string;
  preview: CsvImportPreview;
}

export function useDatasetController(options: DatasetControllerOptions) {
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);
  const [pendingImport, setPendingImport] = useState<PendingImport | null>(null);
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
  const chooseCsv = () => void run(async () => {
    const path = await open({ multiple: false, directory: false, filters: [{ name: options.csvLabel, extensions: ["csv"] }] });
    if (typeof path === "string") setPendingImport({ path, preview: await previewDatasetCsv(path) });
  });
  const confirmImport = () => void run(async () => {
    if (!selected || !pendingImport) return;
    const result = await importDatasetCsv(selected.id, pendingImport.path);
    setPendingImport(null);
    setNotice(options.successMessage(result.importedItems));
    await options.onChanged(selected.id);
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
  const confirmDelete = () => void run(async () => {
    if (!selected) return;
    await deleteDataset(selected.id);
    setDeleteOpen(false);
    await options.onChanged();
  });
  return { beginRename, busy, chooseCsv, confirmDelete, confirmImport, confirmRename, create, deleteOpen, name, notice, pendingImport, renameName, renameOpen, selected, setDeleteOpen, setName, setPendingImport, setRenameName, setRenameOpen };
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
