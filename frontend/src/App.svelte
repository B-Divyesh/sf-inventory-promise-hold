<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { request, getSession, setSession, ResponseError, type Bootstrap, type InventoryItem, type Hold } from './api';
  import { configureAuth, signIn, signOut, usesCiam } from './auth';
  import { captureLicense, checkLicense, storeLicense, type LicenseState } from './license';
  import { applyRouteMetadata } from './metadata';
  import { formatTime, relativeExpiry } from './time';
  import Legal from './Legal.svelte';

  type Tab = 'desk' | 'outcomes' | 'settings';
  type Modal = 'hold' | 'unlock' | 'inventory' | 'import' | 'privacy' | null;

  function routeFromUrl(input = window.location.href): string {
    const url = new URL(input, window.location.origin);
    return url.searchParams.get('demo') === '1' ? '/demo' : url.pathname;
  }

  let path = routeFromUrl();
  let landing = path === '/';
  let demo = path === '/demo';
  let data: Bootstrap | null = null;
  let loading = true;
  let refreshing = false;
  let fatalError = '';
  let accessRequired = false;
  let online = navigator.onLine;
  let tab: Tab = 'desk';
  let query = '';
  let modal: Modal = null;
  let dialog: HTMLDialogElement;
  let modalOpener: HTMLElement | null = null;
  let selectedItem: InventoryItem | null = null;
  let editingItem: InventoryItem | null = null;
  let supervisor = demo || (!demo && Boolean(getSession()));
  let busy = '';
  let formError = '';
  let announcement = '';
  let now = Date.now();
  let license: LicenseState = { unlocked: false, notice: '', token: null };
  let operatorName = '';
  let supervisorName = '';
  let profiles: string[] = [];
  let reminders = false;
  let auditEntries: Array<Record<string, any>> = [];
  let importReport = '';
  let retentionDays = 90;
  const notified = new Set<string>();
  const buildId = import.meta.env.VITE_BUILD_SHA || 'dev';
  const demoPrefix = 'demo:stock-promise:';
  const demoKey = `${demoPrefix}state`;
  let licenseCheckController: AbortController | null = null;

  $: filteredInventory = (data?.inventory || []).filter((item) =>
    `${item.sku} ${item.name}`.toLowerCase().includes(query.trim().toLowerCase())
  );
  $: totalAvailable = (data?.inventory || []).reduce((sum, item) => sum + item.available, 0);
  $: totalHeld = (data?.inventory || []).reduce((sum, item) => sum + item.held, 0);
  $: expiringSoon = (data?.active_holds || []).filter((hold) => hold.expires_at * 1000 - now < 15 * 60_000).length;

  onMount(() => {
    applyRouteMetadata(path);
    hydrateBrowserPreferences();
    captureLicense(demo);
    runLicenseCheck(false);
    if (demo) load();
    else if (path === '/auth/callback') {
      path = '/'; landing = false; prepareLive();
    } else if (!landing && path !== '/privacy' && path !== '/terms' && path !== '/404') {
      path = '/404'; applyRouteMetadata(path); loading = false;
    } else {
      loading = false;
    }
    const clock = window.setInterval(() => {
      now = Date.now();
      sendReminders();
      if (data?.active_holds.some((hold) => hold.expires_at * 1000 <= now)) load(true);
    }, 30_000);
    const sync = window.setInterval(() => { if (navigator.onLine && !busy) load(true); }, 15_000);
    const handleOnline = () => { online = true; load(true); };
    const handleOffline = () => online = false;
    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);
    const handlePopState = () => changeRoute(routeFromUrl(), true);
    window.addEventListener('popstate', handlePopState);
    return () => {
      clearInterval(clock);
      clearInterval(sync);
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
      window.removeEventListener('popstate', handlePopState);
      licenseCheckController?.abort();
    };
  });

  function navigate(next: string) {
    history.pushState({}, '', next);
    changeRoute(routeFromUrl(next), false);
  }

  function changeRoute(next: string, fromHistory: boolean) {
    const wasDemo = demo;
    path = next;
    demo = next === '/demo';
    landing = next === '/';
    if (wasDemo && !demo) clearDemoStorage();
    if (wasDemo !== demo) {
      licenseCheckController?.abort();
      hydrateBrowserPreferences();
      license = { unlocked: false, notice: '', token: null };
      runLicenseCheck(false);
    }
    applyRouteMetadata(next);
    if (demo) load();
    if (landing) { data = null; accessRequired = false; loading = false; }
    announcement = '';
    if (!fromHistory) window.scrollTo({ top: 0, behavior: 'smooth' });
    tick().then(focusPageHeading);
  }

  function focusPageHeading() {
    const heading = document.querySelector<HTMLElement>('h1');
    if (!heading) return;
    heading.tabIndex = -1;
    heading.focus({ preventScroll: true });
    const name = heading.textContent?.trim() || 'page';
    announcement = `Opened ${name}${/[.!?]$/.test(name) ? '' : '.'}`;
  }

  async function prepareLive() {
    loading = true;
    try {
      const config = await request<{ mode: 'local' | 'ciam' }>('/api/auth/config');
      await configureAuth(config.mode);
      await load();
    } catch (error) {
      fatalError = message(error); loading = false;
    }
  }

  function startLive() {
    landing = false;
    prepareLive();
  }

  function runLicenseCheck(force: boolean) {
    licenseCheckController?.abort();
    const controller = new AbortController();
    const namespaceIsDemo = demo;
    licenseCheckController = controller;
    checkLicense(force, namespaceIsDemo, {
      signal: controller.signal,
      isCurrent: () => !controller.signal.aborted && demo === namespaceIsDemo,
    }).then((value) => {
      if (!controller.signal.aborted && demo === namespaceIsDemo) license = value;
    });
  }

  function preferenceStorage(): Storage {
    return demo ? sessionStorage : localStorage;
  }

  function preferenceKey(name: string): string {
    return demo ? `${demoPrefix}${name}` : `stock-promise:${name}`;
  }

  function readProfiles(): string[] {
    try {
      const value = JSON.parse(preferenceStorage().getItem(preferenceKey('profiles')) || '[]');
      return Array.isArray(value) ? value.filter((item): item is string => typeof item === 'string') : [];
    } catch {
      return [];
    }
  }

  function hydrateBrowserPreferences() {
    const storage = preferenceStorage();
    operatorName = storage.getItem(preferenceKey('operator')) || '';
    supervisorName = storage.getItem(preferenceKey('supervisor-name')) || '';
    profiles = readProfiles();
    reminders = storage.getItem(preferenceKey('reminders')) === 'true';
    notified.clear();
  }

  function savePreference(name: string, value: string) {
    preferenceStorage().setItem(preferenceKey(name), value);
  }

  function clearDemoStorage() {
    for (let index = sessionStorage.length - 1; index >= 0; index--) {
      const key = sessionStorage.key(index);
      if (key?.startsWith(demoPrefix)) sessionStorage.removeItem(key);
    }
  }

  function sampleData(): Bootstrap {
    const nowSeconds = Math.floor(Date.now() / 1000);
    return {
      setup_required: false, location_name: 'Harbor Parts — sample', server_time: nowSeconds, role: 'supervisor',
      inventory: [
        { id: 1, sku: 'VALVE-24', name: 'Brass isolation valve', on_hand: 12, held: 3, available: 9 },
        { id: 2, sku: 'FILTER-8', name: 'Cartridge filter', on_hand: 6, held: 0, available: 6 },
        { id: 3, sku: 'SEAL-11', name: 'Nitrile seal set', on_hand: 4, held: 0, available: 4 },
      ],
      active_holds: [{ id: 'demo-active-1', inventory_id: 1, sku: 'VALVE-24', item_name: 'Brass isolation valve', quantity: 3, customer: 'Northline Plumbing order 418', order_note: 'Counter pickup', operator_name: 'Mina', status: 'active', created_at: nowSeconds - 300, expires_at: nowSeconds + 25 * 60, resolved_at: null, resolved_by: null }],
      recent_outcomes: [{ id: 'demo-outcome-1', inventory_id: 2, sku: 'FILTER-8', item_name: 'Cartridge filter', quantity: 2, customer: 'Tideway Maintenance order 771', order_note: '', operator_name: 'Ravi', status: 'converted', created_at: nowSeconds - 7200, expires_at: nowSeconds - 5400, resolved_at: nowSeconds - 5100, resolved_by: 'Supervisor' }],
    };
  }

  function resetDemo() {
    clearDemoStorage();
    data = sampleData(); supervisor = true; auditEntries = [];
    operatorName = ''; supervisorName = ''; profiles = []; reminders = false;
    license = { unlocked: false, notice: '', token: null };
    retentionDays = 90; notified.clear();
    announcement = 'Demo reset to the shipped sample data.';
  }

  function persistDemo() {
    if (data) sessionStorage.setItem(demoKey, JSON.stringify(data));
  }

  async function load(quiet = false) {
    if (quiet) refreshing = true; else loading = true;
    try {
      if (demo) {
        data = JSON.parse(sessionStorage.getItem(demoKey) || 'null') || sampleData();
        supervisor = true; accessRequired = false;
      } else if (getSession() || usesCiam()) {
        data = await request<Bootstrap>('/api/bootstrap', {}, 'required');
        supervisor = data.role === 'supervisor';
        accessRequired = false;
      } else {
        const status = await request<{ setup_required: boolean; server_time: number }>('/api/status');
        if (status.setup_required) {
          data = { setup_required: true, location_name: null, server_time: status.server_time, inventory: [], active_holds: [], recent_outcomes: [], role: 'supervisor' };
          accessRequired = false;
        } else {
          data = null;
          accessRequired = true;
        }
      }
      fatalError = '';
    } catch (error) {
      if (error instanceof ResponseError && error.status === 401) {
        data = null;
        accessRequired = true;
        supervisor = false;
        fatalError = '';
      } else if (!data) fatalError = message(error);
    } finally {
      loading = false;
      refreshing = false;
    }
  }

  async function openModal(kind: Exclude<Modal, null>, item: InventoryItem | null = null) {
    modalOpener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    modal = kind;
    selectedItem = kind === 'hold' ? item : null;
    editingItem = kind === 'inventory' ? item : null;
    formError = '';
    importReport = '';
    await tick();
    dialog?.showModal();
    const first = dialog?.querySelector<HTMLElement>('input, button, select, textarea');
    first?.focus();
  }

  function closeModal() {
    dialog?.close();
  }

  async function handleDialogClose() {
    modal = null;
    selectedItem = null;
    editingItem = null;
    formError = '';
    const opener = modalOpener;
    modalOpener = null;
    await tick();
    if (opener?.isConnected) opener.focus();
  }

  async function setup(event: SubmitEvent) {
    busy = 'setup'; formError = '';
    const values = Object.fromEntries(new FormData(event.currentTarget as HTMLFormElement));
    try {
      const result = await request<{ token: string }>('/api/setup', { method: 'POST', body: JSON.stringify(values) });
      if (result.token) setSession(result.token); supervisor = true;
      announcement = 'Location ready. Add the stock your team promises.';
      await load(true);
    } catch (error) { formError = message(error); }
    finally { busy = ''; }
  }

  async function unlock(event: SubmitEvent) {
    if (usesCiam()) { await signIn(); return; }
    busy = 'unlock'; formError = '';
    const values = Object.fromEntries(new FormData(event.currentTarget as HTMLFormElement));
    try {
      const result = await request<{ token: string }>('/api/session', { method: 'POST', body: JSON.stringify(values) });
      setSession(result.token); supervisor = true; closeModal();
      announcement = 'Supervisor controls unlocked for this tab.';
      await load(true);
      if (tab === 'settings') loadAudit();
    } catch (error) { formError = message(error); }
    finally { busy = ''; }
  }

  async function lockSupervisor() {
    if (demo) { announcement = 'The demo is already isolated from live data.'; return; }
    if (usesCiam()) { await signOut(); return; }
    try { await request('/api/session', { method: 'DELETE' }, 'required'); } catch { /* local lock still applies */ }
    setSession(null); supervisor = false; auditEntries = []; data = null; accessRequired = true;
    announcement = 'Inventory holds locked.';
  }

  async function saveInventory(event: SubmitEvent) {
    busy = 'inventory'; formError = '';
    const wasEditing = Boolean(editingItem);
    const values = Object.fromEntries(new FormData(event.currentTarget as HTMLFormElement));
    const payload = { ...values, on_hand: Number(values.on_hand) };
    if (demo && data) {
      const sku = String(values.sku).trim().toUpperCase();
      const name = String(values.name).trim();
      if (!sku || !name || !Number.isFinite(payload.on_hand) || payload.on_hand < 0) { formError = 'Enter a SKU, item name, and a non-negative stock count.'; busy = ''; return; }
      if (editingItem) {
        data.inventory = data.inventory.map((item) => item.id === editingItem?.id ? { ...item, sku, name, on_hand: payload.on_hand, available: payload.on_hand - item.held } : item);
      } else {
        const id = Math.max(0, ...data.inventory.map((item) => item.id)) + 1;
        data.inventory = [...data.inventory, { id, sku, name, on_hand: payload.on_hand, held: 0, available: payload.on_hand }];
      }
      persistDemo(); closeModal(); announcement = wasEditing ? 'Sample stock record updated.' : 'Sample stock item added.'; busy = ''; return;
    }
    try {
      await request(editingItem ? `/api/inventory/${editingItem.id}` : '/api/inventory', { method: 'POST', body: JSON.stringify(payload) }, 'required');
      closeModal(); announcement = wasEditing ? 'Stock record updated.' : 'Stock item added.';
      await load(true);
    } catch (error) { formError = message(error); supervisor = Boolean(getSession()); }
    finally { busy = ''; }
  }

  async function createHold(event: SubmitEvent) {
    if (!selectedItem) return;
    busy = 'hold'; formError = '';
    const values = Object.fromEntries(new FormData(event.currentTarget as HTMLFormElement));
    const payload = { ...values, inventory_id: selectedItem.id, quantity: Number(values.quantity), duration_minutes: Number(values.duration_minutes) };
    if (demo && data) {
      const item = data.inventory.find((candidate) => candidate.id === selectedItem?.id);
      if (!item || payload.quantity < 1 || payload.quantity > item.available) { formError = 'Choose a quantity that is available in the sample stockroom.'; busy = ''; return; }
      const createdAt = Math.floor(Date.now() / 1000);
      const hold: Hold = { id: `demo-${crypto.randomUUID()}`, inventory_id: item.id, sku: item.sku, item_name: item.name, quantity: payload.quantity, customer: String(values.customer), order_note: String(values.order_note || ''), operator_name: String(values.operator_name), status: 'active', created_at: createdAt, expires_at: createdAt + payload.duration_minutes * 60, resolved_at: null, resolved_by: null };
      data.inventory = data.inventory.map((candidate) => candidate.id === item.id ? { ...candidate, held: candidate.held + hold.quantity, available: candidate.available - hold.quantity } : candidate);
      data.active_holds = [...data.active_holds, hold];
      operatorName = hold.operator_name; savePreference('operator', operatorName);
      persistDemo(); closeModal(); announcement = `Sample hold created for ${hold.customer}. ${hold.quantity} units are now protected.`; busy = ''; return;
    }
    try {
      await request('/api/holds', { method: 'POST', body: JSON.stringify(payload) }, 'required');
      operatorName = String(values.operator_name); savePreference('operator', operatorName);
      closeModal(); announcement = `Hold created for ${values.customer}. ${payload.quantity} units are now protected.`;
      await load(true);
    } catch (error) { formError = message(error); await load(true); }
    finally { busy = ''; }
  }

  async function resolve(hold: Hold, action: 'convert' | 'release') {
    if (!supervisor) { openModal('unlock'); return; }
    if (action === 'release' && !confirm(`Release ${hold.quantity} × ${hold.sku} held for ${hold.customer}? The stock will become available immediately.`)) return;
    if (action === 'convert' && !confirm(`Convert ${hold.quantity} × ${hold.sku} for ${hold.customer}? This permanently deducts the units from on-hand stock.`)) return;
    busy = hold.id; formError = '';
    if (demo && data) {
      const resolvedAt = Math.floor(Date.now() / 1000);
      data.active_holds = data.active_holds.filter((candidate) => candidate.id !== hold.id);
      data.inventory = data.inventory.map((item) => {
        if (item.id !== hold.inventory_id) return item;
        const on_hand = action === 'convert' ? item.on_hand - hold.quantity : item.on_hand;
        return { ...item, on_hand, held: item.held - hold.quantity, available: on_hand - (item.held - hold.quantity) };
      });
      data.recent_outcomes = [{ ...hold, status: action === 'convert' ? 'converted' : 'released', resolved_at: resolvedAt, resolved_by: 'Sample supervisor' }, ...data.recent_outcomes];
      persistDemo(); announcement = action === 'convert' ? `Sample hold for ${hold.customer} converted.` : `Sample hold for ${hold.customer} released.`; busy = ''; return;
    }
    try {
      await request(`/api/holds/${hold.id}/resolve`, { method: 'POST', body: JSON.stringify({ action, actor: supervisorName || 'Supervisor' }) }, 'required');
      if (supervisorName) savePreference('supervisor-name', supervisorName);
      announcement = action === 'convert' ? `Hold for ${hold.customer} converted. Stock count reduced.` : `Hold for ${hold.customer} released. Stock is available again.`;
      await load(true);
    } catch (error) { announcement = message(error); supervisor = Boolean(getSession()); await load(true); }
    finally { busy = ''; }
  }

  async function downloadExport() {
    if (!supervisor) { openModal('unlock'); return; }
    busy = 'export';
    try {
      if (demo && data) {
        const rows = [...data.active_holds, ...data.recent_outcomes];
        const header = 'hold_id,sku,item,quantity,customer,order_note,operator,outcome\n';
        const quote = (value: string | number) => `"${String(value).replaceAll('"', '""')}"`;
        const csv = header + rows.map((hold) => [hold.id, hold.sku, hold.item_name, hold.quantity, hold.customer, hold.order_note, hold.operator_name, hold.status].map(quote).join(',')).join('\n');
        const url = URL.createObjectURL(new Blob([csv], { type: 'text/csv' }));
        const anchor = document.createElement('a'); anchor.href = url; anchor.download = 'stock-promise-holds.csv'; anchor.click(); URL.revokeObjectURL(url);
        announcement = 'Sample CSV export downloaded.'; return;
      }
      const response = await fetch('/api/export.csv', { headers: { authorization: `Bearer ${getSession()}` } });
      if (!response.ok) throw new Error((await response.json()).error);
      const url = URL.createObjectURL(await response.blob());
      const anchor = document.createElement('a'); anchor.href = url; anchor.download = 'stock-promise-holds.csv'; anchor.click(); URL.revokeObjectURL(url);
      announcement = 'CSV export downloaded.';
    } catch (error) { announcement = message(error); }
    finally { busy = ''; }
  }

  async function importCsv(event: Event) {
    const file = (event.currentTarget as HTMLInputElement).files?.[0];
    if (!file) return;
    busy = 'import'; formError = ''; importReport = '';
    try {
      const lines = (await file.text()).replace(/^\uFEFF/, '').split(/\r?\n/).filter(Boolean);
      if (lines.length < 2) throw new Error('The CSV needs a header and at least one stock row.');
      const headers = lines[0].split(',').map((value) => value.trim().toLowerCase());
      const indexes = ['sku', 'name', 'on_hand'].map((name) => headers.indexOf(name));
      if (indexes.some((index) => index < 0)) throw new Error('Use CSV columns: sku,name,on_hand');
      let imported = 0; const failures: string[] = [];
      for (const [offset, line] of lines.slice(1).entries()) {
        const cells = line.split(',').map((value) => value.trim().replace(/^"|"$/g, ''));
        try {
          const sku = cells[indexes[0]].toUpperCase();
          const name = cells[indexes[1]];
          const onHand = Number(cells[indexes[2]]);
          if (!sku || !name || !Number.isInteger(onHand) || onHand < 0) throw new Error('Enter a SKU, item name, and a non-negative whole stock count.');
          if (demo && data) {
            if (data.inventory.some((item) => item.sku === sku)) throw new Error('That SKU is already in the sample stockroom.');
            const id = Math.max(0, ...data.inventory.map((item) => item.id)) + 1;
            data.inventory = [...data.inventory, { id, sku, name, on_hand: onHand, held: 0, available: onHand }];
          } else {
            await request('/api/inventory', { method: 'POST', body: JSON.stringify({ sku, name, on_hand: onHand }) }, 'required');
          }
          imported++;
        } catch (error) { failures.push(`Row ${offset + 2}: ${message(error)}`); }
      }
      if (demo) persistDemo();
      importReport = `${imported} item${imported === 1 ? '' : 's'} imported.${failures.length ? ` ${failures.length} skipped: ${failures.slice(0, 3).join(' ')}` : ''}`;
      if (!demo) await load(true);
    } catch (error) { formError = message(error); }
    finally { busy = ''; }
  }

  async function restoreLicense(event: SubmitEvent) {
    const value = String(new FormData(event.currentTarget as HTMLFormElement).get('license') || '');
    if (!value.trim()) return;
    storeLicense(value, demo); license = { unlocked: true, notice: 'Checking license…', token: value };
    licenseCheckController?.abort();
    const controller = new AbortController();
    const namespaceIsDemo = demo;
    licenseCheckController = controller;
    license = await checkLicense(true, namespaceIsDemo, {
      signal: controller.signal,
      isCurrent: () => !controller.signal.aborted && demo === namespaceIsDemo,
    });
    announcement = license.unlocked ? 'Stock Promise Pro unlocked.' : license.notice;
  }

  function addProfile() {
    const value = operatorName.trim();
    if (!value || profiles.includes(value)) return;
    profiles = [...profiles, value].slice(-8);
    savePreference('profiles', JSON.stringify(profiles));
  }

  async function enableReminders() {
    if (!license.unlocked) return;
    if (!('Notification' in window)) { announcement = 'This browser does not support notifications.'; return; }
    const permission = await Notification.requestPermission();
    reminders = permission === 'granted'; savePreference('reminders', String(reminders));
    if (reminders) sendReminders();
    announcement = reminders ? 'Five-minute expiry reminders enabled on this device.' : 'Notification permission was not granted.';
  }

  function sendReminders() {
    if (!license.unlocked || !reminders || !data || Notification.permission !== 'granted') return;
    for (const hold of data.active_holds) {
      const remaining = hold.expires_at * 1000 - Date.now();
      if (remaining > 0 && remaining <= 5 * 60_000 && !notified.has(hold.id)) {
        new Notification(`${hold.sku} hold expires soon`, { body: `${hold.quantity} for ${hold.customer} · ${relativeExpiry(hold.expires_at)}`, icon: '/mark.svg' });
        notified.add(hold.id);
      }
    }
  }

  async function loadAudit() {
    if (!supervisor || demo) return;
    try { auditEntries = (await request<{ entries: Array<Record<string, any>> }>('/api/audit', {}, 'required')).entries; }
    catch (error) { announcement = message(error); supervisor = Boolean(getSession()); }
  }

  async function loadRetention() {
    if (!supervisor || demo) return;
    try { retentionDays = (await request<{ retention_days: number }>('/api/data-retention', {}, 'required')).retention_days; }
    catch (error) { announcement = message(error); }
  }

  async function saveRetention(event: SubmitEvent) {
    busy = 'retention'; formError = '';
    const retention_days = Number(new FormData(event.currentTarget as HTMLFormElement).get('retention_days'));
    try {
      if (demo) { retentionDays = retention_days; announcement = 'Sample retention setting changed.'; closeModal(); return; }
      retentionDays = (await request<{ retention_days: number }>('/api/data-retention', { method: 'POST', body: JSON.stringify({ retention_days }) }, 'required')).retention_days;
      announcement = `Shared data retention set to ${retentionDays} days.`; closeModal();
    } catch (error) { formError = message(error); } finally { busy = ''; }
  }

  async function eraseLocation(event: SubmitEvent) {
    busy = 'erase'; formError = '';
    const confirmation = String(new FormData(event.currentTarget as HTMLFormElement).get('confirmation') || '');
    try {
      if (demo) { resetDemo(); closeModal(); return; }
      await request('/api/location', { method: 'DELETE', body: JSON.stringify({ confirmation }) }, 'required');
      setSession(null); data = null; supervisor = false; accessRequired = false; landing = true; navigate('/');
    } catch (error) { formError = message(error); } finally { busy = ''; }
  }

  function chooseTab(next: Tab) {
    tab = next;
    if (next === 'settings') { loadAudit(); loadRetention(); }
  }

  function message(error: unknown): string { return error instanceof Error ? error.message : 'Something went wrong. Try again.'; }
</script>

<div class="live-region" aria-live="polite" aria-atomic="true">{announcement}</div>

{#if path === '/privacy' || path === '/terms'}
  <Legal kind={path === '/privacy' ? 'privacy' : 'terms'} {navigate} />
{:else}
  <a class="skip-link" href="#main">Skip to inventory holds</a>
  <header class="app-header">
    <a class="wordmark" href="/" onclick={(event) => { event.preventDefault(); navigate('/'); }}>
      <img src="/mark.svg" alt="" width="38" height="38" />
      <span>Stock Promise</span>
    </a>
    <nav class="top-nav" aria-label="Site">
      <a href="/demo" onclick={(event) => { event.preventDefault(); navigate('/demo'); }}>Demo</a>
      <a href="/privacy" onclick={(event) => { event.preventDefault(); navigate('/privacy'); }}>Privacy</a>
    </nav>
    <div class="header-status">
      {#if demo}
        <span class="sample-data-status">Sample data</span>
      {/if}
      {#if data && !data.setup_required && !demo}
        <button class="quiet-button header-action" aria-label={usesCiam() ? 'Sign out' : supervisor ? 'Lock supervisor' : 'Supervisor unlock'} onclick={() => usesCiam() || supervisor ? lockSupervisor() : openModal('unlock')}>
          <span class="header-action-full">{usesCiam() ? 'Sign out' : supervisor ? 'Lock supervisor' : 'Supervisor unlock'}</span>
          <span class="header-action-compact" aria-hidden="true">{usesCiam() ? '↗' : supervisor ? '🔒' : '🔓'}</span>
        </button>
      {/if}
    </div>
  </header>

  {#if !online}<div class="offline-banner" role="status">You’re offline. Current figures may be stale; new promises are paused until the shared server reconnects.</div>{/if}
  {#if demo}<div class="demo-banner" role="status"><span><strong>Demo</strong> — sample data, nothing is saved.</span><button class="text-button" onclick={resetDemo}>Reset demo</button><a class="text-button" href="/" onclick={(event) => { event.preventDefault(); navigate('/'); }}>Leave demo</a></div>{/if}
  {#if path === '/404'}
    <main id="main" class="center-state"><p class="eyebrow">404</p><h1>Page not found</h1><p>Open inventory holds or the sample stockroom to continue.</p><a class="primary-button" href="/" onclick={(event) => { event.preventDefault(); navigate('/'); }}>Return home</a></main>
  {:else if landing}
    <main id="main" class="landing-page">
      <section class="landing-hero">
        <div class="landing-copy">
          <p class="eyebrow">One shared location</p>
          <h1>Hold scarce stock before it is promised twice.</h1>
          <p>For distributors and resellers taking orders in parallel, Stock Promise shows a timed team hold before stock is promised.</p>
          <div class="landing-actions"><a class="primary-button" href="/?demo=1" onclick={(event) => { event.preventDefault(); navigate('/?demo=1'); }}>Try it with sample data</a><span>Open a sample stockroom.</span></div>
          <div class="plain-facts"><span>Timed holds expire automatically.</span><span>The sample never changes a live stockroom.</span><span>New Pro purchases are temporarily unavailable.</span></div>
          <button class="secondary-button" onclick={startLive}>Open inventory holds</button>
        </div>
        <picture><source media="(max-width: 700px)" srcset="/assets/stockroom-watch-640.webp" /><img src="/assets/stockroom-watch-1536.webp" width="1536" height="1024" alt="An orderly stockroom aisle with a small carton group under a warm work light" fetchpriority="high" decoding="async" /></picture>
      </section>
      <section class="landing-section" aria-labelledby="how-it-works"><h2 id="how-it-works">How it works</h2><ol><li><strong>List stock.</strong> Add the SKUs that one location can promise.</li><li><strong>Place a hold.</strong> Staff name the customer, quantity, and expiry.</li><li><strong>Resolve it.</strong> A supervisor converts or releases the hold.</li></ol></section>
      <section class="landing-section" aria-labelledby="limits"><h2 id="limits">Limits and data retention</h2><p>It is not a legal reservation, warehouse system, storefront, or replacement for your system of record.</p><p>Supervisors choose when resolved customer references, notes, and operator names are removed.</p></section>
      <section class="landing-section" aria-labelledby="pricing"><h2 id="pricing">Pro profiles and reminders</h2><p>A verified Pro license enables local operator profiles and on-device expiry reminders. Core holds and CSV export do not require Pro.</p><p class="muted">New Pro purchases are temporarily unavailable.</p></section>
    </main>
  {:else if loading}
    <main id="main" class="loading-state" aria-busy="true">
      <p class="eyebrow">Opening the stockroom</p><h1>Finding today’s promises…</h1><div class="loader"></div>
    </main>
  {:else if accessRequired}
    <main id="main" class="center-state access-gate">
      <p class="eyebrow">Staff access</p>
      <h1>Open inventory holds.</h1>
      <p>Operational stock and customer references are private to this location.</p>
      {#if usesCiam()}
        <p>Sign in with your Sociobot account. Staff can create holds; supervisors can change stock and resolve holds.</p>
        <button class="primary-button" onclick={() => signIn()}>Sign in with Sociobot</button>
      {:else}
        <form onsubmit={(event) => { event.preventDefault(); unlock(event); }}>
          <label for="access-pin">Supervisor PIN <span>6–12 digits</span></label>
          <input id="access-pin" name="pin" type="password" inputmode="numeric" pattern="[0-9]+" minlength="6" maxlength="12" autocomplete="current-password" required />
          {#if formError}<p class="form-error" role="alert">{formError}</p>{/if}
          <button class="primary-button" disabled={busy === 'unlock'}>{busy === 'unlock' ? 'Opening inventory…' : 'Open inventory holds'}</button>
        </form>
      {/if}
    </main>
  {:else if fatalError}
    <main id="main" class="center-state">
      <p class="eyebrow alarm">Shared server unavailable</p><h1>Inventory holds can’t open yet.</h1><p>{fatalError}</p><button class="primary-button" onclick={() => load()}>Try again</button>
    </main>
  {:else if data?.setup_required}
    <main id="main" class="setup-layout">
      <section class="setup-art">
        <picture><source media="(max-width: 700px)" srcset="/assets/stockroom-watch-640.webp" /><img src="/assets/stockroom-watch-1536.webp" srcset="/assets/stockroom-watch-1024.webp 1024w, /assets/stockroom-watch-1536.webp 1536w" sizes="(max-width: 800px) 100vw, 58vw" width="1536" height="1024" alt="An orderly stockroom aisle where one small group of cartons is picked out by a warm work light" fetchpriority="high" decoding="async" /></picture>
        <div class="art-copy"><p class="eyebrow">One location · one live truth</p><h1>Promise what’s there. Once.</h1><p>Create a visible, timed claim while the order is still being written.</p></div>
      </section>
      <section class="setup-form-wrap" aria-labelledby="setup-title">
        <p class="step">First shift setup</p><h2 id="setup-title">Name this stockroom</h2><p>This takes about a minute. Supervisors protect stock edits, conversions, and exports.</p>
        <form onsubmit={(event) => { event.preventDefault(); setup(event); }}>
          <label for="location">Location name</label><input id="location" name="location_name" autocomplete="organization" maxlength="80" required placeholder="e.g. Main counter" />
          {#if !usesCiam()}<label for="setup-pin">Supervisor PIN <span>6–12 digits</span></label><input id="setup-pin" name="pin" type="password" inputmode="numeric" pattern="[0-9]+" minlength="6" maxlength="12" autocomplete="new-password" required />{/if}
          {#if formError}<p class="form-error" role="alert">{formError}</p>{/if}
          <button class="primary-button" disabled={busy === 'setup'}>{busy === 'setup' ? 'Securing location…' : 'Open inventory holds'}</button>
        </form>
        <p class="fine-print">This hosted service stores your stock, customer references, names, and hold notes. A supervisor can set retention and erase the location.</p>
      </section>
    </main>
  {:else if data}
    <div class="app-frame">
      <aside class="scene-rail">
        <img src="/assets/stockroom-watch-640.webp" width="640" height="427" alt="An orderly stockroom with a finite carton group under a warm work light" fetchpriority="high" decoding="async" />
        <div class="scene-shade"></div>
        <div class="scene-content">
          <p class="eyebrow">{data.location_name}</p>
          <p class="scene-heading">Inventory holds</p>
          <p class="scene-note">A soft hold is a team signal, not a legal reservation.</p>
          <dl class="rail-metrics">
            <div><dt>Available now</dt><dd>{totalAvailable.toLocaleString()}</dd></div>
            <div><dt>On hold</dt><dd>{totalHeld.toLocaleString()}</dd></div>
            <div><dt>Due in 15 min</dt><dd>{expiringSoon}</dd></div>
          </dl>
        </div>
      </aside>

      <main id="main" class="workspace">
        <h1 class="sr-only">{demo ? 'Manage sample inventory holds' : 'Manage inventory holds'}</h1>
        <nav class="section-nav" aria-label="Inventory hold sections">
          <button class:active={tab === 'desk'} aria-current={tab === 'desk' ? 'page' : undefined} onclick={() => chooseTab('desk')}>Inventory holds <span>{data.active_holds.length}</span></button>
          <button class:active={tab === 'outcomes'} aria-current={tab === 'outcomes' ? 'page' : undefined} onclick={() => chooseTab('outcomes')}>Outcomes</button>
          <button class:active={tab === 'settings'} aria-current={tab === 'settings' ? 'page' : undefined} onclick={() => chooseTab('settings')}>Stock & settings</button>
        </nav>

        {#if tab === 'desk'}
          <section class="desk-head" aria-labelledby="inventory-title">
            <div><p class="eyebrow">Live availability</p><h2 id="inventory-title">Choose stock to hold</h2></div>
            <label class="search"><span class="sr-only">Search inventory</span><input type="search" bind:value={query} placeholder="Search SKU or item" /></label>
          </section>
          {#if data.inventory.length === 0}
            <section class="empty-state">
              <span class="empty-mark" aria-hidden="true">＋</span><h2>No stock is listed yet</h2><p>Unlock supervisor controls, then add an item or import a CSV to give the team one shared list.</p>
              <button class="primary-button" onclick={() => supervisor ? openModal('inventory') : openModal('unlock')}>{supervisor ? 'Add first item' : 'Unlock to add stock'}</button>
            </section>
          {:else if filteredInventory.length === 0}
            <section class="empty-state compact"><h2>No items match “{query}”</h2><p>Check the SKU or item name, then try again.</p><button class="text-button" onclick={() => query = ''}>Clear search</button></section>
          {:else}
            <ul class="inventory-list" aria-label="Inventory availability">
              {#each filteredInventory as item (item.id)}
                <li class:scarce={item.available <= 3}>
                  <div class="item-identity"><strong>{item.name}</strong><span>{item.sku}</span></div>
                  <div class="stock-figures"><span><b>{item.available}</b> available</span><span>{item.held} held · {item.on_hand} on hand</span></div>
                  <button class="hold-button" disabled={!online || item.available < 1} onclick={() => openModal('hold', item)}>{item.available < 1 ? 'Fully held' : 'Create hold'}</button>
                </li>
              {/each}
            </ul>
          {/if}

          <section class="active-section" aria-labelledby="active-title">
            <div class="section-title"><div><p class="eyebrow">In progress</p><h2 id="active-title">Active holds</h2></div><button class="icon-button" aria-label="Refresh live figures" disabled={refreshing} onclick={() => load(true)}>↻</button></div>
            {#if data.active_holds.length === 0}
              <div class="quiet-empty"><span aria-hidden="true">✓</span><p><strong>No stock is tied up.</strong><br />New holds will appear here for everyone.</p></div>
            {:else}
              <ol class="hold-list">
                {#each data.active_holds as hold (hold.id)}
                  <li class:urgent={hold.expires_at * 1000 - now < 5 * 60_000}>
                    <div class="hold-edge"></div>
                    <div class="hold-main"><div><span class="sku">{hold.sku}</span><strong>{hold.quantity} × {hold.item_name}</strong></div><p>For <b>{hold.customer}</b> · by {hold.operator_name}{hold.order_note ? ` · ${hold.order_note}` : ''}</p></div>
                    <div class="hold-time"><strong>{relativeExpiry(hold.expires_at, now)}</strong><span>until {formatTime(hold.expires_at)}</span></div>
                    <div class="hold-actions">
                      {#if supervisor}
                        <button class="convert-button" disabled={busy === hold.id || !online} onclick={() => resolve(hold, 'convert')}>{busy === hold.id ? 'Working…' : 'Convert'}</button>
                        <button class="release-button" disabled={busy === hold.id || !online} onclick={() => resolve(hold, 'release')}>Release</button>
                      {:else}<span class="muted">A supervisor resolves this hold.</span>{/if}
                    </div>
                  </li>
                {/each}
              </ol>
            {/if}
          </section>
        {:else if tab === 'outcomes'}
          <section class="panel-head"><div><p class="eyebrow">The completed ledger</p><h2>Recent outcomes</h2><p>Converted, released, and automatically expired promises.</p></div>{#if supervisor}<button class="primary-button small" onclick={downloadExport} disabled={busy === 'export'}>{busy === 'export' ? 'Preparing…' : 'Export CSV'}</button>{/if}</section>
          {#if data.recent_outcomes.length === 0}
            <section class="empty-state compact"><h2>No outcomes yet</h2><p>Resolve a live hold and its outcome will be recorded here permanently.</p></section>
          {:else}
            <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
            <div class="outcome-table" role="region" aria-label="Recent hold outcomes" tabindex="0">
              <table><thead><tr><th>Outcome</th><th>Stock</th><th>Customer</th><th>Created by</th><th>Resolved</th></tr></thead><tbody>
                {#each data.recent_outcomes as hold}
                  <tr><td><span class="status {hold.status}">{hold.status}</span></td><td><strong>{hold.quantity} × {hold.sku}</strong><small>{hold.item_name}</small></td><td>{hold.customer}</td><td>{hold.operator_name}</td><td>{hold.resolved_at ? formatTime(hold.resolved_at) : '—'}<small>{hold.resolved_by || ''}</small></td></tr>
                {/each}
              </tbody></table>
            </div>
          {/if}
        {:else}
          <section class="panel-head"><div><p class="eyebrow">Supervisor station</p><h2>Stock & settings</h2><p>Keep the shared list current and review its audit record.</p></div>{#if supervisor}<div class="action-row"><button class="secondary-button" onclick={() => openModal('import')}>Import CSV</button><button class="primary-button small" onclick={() => openModal('inventory')}>Add stock</button></div>{/if}</section>
          <section class="settings-section"><div class="section-title"><div><h3>Inventory</h3><p>{data.inventory.length} shared SKU{data.inventory.length === 1 ? '' : 's'}</p></div>{#if !supervisor}<span class="muted">Supervisor access required to edit.</span>{/if}</div>
            <ul class="settings-stock">{#each data.inventory as item}<li><div><strong>{item.sku}</strong><span>{item.name}</span></div><div><b>{item.on_hand}</b> on hand</div><button disabled={!supervisor} onclick={() => openModal('inventory', item)}>Edit</button></li>{/each}</ul>
          </section>
          {#if supervisor}
            <section class="settings-section privacy-controls"><div class="section-title"><div><h3>Data retention</h3><p>Remove resolved hold details after {retentionDays} days.</p></div><button class="secondary-button" onclick={() => openModal('privacy')}>Manage data</button></div><p class="muted">Retention removes resolved customer references, notes, and operator names. Erasing this location permanently removes its inventory, holds, sessions, and audit record.</p></section>
          {/if}
          <section class="pro-section"><div><p class="eyebrow">Optional team convenience</p><h3>{license.unlocked ? 'Stock Promise Pro is active' : 'Pro reminders & profiles'}</h3><p>Core holds and CSV export do not require Pro.</p></div>
            {#if license.unlocked}
              <div class="pro-controls"><label for="profile-name">Operator profile name</label><div class="inline-form"><input id="profile-name" bind:value={operatorName} maxlength="80" /><button class="secondary-button" onclick={addProfile}>Save profile</button></div>{#if profiles.length}<div class="chips">{#each profiles as profile}<button onclick={() => operatorName = profile}>{profile}</button>{/each}</div>{/if}<button class="primary-button small" onclick={enableReminders}>{reminders ? 'Reminders enabled' : 'Enable 5-minute reminders'}</button></div>
            {:else}
              <div class="price-lock"><p>Saved operator profiles and on-device expiry notifications need a verified Pro license.</p><p class="muted">New Pro purchases are temporarily unavailable. Existing license holders can restore a license below.</p></div>
            {/if}
            {#if license.notice}<p class="license-notice">{license.notice}</p>{/if}
            <form class="restore-form" onsubmit={(event) => { event.preventDefault(); restoreLicense(event); }}><label for="license">Have a license? Paste it here</label><div class="inline-form"><input id="license" name="license" autocomplete="off" /><button class="secondary-button">Verify license</button></div></form>
          </section>
          <section class="settings-section"><div class="section-title"><div><h3>Audit record</h3><p>Past changes cannot be edited; newest first</p></div>{#if supervisor}<button class="icon-button" aria-label="Refresh audit record" onclick={loadAudit}>↻</button>{/if}</div>
            {#if !supervisor}<div class="locked-copy"><p>Supervisor access is required to inspect the audit record.</p></div>
            {:else if auditEntries.length === 0}<p class="muted">No recorded activity yet.</p>
            {:else}<ol class="audit-list">{#each auditEntries.slice(0, 30) as entry}<li><span class="audit-dot"></span><div><strong>{String(entry.event).replace('.', ' ')}</strong><p>{entry.actor} · {formatTime(entry.created_at)}</p></div></li>{/each}</ol>{/if}
          </section>
        {/if}
      </main>
    </div>
  {/if}

  {#if modal}
    <dialog bind:this={dialog} onclose={handleDialogClose} oncancel={(event) => { event.preventDefault(); closeModal(); }} aria-labelledby="dialog-title">
      <button class="dialog-close" aria-label="Close dialog" onclick={closeModal}>×</button>
      {#if modal === 'hold' && selectedItem}
        <p class="eyebrow">Temporary promise</p><h2 id="dialog-title">Hold {selectedItem.name}</h2><p><span class="sku">{selectedItem.sku}</span> · <strong>{selectedItem.available} available now</strong></p>
        <form onsubmit={(event) => { event.preventDefault(); createHold(event); }}>
          <div class="form-grid"><label for="quantity">Quantity</label><input id="quantity" name="quantity" type="number" min="1" max={selectedItem.available} value="1" required /><label for="duration">Hold for</label><select id="duration" name="duration_minutes"><option value="15">15 minutes</option><option value="30" selected>30 minutes</option><option value="60">1 hour</option><option value="120">2 hours</option><option value="240">4 hours</option><option value="480">8 hours</option></select></div>
          <label for="customer">Customer or order reference</label><input id="customer" name="customer" maxlength="120" required autocomplete="off" />
          <label for="operator">Your name</label><input id="operator" name="operator_name" maxlength="80" bind:value={operatorName} required autocomplete="name" />
          {#if license.unlocked && profiles.length}<div class="chips">{#each profiles as profile}<button type="button" onclick={() => operatorName = profile}>{profile}</button>{/each}</div>{/if}
          <label for="order-note">Order note <span>optional</span></label><textarea id="order-note" name="order_note" maxlength="300" rows="3"></textarea>
          <p class="form-help">This will be visible to everyone at {data?.location_name}. It expires automatically; it is not a legal reservation.</p>
          {#if formError}<p class="form-error" role="alert">{formError}</p>{/if}<button class="primary-button full" disabled={busy === 'hold' || !online}>{busy === 'hold' ? 'Protecting stock…' : `Hold ${selectedItem.sku}`}</button>
        </form>
      {:else if modal === 'unlock'}
        <p class="eyebrow">Restricted controls</p><h2 id="dialog-title">Supervisor unlock</h2><p>Unlock stock edits, hold outcomes, audit history, and CSV export for this browser tab.</p>
        <form onsubmit={(event) => { event.preventDefault(); unlock(event); }}><label for="unlock-pin">Supervisor PIN</label><input id="unlock-pin" name="pin" type="password" inputmode="numeric" pattern="[0-9]+" minlength="6" maxlength="12" autocomplete="current-password" required />{#if formError}<p class="form-error" role="alert">{formError}</p>{/if}<button class="primary-button full" disabled={busy === 'unlock'}>{busy === 'unlock' ? 'Checking…' : 'Unlock controls'}</button></form>
      {:else if modal === 'inventory'}
        <p class="eyebrow">Shared stock list</p><h2 id="dialog-title">{editingItem ? 'Edit stock' : 'Add stock'}</h2><p>On-hand is physical stock before active holds are deducted.</p>
        <form onsubmit={(event) => { event.preventDefault(); saveInventory(event); }}><label for="sku">SKU</label><input id="sku" name="sku" maxlength="48" value={editingItem?.sku || ''} required autocapitalize="characters" /><label for="item-name">Item name</label><input id="item-name" name="name" maxlength="120" value={editingItem?.name || ''} required /><label for="on-hand">On-hand quantity</label><input id="on-hand" name="on_hand" type="number" min={editingItem?.held || 0} max="100000000" value={editingItem?.on_hand ?? 0} required />{#if editingItem?.held}<p class="form-help">At least {editingItem.held} units are currently held, so stock cannot be set lower.</p>{/if}{#if formError}<p class="form-error" role="alert">{formError}</p>{/if}<button class="primary-button full" disabled={busy === 'inventory'}>{busy === 'inventory' ? 'Saving…' : editingItem ? 'Save stock record' : 'Add to stock list'}</button></form>
      {:else if modal === 'import'}
        <p class="eyebrow">Bulk setup</p><h2 id="dialog-title">Import stock CSV</h2><p>Use a simple UTF-8 CSV with <code>sku,name,on_hand</code> headers. Existing SKUs are skipped so a bulk file never overwrites live counts.</p><label class="file-picker" for="csv-file">Choose CSV file</label><input id="csv-file" class="sr-only" type="file" accept=".csv,text/csv" onchange={importCsv} />{#if busy === 'import'}<p role="status">Importing rows…</p>{/if}{#if importReport}<p class="success-box" role="status">{importReport}</p>{/if}{#if formError}<p class="form-error" role="alert">{formError}</p>{/if}<button class="secondary-button full" onclick={closeModal}>Done</button>
      {:else if modal === 'privacy'}
        <p class="eyebrow">Supervisor control</p><h2 id="dialog-title">Manage shared data</h2><p>Set when resolved customer references, notes, and operator names are removed. This does not affect browser-only license or reminder preferences.</p>
        <form onsubmit={(event) => { event.preventDefault(); saveRetention(event); }}><label for="retention-days">Retention period</label><select id="retention-days" name="retention_days" bind:value={retentionDays}><option value="30">30 days</option><option value="90">90 days</option><option value="180">180 days</option><option value="365">365 days</option><option value="730">730 days</option></select><button class="secondary-button full" disabled={busy === 'retention'}>{busy === 'retention' ? 'Saving…' : 'Save retention'}</button></form>
        <hr /><h3>Erase this location</h3><p>This permanently deletes the shared inventory, customer references, holds, sessions, and audit record. It cannot be undone.</p><form onsubmit={(event) => { event.preventDefault(); eraseLocation(event); }}><label for="erase-confirmation">Type DELETE to confirm</label><input id="erase-confirmation" name="confirmation" autocomplete="off" required /><button class="release-button full" disabled={busy === 'erase'}>{busy === 'erase' ? 'Erasing…' : 'Erase location data'}</button></form>{#if formError}<p class="form-error" role="alert">{formError}</p>{/if}
      {/if}
    </dialog>
  {/if}

  <footer class="site-footer">
    <span>Timed shared holds for one location.</span><nav aria-label="Legal"><a href="/privacy" onclick={(event) => { event.preventDefault(); navigate('/privacy'); }}>Privacy</a><a href="/terms" onclick={(event) => { event.preventDefault(); navigate('/terms'); }}>Terms</a></nav><span>Built by Param Factory · build {buildId.slice(0, 12)} · AI-assisted image.</span>
  </footer>
{/if}
