import { useEffect, useRef, useState, type KeyboardEvent, type PointerEvent } from "react";

import type { DatasetSummary } from "../../lib/commands";
import { useI18n } from "../../lib/i18n";

interface DatasetListProps {
  busy: boolean;
  datasets: DatasetSummary[];
  onReorder: (datasetIds: string[]) => Promise<void>;
  onSelect: (id: string) => void;
  selectedId: string;
}

export function DatasetList(props: DatasetListProps) {
  const { t } = useI18n();
  const reorder = useDatasetReordering(props);
  return (
    <div className="dataset-list">
      {reorder.ordered.map((dataset) => (
        <DatasetListItem
          busy={props.busy}
          dataset={dataset}
          dragging={dataset.id === reorder.draggingId}
          key={dataset.id}
          onBeginDrag={reorder.beginDrag}
          onDragOver={reorder.dragOver}
          onFinishDrag={reorder.finishDrag}
          onMoveByKeyboard={reorder.moveByKeyboard}
          onSelect={props.onSelect}
          reorderHint={t("dataset.reorderHint")}
          reorderLabel={t("dataset.reorder", { name: dataset.name })}
          selected={dataset.id === props.selectedId}
          wordCountLabel={t("dataset.words", { count: dataset.wordCount })}
        />
      ))}
    </div>
  );
}

function useDatasetReordering(props: DatasetListProps) {
  const [ordered, setOrdered] = useState(props.datasets);
  const [draggingId, setDraggingId] = useState<string>();
  const draggingRef = useRef<string | undefined>(undefined);
  const orderedRef = useRef(props.datasets);

  useEffect(() => {
    if (draggingRef.current) return;
    orderedRef.current = props.datasets;
    setOrdered(props.datasets);
  }, [props.datasets]);

  const updateOrder = (activeId: string, overId: string) => {
    const next = moveDataset(orderedRef.current, activeId, overId);
    orderedRef.current = next;
    setOrdered(next);
  };
  const finishDrag = () => {
    if (!draggingRef.current) return;
    draggingRef.current = undefined;
    setDraggingId(undefined);
    void persistIfChanged(orderedRef.current, props.datasets, props.onReorder);
  };
  const moveByKeyboard = (event: KeyboardEvent, datasetId: string) => {
    const offset = event.key === "ArrowUp" ? -1 : event.key === "ArrowDown" ? 1 : 0;
    if (!offset || props.busy) return;
    event.preventDefault();
    const next = moveDatasetByOffset(orderedRef.current, datasetId, offset);
    orderedRef.current = next;
    setOrdered(next);
    void persistIfChanged(next, props.datasets, props.onReorder);
  };
  const beginDrag = (event: PointerEvent<HTMLButtonElement>, datasetId: string) => {
    if (props.busy) return;
    draggingRef.current = datasetId;
    setDraggingId(datasetId);
    event.currentTarget.setPointerCapture(event.pointerId);
  };
  const dragOver = (event: PointerEvent<HTMLButtonElement>) => {
    const activeId = draggingRef.current;
    const target = document
      .elementFromPoint(event.clientX, event.clientY)
      ?.closest<HTMLElement>("[data-dataset-id]");
    const overId = target?.dataset.datasetId;
    if (activeId && overId && activeId !== overId) updateOrder(activeId, overId);
  };

  return { beginDrag, dragOver, draggingId, finishDrag, moveByKeyboard, ordered };
}

interface DatasetListItemProps {
  busy: boolean;
  dataset: DatasetSummary;
  dragging: boolean;
  onBeginDrag: (event: PointerEvent<HTMLButtonElement>, datasetId: string) => void;
  onDragOver: (event: PointerEvent<HTMLButtonElement>) => void;
  onFinishDrag: () => void;
  onMoveByKeyboard: (event: KeyboardEvent, datasetId: string) => void;
  onSelect: (id: string) => void;
  reorderHint: string;
  reorderLabel: string;
  selected: boolean;
  wordCountLabel: string;
}

function DatasetListItem(props: DatasetListItemProps) {
  return (
    <div className="dataset-list-item" data-active={props.selected} data-dataset-id={props.dataset.id} data-dragging={props.dragging}>
      <button className="dataset-select" onClick={() => props.onSelect(props.dataset.id)} type="button">
        <span>{props.dataset.name}</span><small>{props.wordCountLabel}</small>
      </button>
      <button
        aria-label={`${props.reorderLabel}. ${props.reorderHint}`}
        className="dataset-drag-handle"
        disabled={props.busy}
        onKeyDown={(event) => props.onMoveByKeyboard(event, props.dataset.id)}
        onLostPointerCapture={props.onFinishDrag}
        onPointerCancel={props.onFinishDrag}
        onPointerDown={(event) => props.onBeginDrag(event, props.dataset.id)}
        onPointerMove={props.onDragOver}
        onPointerUp={props.onFinishDrag}
        title={props.reorderHint}
        type="button"
      >
        <span aria-hidden="true">≡</span>
      </button>
    </div>
  );
}

export function moveDataset(
  datasets: DatasetSummary[],
  activeId: string,
  overId: string,
): DatasetSummary[] {
  const from = datasets.findIndex((dataset) => dataset.id === activeId);
  const to = datasets.findIndex((dataset) => dataset.id === overId);
  if (from < 0 || to < 0 || from === to) return datasets;
  const next = [...datasets];
  const [active] = next.splice(from, 1);
  if (!active) return datasets;
  next.splice(to, 0, active);
  return next;
}

function moveDatasetByOffset(datasets: DatasetSummary[], datasetId: string, offset: number) {
  const from = datasets.findIndex((dataset) => dataset.id === datasetId);
  const over = datasets[from + offset];
  return over ? moveDataset(datasets, datasetId, over.id) : datasets;
}

async function persistIfChanged(
  current: DatasetSummary[],
  previous: DatasetSummary[],
  persist: (datasetIds: string[]) => Promise<void>,
) {
  const ids = current.map((dataset) => dataset.id);
  if (ids.some((id, index) => id !== previous[index]?.id)) await persist(ids);
}
