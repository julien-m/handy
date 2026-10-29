import React from "react";
import SelectComponent from "react-select";
import CreatableSelect from "react-select/creatable";
import type {
  ActionMeta,
  Props as ReactSelectProps,
  SingleValue,
  StylesConfig,
} from "react-select";

export type SelectOption = {
  value: string;
  label: string;
  isDisabled?: boolean;
};

type BaseProps = {
  value: string | null;
  options: SelectOption[];
  placeholder?: string;
  disabled?: boolean;
  isLoading?: boolean;
  isClearable?: boolean;
  onChange: (value: string | null, action: ActionMeta<SelectOption>) => void;
  onBlur?: () => void;
  className?: string;
  /** Accessible name when no label element is tied to the control. */
  ariaLabel?: string;
  formatCreateLabel?: (input: string) => string;
};

type CreatableProps = {
  isCreatable: true;
  onCreateOption: (value: string) => void;
};

type NonCreatableProps = {
  isCreatable?: false;
  onCreateOption?: never;
};

export type SelectProps = BaseProps & (CreatableProps | NonCreatableProps);

const baseBackground =
  "color-mix(in srgb, var(--color-mid-gray) 10%, transparent)";
const hoverBackground =
  "color-mix(in srgb, var(--color-logo-primary) 12%, transparent)";
const focusBackground =
  "color-mix(in srgb, var(--color-logo-primary) 20%, transparent)";
const neutralBorder =
  "color-mix(in srgb, var(--color-mid-gray) 80%, transparent)";

/**
 * Box of the shared `Dropdown` button (`px-2 py-[5px] text-sm rounded-md`, 1px border): the
 * select uses the same height, padding, radius and chevron size so both sit on one row
 * (005 prompt editor: provider and model side by side).
 */
// Line height of `text-sm` (1.25rem) + 2 × 5px padding + 2 × 1px border, like the button.
const CONTROL_HEIGHT = "calc(1.25rem + 12px)";
// Text inset of the button (`px-2`): value container 4px + value margin 4px.
const VALUE_INSET = 4;
// Vertical room of the indicators, small enough to keep the control height.
const INDICATOR_PADDING_BLOCK = 4;
const INDICATOR_SIZE = "1rem"; // `w-4 h-4` chevron of the button

const selectStyles: StylesConfig<SelectOption, false> = {
  control: (base, state) => ({
    ...base,
    minHeight: CONTROL_HEIGHT,
    borderRadius: "0.375rem",
    borderColor: state.isFocused ? "var(--color-logo-primary)" : neutralBorder,
    boxShadow: state.isFocused ? "0 0 0 1px var(--color-logo-primary)" : "none",
    backgroundColor: state.isFocused ? focusBackground : baseBackground,
    fontSize: "0.875rem",
    color: "var(--color-text)",
    transition: "all 150ms ease",
    ":hover": {
      borderColor: "var(--color-logo-primary)",
      backgroundColor: hoverBackground,
    },
  }),
  valueContainer: (base) => ({
    ...base,
    paddingInline: VALUE_INSET,
    // No block padding: the input already brings 2 × (2px padding + 2px margin).
    paddingBlock: "0px",
  }),
  input: (base) => ({
    ...base,
    color: "var(--color-text)",
  }),
  singleValue: (base) => ({
    ...base,
    marginInline: VALUE_INSET,
    fontWeight: 600,
    color: "var(--color-text)",
  }),
  dropdownIndicator: (base, state) => ({
    ...base,
    // Chevron then the button's `px-2` right padding.
    paddingBlock: INDICATOR_PADDING_BLOCK,
    paddingInline: `${VALUE_INSET}px ${2 * VALUE_INSET}px`,
    "& svg": { width: INDICATOR_SIZE, height: INDICATOR_SIZE },
    color: state.isFocused
      ? "var(--color-logo-primary)"
      : "color-mix(in srgb, var(--color-mid-gray) 80%, transparent)",
    ":hover": {
      color: "var(--color-logo-primary)",
    },
  }),
  clearIndicator: (base) => ({
    ...base,
    paddingBlock: INDICATOR_PADDING_BLOCK,
    paddingInline: VALUE_INSET,
    "& svg": { width: INDICATOR_SIZE, height: INDICATOR_SIZE },
    color: "color-mix(in srgb, var(--color-mid-gray) 80%, transparent)",
    ":hover": {
      color: "var(--color-logo-primary)",
    },
  }),
  menu: (provided) => ({
    ...provided,
    zIndex: 30,
    backgroundColor: "var(--color-background)",
    color: "var(--color-text)",
    border:
      "1px solid color-mix(in srgb, var(--color-mid-gray) 30%, transparent)",
    boxShadow: "0 10px 30px rgba(15, 15, 15, 0.2)",
  }),
  option: (base, state) => ({
    ...base,
    backgroundColor: state.isSelected
      ? focusBackground
      : state.isFocused
        ? hoverBackground
        : "transparent",
    color: "var(--color-text)",
    cursor: state.isDisabled ? "not-allowed" : base.cursor,
    opacity: state.isDisabled ? 0.5 : 1,
  }),
  placeholder: (base) => ({
    ...base,
    marginInline: VALUE_INSET,
    overflow: "hidden",
    textOverflow: "ellipsis",
    whiteSpace: "nowrap",
    color: "color-mix(in srgb, var(--color-mid-gray) 65%, transparent)",
  }),
};

export const Select: React.FC<SelectProps> = React.memo(
  ({
    value,
    options,
    placeholder,
    disabled,
    isLoading,
    isClearable = true,
    onChange,
    onBlur,
    className = "",
    ariaLabel,
    isCreatable,
    formatCreateLabel,
    onCreateOption,
  }) => {
    const selectValue = React.useMemo(() => {
      if (!value) return null;
      const existing = options.find((option) => option.value === value);
      if (existing) return existing;
      return { value, label: value, isDisabled: false };
    }, [value, options]);

    const handleChange = (
      option: SingleValue<SelectOption>,
      action: ActionMeta<SelectOption>,
    ) => {
      onChange(option?.value ?? null, action);
    };

    const sharedProps: Partial<ReactSelectProps<SelectOption, false>> = {
      className,
      classNamePrefix: "app-select",
      value: selectValue,
      options,
      onChange: handleChange,
      placeholder,
      isDisabled: disabled,
      isLoading,
      onBlur,
      isClearable,
      styles: selectStyles,
      // No separator: the button of `Dropdown` shows only its chevron.
      components: { IndicatorSeparator: null },
      "aria-label": ariaLabel,
    };

    if (isCreatable) {
      return (
        <CreatableSelect<SelectOption, false>
          {...sharedProps}
          onCreateOption={onCreateOption}
          formatCreateLabel={formatCreateLabel}
        />
      );
    }

    return <SelectComponent<SelectOption, false> {...sharedProps} />;
  },
);

Select.displayName = "Select";
