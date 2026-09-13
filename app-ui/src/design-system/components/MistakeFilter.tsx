interface FilterOption {
  label: string;
  lastWrongOnly: boolean;
  minimum: number;
}

interface MistakeFilterProps {
  lastWrongOnly: boolean;
  minimum: number;
  onChange: (minimum: number, lastWrongOnly: boolean) => void;
  options: FilterOption[];
}

export function MistakeFilter(props: MistakeFilterProps) {
  return (
    <div className="pb-filter-row">
      {props.options.map((option) => {
        const active = props.minimum === option.minimum && props.lastWrongOnly === option.lastWrongOnly;
        return <button className="pb-filter-chip" data-active={active} key={option.label} onClick={() => props.onChange(option.minimum, option.lastWrongOnly)} type="button">{option.label}</button>;
      })}
    </div>
  );
}
