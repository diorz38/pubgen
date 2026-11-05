# POS app icon

The Publication Orchestrator Studio (POS) icon: a gold **PCS** monogram inside a rounded-square
outline, on a transparent field.

## Palette

| Colour | Hex | Used for |
|---|---|---|
| Gold | `#f2b34a` | The monogram and the rounded-square outline |

The source artwork is `pos_logo.png` (with `pos_logo.svg` as its traced vector companion).

## Geometry

The icon is rendered on a square canvas with the artwork centred and a small transparent margin.
Windows and Linux use the full tile. The macOS icons (`.icns`, `pos-macos-512.png`) sit on
Apple's grid: a smaller body with a transparent margin.

## Files

| File | Use |
|---|---|
| `pos_logo.png` | Source raster artwork |
| `pos_logo.svg` | Traced vector companion of the source artwork |
| `pos.svg` | Square, padded SVG wrapper used as the canonical artwork |
| `pos-small.svg` | Lighter square SVG wrapper; also `hicolor/scalable` |
| `pos-1024.png` | 1024 px render of the full tile |
| `pos-macos-512.png` | Runtime Dock icon on macOS |
| `pos.icns` | macOS bundle icon (`CFBundleIconFile`) |
| `pos.ico` | Windows exe icon, 16–256 px |
| `hicolor/<size>/apps/ai.storyteller.pos.png` | Linux icon theme, 16–512 px; the 256 px one is also the runtime window icon on Windows and Linux |
| `hicolor/scalable/apps/ai.storyteller.pos.svg` | Linux scalable icon |
| `LICENSE.txt` | MIT OR Apache-2.0 |

The icon theme files use the app ID `ai.storyteller.pos`. Keep the Wayland app ID and the Linux
`.desktop` file in sync with that name.

## Regenerate

The raster sizes, `.ico` and `.icns` are generated from `pos_logo.png` with Pillow. The square SVG
wrappers embed the same artwork so they render consistently at any size.
