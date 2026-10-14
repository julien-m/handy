import React from "react";

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  variant?: "default" | "compact";
  /** Shows the value as refused: red border; pair it with `aria-invalid` text. */
  invalid?: boolean;
}

const VARIANT_CLASSES = {
  default: "px-3 py-2",
  compact: "px-2 py-1",
} as const;

/** What the classes of an input depend on. */
interface InputLook {
  variant: NonNullable<InputProps["variant"]>;
  invalid: boolean;
  disabled: boolean | undefined;
  className: string;
}

/**
 * Classes of an input: base look, red border when refused, and the hover/focus look of an
 * enabled field (none when disabled; no hover accent on a refused value).
 * @param look variant, refused and disabled states, and extra classes
 * @returns the class string
 */
const inputClasses = ({
  variant,
  invalid,
  disabled,
  className,
}: InputLook): string => {
  const borderClass = invalid ? "border-red-500" : "border-mid-gray/80";
  const baseClasses = `px-2 py-1 text-sm font-semibold bg-mid-gray/10 border ${borderClass} rounded-md text-start transition-all duration-150`;
  const interactiveClasses = disabled
    ? "opacity-60 cursor-not-allowed bg-mid-gray/10 border-mid-gray/40"
    : invalid
      ? "focus:outline-none focus:bg-logo-primary/20"
      : "hover:bg-logo-primary/10 hover:border-logo-primary focus:outline-none focus:bg-logo-primary/20 focus:border-logo-primary";
  return `${baseClasses} ${VARIANT_CLASSES[variant]} ${interactiveClasses} ${className}`;
};

// Forwards its ref to the <input> so editors can move the focus (replacement rules, 002).
export const Input = React.forwardRef<HTMLInputElement, InputProps>(
  function Input(
    {
      className = "",
      variant = "default",
      disabled,
      invalid = false,
      ...props
    },
    ref,
  ) {
    return (
      <input
        ref={ref}
        className={inputClasses({ variant, invalid, disabled, className })}
        disabled={disabled}
        aria-invalid={invalid || undefined}
        {...props}
      />
    );
  },
);
