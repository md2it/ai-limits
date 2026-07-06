# Desktop Icon Generation

Status: active.

Icon source files:

```text
src-tauri/icons/icon-master.svg
src-tauri/icons/icon-desktop.svg
```

Rules:

- `icon-master.svg` is the master artwork. It intentionally has no internal
  padding.
- `icon-desktop.svg` is the desktop icon source derived from the master artwork.
  It keeps the black background and adds desktop-safe internal padding.
- Desktop PNG, `.ico`, and `.icns` files must be generated from
  `icon-desktop.svg`, not directly from `icon-master.svg`.
- Android and iOS icon folders are not part of this desktop application.

Required local tools:

```text
qlmanage
magick
iconutil
sips
```

Tool roles:

- Use macOS QuickLook through `qlmanage` to render SVG into PNG. This is the
  confirmed renderer for the current SVG artwork.
- Do not use ImageMagick as the SVG renderer for this icon. During verification,
  ImageMagick rendered the small sparkles but dropped the main logo shape.
- Use ImageMagick only to assemble `icon.ico` from already rendered PNG files.
- Use a direct ICNS container build from already rendered PNG files, then verify
  the result with `iconutil`.
- Use `sips` for dimension checks.

Expected desktop icon files:

```text
src-tauri/icons/32x32.png
src-tauri/icons/64x64.png
src-tauri/icons/128x128.png
src-tauri/icons/128x128@2x.png
src-tauri/icons/icon.png
src-tauri/icons/icon-1024.png
src-tauri/icons/icon.ico
src-tauri/icons/icon.icns
```

Windows logo PNG files in `src-tauri/icons/Square*Logo.png` and
`src-tauri/icons/StoreLogo.png` are also generated from `icon-desktop.svg`.

Current desktop icon padding:

```text
source: src-tauri/icons/icon-desktop.svg
canvas: 1024x1024
background: black
artwork scale: 80%
internal padding: about 10% per side
```

Verified behavior:

- Local Tauri macOS build copies `src-tauri/icons/icon.icns` into the `.app`
  bundle without changing it.
- GitHub Actions macOS build also copied `icon.icns` without changing it.
- GitHub Actions Linux `.deb` package copied the checked PNG files without
  changing them.
- Therefore desktop icon padding is controlled by the source icon files, not by
  GitHub Actions or Tauri at build time.
