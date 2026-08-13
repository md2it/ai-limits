# Tauri UI Provider Block Accent Color

## Provider Accent Color

Each provider block owns one provider accent token. The token defines the provider-specific brand color used by prominent provider-scoped UI elements.

Provider-specific color values, including theme-tuned variants, should live together in the provider token block. Theme and component selectors should map or consume those tokens rather than scattering provider color values across the stylesheet.

The provider block border and any internal divider must use the same provider border token stack. This includes the divider that separates provider metadata from provider controls, and the labelled section dividers that open the Limits and Subscription sections (see [provider-blocks.md](provider-blocks.md)). It also includes the base provider border color and any theme-specific internal overlay/highlight layer used to tune the visible card border. This keeps every divider visible in both light and dark themes and prevents provider-specific overrides, such as Cursor dark theme tuning, from drifting between the card border and its internal separators.

## Section Heading Label

The section heading label sits centered in its divider and is styled as secondary text, not as a provider-accented element: it names the content below it and must not compete with the limit bars or the provider heading for attention. It uses the muted/secondary foreground token already used for other supporting text in the block, such as the source line, so that a card reads as one heading, one set of numbers, and quiet structure between them.

The label does not take the provider accent color. Section headings are identical across providers by design, since their purpose is to let the eye compare the same metric across cards; tinting them per provider would work against that.

The provider background may use the same provider accent color with lower opacity. Theme-specific overrides may tune opacity or border brightness per provider, but they must do so through the shared provider tokens rather than hard-coding separate divider colors.
