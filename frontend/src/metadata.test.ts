import { describe, expect, it } from 'vitest';
import { applyRouteMetadata, metadataForPath } from './metadata';

describe('route metadata', () => {
  it('uses a unique title, description, and canonical route for every public page', () => {
    for (const [path, canonical] of [
      ['/', '/'],
      ['/demo', '/demo'],
      ['/privacy', '/privacy'],
      ['/terms', '/terms'],
      ['/404', '/404'],
    ] as const) {
      const metadata = metadataForPath(path);
      expect(metadata.canonicalPath).toBe(canonical);
      expect(metadata.title).toBeTruthy();
      expect(metadata.description).toBeTruthy();
    }
    expect(metadataForPath('/demo').canonicalPath).not.toBe(metadataForPath('/').canonicalPath);
    expect(metadataForPath('/privacy').description).not.toBe(metadataForPath('/').description);
  });

  it('updates the existing head tags instead of appending duplicates', () => {
    document.head.innerHTML = `
      <title>Stock Promise — timed inventory holds</title>
      <meta name="description" content="home" />
      <link rel="canonical" href="https://inventory-promise-hold.sociobot.in/" />
      <meta property="og:title" content="home" />
      <meta property="og:description" content="home" />
      <meta name="twitter:title" content="home" />
      <meta name="twitter:description" content="home" />`;

    applyRouteMetadata('/privacy');

    expect(document.title).toBe('Privacy — Stock Promise');
    expect(document.head.querySelectorAll('link[rel="canonical"]')).toHaveLength(1);
    expect(document.head.querySelectorAll('meta[name="description"]')).toHaveLength(1);
    expect(document.head.querySelector('link[rel="canonical"]')?.getAttribute('href'))
      .toBe('https://inventory-promise-hold.sociobot.in/privacy');
  });
});
