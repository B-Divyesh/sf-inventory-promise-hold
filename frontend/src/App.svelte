<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { request, getSession, setSession, ResponseError, type Bootstrap, type InventoryItem, type Hold } from './api';
  import { buyUrl, captureLicense, checkLicense, storeLicense, type LicenseState } from './license';
  import { formatTime, relativeExpiry } from './time';
  import Legal from './Legal.svelte';

  type Tab = 'desk' | 'outcomes' | 'settings';
  type Modal = 'hold' | 'unlock' | 'inventory' | 'import' | null;

  let path = window.location.pathname;
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
  let selectedItem: InventoryItem | null = null;
  let editingItem: InventoryItem | null = null;
  let supervisor = Boolean(getSession());
  let busy = '';
  let formError = '';
  let announcement = '';
  let now = Date.now();
  let license: LicenseState = { unlocked: false, notice: '', token: null };
  let operatorName = localStorage.getItem('stock-promise:operator') || '';
  let supervisorName = localStorage.getItem('stock-promise:supervisor-name') || '';
  let profiles: string[] = JSON.parse(localStorage.getItem('stock-promise:profiles') || '[]');
  let reminders = localStorage.getItem('stock-promise:reminders') === 'true';
  let auditEntries: Array<Record<string, any>> = [];
  let importReport = '';
  const notified = new Set<string>();

  $: filteredInventory = (data?.inventory || []).filter((item) =>
    `${item.sku} ${item.name}`.toLowerCase().includes(query.trim().toLowerCase())
  );
  $: totalAvailable = (data?.inventory || []).reduce((sum, item) => sum + item.available, 0);
  $: totalHeld = (data?.inventory || []).reduce((sum, item) => sum + item.held, 0);
  $: expiringSoon = (data?.active_holds || []).filter((hold) => hold.expires_at * 1000 - now < 15 * 60_000).length;

  onMount(() => {
    captureLicense();
    checkLicense().then((value) => license = value);
    load();
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
    window.addEventListener('popstate', () => path = window.location.pathname);
    return () => {
      clearInterval(clock);
      clearInterval(sync);
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  });

  function navigate(next: string) {
    history.pushState({}, '', next);
    path = next;
    window.scrollTo({ top: 0, behavior: 'smooth' });
  }

  async function load(quiet = false) {
    if (quiet) refreshing = true; else loading = true;
    try {
      if (getSession()) {
        data = await request<Bootstrap>('/api/bootstrap', {}, 'required');
        accessRequired = false;
      } else {
        const status = await request<{ setup_required: boolean; server_time: number }>('/api/status');
        if (status.setup_required) {
          data = { setup_required: true, location_name: null, server_time: status.server_time, inventory: [], active_holds: [], recent_outcomes: [] };
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
    modal = null;
    selectedItem = null;
    editingItem = null;
    formError = '';
  }

  async function setup(event: SubmitEvent) {
    busy = 'setup'; formError = '';
    const values = Object.fromEntries(new FormData(event.currentTarget as HTMLFormElement));
    try {
      const result = await request<{ token: string }>('/api/setup', { method: 'POST', body: JSON.stringify(values) });
      setSession(result.token); supervisor = true;
      announcement = 'Location ready. Add the stock your team promises.';
      await load(true);
    } catch (error) { formError = message(error); }
    finally { busy = ''; }
  }

  async function unlock(event: SubmitEvent) {
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
    try { await request('/api/session', { method: 'DELETE' }, 'required'); } catch { /* local lock still applies */ }
    setSession(null); supervisor = false; auditEntries = []; data = null; accessRequired = true;
    announcement = 'Promise desk locked.';
  }

  async function saveInventory(event: SubmitEvent) {
    busy = 'inventory'; formError = '';
    const wasEditing = Boolean(editingItem);
    const values = Object.fromEntries(new FormData(event.currentTarget as HTMLFormElement));
    const payload = { ...values, on_hand: Number(values.on_hand) };
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
    try {
      await request('/api/holds', { method: 'POST', body: JSON.stringify(payload) }, 'required');
      operatorName = String(values.operator_name); localStorage.setItem('stock-promise:operator', operatorName);
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
    try {
      await request(`/api/holds/${hold.id}/resolve`, { method: 'POST', body: JSON.stringify({ action, actor: supervisorName || 'Supervisor' }) }, 'required');
      if (supervisorName) localStorage.setItem('stock-promise:supervisor-name', supervisorName);
      announcement = action === 'convert' ? `Hold for ${hold.customer} converted. Stock count reduced.` : `Hold for ${hold.customer} released. Stock is available again.`;
      await load(true);
    } catch (error) { announcement = message(error); supervisor = Boolean(getSession()); await load(true); }
    finally { busy = ''; }
  }

  async function downloadExport() {
    if (!supervisor) { openModal('unlock'); return; }
    busy = 'export';
    try {
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
          await request('/api/inventory', { method: 'POST', body: JSON.stringify({ sku: cells[indexes[0]], name: cells[indexes[1]], on_hand: Number(cells[indexes[2]]) }) }, 'required');
          imported++;
        } catch (error) { failures.push(`Row ${offset + 2}: ${message(error)}`); }
      }
      importReport = `${imported} item${imported === 1 ? '' : 's'} imported.${failures.length ? ` ${failures.length} skipped: ${failures.slice(0, 3).join(' ')}` : ''}`;
      await load(true);
    } catch (error) { formError = message(error); }
    finally { busy = ''; }
  }

  async function restoreLicense(event: SubmitEvent) {
    const value = String(new FormData(event.currentTarget as HTMLFormElement).get('license') || '');
    if (!value.trim()) return;
    storeLicense(value); license = { unlocked: true, notice: 'Checking license…', token: value };
    license = await checkLicense(true);
    announcement = license.unlocked ? 'Stock Promise Pro unlocked.' : license.notice;
  }

  function addProfile() {
    const value = operatorName.trim();
    if (!value || profiles.includes(value)) return;
    profiles = [...profiles, value].slice(-8);
    localStorage.setItem('stock-promise:profiles', JSON.stringify(profiles));
  }

  async function enableReminders() {
    if (!license.unlocked) return;
    if (!('Notification' in window)) { announcement = 'This browser does not support notifications.'; return; }
    const permission = await Notification.requestPermission();
    reminders = permission === 'granted'; localStorage.setItem('stock-promise:reminders', String(reminders));
    announcement = reminders ? 'Five-minute expiry reminders enabled on this device.' : 'Notification permission was not granted.';
  }

  function sendReminders() {
    if (!license.unlocked || !reminders || !data || Notification.permission !== 'granted') return;
    for (const hold of data.active_holds) {
      if (hold.expires_at * 1000 - Date.now() <= 5 * 60_000 && !notified.has(hold.id)) {
        new Notification(`${hold.sku} hold expires soon`, { body: `${hold.quantity} for ${hold.customer} · ${relativeExpiry(hold.expires_at)}`, icon: '/mark.svg' });
        notified.add(hold.id);
      }
    }
  }

  async function loadAudit() {
    if (!supervisor) return;
    try { auditEntries = (await request<{ entries: Array<Record<string, any>> }>('/api/audit', {}, 'required')).entries; }
    catch (error) { announcement = message(error); supervisor = Boolean(getSession()); }
  }

  function chooseTab(next: Tab) {
    tab = next;
    if (next === 'settings') loadAudit();
  }

  function message(error: unknown): string { return error instanceof Error ? error.message : 'Something went wrong. Try again.'; }
</script>

{#if path === '/privacy' || path === '/terms'}
  <Legal kind={path === '/privacy' ? 'privacy' : 'terms'} {navigate} />
{:else}
  <a class="skip-link" href="#main">Skip to promise desk</a>
  <header class="app-header">
    <a class="wordmark" href="/" onclick={(event) => { event.preventDefault(); chooseTab('desk'); }}>
      <img src="/mark.svg" alt="" width="38" height="38" />
      <span>Stock Promise</span>
    </a>
    <div class="header-status">
      <span class:offline={!online} class="connection"><i></i>{online ? 'Shared live' : 'Offline'}</span>
      {#if data && !data.setup_required}
        <button class="quiet-button" onclick={() => supervisor ? lockSupervisor() : openModal('unlock')}>
          {supervisor ? 'Lock supervisor' : 'Supervisor unlock'}
        </button>
      {/if}
    </div>
  </header>

  {#if !online}<div class="offline-banner" role="status">You’re offline. Current figures may be stale; new promises are paused until the shared server reconnects.</div>{/if}
  <div class="live-region" aria-live="polite">{announcement}</div>

  {#if loading}
    <main id="main" class="loading-state" aria-busy="true">
      <p class="eyebrow">Opening the stockroom</p><h1>Finding today’s promises…</h1><div class="loader"></div>
    </main>
  {:else if accessRequired}
    <main id="main" class="center-state access-gate">
      <p class="eyebrow">Staff access</p>
      <h1>Open the promise desk.</h1>
      <p>Operational stock and customer references are private to this location. Enter the shared supervisor PIN to continue.</p>
      <form onsubmit={(event) => { event.preventDefault(); unlock(event); }}>
        <label for="access-pin">Supervisor PIN <span>6–12 digits</span></label>
        <input id="access-pin" name="pin" type="password" inputmode="numeric" pattern="[0-9]+" minlength="6" maxlength="12" autocomplete="current-password" required />
        {#if formError}<p class="form-error" role="alert">{formError}</p>{/if}
        <button class="primary-button" disabled={busy === 'unlock'}>{busy === 'unlock' ? 'Opening desk…' : 'Open promise desk'}</button>
      </form>
    </main>
  {:else if fatalError}
    <main id="main" class="center-state">
      <p class="eyebrow alarm">Shared server unavailable</p><h1>The promise desk can’t open yet.</h1><p>{fatalError}</p><button class="primary-button" onclick={() => load()}>Try again</button>
    </main>
  {:else if data?.setup_required}
    <main id="main" class="setup-layout">
      <section class="setup-art">
        <picture><source media="(max-width: 700px)" srcset="/assets/stockroom-watch-640.webp" /><img src="/assets/stockroom-watch-1536.webp" srcset="/assets/stockroom-watch-1024.webp 1024w, /assets/stockroom-watch-1536.webp 1536w" sizes="(max-width: 800px) 100vw, 58vw" width="1536" height="1024" alt="An orderly stockroom aisle where one small group of cartons is picked out by a warm work light" fetchpriority="high" decoding="async" /></picture>
        <div class="art-copy"><p class="eyebrow">One location · one live truth</p><h1>Promise what’s there. Once.</h1><p>Create a visible, timed claim while the order is still being written.</p></div>
      </section>
      <section class="setup-form-wrap" aria-labelledby="setup-title">
        <p class="step">First shift setup</p><h2 id="setup-title">Name this stockroom</h2><p>This takes about a minute. The PIN protects stock edits, conversions, and exports.</p>
        <form onsubmit={(event) => { event.preventDefault(); setup(event); }}>
          <label for="location">Location name</label><input id="location" name="location_name" autocomplete="organization" maxlength="80" required placeholder="e.g. Main counter" />
          <label for="setup-pin">Supervisor PIN <span>6–12 digits</span></label><input id="setup-pin" name="pin" type="password" inputmode="numeric" pattern="[0-9]+" minlength="6" maxlength="12" autocomplete="new-password" required />
          {#if formError}<p class="form-error" role="alert">{formError}</p>{/if}
          <button class="primary-button" disabled={busy === 'setup'}>{busy === 'setup' ? 'Securing location…' : 'Open the promise desk'}</button>
        </form>
        <p class="fine-print">Your operational data stays in this installation’s local database.</p>
      </section>
    </main>
  {:else if data}
    <div class="app-frame">
      <aside class="scene-rail">
        <img src="/assets/stockroom-watch-640.webp" width="640" height="427" alt="An orderly stockroom with a finite carton group under a warm work light" fetchpriority="high" decoding="async" />
        <div class="scene-shade"></div>
        <div class="scene-content">
          <p class="eyebrow">{data.location_name}</p>
          <h1>Promise desk</h1>
          <p class="scene-note">A soft hold is a team signal, not a legal reservation.</p>
          <dl class="rail-metrics">
            <div><dt>Available now</dt><dd>{totalAvailable.toLocaleString()}</dd></div>
            <div><dt>On hold</dt><dd>{totalHeld.toLocaleString()}</dd></div>
            <div><dt>Due in 15 min</dt><dd>{expiringSoon}</dd></div>
          </dl>
        </div>
      </aside>

      <main id="main" class="workspace">
        <nav class="section-nav" aria-label="Promise desk sections">
          <button class:active={tab === 'desk'} aria-current={tab === 'desk' ? 'page' : undefined} onclick={() => chooseTab('desk')}>Live desk <span>{data.active_holds.length}</span></button>
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
                      <button class="convert-button" disabled={busy === hold.id || !online} onclick={() => resolve(hold, 'convert')}>{busy === hold.id ? 'Working…' : 'Convert'}</button>
                      <button class="release-button" disabled={busy === hold.id || !online} onclick={() => resolve(hold, 'release')}>Release</button>
                    </div>
                  </li>
                {/each}
              </ol>
            {/if}
          </section>
        {:else if tab === 'outcomes'}
          <section class="panel-head"><div><p class="eyebrow">The completed ledger</p><h2>Recent outcomes</h2><p>Converted, released, and automatically expired promises.</p></div><button class="primary-button small" onclick={downloadExport} disabled={busy === 'export'}>{busy === 'export' ? 'Preparing…' : 'Export CSV'}</button></section>
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
          <section class="panel-head"><div><p class="eyebrow">Supervisor station</p><h2>Stock & settings</h2><p>Keep the shared list current and review its immutable activity trail.</p></div><div class="action-row"><button class="secondary-button" onclick={() => supervisor ? openModal('import') : openModal('unlock')}>Import CSV</button><button class="primary-button small" onclick={() => supervisor ? openModal('inventory') : openModal('unlock')}>Add stock</button></div></section>
          <section class="settings-section"><div class="section-title"><div><h3>Inventory</h3><p>{data.inventory.length} shared SKU{data.inventory.length === 1 ? '' : 's'}</p></div>{#if !supervisor}<button class="quiet-button" onclick={() => openModal('unlock')}>Unlock to edit</button>{/if}</div>
            <ul class="settings-stock">{#each data.inventory as item}<li><div><strong>{item.sku}</strong><span>{item.name}</span></div><div><b>{item.on_hand}</b> on hand</div><button disabled={!supervisor} onclick={() => openModal('inventory', item)}>Edit</button></li>{/each}</ul>
          </section>
          <section class="pro-section"><div><p class="eyebrow">Optional team convenience</p><h3>{license.unlocked ? 'Stock Promise Pro is active' : 'Add Pro reminders & profiles'}</h3><p>Core holds, supervisor controls, safety checks, audit history, and CSV export always remain available.</p></div>
            {#if license.unlocked}
              <div class="pro-controls"><label for="profile-name">Operator profile name</label><div class="inline-form"><input id="profile-name" bind:value={operatorName} maxlength="80" /><button class="secondary-button" onclick={addProfile}>Save profile</button></div>{#if profiles.length}<div class="chips">{#each profiles as profile}<button onclick={() => operatorName = profile}>{profile}</button>{/each}</div>{/if}<button class="primary-button small" onclick={enableReminders}>{reminders ? 'Reminders enabled' : 'Enable 5-minute reminders'}</button></div>
            {:else}
              <div class="price-lock"><strong>$39 <span>one-time</span></strong><p>Saved operator profiles and on-device expiry notifications.</p><a class="primary-button" href={buyUrl}>Buy Pro securely</a></div>
            {/if}
            {#if license.notice}<p class="license-notice">{license.notice} <a href={buyUrl}>View purchase options</a></p>{/if}
            <form class="restore-form" onsubmit={(event) => { event.preventDefault(); restoreLicense(event); }}><label for="license">Have a license? Paste it here</label><div class="inline-form"><input id="license" name="license" autocomplete="off" /><button class="secondary-button">Verify license</button></div></form>
          </section>
          <section class="settings-section"><div class="section-title"><div><h3>Audit trail</h3><p>Append-only record, newest first</p></div>{#if supervisor}<button class="icon-button" aria-label="Refresh audit trail" onclick={loadAudit}>↻</button>{/if}</div>
            {#if !supervisor}<div class="locked-copy"><p>Unlock supervisor access to inspect the audit trail.</p><button class="secondary-button" onclick={() => openModal('unlock')}>Supervisor unlock</button></div>
            {:else if auditEntries.length === 0}<p class="muted">No recorded activity yet.</p>
            {:else}<ol class="audit-list">{#each auditEntries.slice(0, 30) as entry}<li><span class="audit-dot"></span><div><strong>{String(entry.event).replace('.', ' ')}</strong><p>{entry.actor} · {formatTime(entry.created_at)}</p></div></li>{/each}</ol>{/if}
          </section>
        {/if}
      </main>
    </div>
  {/if}

  {#if modal}
    <dialog bind:this={dialog} onclose={() => modal = null} oncancel={() => modal = null} aria-labelledby="dialog-title">
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
      {/if}
    </dialog>
  {/if}

  <footer class="site-footer">
    <span>Soft holds, clearly seen.</span><nav aria-label="Legal"><a href="/privacy" onclick={(event) => { event.preventDefault(); navigate('/privacy'); }}>Privacy</a><a href="/terms" onclick={(event) => { event.preventDefault(); navigate('/terms'); }}>Terms</a></nav><span>Environmental image created with AI; no depicted people or products.</span>
  </footer>
{/if}
