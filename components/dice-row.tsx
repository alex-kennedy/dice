import { Input } from "@/components/input";
import type { RowComputation } from "@/lib/distribution";
import { chartColor } from "@/lib/palette";

type DiceRowProps = {
  index: number;
  expression: string;
  result: RowComputation;
  disabled: boolean;
  onChange: (expression: string) => void;
  onRemove: () => void;
};

export function DiceRow({
  index,
  expression,
  result,
  disabled,
  onChange,
  onRemove,
}: DiceRowProps) {
  return (
    <div className="flex flex-col gap-1">
      <div className="flex items-center gap-2">
        <span
          className="h-2.5 w-2.5 shrink-0 rounded-full"
          style={{ backgroundColor: chartColor(index) }}
        />
        <Input
          value={expression}
          onChange={(event) => onChange(event.target.value)}
          placeholder="2d6 + 3"
          disabled={disabled}
        />
        <button
          type="button"
          onClick={onRemove}
          aria-label="Remove expression"
          className="text-muted-foreground hover:text-foreground"
        >
          ×
        </button>
      </div>

      {result.status === "error" && (
        <p className="pl-4.5 text-sm text-destructive">{result.message}</p>
      )}
      {result.status === "ok" && (
        <p className="pl-4.5 text-sm">
          min {result.minimum} · max {result.maximum} · most likely{" "}
          {result.mode}
        </p>
      )}
    </div>
  );
}
