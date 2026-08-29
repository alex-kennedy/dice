export function D20Icon({ className }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.5}
      strokeLinejoin="round"
      strokeLinecap="round"
      className={className}
    >
      <polygon points="12,2 21,7 21,17 12,22 3,17 3,7" />
      <polygon points="12,2 21,17 3,17" />
    </svg>
  );
}
