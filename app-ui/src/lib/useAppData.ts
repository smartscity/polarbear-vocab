import { useCallback, useEffect, useState } from "react";

import { browserAppInfo } from "./branding";
import {
  getAppInfo,
  getHome,
  listDatasets,
  type AppInfo,
  type DatasetSummary,
  type HomeDto,
} from "./commands";

export function useAppData() {
  const [appInfo, setAppInfo] = useState<AppInfo>(browserAppInfo);
  const [datasets, setDatasets] = useState<DatasetSummary[]>([]);
  const [selectedDatasetId, setSelectedDatasetId] = useState("");
  const [home, setHome] = useState<HomeDto | null>(null);
  const [error, setError] = useState<string | null>(null);
  const reportError = useCallback((reason: unknown) => setError(String(reason)), []);

  const refreshHome = useCallback(async (datasetId: string) => {
    setHome(await getHome(datasetId));
  }, []);

  const refreshDatasets = useCallback(async (preferredDatasetId?: string) => {
    const available = await listDatasets();
    setDatasets(available);
    const preferredExists = available.some((dataset) => dataset.id === preferredDatasetId);
    const nextId = preferredExists ? preferredDatasetId ?? "" : available[0]?.id ?? "";
    setSelectedDatasetId(nextId);
    if (nextId) await refreshHome(nextId);
    else setHome(null);
  }, [refreshHome]);

  const changeDataset = useCallback(async (datasetId: string) => {
    setSelectedDatasetId(datasetId);
    await refreshHome(datasetId).catch(reportError);
  }, [refreshHome, reportError]);

  useEffect(() => {
    if (!("__TAURI_INTERNALS__" in window)) return;
    void getAppInfo()
      .then((info) => setAppInfo(info))
      .then(() => refreshDatasets())
      .catch(reportError);
  }, [refreshDatasets, reportError]);

  return {
    appInfo,
    changeDataset,
    datasets,
    error,
    home,
    refreshDatasets,
    refreshHome,
    reportError,
    selectedDatasetId,
    setError,
  };
}
