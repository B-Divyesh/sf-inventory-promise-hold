export const productSlug = 'inventory-promise-hold';
const key = `sb_license:${productSlug}`;
const verdictKey = `${key}:verdict`;

export interface LicenseState { unlocked: boolean; notice: string; token: string | null }

export function captureLicense(): void {
  const url = new URL(location.href);
  const token = url.searchParams.get('license');
  if (!token) return;
  localStorage.setItem(key, token);
  url.searchParams.delete('license');
  history.replaceState({}, '', url.pathname + url.search + url.hash);
}

export function storeLicense(token: string): void {
  localStorage.setItem(key, token.trim());
  localStorage.removeItem(verdictKey);
}

export async function checkLicense(force = false): Promise<LicenseState> {
  const token = localStorage.getItem(key);
  if (!token) return { unlocked: false, notice: '', token: null };
  const cached = JSON.parse(localStorage.getItem(verdictKey) || 'null') as { valid: boolean; checked: number } | null;
  const fresh = cached && Date.now() - cached.checked < 86_400_000;
  if (fresh && !force) return { unlocked: cached.valid, notice: cached.valid ? '' : 'License no longer active.', token };
  try {
    const response = await fetch(`https://api.sociobot.in/api/v1/products/${productSlug}/verify?license=${encodeURIComponent(token)}`);
    const verdict = await response.json() as { valid: boolean };
    localStorage.setItem(verdictKey, JSON.stringify({ valid: verdict.valid, checked: Date.now() }));
    return { unlocked: verdict.valid, notice: verdict.valid ? '' : 'License no longer active.', token };
  } catch {
    return { unlocked: cached?.valid ?? false, notice: 'License check will retry when you are online.', token };
  }
}
