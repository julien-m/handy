import React from "react";

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  variant?: "default" | "compact";
  /** Shows the value as refused: red border; pair it with `aria-invalid` text. */
  invalid?: boolean;
}

export const Input: React.FC<InputProps> = ({
  className = "",
  variant = "default",
  disabled,
  invalid = false,
  ...props
}) => {
  const borderClass = invalid ? "border-red-500" : "border-mid-gray/80";
  const baseClasses = `px-2 py-1 text-sm font-semibold bg-mid-gray/10 border ${borderClass} rounded-md text-start transition-all duration-150`;

  const interactiveClasses = disabled
    ? "opacity-60 cursor-not-allowed bg-mid-gray/10 border-mid-gray/40"
    : invalid
      ? "focus:outline-none focus:bg-logo-primary/20"
      : "hover:bg-logo-primary/10 hover:border-logo-primary focus:outline-none focus:bg-logo-primary/20 focus:border-logo-primary";

  const variantClasses = {
    default: "px-3 py-2",
    compact: "px-2 py-1",
  } as const;

  return (
    <input
      className={`${baseClasses} ${variantClasses[variant]} ${interactiveClasses} ${className}`}
      disabled={disabled}
      aria-invalid={invalid || undefined}
      {...props}
    />
  );
};
