export interface RouteMetadata {
  title: string;
  description: string;
  canonicalPath: string;
}

const origin = 'https://inventory-promise-hold.sociobot.in';

export function metadataForPath(path: string): RouteMetadata {
  switch (path) {
    case '/demo':
      return {
        title: 'Demo — Stock Promise',
        description: 'Try sample inventory holds without changing a live stockroom.',
        canonicalPath: '/demo',
      };
    case '/privacy':
      return {
        title: 'Privacy — Stock Promise',
        description: 'How Stock Promise stores, retains, and erases shared stockroom data.',
        canonicalPath: '/privacy',
      };
    case '/terms':
      return {
        title: 'Terms — Stock Promise',
        description: 'Terms for using Stock Promise to coordinate temporary inventory holds.',
        canonicalPath: '/terms',
      };
    case '/404':
      return {
        title: 'Page not found — Stock Promise',
        description: 'Return to Stock Promise or open the sample stockroom.',
        canonicalPath: '/404',
      };
    default:
      return {
        title: 'Stock Promise — timed inventory holds',
        description: 'Create timed, shared inventory holds so scarce stock is not promised twice.',
        canonicalPath: '/',
      };
  }
}

function setMeta(selector: string, value: string): void {
  const element = document.head.querySelector<HTMLMetaElement>(selector);
  element?.setAttribute('content', value);
}

/** Keep the one static document head authoritative as SPA routes change. */
export function applyRouteMetadata(path: string): void {
  const metadata = metadataForPath(path);
  document.title = metadata.title;
  document.head.querySelector<HTMLLinkElement>('link[rel="canonical"]')
    ?.setAttribute('href', `${origin}${metadata.canonicalPath}`);
  setMeta('meta[name="description"]', metadata.description);
  setMeta('meta[property="og:title"]', metadata.title);
  setMeta('meta[property="og:description"]', metadata.description);
  setMeta('meta[name="twitter:title"]', metadata.title);
  setMeta('meta[name="twitter:description"]', metadata.description);
}
