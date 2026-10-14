import React, { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { dropdownKeyAction, initialHighlight } from "./dropdown-navigation";

export interface DropdownOption {
  value: string;
  label: string;
  description?: string;
  disabled?: boolean;
}

interface DropdownProps {
  options: DropdownOption[];
  className?: string;
  menuClassName?: string;
  selectedValue: string | null;
  onSelect: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  onOpen?: () => void;
  /** Accessible name of the control (the setting title). */
  ariaLabel?: string;
  /** Test identifier of the control button. */
  testId?: string;
}

export const Dropdown: React.FC<DropdownProps> = ({
  options,
  selectedValue,
  onSelect,
  className = "",
  menuClassName,
  placeholder = "Select an option...",
  disabled = false,
  onOpen,
  ariaLabel,
  testId,
}) => {
  const { t } = useTranslation();
  const [isOpen, setIsOpen] = useState(false);
  const [highlighted, setHighlighted] = useState(-1);
  const listId = useId();
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const selectedOption = options.find(
    (option) => option.value === selectedValue,
  );

  const handleSelect = (value: string) => {
    onSelect(value);
    setIsOpen(false);
  };

  // Select-only combobox: focus stays on the button, the highlight moves over the options.
  const handleKeyDown = (event: React.KeyboardEvent<HTMLButtonElement>) => {
    if (disabled) return;
    const action = dropdownKeyAction(
      event.key,
      isOpen,
      highlighted,
      options,
      selectedValue,
    );
    if (action.kind === "ignore") return;
    // Tab keeps its default so focus moves on in reading order.
    if (event.key !== "Tab") event.preventDefault();
    if (action.kind === "open") onOpen?.();
    if (action.kind === "open" || action.kind === "move") {
      setIsOpen(true);
      setHighlighted(action.highlighted);
    } else if (action.kind === "select") {
      handleSelect(action.value);
    } else {
      setIsOpen(false);
    }
  };

  const handleToggle = () => {
    if (disabled) return;
    if (!isOpen) {
      onOpen?.();
      setHighlighted(initialHighlight(options, selectedValue));
    }
    setIsOpen(!isOpen);
  };

  return (
    <div className={`relative ${className}`} ref={dropdownRef}>
      <button
        type="button"
        className={`px-2 py-[5px] text-sm font-semibold bg-mid-gray/10 border border-mid-gray/80 rounded-md min-w-[200px] w-full text-start grid grid-cols-[1fr_auto] gap-2 items-center transition-all duration-150 ${
          disabled
            ? "opacity-50 cursor-not-allowed"
            : "hover:bg-logo-primary/10 cursor-pointer hover:border-logo-primary"
        } focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-logo-primary`}
        onClick={handleToggle}
        onKeyDown={handleKeyDown}
        disabled={disabled}
        role="combobox"
        data-testid={testId}
        aria-label={ariaLabel}
        aria-haspopup="listbox"
        aria-expanded={isOpen}
        aria-controls={listId}
        aria-activedescendant={
          isOpen && highlighted >= 0 ? `${listId}-${highlighted}` : undefined
        }
      >
        <span className="truncate">{selectedOption?.label || placeholder}</span>
        <svg
          className={`w-4 h-4 transition-transform duration-200 ${isOpen ? "transform rotate-180" : ""}`}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M19 9l-7 7-7-7"
          />
        </svg>
      </button>
      {isOpen && !disabled && (
        <div
          id={listId}
          role="listbox"
          aria-label={ariaLabel}
          className={`absolute top-full mt-1 bg-background border border-mid-gray/80 rounded-md shadow-lg z-50 max-h-60 overflow-y-auto ${
            menuClassName ?? "left-0 right-0"
          }`}
        >
          {options.length === 0 ? (
            <div className="px-2 py-1 text-sm text-mid-gray">
              {t("common.noOptionsFound")}
            </div>
          ) : (
            options.map((option, index) => (
              <button
                key={option.value}
                id={`${listId}-${index}`}
                type="button"
                role="option"
                aria-selected={selectedValue === option.value}
                aria-disabled={option.disabled}
                tabIndex={-1}
                onMouseEnter={() => setHighlighted(index)}
                className={`w-full text-sm text-start hover:bg-logo-primary/10 transition-colors duration-150 ${
                  option.description ? "px-3 py-2" : "px-2 py-1"
                } ${
                  selectedValue === option.value ? "bg-logo-primary/20" : ""
                } ${highlighted === index ? "ring-1 ring-inset ring-logo-primary" : ""} ${option.disabled ? "opacity-50 cursor-not-allowed" : ""}`}
                onClick={() => handleSelect(option.value)}
                disabled={option.disabled}
              >
                <span
                  className={`block whitespace-normal break-words ${
                    option.description || selectedValue === option.value
                      ? "font-semibold"
                      : ""
                  }`}
                >
                  {option.label}
                </span>
                {option.description && (
                  <span className="mt-0.5 block whitespace-normal text-xs font-normal leading-snug text-mid-gray">
                    {option.description}
                  </span>
                )}
              </button>
            ))
          )}
        </div>
      )}
    </div>
  );
};
