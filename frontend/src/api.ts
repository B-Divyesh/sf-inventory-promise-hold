export interface InventoryItem {
  id: number;
  sku: string;
  name: string;
  on_hand: number;
  held: number;
  available: number;
}

export type HoldStatus = 'active' | 'converted' | 'released' | 'expired';
export interface Hold {
  id: string;
  inventory_id: number;
  sku: string;
  item_name: string;
  quantity: number;
  customer: string;
  order_note: string;
  operator_name: string;
  status: HoldStatus;
  created_at: number;
  expires_at: number;
  resolved_at: number | null;
  resolved_by: string | null;
}

export interface Bootstrap {
  setup_required: boolean;
  location_name: string | null;
  server_time: number;
  inventory: InventoryItem[];
  active_holds: Hold[];
  recent_outcomes: Hold[];
}

const sessionKey = 'stock-promise:supervisor-session';

export class ResponseError extends Error {
  constructor(message: string, public status: number) { super(message); }
}

export function getSession(): string | null {
  return sessionStorage.getItem(sessionKey);
}

export function setSession(token: string | null): void {
  if (token) sessionStorage.setItem(sessionKey, token);
  else sessionStorage.removeItem(sessionKey);
}

export async function request<T>(path: string, init: RequestInit = {}, access: 'none' | 'optional' | 'required' = 'none'): Promise<T> {
  const headers = new Headers(init.headers);
  if (init.body && !headers.has('content-type')) headers.set('content-type', 'application/json');
  const token = getSession();
  if (token && access !== 'none') headers.set('authorization', `Bearer ${token}`);
  if (!token && access === 'required') throw new ResponseError('Enter the supervisor PIN to continue.', 401);
  let response: Response;
  try {
    response = await fetch(path, { ...init, headers });
  } catch {
    throw new Error('Stock Promise cannot reach the shared server. Check this device’s connection and try again.');
  }
  if (!response.ok) {
    const payload = await response.json().catch(() => null);
    if (response.status === 401) setSession(null);
    throw new ResponseError(payload?.error || `The server returned ${response.status}. Try again.`, response.status);
  }
  if (response.status === 204) return undefined as T;
  return response.json() as Promise<T>;
}
