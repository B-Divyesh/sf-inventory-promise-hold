import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig, loadEnv } from 'vite';

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, '.', '');
  return {
  root: 'frontend',
  define: {
    'import.meta.env.VITE_BUILD_SHA': JSON.stringify(env.BUILD_SHA || env.VITE_BUILD_SHA || 'dev'),
  },
  plugins: [svelte()],
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
