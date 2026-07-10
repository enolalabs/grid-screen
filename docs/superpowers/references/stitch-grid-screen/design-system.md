# Design System — Grid Screen (Stitch)

Extracted from the Stitch project "Grid Screen Architect" (3286229551374494803), design system `assets/a44c86b397264e5be1ac75a7ded7975`.

## Color Tokens

| Token | Hex | Usage |
|-------|-----|-------|
| Primary | `#8B5CF6` | Primary buttons, active states, focus rings |
| Primary Bright | `#D0BCFF` | Primary on dark backgrounds, hover/intent |
| Surface Lowest | `#0F0D15` | Canvas background |
| Surface Low | `#1D1A23` | Sidebar, secondary surfaces |
| Surface Container | `#211E27` | Card backgrounds, container elements |
| Surface High | `#2C2832` | Elevated surfaces, hover states |
| Surface Highest | `#37333D` | Topmost surfaces, pressed states |
| Outline | `#494454` | Default borders, dividers |
| Outline Variant | `#958EA0` | Subtle borders, secondary outlines |
| On-Surface | `#E7E0ED` | Primary text |
| On-Surface Variant | `#CBC3D7` | Secondary text, labels |

## Typography

| Level | Size/Weight | Font | Usage |
|-------|-------------|------|-------|
| Headline LG | 24px / 600 | Geist | Page titles |
| Headline MD | 20px / 600 | Geist | Section headers |
| Body MD | 14px / 400 | Geist | Body copy, descriptions |
| Body SM | 12px / 400 | Geist | Secondary text, meta |
| Mono Label | 12px / 500 | JetBrains Mono | Input labels, data keys |
| Mono Data | 11px / 400 | JetBrains Mono | Code, coordinates, values |

## Spacing & Layout

- Base grid: **4px**
- Sidebar: **280px** fixed width
- Component gaps follow 4px multiples (4, 8, 12, 16, 20, 24, 32, 40, 48)

## Borders & Radius

- Border radius: **4px** (roundness)
- Inputs: dark background, mono font, 4px radius
- Cards: minimal padding, 1px dividers
- Chip/badge: **2px** radius
- List items: 1px dividers + 2px violet intent bar on active

## Component Notes

### Buttons
- Height: **32px**
- Radius: **4px**
- Primary: solid fill `#8B5CF6`
- Ghost variant: transparent fill with 1px border
- Focus: 1px violet (`#8B5CF6`) border

### Zones / Grid
- Dashed borders in inactive state
- Solid violet (`#8B5CF6`) border when active/selected

### Inputs
- Dark surface background
- JetBrains Mono font for values
- 4px radius

### Lists
- 1px dividers between items
- 2px violet intent bar on active/focused item

### Cards
- Minimal internal padding
- 1px bottom/section dividers

## Color Mode

- **DARK** mode — all surfaces are dark tones
- Primary seed color: `#8B5CF6` (Electric Violet)

## Fonts

- **Headline & Body:** Geist (SIL OFL 1.1)
- **Labels & Data:** JetBrains Mono (SIL OFL 1.1)
- Fallback stacks: `system-ui, -apple-system, sans-serif` and `ui-monospace, monospace`
