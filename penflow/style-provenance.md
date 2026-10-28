# Style provenance — Handy mockups (pass B)

The pen.dev mockups in `penflow/ui.pen` reproduce the current Handy UI from its
source code. This file records where every style value comes from, the values
that fall outside Julien's spacing scale, and what pen.dev cannot reproduce.
Token values live in [theme.css](theme.css); they are mirrored as `$pf-*`
variables in `ui.pen`.

## Root unit: 1rem = 15px

`src/App.css:20` sets `:root { font-size: 15px }`. Handy uses Tailwind v4.1.16,
whose spacing, type and radius utilities are in `rem`
(`node_modules/tailwindcss/theme.css`). Every Tailwind value is therefore
15/16 of its usual pixel size:

| Utility | Tailwind | Handy px |
|---|---|---|
| spacing unit `--spacing` (theme.css:277) | 0.25rem | 3.75 |
| `p-1`, `gap-1`, `py-1` | 1 unit | 3.75 |
| `p-2`, `gap-2`, `px-2`, `space-y-2` | 2 units | 7.5 |
| `pt-3`, `pb-3`, `gap-3`, `px-3` | 3 units | 11.25 |
| `p-4`, `px-4`, `gap-4`, `m-4` | 4 units | 15 |
| `space-y-6` | 6 units | 22.5 |
| `w-4 h-4` (icons) | 4 units | 15 |
| `w-5 h-5` (toggle knob, alert icon) | 5 units | 18.75 |
| `h-6` (toggle track) | 6 units | 22.5 |
| `w-11` (toggle track) | 11 units | 41.25 |
| `min-h-12` (setting row) | 12 units | 45 |
| `w-40` (sidebar) | 40 units | 150 |
| `max-w-xs` | 20rem | 300 |
| `text-xs` / line height (theme.css:299-300) | 0.75rem / 1rem | 11.25 / 15 |
| `text-sm` / line height (theme.css:301-302) | 0.875rem / 1.25rem | 13.125 / 18.75 |
| `text-base` / line height (theme.css:303-304) | 1rem / 1.5rem | 15 / 22.5 |
| `tracking-wide` (theme.css:339) | 0.025em | 0.28 at 11.25px |
| `rounded` (theme.css:350) | 0.25rem | 3.75 |
| `rounded-md` (theme.css:351) | 0.375rem | 5.625 |
| `rounded-lg` (theme.css:352) | 0.5rem | 7.5 |

Arbitrary values in brackets are literal pixels: `py-[5px]`, `min-w-[200px]`,
`after:top-[2px]`. Icon sizes passed as props are literal pixels too
(`width={24}`, `size={14}`, `ResetIcon` 20, `HandyTextLogo width={120}`).
pen.dev strokes are drawn inside the box (`strokeAlignment: inner`), so CSS
`border` widths are added to the padding: `px-2` + 1px border = 8.5,
`py-[5px]` + 1px = 6, `p-1` + 1px = 4.75, `px-4` + 1px = 16.

## Colors (dark mode)

| Token | Value | Source |
|---|---|---|
| background | `#2C2B29` | `src/styles/theme.css:13` `--dark-color-background` |
| foreground (text) | `#FBFBFB` | `src/styles/theme.css:12` `--dark-color-text` |
| logo pink (`logo-primary`) | `#F28CBB` | `src/styles/theme.css:14` |
| logo stroke | `#FAD1ED` | `src/styles/theme.css:15` |
| logo highlight | `#F9C5E8` | `src/components/icons/HandyTextLogo.tsx` last path `fill` |
| UI pink (`background-ui`) | `#DA5893` | `src/styles/theme.css:26` (primary button, toggle on) |
| mid-gray | `#808080` | `src/styles/theme.css:28` |
| warning | `#FBBF24` | `src/styles/theme.css:22` (theme only; components use Tailwind yellows below) |
| error | `#F87171` | `src/styles/theme.css:23` (theme only; components use Tailwind reds below) |
| white | `#FFFFFF` | Tailwind `--color-white` (theme.css:275), `Button.tsx:27` |
| gray-300 | `#D1D5DC` | Tailwind theme.css:229, `ToggleSwitch.tsx:47` knob border |
| red-400 / 500 / 600 | `#FF6467` / `#FB2C36` / `#E7000B` | Tailwind theme.css:14-16 (oklch → sRGB) |
| yellow-400 / 500 / 600 | `#FDC700` / `#F0B100` / `#D08700` | Tailwind theme.css:50-52 |
| green-400 / 500 | `#05DF72` / `#00C950` | Tailwind theme.css:74-75 |
| blue-400 / 500 | `#51A2FF` / `#2B7FFF` | Tailwind theme.css:134-135 |

Opacity utilities keep the base color with an alpha byte:
`mid-gray/5 /10 /20 /40 /60 /80` = `#808080` + `0D 1A 33 66 99 CC`;
`logo-primary/10 /20 /30 /80` = `#F28CBB` + `1A 33 4D CC`;
`text/40 /60 /70 /80 /90` = `#FBFBFB` + `66 99 B3 CC E6`;
`*-500/10` alert surfaces and `*-500/20` badge surfaces; `black/50` dialog backdrop;
`rgb(0 0 0 / 0.1)` shadows (`shadow-lg`, theme.css:362; `shadow-xl`, theme.css:363).

Recording overlay, `src/overlay/RecordingOverlay.css` (dark values):
surface `#2C2B29FA` (:7, background at 98%), border `#FBFBFB1F` (:36, text 12%),
hairline `#FBFBFB14` (:38, text 8%), muted `#A3A09A` (:29), faint `#6F6C66` (:31),
accent soft `#F28CBB33` (:21-25), live text `#FBFBFBE6` (:256).

## Typography

Handy sets no `font-family`: the settings window uses Tailwind's default stack
`ui-sans-serif, system-ui, sans-serif, …` (tailwind theme.css:2-4) and the
overlay uses `-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`
(`RecordingOverlay.css:4`). Both resolve to the OS UI font (SF Pro on macOS).
The theme token is `--font-sans: system-ui`.

| Use | Size / weight | Source |
|---|---|---|
| Sidebar label | text-sm, medium | `Sidebar.tsx:115` |
| Group title | text-xs, medium, uppercase, tracking-wide, mid-gray | `SettingsGroup.tsx:18` |
| Setting title | text-sm, medium | `SettingContainer.tsx:132-133` |
| Inline description | text-sm, normal, text color | `SettingContainer.tsx:187` |
| Dropdown value, input, textarea, shortcut chip | text-sm, semibold | `Dropdown.tsx:68`, `Input.tsx:14`, `Textarea.tsx:14`, `shortcut-chip.tsx:46` |
| Button | text-sm (md) / text-xs (sm), medium | `Button.tsx:23,46-47` |
| Badge | text-xs, medium | `Badge.tsx:22` |
| Alert | text-sm, `*-400` | `Alert.tsx:61` |
| Form label | text-sm, semibold | `PostProcessingSettings.tsx:303` |
| Dialog title | text-base, semibold | `Dialog.tsx:175`, `UpdateChecker.tsx:214` |
| Footer | text-xs, text/60 | `Footer.tsx:26` |
| Overlay labels | 12px, muted | `RecordingOverlay.css:348,463` |
| Overlay live text | 15px italic, line height 1.35, -0.003em | `RecordingOverlay.css:252-256` |

## Components (geometry)

| Component | Values | Source |
|---|---|---|
| Window | 680 × 570 | `src-tauri/src/lib.rs:991` |
| Sidebar | width 150, `px-2`, right border mid-gray/20; logo 120 wide, `m-4`; nav `pt-2`, top border, `gap-1`; item `p-2 gap-2 rounded-lg`, icon 24, active `bg-logo-primary/80`, inactive opacity 0.85 | `Sidebar.tsx:96-119` |
| Sidebar icons | General = `HandyHand` SVG; History, Cpu, Cog, Sparkles, Info = lucide-react 0.542 | `Sidebar.tsx:3-76` |
| Content | `p-4 gap-4`, groups `space-y-6`, `max-w-3xl` | `App.tsx:378`, `GeneralSettings.tsx:22` |
| Settings group | `space-y-2`; title wrapper `px-4`; card `bg-background border border-mid-gray/20 rounded-lg`, `divide-y` | `SettingsGroup.tsx:15-27` |
| Setting row | `min-h-12 px-4 p-2`, justify-between, title + 15px info icon (`gap-2`) | `SettingContainer.tsx:124-165` |
| Info icon | heroicons information-circle path, 15px, stroke 2, mid-gray | `SettingContainer.tsx:144-164` |
| Dropdown | `px-2 py-[5px]`, `min-w-[200px]`, `bg-mid-gray/10 border-mid-gray/80 rounded-md`, chevron 15px `M19 9l-7 7-7-7`; open: `bg-logo-primary/10 border-logo-primary`, menu `mt-1 bg-background border-mid-gray/80 rounded-md shadow-lg`, option `px-2 py-1` (or `px-3 py-2` with description), selected `bg-logo-primary/20` semibold | `Dropdown.tsx:66-131` |
| Toggle | track 41.25 × 22.5 `bg-mid-gray/20`, checked `bg-background-ui`; knob 18.75 white at 2px, `border-gray-300` | `ToggleSwitch.tsx:47` |
| Shortcut chip | `px-2 py-1 text-sm font-semibold border rounded-md`, idle `bg-mid-gray/10 border-mid-gray/80`, recording `border-logo-primary bg-logo-primary/30` | `shortcut-chip.tsx:32-46` |
| Shortcut list | column `items-end gap-1`; row `gap-1`: warning (AlertTriangle 14, yellow-500), chip, remove (`p-1 rounded-md`, X 14); add chip + ResetButton; messages `text-xs max-w-xs text-right` | `shortcut-list.tsx:53-275` |
| Reset button | `p-1 rounded-md border-transparent`, `ResetIcon` 20px stroke 1.5, text/80 | `ResetButton.tsx:14-25`, `icons/ResetIcon.tsx` |
| Button | `rounded-lg border font-medium`; md `px-4 py-[5px] text-sm`, sm `px-2 py-1 text-xs`; primary `bg-background-ui text-white`, secondary `bg-mid-gray/10 border-mid-gray/20`, primary-soft `bg-logo-primary/20`, danger-ghost `text-red-400` | `Button.tsx:22-49` |
| Input / Textarea | `px-3 py-2 bg-mid-gray/10 border-mid-gray/80 rounded-md text-sm font-semibold`, focus `bg-logo-primary/20 border-logo-primary`; textarea `min-h-[100px]` | `Input.tsx:13-22`, `Textarea.tsx:13-18` |
| Word chip | Button secondary sm + X 11.25 (`M6 18L18 6M6 6l12 12`) | `CustomWords.tsx:95-118` |
| Badge | `px-3 py-1 rounded-full text-xs font-medium`; success `bg-green-500/20 text-green-400`, secondary `bg-mid-gray/20 text-text/70` | `Badge.tsx:14-23` |
| Alert | `flex gap-3 p-4 rounded-lg bg-*-500/10`, icon 18.75 `*-500`, text-sm `*-400`; `contained` drops the radius | `Alert.tsx:18-62` |
| Info box | `p-3 bg-mid-gray/5 rounded-md border border-mid-gray/20 text-sm text-mid-gray` | `PostProcessingSettings.tsx:358-359` |
| Dialog | `rounded-lg border-mid-gray/20 bg-background shadow-xl`, backdrop `bg-black/50` | `Dialog.tsx:161-202` |
| Tooltip | width 200, `px-3 py-2 bg-background border-mid-gray/80 rounded-lg shadow-lg`, text-sm centered, `leading-relaxed` | `Tooltip.tsx:19,104` |
| Toast | `bg-background border-mid-gray/20 rounded-lg shadow-lg px-4 py-3 gap-3 text-sm` | `App.tsx:315-316` |
| Footer | `border-t pt-3`, `px-4 pb-3 text-xs text-text/60`; status dot 7.5 (`w-2 h-2`, ready `bg-green-400`), name `max-w-28 truncate`, chevron 11.25; update text, `•`, version, `gap-1` | `Footer.tsx:25-37`, `ModelStatusButton.tsx:52-68`, `UpdateChecker.tsx:244-259` |
| Overlay pill | 172 × 40 + 1px border (content-box), radius 24, `padding: 0 10px`, three-column grid (22px min sides); dot 7px; 9 bars 4px wide, gap 3, radius 2, 3-18px high; cancel 22px circle `--s-hair`, X 9px stroke 1.6 | `RecordingOverlay.css:53-60,162-182,295-445`, `RecordingOverlay.tsx:19,232-270` |
| Overlay working pill | 216 wide, radius 18, spinner 13px + 2px ring, label 12px muted | `RecordingOverlay.css:56,205-221,448-471` |
| Overlay open card | radius 16, text region padding 12/14, 15px italic text | `RecordingOverlay.css:201-204,240-256` |

## Values outside the 2/4/8/12/16/24/32/40/48/64/80 scale

Fidelity to the code wins over the spacing scale (`ai-ressources/design/tokens/spacing.md`).
Every off-scale value below comes from Handy:

| Value | Where | Why |
|---|---|---|
| 1 | dropdown menu padding | 1px border kept inside the box |
| 1.875 | text stacks, alert icon offset | Tailwind `0.5` / `mt-0.5` at 15px |
| 3 | overlay waveform gap | `RecordingOverlay.css:365` |
| 3.75 | most `gap-1`, `p-1` | Tailwind `1` at 15px |
| 4.75 | chips, reset button, small buttons | `p-1`/`py-1` + 1px border |
| 5 | overlay pill left column | `.sbase-l` padding-left 5px (:308) |
| 6 | overlay shortcut badge (horizontal) | derived from the shortcut chip at overlay scale (indicator revision) |
| 6 | dropdown, buttons (vertical) | `py-[5px]` + 1px border |
| 7.5 | rows, gaps, nav items | Tailwind `2` at 15px |
| 8.5 | dropdown, chips (horizontal) | `px-2` + 1px border |
| 10 | overlay right column gap | `.sbase-r` gap 10px (:315) |
| 11 | overlay pill padding | `.sbase` padding 10px + 1px border |
| 11.25 | alerts gap, info boxes | Tailwind `3` at 15px |
| 12.25 | inputs, tooltip (horizontal) | `px-3` + 1px border |
| 14 | overlay transcript padding | `.stext-cap` padding-left/right 14px (:250-251) |
| 15 | content, rows, footer, alerts | Tailwind `4` at 15px |
| 22.5 | gap between settings groups | `space-y-6` at 15px |
| radius 3.75 / 5.625 / 7.5 / 11.25 | `rounded`, `rounded-md`, `rounded-lg`, `rounded-full` on 22.5px | Tailwind radii at 15px |
| radius 11 / 18 / 24 | overlay cancel circle (22px, 50%), working pill, pill | `RecordingOverlay.css:167,207,416` |
| font 11.25 / 13.125 | `text-xs`, `text-sm` | Tailwind type at 15px |

`lint-tokens` runs with these scales passed explicitly:
`--spacing-scale 0,1,1.875,2,3,3.75,4,4.75,5,6,7.5,8,8.5,10,11,11.25,12,12.25,14,15,16,22.5,24,32,40,48,64,80`,
`--radius-scale 0,2,3.75,5.625,7.5,11,11.25,12,16,18,24`,
`--fontsize-scale 11.25,12,13.125,15,16.875,18.75`. Penflow rounds fractional `px` theme values when it loads
`theme.css` (for example 7.5 → 8); the pen variables keep the exact values.

## New elements (no Handy equivalent)

Feature elements that Handy does not have yet are built from the primitives above:

- Microphone priority list, replacement and vocabulary lists: nested settings card + setting rows + remove buttons (`p-1 rounded-md`, X 14).
- Status pills: `Badge` geometry; warning, error and info badges reuse the `*-500/20` + `*-400` recipe of the success badge.
- List / Text switch: Button `primary-soft` (selected) and `ghost`.
- Import mode radio: 15px circle with the Input border and the logo-pink focus color (Handy has no radio control).
- Recording indicator layout (revision of 2026-10-14, requested by Julien from two Spokenly captures; only the
  layout is borrowed, never Spokenly's brand or colors). It applies to every screen that shows the indicator:
  `recording-indicator`, `-destination-menu`, `-no-destination`, `-prompt-menu` and `-pinned`.
  - The `bar` is a vertical stack (`gap` 8, centered): the prompt pill sits **above** the indicator.
  - Prompt pill (`<screen>.prompt`): overlay open card (surface, border, radius 16), padding 8/12. It shows the
    prompt shortcut badge, the prompt name (12px, semibold, `--s-text`) and a secondary line
    "Hover to see options" (11.25px, `--s-muted`). The shortcut shown is the prompt's own shortcut from
    feature 005 (the `prompt:<id>` target of feature 004): "⌃ ⌥ M" for Minimal correction, "⌃ ⌥ N" for Notes,
    nothing for prompts without a shortcut, as listed on the AI prompts screen.
  - The prompt pill uses `layout: none` (absolute children) on purpose: Penflow derives the button copy from a
    *direct* inherited `content` child, so the name cannot be wrapped in a text-column frame. The badge sits at
    12/15, the name at 69/8 and the secondary line at 69/26 (pill 192 × 49).
  - Shortcut badge: Handy shortcut-chip recipe at overlay scale — border `--s-border`, radius 5.625
    (`rounded-md`), padding 2/6, text 11.25px semibold `--s-muted` (no fill, so the muted text keeps WCAG AA on
    the overlay surface; on the selected menu row the badge text uses `--s-text`).
  - Indicator: one overlay pill (surface, border, radius 24, 42px high, padding 3, gap 4) holding the
    destination (feature 006: app icon and name, remove button) on the left, a 1 × 20 hairline divider
    (`--s-border`), then the Handy recording pill content (live dot, level bars, timer, cancel). The destination
    keeps its click target (destination menu) without a chip fill.
  - Hover state (`recording-indicator-prompt-menu`, state `prompt-menu-open`): the prompt pill takes the
    Handy dropdown open style (border `logo-primary`, `Dropdown.tsx`) and drops the secondary line; the list
    opens above it (`gap` 8) and gives every row a fixed 48px shortcut slot, then the name; the active prompt
    keeps `--s-accent-soft` and the check. The contract schema only accepts `click`/`submit` triggers, so the
    action stays `click`; opening on hover is a presentation rule for the implementation (click remains the
    keyboard-free fallback).
  - No new screen was needed: the expanded state already exists as `recording-indicator-prompt-menu`
    (`prompt-menu-open`); the collapsed state is the prompt pill of the other indicator screens. New contract
    nodes: `<screen>.prompt.shortcut`, `<screen>.prompt.hint` (not on the expanded screen) and
    `recording-indicator-prompt-menu.menu.prompt.shortcut`; `bar` children are reordered to the visual order
    (prompt, destination, pill).
- Overlay menus and recovery bubble: overlay open card (radius 16); selected item uses `--s-accent-soft`; inner corner radius 12 is derived (16 minus the 4px padding), not read from the code.
- The desktop behind the overlay (`#1E1D1C`) is scenery, not a Handy style.
- App icons (006, decision of 2026-10-17): the monogram tile Handy draws while `icon_ref` is empty, `src/overlay/indicator.css:139-154` (20 × 20, radius 5.625, `--s-accent-soft` = `$pf-palette-overlay-accent-soft`, first letter at 11.25 / 600 in the text colour).
- State variants (alignment of 2026-10-17): states that the delivered UI shows one at a time (loading, refusals, dialogs, platform options, gate-dependent rows) are hidden layers (`enabled: false`) carrying their own `context`, built from the same Handy styles. The screen PNGs show the default state; toggle a layer in pen.dev to see its variant.

## What pen.dev cannot reproduce

- **System font.** pen.dev only renders Google Fonts; `system-ui` (SF Pro on macOS) is reported as invalid and pen.dev falls back to its default sans. The files keep `system-ui` so code generation and `lint-tokens` stay faithful; the PNGs show the fallback face, not SF Pro.
- **`uppercase`.** pen.dev has no text transform. Group titles keep the DOM string ("General") in a hidden inherited text node, and a decorative twin shows the rendered capitals ("GENERAL").
- **Truncation.** `truncate` (ellipsis) is approximated: dropdown values clip to one line; the footer model name is written "Parakeet TDT 0.6...".
- **Window chrome.** The macOS title bar is native, outside the webview; the mockups show the 680 × 570 webview only.
- **Shortcut labels.** Handy formats shortcuts as words ("Option + Space"). The prompt list and the prompt editor use Handy's wording ("Ctrl + Option + M") since the 2026-10-17 alignment; the dictation shortcut screens and the overlay still show the pass A glyph strings ("⌥ Space").
- **Archive lifecycle.** pen.dev 1.2 rejects the `penflow.lifecycle` property on a frame ("unexpected property"), so a region cannot be archived inside `ui.pen`; an obsolete screen is removed from the canvas and kept in git history.
- **Light mode.** Handy supports it (`src/styles/theme.css:7-10,20-21`), but the mockups are dark only, like the reference capture.
