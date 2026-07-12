// Age is shown as a bucket, not a continuum — four distinct steps read at a
// glance while scrolling; a smooth gradient wouldn't.
export function clipAge(unixSeconds: number): { bar: string; opacity: number } {
  const hours = (Date.now() / 1000 - unixSeconds) / 3600;
  if (hours < 1) return { bar: "var(--primary)", opacity: 1 };
  if (hours < 24) return { bar: "color-mix(in srgb, var(--primary) 55%, transparent)", opacity: 0.95 };
  if (hours < 24 * 7) return { bar: "color-mix(in srgb, var(--primary) 25%, transparent)", opacity: 0.87 };
  return { bar: "color-mix(in srgb, var(--primary) 10%, transparent)", opacity: 0.78 };
}

export function timeLeft(expiresAtUnixSeconds: number): string {
  const secs = expiresAtUnixSeconds - Date.now() / 1000;
  if (secs <= 0) return "expiring…";
  if (secs < 3600) return `${Math.max(1, Math.round(secs / 60))}m left`;
  if (secs < 24 * 3600) return `${Math.round(secs / 3600)}h left`;
  return `${Math.round(secs / (24 * 3600))}d left`;
}
