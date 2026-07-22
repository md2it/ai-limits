# Tauri UI Provider Blocks

## Block Structure

Each provider square contains:

- provider name
- limit rows
- credits line, when available
- manual limit-reset availability, when available
- source line with data origin label and timestamp on one line by default
- update frequency dropdown near the bottom
- provider-specific manual update button at the bottom

The provider content should roughly match the current terminal output model.

## Provider Accent Color

Each provider block owns one provider accent token. The token defines the provider-specific brand color used by prominent provider-scoped UI elements.

Provider-specific color values, including theme-tuned variants, should live together in the provider token block. Theme and component selectors should map or consume those tokens rather than scattering provider color values across the stylesheet.

The provider block border and any internal divider that separates provider metadata from provider controls must use the same provider border token stack. This includes the base provider border color and any theme-specific internal overlay/highlight layer used to tune the visible card border. This keeps the divider visible in both light and dark themes and prevents provider-specific overrides, such as Cursor dark theme tuning, from drifting between the card border and its internal separators.

The provider background may use the same provider accent color with lower opacity. Theme-specific overrides may tune opacity or border brightness per provider, but they must do so through the shared provider tokens rather than hard-coding separate divider colors.

Example data shape:

```text
     --------- CURSOR --------
plan | 54.6% left
■■■■■■■■■■■■□□□
reset Jul 28, 03:00
auto | 63.7% left
■■■■■■■■■■■■■■□□□□
api | 24.5% left
■■■■■□□□□□□□□□□□□□□□
API2, as of Jul 5, 19:28

     --------- CODEX ---------
5h | 92.0% left
■■■■■■■■■■■■■■■■■■■■■■■□□
reset 20:48
7d | 35.0% left
■■■■■■■■■□□□□□□□□□□□□□□□□
reset Jul 10, 03:55
Credits: 344.2
Local files, as of Jul 5, 19:28

     --------- CLAUDE --------
5h | 100.0% left
■■■■■■■■■■■■■■■■■■■■■■■■■
reset Jul 6, 00:20
7d | 84.0% left
■■■■■■■■■■■■■■■■■■■■□□□□
reset Jul 7, 13:00
CLI, as of Jul 5, 19:29
```

The UI does not need to use terminal-style ASCII rendering. The example defines the information that must be visible.

## Limit Rows

Each limit row is rendered as a vertical group:

1. Top text line above the bar: `{window} | {remaining}% left`, for example `5h | 59.0% left`.
2. Full-width remaining bar.
3. Reset text line below the bar: `reset {time}`, for example `reset Jul 6, 01:49`.

The limit type, such as `5h`, `7d`, `plan`, `auto`, or `api`, must not consume a separate left column. This lets every bar use 100% of the provider block content width.

The remaining bar shows:

- filled segment width equal to remaining percentage
- unfilled spent segment in white or another very light neutral color
- one solid color for the whole filled segment

The filled segment color is calculated from remaining percentage:

- `100%` is green
- `50%` is yellow
- `1%` is red
- intermediate values are interpolated between these anchors

The bar must not use a left-to-right rainbow gradient inside the filled segment. For example, if `10%` remains, the filled 10% segment is a near-red color and the spent 90% segment stays light.

## Credits Line

When the provider has remaining credits, show one text line directly below the limit rows:

```text
Credits: 344.2
```

The line is hidden when credits are unavailable.

---

## Manual Limit Resets

When `availableLimitResets` is greater than zero, show an informational section after credits and before the source line:

```text
Resets: 1
```

This section shows availability only; it must not contain a control that redeems a reset.

---

## Source Line

Provider source information is shown on one line by default:

```text
Local files, as of Jul 5, 22:12
```

The line has two parts:

- origin label, for example `Local files`, `CLI`, `Statusline`, or `API2`
- timestamp from `dataTimestamp`, rendered as `as of {timestamp}`

Possible origin labels: `Local files`, `CLI`, `Statusline`, `API2`, `Unknown`.

Each part is a non-breaking unit: `{origin label},` and `as of {timestamp}` must not wrap in the middle. If the provider block is too narrow for the full line, the line may break only between these two units.
