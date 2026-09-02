import type { AccountInfo, AuthenticationResult, PublicClientApplication } from '@azure/msal-browser';

export type AuthMode = 'local' | 'ciam';

const tenant = '35c6fe40-0ec0-46b6-98c6-213ad4de6650';
const clientId = '25c704f4-465a-47af-80ab-2c489466b697';
const scopes = ['openid', 'profile', 'email'];
let mode: AuthMode = 'local';

let client: PublicClientApplication | null = null;

function fixtureToken(claims: Record<string, unknown>): string {
  const encode = (value: unknown) => btoa(JSON.stringify(value))
    .replaceAll('+', '-').replaceAll('/', '_').replaceAll('=', '');
  return `${encode({ alg: 'none', typ: 'JWT' })}.${encode(claims)}.fixture`;
}

/**
 * The hosted-auth browser contract uses a build-only callback fixture. It calls
 * MSAL's real cache hydrator with the result of a stubbed identity exchange,
 * allowing the test to observe the exact browser storage MSAL uses. This branch
 * is absent from release builds because VITE_HOSTED_AUTH_FIXTURE is never set.
 */
async function completeHostedFixture(nextClient: PublicClientApplication): Promise<void> {
  if (import.meta.env.VITE_HOSTED_AUTH_FIXTURE !== '1') return;
  if (new URL(location.href).searchParams.get('test-hosted-auth') !== '1') return;

  const now = Math.floor(Date.now() / 1000);
  const claims = {
    aud: clientId,
    exp: now + 3600,
    iat: now,
    iss: `https://sociobotcustomers.ciamlogin.com/${tenant}/v2.0`,
    name: 'Hosted fixture staff',
    oid: 'hosted-fixture-user',
    preferred_username: 'fixture.staff@example.test',
    roles: ['stockpromise.staff'],
    sub: 'hosted-fixture-user',
    tid: tenant,
  };
  const idToken = fixtureToken(claims);
  const account: AccountInfo = {
    homeAccountId: `${claims.oid}.${tenant}`,
    environment: 'sociobotcustomers.ciamlogin.com',
    tenantId: tenant,
    username: claims.preferred_username,
    localAccountId: claims.oid,
    name: claims.name,
    idToken,
    idTokenClaims: claims,
  };
  const exchange: AuthenticationResult = {
    authority: `https://sociobotcustomers.ciamlogin.com/${tenant}`,
    uniqueId: claims.oid,
    tenantId: tenant,
    scopes,
    account,
    idToken,
    idTokenClaims: claims,
    accessToken: 'hosted-fixture-access-token',
    fromCache: false,
    expiresOn: new Date((now + 3600) * 1000),
    tokenType: 'Bearer',
    correlationId: 'hosted-storage-fixture',
  };
  await nextClient.hydrateCache(exchange, { scopes });
  nextClient.setActiveAccount(account);
}

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
    await completeHostedFixture(client);
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
