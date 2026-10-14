import React from "react";
import { AlertCircle, AlertTriangle, Info, CheckCircle } from "lucide-react";

type AlertVariant = "error" | "warning" | "info" | "success";

interface AlertProps {
  variant?: AlertVariant;
  /** When true, removes rounded corners for use inside containers */
  contained?: boolean;
  children: React.ReactNode;
  className?: string;
  /** Optional first line, in medium weight; its test id is `${testId}-title`. */
  title?: React.ReactNode;
  /** Optional control shown on the right, such as a Reload button. */
  action?: React.ReactNode;
  testId?: string;
  role?: "alert" | "status";
}

const variantStyles: Record<
  AlertVariant,
  { container: string; icon: string; text: string }
> = {
  error: {
    container: "bg-red-500/10",
    icon: "text-red-500",
    text: "text-red-400",
  },
  warning: {
    container: "bg-yellow-500/10",
    icon: "text-yellow-500",
    text: "text-yellow-400",
  },
  info: {
    container: "bg-blue-500/10",
    icon: "text-blue-500",
    text: "text-blue-400",
  },
  success: {
    container: "bg-green-500/10",
    icon: "text-green-500",
    text: "text-green-400",
  },
};

const variantIcons: Record<AlertVariant, React.ElementType> = {
  error: AlertCircle,
  warning: AlertTriangle,
  info: Info,
  success: CheckCircle,
};

/**
 * Colored message with its icon. With a title or an action it becomes a titled alert: the
 * title, then the message, and the action on the right.
 */
export const Alert: React.FC<AlertProps> = ({
  variant = "error",
  contained = false,
  children,
  className = "",
  title,
  action,
  testId,
  role,
}) => {
  const styles = variantStyles[variant];
  const Icon = variantIcons[variant];
  const hasAction = action !== undefined && action !== null && action !== false;
  const isTitled = title !== undefined || hasAction;

  return (
    <div
      role={role}
      data-testid={testId}
      className={`flex items-start gap-3 p-4 ${styles.container} ${contained ? "" : "rounded-lg"} ${className}`}
    >
      <Icon className={`w-5 h-5 shrink-0 mt-0.5 ${styles.icon}`} />
      {isTitled ? (
        <div className={`flex-1 min-w-0 text-sm ${styles.text}`}>
          {title !== undefined && (
            <p
              className="font-medium"
              data-testid={testId ? `${testId}-title` : undefined}
            >
              {title}
            </p>
          )}
          <div>{children}</div>
        </div>
      ) : (
        <p className={`text-sm ${styles.text}`}>{children}</p>
      )}
      {hasAction && <div className="shrink-0">{action}</div>}
    </div>
  );
};
