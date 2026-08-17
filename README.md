# ai-limits

<p align="center">| <a href="docs/readmes/DE.md">DE</a> | EN | <a href="docs/readmes/ES.md">ES</a> | <a href="docs/readmes/FR.md">FR</a> | <a href="docs/readmes/PT.md">PT</a> | <a href="docs/readmes/RU.md">RU</a> | <a href="docs/readmes/ZH.md">中文</a> | <a href="docs/readmes/AR.md">عربي</a> |</p>

<p align="center">
   A local app for tracking AI subscription limits and usage across Codex, Claude, and Cursor.
</p>

<p align="center">
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.5.1/AI-Limits-v0.5.1-macos-arm64.dmg"><img src="https://shieldcn.dev/badge/macOS-v0.5.1-grey.svg?logo=apple" alt="Download for macOS"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.5.1/AI-Limits-v0.5.1-windows-setup.exe"><img src="https://shieldcn.dev/badge/Windows-v0.5.1-blue.svg?logo=ri:FaWindows" alt="Download for Windows"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.5.1/AI-Limits-v0.5.1-linux.AppImage"><img src="https://shieldcn.dev/badge/Linux-v0.5.1-yellow.svg?logo=linux" alt="Download for Linux"></a>
</p>

<p align="center"><a href="https://github.com/md2it/ai-limits/releases/tag/v0.5.1">All downloads</a></p>

---

![ai-limits macOS](docs/readmes/screenshots/macos.png)

<p align="center">
  <img src="docs/readmes/screenshots/windows.png" alt="ai-limits on Windows" width="24%">
  <img src="docs/readmes/screenshots/linux.png" alt="ai-limits on Linux" width="24%">
  <img src="docs/readmes/screenshots/macos-light-settings.png" alt="ai-limits settings" width="24%">
  <img src="docs/readmes/screenshots/macos-help.png" alt="ai-limits help" width="24%">
</p>

<p align="center">
  <img src="docs/readmes/screenshots/macos-popover-dark.png" alt="ai-limits macOS popover in dark appearance" width="24%">
  <img src="docs/readmes/screenshots/macos-popover-light.png" alt="ai-limits macOS popover in light appearance" width="24%">
  <img src="docs/readmes/screenshots/macos-notification-center-dark.png" alt="ai-limits notifications in macOS Notification Center" width="24%">
  <img src="docs/readmes/screenshots/macos-notification-center-light.png" alt="ai-limits light notifications in macOS Notification Center" width="24%">
</p>

## Benefits

- Works without an API subscription,
- No separate AI Limits account: use your existing provider authorization,
- All providers in one place,
- Completely free,
- Private: no third-party services, proxies, or registrations,
- Lightweight desktop app for macOS, Windows, and Linux,
- Limit notifications,
- Open source.

## Features

- Shows limits, reset date and time, available tokens, and available manual resets,
- Works with Codex, Claude, and Cursor,
- Retrieves data from local files, provider CLIs, and APIs,
- Falls back to another source when one is unavailable,
- Lightweight desktop app for macOS, Windows, and Linux,
- CLI with several output formats,
- Native system notifications when limits reach configured thresholds,
- Manual refresh of all data and one shared automatic refresh frequency.

## Selected alternatives

This selected comparison covers capabilities available in ai-limits; it is not a complete feature comparison of every alternative.

| | **ai-limits** | [CodexBar](https://github.com/steipete/CodexBar) | [caut](https://github.com/Dicklesworthstone/coding_agent_usage_tracker) | [OpenUsage](https://github.com/janekbaraniewski/openusage) | [ClaudeBar](https://github.com/tddworks/ClaudeBar) | [ccusage](https://github.com/ccusage/ccusage) |
| --- | :---: | :---: | :---: | :---: | :---: | :---: |
| Desktop app and CLI | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ |
| Codex, Claude, and Cursor | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| macOS, Windows, and Linux | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |

The full comparison covers 18 alternatives and 16 criteria: [alternatives catalog](docs/product/analogues.tsv).

## Platform support and limitations

- macOS: supported release; the app is signed, notarized, and stapled; notifications work,
- Windows and Linux: unsigned pre-release builds are available; support evolves based on user feedback,
- Desktop notifications are currently available only on macOS,
- Some local Codex and Claude sources may not work on Windows and Linux yet; CLI sources work everywhere.
- Codex and Claude CLI sources require authorization with the respective provider; Cursor requires a valid token from an authorized Cursor Agent.

## License

[MIT License](LICENSE)
