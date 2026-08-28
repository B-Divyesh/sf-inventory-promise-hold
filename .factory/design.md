# Stock Promise — visual thesis

## Direction: the blue-hour stockroom

Stock Promise uses **cinematic environmental art** to make a mundane but tense
moment visible: the warehouse is quiet, stock is finite, and one warm pool of
light marks inventory that somebody has already promised. The interface should
feel like a calm control desk at the edge of that room—not an ERP, storefront,
or generic dashboard. The generated environmental scene explains the product's
world; every other flourish is restrained.

The product is intentionally single-mode. A painted, near-black navy backdrop
preserves the nocturnal warehouse atmosphere and keeps the live amber promise
signals visually dominant. There is no automatic light theme because it would
break the core environmental metaphor; surfaces and text are explicitly tuned
to accessible contrast in this treatment.

## Palette

| Token | Value | Use |
| --- | --- | --- |
| Night | `#071318` | Canvas; the unlit warehouse |
| Slate | `#10262c` | Raised working surfaces |
| Steel | `#1a343a` | Dividers and inactive controls |
| Chalk | `#f4f0e6` | Primary text |
| Fog | `#aebfc0` | Secondary text (7.4:1 on Night) |
| Promise amber | `#f2ae49` | Primary actions and held stock |
| Ink on amber | `#172023` | Accent contrast |
| Available mint | `#74c9a0` | Confirmed available state |
| Warning sand | `#ffd58a` | Expiry approaching |
| Alarm coral | `#ff8b79` | Errors and destructive actions |

Color never carries meaning alone. Every stock state has a label and, where
useful, an authored icon. Borders remain at least 3:1 against adjacent surfaces;
body text is at least 4.5:1.

## Typography

The product uses two local/system families and makes no font request:

- **Georgia** for the single display heading and occasional numeric statement.
  Its editorial, human cadence makes “promises” feel deliberate.
- **Inter-compatible system sans** (`ui-sans-serif`, `system-ui`, sans-serif)
  for controls and data. It is fast, neutral, and highly legible.

Scale: 14 px metadata, 16 px body/control minimum, 20 px section heading,
32–44 px page heading, 56 px hero stock figure. Inventory numbers use tabular
figures. Text measures stay below 72 characters.

## Space and composition

An 8 px base rhythm (`4, 8, 12, 16, 24, 32, 48, 64`) shapes the interface.
Desktop uses a 280 px contextual rail and a broad operations table; phone drops
the environmental rail after its compact status summary and turns rows into
stacked, glanceable records. Controls are at least 44 px tall, with 8 px between
targets. Cards are used only for independently actionable inventory or hold
records; related controls use proximity and quiet dividers.

Corners are clipped (`2–8 px`) rather than pill-shaped. A narrow amber edge and
soft directional shadow suggest a physical tag laid across a stock ledger.

## Interaction grammar

- **Enter:** a quiet, centered staff gate withholds every operational name and
  count until the shared PIN is accepted; errors stay inline and preserve focus.
- **Create:** a hold drawer enters from the inventory row that originated it.
- **Commit:** successful holds receive one brief amber edge sweep, then settle.
- **Resolve:** conversion recedes into the audit trail; release returns capacity
  to its SKU in place.
- **Urgency:** time remaining is written in plain language and ticks every 30
  seconds. Under five minutes adds “Due soon,” never a flashing effect.
- **Feedback:** buttons show busy labels, server errors explain the recovery,
  and a polite live region announces every mutation.

Motion lasts 160–240 ms and changes only transform/opacity. With
`prefers-reduced-motion: reduce`, drawers and state updates appear instantly and
the decorative light sweep is removed. Nothing loops.

## Asset plan and provenance

### `stockroom-watch`

Subject: a small distributor's aisle seen from a control desk, orderly metal
shelves and plain unbranded cartons, one finite cluster picked out by a warm
temporary work light. World/materials: brushed steel, kraft paper, concrete,
light atmospheric dust. Light/lens: deep blue-hour ambient light, warm tungsten
beam, cinematic 35 mm composition, quiet negative space at upper left. Palette
words: midnight navy, oxidized teal, chalk, promise amber. Negative list: no
people, no text, no labels, no logos, no watermark, no branded products, no
science-fiction holograms, no excessive bloom, no unsafe or chaotic warehouse.

Full generation prompt:

> Cinematic environmental key art for a small distributor inventory control
> tool. View down a calm, orderly compact stockroom aisle from the edge of a
> dark control desk; steel shelving, plain kraft cartons and a few reusable
> totes, one finite cluster of cartons isolated by a warm amber tungsten work
> light suggesting a temporary promise while the rest recedes into midnight
> navy and oxidized teal blue-hour shadow. Brushed steel, paper, concrete,
> subtle atmospheric dust, physically plausible practical lighting, 35mm lens,
> slightly elevated eye line, editorial film still, broad quiet negative space
> in the upper left for interface composition, high detail but restrained. No
> people, no text, no numbers, no labels, no logos, no watermark, no branded
> products, no science-fiction holograms, no excessive bloom, no messy or
> unsafe warehouse.

Generated with the factory Azure image model (`factory-image`) on 2026-08-28.
The output is original to this product. Source PNG and prompt sidecar live in
`assets/src/`; responsive WebP derivatives live in `frontend/public/assets/`.
The footer discloses AI-assisted imagery. Authored SVG interface icons are MIT
licensed with the application.
