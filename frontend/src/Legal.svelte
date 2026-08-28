<script lang="ts">
  export let kind: 'privacy' | 'terms';
  export let navigate: (path: string) => void;
</script>

<a class="skip-link" href="#legal-main">Skip to content</a>
<header class="legal-header">
  <a href="/" onclick={(event) => { event.preventDefault(); navigate('/'); }} aria-label="Back to Stock Promise">
    <img src="/mark.svg" alt="" width="40" height="40" />
    <span>Stock Promise</span>
  </a>
</header>
<main id="legal-main" class="legal-copy">
  {#if kind === 'privacy'}
    <p class="eyebrow">Product policy · 28 August 2026</p>
    <h1>Privacy, kept close to the stockroom.</h1>
    <p>Stock Promise stores the inventory, customer references, operator names, and hold notes that your team enters. In a self-hosted installation, that operational data stays in your SQLite database on the server you control.</p>
    <h2>What is stored</h2>
    <p>Inventory records, temporary holds, their outcomes, and an append-only audit trail are stored so the shared promise ledger works. A supervisor session token is kept only in the browser tab. Your operator name, optional Pro license, and reminder preferences are stored in this browser.</p>
    <h2>What is sent elsewhere</h2>
    <p>There is no advertising, behavioral analytics, or third-party runtime script. If you buy or verify a Pro license, the license token is sent to Sociobot’s billing API. Checkout is operated by Sociobot/Dodo as merchant of record under their policies.</p>
    <h2>Retention and control</h2>
    <p>The operator of your Stock Promise server controls retention and database backups. CSV exports are created only when an unlocked supervisor requests one. Clearing site data removes browser-only preferences and license tokens; it does not erase shared server records.</p>
    <h2>Contact</h2>
    <p>For a data request, contact the person or business operating your Stock Promise installation. Product questions can be sent to <a href="mailto:privacy@sociobot.in">privacy@sociobot.in</a>.</p>
  {:else}
    <p class="eyebrow">Product terms · 28 August 2026</p>
    <h1>A visible promise, not a legal reservation.</h1>
    <p>Stock Promise is an operational coordination tool. A soft hold helps coworkers avoid duplicate promises while an order is being prepared; it does not create ownership, a sale, a warehouse allocation, or any legal right to goods.</p>
    <h2>Your responsibilities</h2>
    <p>You are responsible for keeping on-hand counts accurate, choosing suitable hold durations, protecting the supervisor PIN, and confirming an order in your system of record. Review expired and unresolved holds as part of normal operations.</p>
    <h2>Service and data</h2>
    <p>The software is provided “as is,” without a guarantee of uninterrupted availability. Maintain backups of the SQLite data volume. Do not enter sensitive personal, payment, health, or authentication data in customer names or order notes.</p>
    <h2>Pro license</h2>
    <p>Stock Promise Pro is a $39 one-time purchase for the features described at checkout. Sociobot/Dodo is the merchant of record and handles payment and refunds. A refund, expiry, or revocation disables the Pro convenience features; core hold safety and CSV export remain available.</p>
    <h2>Fair use</h2>
    <p>You may use the service for lawful inventory coordination. Do not attempt to disrupt the service, bypass access controls, or use it to misrepresent stock availability to customers.</p>
  {/if}
  <button class="text-button" onclick={() => navigate('/')}>Return to the promise desk</button>
</main>
<footer class="site-footer legal-footer">Stock Promise · Built for one shared location</footer>

