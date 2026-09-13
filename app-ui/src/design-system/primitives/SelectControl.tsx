import { Select } from "radix-ui";

interface SelectOption<T extends string> {
  label: string;
  value: T;
}

interface SelectControlProps<T extends string> {
  ariaLabel: string;
  onValueChange: (value: T) => void;
  options: Array<SelectOption<T>>;
  value: T;
}

export function SelectControl<T extends string>(props: SelectControlProps<T>) {
  return (
    <Select.Root onValueChange={(value) => props.onValueChange(value as T)} value={props.value}>
      <Select.Trigger aria-label={props.ariaLabel} className="pb-button pb-select-trigger">
        <Select.Value />
        <Select.Icon aria-hidden="true">⌄</Select.Icon>
      </Select.Trigger>
      <Select.Portal>
        <Select.Content className="pb-select-content" position="popper" sideOffset={6}>
          <Select.Viewport>
            {props.options.map((option) => (
              <Select.Item className="pb-select-item" key={option.value} value={option.value}>
                <Select.ItemText>{option.label}</Select.ItemText>
              </Select.Item>
            ))}
          </Select.Viewport>
        </Select.Content>
      </Select.Portal>
    </Select.Root>
  );
}
