import type { AccountInfo, PublicClientApplication } from '@azure/msal-browser';

export type AuthMode = 'local' | 'ciam';

const tenant = '35c6fe40-0ec0-46b6-98c6-213ad4de6650';
const clientId = '25c704f4-465a-47af-80ab-2c489466b697';
const scopes = ['openid', 'profile', 'email'];
let mode: AuthMode = 'local';

let client: PublicClientApplication | null = null;

async function getClient(): Promise<PublicClientApplication> {
  if (client) return client;
  const { PublicClientApplication } = await import('@azure/msal-browser');
  client = new PublicClientApplication({
    auth: {
      clientId,
      authority: `https://sociobotcustomers.ciamlogin.com/${tenant}`,
      redirectUri: `${location.origin}/auth/callback`,
      navigateToLoginRequestUrl: true,
    },
    cache: { cacheLocation: 'sessionStorage' },
  });
  return client;
}

export async function configureAuth(next: AuthMode): Promise<void> {
  mode = next;
  if (mode === 'ciam') {
    const client = await getClient();
    await client.initialize();
    const result = await client.handleRedirectPromise();
    if (result?.account) client.setActiveAccount(result.account);
    if (!client.getActiveAccount()) {
      const [account] = client.getAllAccounts();
      if (account) client.setActiveAccount(account);
    }
  }
}

async function account(): Promise<AccountInfo | null> {
  const client = await getClient();
  return client.getActiveAccount() || client.getAllAccounts()[0] || null;
}

export async function accessToken(): Promise<string | null> {
  if (mode !== 'ciam') return null;
  const client = await getClient();
  const active = await account();
  if (!active) return null;
  try {
    return (await client.acquireTokenSilent({ scopes, account: active })).accessToken;
  } catch {
    return null;
  }
}

export async function signIn(): Promise<void> {
  const client = await getClient();
  return client.loginRedirect({ scopes });
}

export async function signOut(): Promise<void> {
  const client = await getClient();
  const active = await account();
  return client.logoutRedirect({ account: active || undefined, postLogoutRedirectUri: location.origin });
}

export function usesCiam(): boolean {
  return mode === 'ciam';
}
