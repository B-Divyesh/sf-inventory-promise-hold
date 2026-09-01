import { mount } from 'svelte';
import App from './App.svelte';
import './styles.css';

mount(App, { target: document.getElementById('app')! });

if ('serviceWorker' in navigator && import.meta.env.PROD) {
  const version = import.meta.env.VITE_BUILD_SHA || 'dev';
  window.addEventListener('load', () => navigator.serviceWorker.register(`/sw.js?v=${encodeURIComponent(version)}`));
}
