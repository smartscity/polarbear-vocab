interface ProgressStatusProps {
  label: string;
}

export function ProgressStatus({ label }: ProgressStatusProps) {
  return (
    <div aria-busy="true" aria-live="polite" className="pb-progress-status" role="status">
      <span>{label}</span>
      <progress aria-label={label} />
    </div>
  );
}
