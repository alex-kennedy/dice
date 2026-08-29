"use client";

import { FiCheck, FiShare2 } from "react-icons/fi";
import { toast } from "sonner";

export function ShareButton() {
  const copyLink = async () => {
    await navigator.clipboard.writeText(window.location.href);
    toast(
      <span className="flex items-center gap-1.5 whitespace-nowrap">
        Link copied to clipboard <FiCheck className="shrink-0" />
      </span>,
    );
  };

  return (
    <button
      type="button"
      onClick={copyLink}
      aria-label="Copy link to this comparison"
      className="text-muted-foreground hover:text-foreground"
    >
      <FiShare2 className="h-5 w-5" />
    </button>
  );
}
