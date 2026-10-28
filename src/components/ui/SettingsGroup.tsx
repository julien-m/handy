import React from "react";

interface SettingsGroupProps {
  title?: string;
  description?: string;
  /** Makes the title a programmatic focus point (tabIndex -1), the section's stable focus target. */
  titleRef?: React.Ref<HTMLHeadingElement>;
  children: React.ReactNode;
}

export const SettingsGroup: React.FC<SettingsGroupProps> = ({
  title,
  description,
  titleRef,
  children,
}) => {
  return (
    <div className="space-y-2">
      {title && (
        <div className="px-4">
          <h2
            ref={titleRef}
            tabIndex={titleRef ? -1 : undefined}
            className="text-xs font-medium text-mid-gray uppercase tracking-wide focus:outline-none"
          >
            {title}
          </h2>
          {description && (
            <p className="text-xs text-mid-gray mt-1">{description}</p>
          )}
        </div>
      )}
      <div className="bg-background border border-mid-gray/20 rounded-lg overflow-visible">
        <div className="divide-y divide-mid-gray/20">{children}</div>
      </div>
    </div>
  );
};
