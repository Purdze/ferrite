export function formatDate(ts: number) {
  return new Date(ts * 1000).toLocaleDateString();
}

export function formatPastRelativeDate(ts: number, withAgo = false) {
  const now = Date.now();
  const diffMs = now - ts * 1000;

  const minutes = Math.floor(diffMs / (1000 * 60));
  const hours = Math.floor(diffMs / (1000 * 60 * 60));
  const days = Math.floor(diffMs / (1000 * 60 * 60 * 24));
  const weeks = Math.floor(days / 7);

  const nowDate = new Date();
  const pastDate = new Date(ts * 1000);

  const months =
    (nowDate.getFullYear() - pastDate.getFullYear()) * 12 +
    (nowDate.getMonth() - pastDate.getMonth());

  let result: string;

  if (minutes < 1) return "now";

  if (minutes < 60) result = `${minutes} min`;
  else if (hours < 24) result = `${hours} h`;
  else if (days === 1) return "yesterday";
  else if (days < 7) result = `${days} days`;
  else if (weeks === 1) result = `1 week`;
  else if (weeks < 4) result = `${weeks} weeks`;
  else if (months === 1) result = `1 month`;
  else if (months < 4) result = `${months} months`;
  else return pastDate.toLocaleDateString();

  return withAgo ? `${result} ago` : result;
}

export function formatRelativeDate(ts: number) {
  const now = Date.now();
  const diffMs = ts * 1000 - now;
  const isPast = diffMs < 0;

  const absMs = Math.abs(diffMs);

  const minutes = Math.floor(absMs / (1000 * 60));
  const hours = Math.floor(absMs / (1000 * 60 * 60));
  const days = Math.floor(absMs / (1000 * 60 * 60 * 24));
  const weeks = Math.floor(days / 7);

  const nowDate = new Date();
  const date = new Date(ts * 1000);

  const months =
    (date.getFullYear() - nowDate.getFullYear()) * 12 + (date.getMonth() - nowDate.getMonth());

  const wrap = (text: string) => (isPast ? `${text} ago` : `in ${text}`);

  if (minutes < 1) return "now";

  if (minutes < 60) return wrap(`${minutes} min`);
  if (hours < 24) return wrap(`${hours} h`);

  if (days === 1) return isPast ? "yesterday" : "tomorrow";
  if (days < 7) return wrap(`${days} days`);

  if (weeks === 1) return wrap("1 week");
  if (weeks < 4) return wrap(`${weeks} weeks`);

  if (Math.abs(months) === 1) return wrap("1 month");
  if (Math.abs(months) < 4) return wrap(`${Math.abs(months)} months`);

  return date.toLocaleDateString();
}
