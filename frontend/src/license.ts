export const productSlug = 'inventory-promise-hold';
const liveKey = `sb_license:${productSlug}`;
const demoKey = 'demo:stock-promise:license';

export interface LicenseState { unlocked: boolean; notice: string; token: string | null }

function licenseStore(demo: boolean): { storage: Storage; key: string; verdictKey: string } {
  const key = demo ? demoKey : liveKey;
  return {
    storage: demo ? sessionStorage : localStorage,
    key,
    verdictKey: `${key}:verdict`,
  };
}

export function captureLicense(demo = false): void {
  const url = new URL(location.href);
  const token = url.searchParams.get('license');
  if (!token) return;
  const { storage, key } = licenseStore(demo);
  storage.setItem(key, token);
  url.searchParams.delete('license');
  history.replaceState({}, '', url.pathname + url.search + url.hash);
}

export function storeLicense(token: string, demo = false): void {
  const { storage, key, verdictKey } = licenseStore(demo);
  storage.setItem(key, token.trim());
  storage.removeItem(verdictKey);
}

export async function checkLicense(force = false, demo = false): Promise<LicenseState> {
  const { storage, key, verdictKey } = licenseStore(demo);
  const token = storage.getItem(key);
  if (!token) return { unlocked: false, notice: '', token: null };
  let cached: { valid: boolean; checked: number } | null = null;
  try {
    cached = JSON.parse(storage.getItem(verdictKey) || 'null') as { valid: boolean; checked: number } | null;
  } catch {
    storage.removeItem(verdictKey);
  }
  if (cached && Date.now() - cached.checked < 86_400_000 && !force) {
    return { unlocked: cached.valid, notice: cached.valid ? '' : 'License no longer active.', token };
  }
  try {
    const response = await fetch(`https://api.sociobot.in/api/v1/products/${productSlug}/verify?license=${encodeURIComponent(token)}`);
    const verdict = await response.json() as { valid: boolean };
    storage.setItem(verdictKey, JSON.stringify({ valid: verdict.valid, checked: Date.now() }));
    return { unlocked: verdict.valid, notice: verdict.valid ? '' : 'License no longer active.', token };
  } catch {
    return { unlocked: cached?.valid ?? false, notice: 'License check will retry when you are online.', token };
  }
}
