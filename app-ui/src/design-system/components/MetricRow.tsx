interface Metric {
  label: string;
  onClick: () => void;
  value: number;
}

export function MetricRow({ metrics }: { metrics: Metric[] }) {
  return (
    <div className="pb-metric-row">
      {metrics.map((metric) => (
        <button className="pb-metric" disabled={metric.value === 0} key={metric.label} onClick={metric.onClick} type="button">
          <span>{metric.label}</span><strong>{metric.value.toLocaleString()}</strong>
        </button>
      ))}
    </div>
  );
}
