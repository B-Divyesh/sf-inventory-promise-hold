export function relativeExpiry(expiresAt: number, now = Date.now()): string {
  const seconds = Math.max(0, expiresAt - Math.floor(now / 1000));
  if (seconds === 0) return 'Expired — refreshing';
  if (seconds < 60) return `Less than a minute left`;
  const minutes = Math.ceil(seconds / 60);
  if (minutes < 60) return `${minutes} min left`;
  const hours = Math.floor(minutes / 60);
  const remaining = minutes % 60;
  return remaining ? `${hours} hr ${remaining} min left` : `${hours} hr left`;
}

export function formatTime(value: number): string {
  return new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(value * 1000);
}

