# QuireForge Brand Sources

QuireForge is an unofficial native Linux workspace for Codex.

> QuireForge is an independent community project. It is not made, endorsed,
> supported, or distributed by OpenAI.

These files are the maintainable source of the QuireForge visual identity. They
are intentionally independent of OpenAI and ChatGPT logos, iconography, and
visual systems.

## Concept

The mark is an angular `Q`. Its counter is shaped like an open folio—a quire of
pages—and the diagonal ember stroke forms both the `Q` tail and a piece of hot,
forged material. The octagonal construction gives the mark a tool-like Linux
launcher silhouette without imitating a distribution or vendor logo.

The identity uses flat color, strong geometry, and restrained typography. It
does not depend on gradients, animation, raster effects, or remote assets.

## Source assets

| File | Intended use |
|---|---|
| `quireforge-mark.svg` | Transparent mark on light backgrounds |
| `quireforge-mark-dark.svg` | Transparent mark on dark backgrounds |
| `quireforge-mark-monochrome.svg` | Single-color printing and masks |
| `quireforge-wordmark.svg` | Name without the mark or tagline |
| `quireforge-lockup.svg` | Horizontal mark, name, and tagline on light backgrounds |
| `quireforge-lockup-dark.svg` | Horizontal lockup on dark backgrounds |
| `quireforge-app-icon.svg` | Rounded application-icon source |
| `quireforge-favicon.svg` | Simplified 64-unit browser-icon source |
| `quireforge-social-card.svg` | 1200×630 social-sharing source |

All user-facing SVGs provide a `<title>`, `<desc>`, and `aria-labelledby`.
Lettering is stored as paths, so the assets do not require installed fonts or
make a network font request.

## Palette

| Token | Hex | Use |
|---|---|---|
| Forge ink | `#1B2130` | Primary light-theme mark and text |
| Forge night | `#111620` | Dark tile and social-card background |
| Quire paper | `#F6F1E7` | Primary dark-theme mark and text |
| Ember | `#E45B2B` | Graphic accent and large light-theme wordmark only |
| Ember accessible | `#C9481D` | Small or normal-size text on white |
| Ember bright | `#FF7142` | Accent and text on forge night |
| Steel text | `#4E5668` | Secondary text on white |
| Mist text | `#BAC1CF` | Secondary text on forge night |
| Cloud text | `#AEB7C8` | Tertiary text on forge night |

Measured WCAG contrast ratios include:

- Forge ink on white: 16.07:1.
- Steel text on white: 7.36:1.
- Ember accessible on white: 4.74:1.
- Quire paper on forge night: 16.09:1.
- Ember bright on forge night: 6.63:1.
- Mist text on forge night: 10.02:1.
- Cloud text on forge night: 8.98:1.

Ember `#E45B2B` is 3.60:1 on white. It is appropriate for the existing large,
bold wordmark and non-text graphics, not normal-size light-theme text. Product
UI tokens must use Ember accessible where WCAG normal-text contrast applies.

## Typography and attribution

The wordmark begins with the shapes of Noto Sans Bold. Supporting lockup and
social-card text begins with Noto Sans Regular or Bold. All lettering is
converted to vector outlines for stable rendering.

Noto Sans is Copyright 2010, 2012–2020 Google Inc. and 2015–2020 Google LLC and
is distributed under the SIL Open Font License 1.1. Converting text to outlines
does not embed the font files in these assets. The future website may choose a
different open-licensed interface typeface while retaining the path-based
wordmark.

## Usage rules

- Keep clear space equal to at least one-eighth of the mark width.
- Use the standalone mark at 24 CSS pixels or larger in general UI.
- Use the app-icon source at 32 pixels or larger; the dedicated favicon source
  may be rendered at 16 or 32 pixels.
- Use a horizontal lockup at 180 CSS pixels or larger.
- Do not rotate, skew, outline, add shadows, or rearrange mark components.
- Do not recolor the identity with OpenAI green or place it next to language
  implying official OpenAI ownership or endorsement.
- Use the monochrome mark when production constraints permit only one color.
- When the image is informative, use alt text such as “QuireForge.” When nearby
  text already names the product, treat the image as decorative.

## Export matrix

Production exports belong to the milestone that owns their consuming package.
Generate them from these committed vectors rather than redrawing the mark.

| Consumer | Required exports |
|---|---|
| Tauri/Linux application | 32, 64, 128, 256, and 512 px PNG; toolchain-required platform variants |
| AppImage | 512 px PNG named for QuireForge by the packaging workflow |
| Desktop entry | Installed icon sizes selected by the packaging baseline |
| Astro website | SVG mark/lockups, responsive raster fallbacks where measured useful |
| Browser | SVG favicon plus generated 16 and 32 px fallbacks |
| Social metadata | 1200×630 raster rendered from `quireforge-social-card.svg` |
| GitHub social preview | Approved 1200×630 export uploaded manually in a later gated operation |

Do not commit generated exports until their consumer, build validation, and
update policy exist. Final package filenames and icon installation paths are
tested in the desktop, website, and packaging milestones.

## Source and security rules

- Keep SVGs free of scripts, event handlers, foreign objects, embedded binary
  data, and remote references.
- Convert any future asset text to paths before release.
- Preserve accessible titles and descriptions in source SVGs.
- Validate XML and render assets at their minimum size before committing.
- Treat the social-card copy as public website content; never inject local
  paths, integration state, account data, or release claims automatically.

The repository does not yet contain its final project `LICENSE`; Milestone 1
must establish repository licensing before the assets are represented as a
released open-source brand package.
