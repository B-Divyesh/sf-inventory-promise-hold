# Stock Promise demo

Open `/?demo=1` or select **Try it with sample data** on the home page. The
`/demo` route remains an equivalent direct link.

The sample stockroom contains three plausible counter items, one timed hold for
Northline Plumbing, and one converted outcome for Tideway Maintenance. It is
usable without a sign-in and does not call any live data write endpoint.

Demo stock, operator names, profiles, reminders, and license state are stored
only in the browser session under `demo:stock-promise:*`. The demo never reads
or writes live workspace keys. The persistent banner says **Demo — sample data,
nothing is saved**. **Reset demo** clears every demo key and restores the
shipped sample. **Leave demo** returns home and discards the demo
namespace.

The sample shell is available offline after the first visit. Live stock remains
online-only so a stale device cannot make a promise.
