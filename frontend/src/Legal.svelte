<script lang="ts">
  export let kind: 'privacy' | 'terms';
  export let navigate: (path: string) => void;
  const buildId = import.meta.env.VITE_BUILD_SHA || 'dev';
</script>

<a class="skip-link" href="#legal-main">Skip to content</a>
<header class="legal-header">
  <a href="/" onclick={(event) => { event.preventDefault(); navigate('/'); }} aria-label="Back to Stock Promise">
    <img src="/mark.svg" alt="" width="40" height="40" />
    <span>Stock Promise</span>
  </a>
  <nav class="top-nav" aria-label="Site"><a href="/demo" onclick={(event) => { event.preventDefault(); navigate('/demo'); }}>Demo</a><a href="/privacy" onclick={(event) => { event.preventDefault(); navigate('/privacy'); }}>Privacy</a></nav>
</header>
<main id="legal-main" class="legal-copy">
  {#if kind === 'privacy'}
    <p class="eyebrow">Product policy · 1 September 2026</p>
    <h1>Privacy for Stock Promise</h1>
    <p>Stock Promise stores the inventory, customer references, operator names, and hold notes that your team enters. The hosted service stores this operational data in its durable database.</p>
    <h2>What is stored</h2>
    <p>Inventory records, temporary holds, outcomes, and the audit record are stored so coworkers see the same stock. The audit record keeps past changes and cannot be edited. Sociobot sign-in tokens stay in the current browser session. Operator preferences, an optional Pro license, and reminder preferences stay in this browser.</p>
    <h2>What is sent elsewhere</h2>
    <p>There is no advertising or behavioral analytics. Sign-in uses Sociobot Microsoft Entra External ID. When an existing Pro license is verified, its token is sent to Sociobot’s billing API. New Pro purchases are temporarily unavailable.</p>
    <h2>Retention and control</h2>
    <p>A supervisor can set a 30–730 day period before resolved customer references, hold notes, and operator names are removed. The audit record does not include those values for new holds. A supervisor can also permanently erase the whole location, including inventory, holds, sessions, and its audit record. Clearing browser data removes browser-only preferences and license tokens, but does not erase shared records.</p>
    <h2>Contact</h2>
    <p>For a data request, ask a supervisor for the location. Product privacy questions can be sent to <a href="mailto:privacy@sociobot.in">privacy@sociobot.in</a>.</p>
  {:else}
    <p class="eyebrow">Product terms · 1 September 2026</p>
    <h1>Terms for temporary stock holds</h1>
    <p>Stock Promise is an operational coordination tool. A soft hold helps coworkers avoid duplicate promises while an order is being prepared; it does not create ownership, a sale, a warehouse allocation, or any legal right to goods.</p>
    <h2>Your responsibilities</h2>
    <p>You are responsible for keeping on-hand counts accurate, assigning staff and supervisor roles, choosing suitable hold durations, and confirming orders in your system of record. Review expired and unresolved holds as part of normal operations.</p>
    <h2>Service and data</h2>
    <p>The service is provided “as is,” without a guarantee of uninterrupted availability. Do not enter sensitive personal, payment, health, or authentication data in customer names or order notes.</p>
    <h2>Pro license</h2>
    <p>New Stock Promise Pro purchases are temporarily unavailable. An existing valid Pro license enables saved operator profiles and on-device expiry reminders. Core holds and CSV export do not require Pro.</p>
    <h2>Fair use</h2>
    <p>You may use the service for lawful inventory coordination. Do not interfere with normal service use or present inaccurate stock availability to customers.</p>
  {/if}
  <a class="text-button" href="/" onclick={(event) => { event.preventDefault(); navigate('/'); }}>Return home</a>
</main>
<footer class="site-footer legal-footer"><span>Timed shared holds for one location.</span><nav aria-label="Legal"><a href="/privacy" onclick={(event) => { event.preventDefault(); navigate('/privacy'); }}>Privacy</a><a href="/terms" onclick={(event) => { event.preventDefault(); navigate('/terms'); }}>Terms</a></nav><span>Built by Param Factory · build {buildId.slice(0, 12)}</span></footer>
