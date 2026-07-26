# Widget Presentation

## Content

Each provider card follows the informational content of the [Tauri provider block](../desktop/ui/provider-blocks.md), including the provider name, limit bars, credits, and reset times.

Widgets have no manual refresh or other controls.

Shared presentation and unavailable-data rules are not redefined for widgets.

## Sizes

- small: the highest-priority provider
- medium: the two highest-priority providers
- large: up to three providers

Medium uses two horizontal square cells. Large uses a 2 × 2 grid filled from left to right and top to bottom; its fourth cell shows the application logo.

Extra-large widgets are not currently supported.

## Configuration

The user selects providers and orders them by personal priority.

Changing the widget size changes how many providers are visible without removing hidden selections or changing their order.

## Appearance and Behavior

The widget uses native SwiftUI and WidgetKit composition and reuses the existing [provider accent colors](../desktop/ui/provider-block-colors.md).

Selecting any provider card opens or focuses the desktop application without provider-specific navigation.

The widget gallery shows realistic sample content. An installed widget loads real data.
