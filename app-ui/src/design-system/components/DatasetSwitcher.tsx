import type { DatasetSummary } from "../../lib/commands";

interface DatasetSwitcherProps {
  datasets: DatasetSummary[];
  label: string;
  onChange: (datasetId: string) => void;
  value: string;
}

export function DatasetSwitcher({ datasets, label, onChange, value }: DatasetSwitcherProps) {
  return (
    <label className="dataset-picker">
      <span>{label}</span>
      <select onChange={(event) => onChange(event.target.value)} value={value}>
        {datasets.map((dataset) => <option key={dataset.id} value={dataset.id}>{dataset.name}</option>)}
      </select>
    </label>
  );
}
