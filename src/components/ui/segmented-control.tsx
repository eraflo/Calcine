import type * as React from "react";
import { cn } from "@/lib/utils";

export type SegmentedOption<T extends string> = {
  value: T;
  label: React.ReactNode;
  icon?: React.ComponentType<{ className?: string }>;
  /** Extra classes when selected, e.g. a compute-unit color. */
  activeClassName?: string;
  disabled?: boolean;
};

/** A row of mutually exclusive choices (radio inputs underneath). */
export function SegmentedControl<T extends string>({
  name,
  label,
  value,
  options,
  onChange,
  disabled = false,
  className,
}: {
  name: string;
  /** Read by screen readers. */
  label: string;
  value: T | null;
  options: readonly SegmentedOption<T>[];
  onChange: (value: T) => void;
  disabled?: boolean;
  className?: string;
}) {
  return (
    <fieldset
      className={cn("inline-flex gap-0.5 rounded-md border p-0.5", className)}
      disabled={disabled}
    >
      <legend className="sr-only">{label}</legend>
      {options.map((option) => {
        const active = value === option.value;
        const Icon = option.icon;
        return (
          <label
            key={option.value}
            className={cn(
              "inline-flex h-7 flex-1 cursor-pointer items-center justify-center gap-1.5 rounded-sm px-3 text-xs font-medium whitespace-nowrap text-muted-foreground transition-colors has-[:disabled]:cursor-not-allowed has-[:focus-visible]:ring-2 has-[:focus-visible]:ring-ring",
              active
                ? cn("bg-accent text-foreground", option.activeClassName)
                : "hover:text-foreground has-[:disabled]:opacity-40 has-[:disabled]:hover:text-muted-foreground",
            )}
          >
            <input
              type="radio"
              name={name}
              value={option.value}
              checked={active}
              disabled={option.disabled}
              onChange={() => onChange(option.value)}
              className="sr-only"
            />
            {Icon && <Icon className="size-3.5" />}
            {option.label}
          </label>
        );
      })}
    </fieldset>
  );
}
