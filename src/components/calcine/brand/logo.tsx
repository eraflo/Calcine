import { cn } from "@/lib/utils";

/** The Calcine mark: a Hexagon outline with an ember inside. */
export function LogoMark({ className }: { className?: string }) {
  return (
    <svg viewBox="200 160 624 704" className={cn("size-6", className)} aria-hidden="true">
      <path
        d="M512 212 L771.8 362 L771.8 662 L512 812 L252.2 662 L252.2 362 Z"
        fill="none"
        stroke="var(--primary)"
        strokeWidth="64"
        strokeLinejoin="round"
      />
      <path
        d="M512 352 C592 446 624 520 603 592 C588 650 548 684 512 684 C476 684 436 650 421 592 C400 520 432 446 512 352 Z"
        fill="var(--primary)"
      />
      <path
        d="M512 498 C551 544 562 584 552 616 C544 642 529 656 512 656 C495 656 480 642 472 616 C462 584 473 544 512 498 Z"
        fill="#FFB547"
      />
    </svg>
  );
}
