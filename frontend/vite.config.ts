import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig, loadEnv } from 'vite';
import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, '.', '');
  const buildSha = env.BUILD_SHA || env.VITE_BUILD_SHA || process.env.BUILD_SHA || process.env.VITE_BUILD_SHA || 'dev';
  return {
  root: 'frontend',
  define: {
    'import.meta.env.VITE_BUILD_SHA': JSON.stringify(buildSha),
  },
  plugins: [
    svelte(),
    {
      name: 'inject-404-build-id',
      async closeBundle() {
        const output = resolve('dist/404.html');
        const html = await readFile(output, 'utf8');
        await writeFile(output, html.replace('__BUILD_SHA__', buildSha.slice(0, 12)));
      },
    },
  ],
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    target: 'es2022',
  },
  server: {
    host: '0.0.0.0',
    proxy: { '/api': 'http://localhost:8080', '/health': 'http://localhost:8080' },
  },
  test: {
    environment: 'jsdom',
    include: ['src/**/*.test.ts'],
  },
  };
});
