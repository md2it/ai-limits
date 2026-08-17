# ai-limits

<p align="center">| DE | <a href="../../README.md">EN</a> | <a href="ES.md">ES</a> | <a href="FR.md">FR</a> | <a href="PT.md">PT</a> | <a href="RU.md">RU</a> | <a href="ZH.md">中文</a> | <a href="AR.md">عربي</a> |</p>

<p align="center">
   Lokale App zur Kontrolle von Limits und Nutzung von KI-Abonnements in Codex, Claude und Cursor.
</p>

<p align="center">
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.5.1/AI-Limits-v0.5.1-macos-arm64.dmg"><img src="https://shieldcn.dev/badge/macOS-v0.5.1-grey.svg?logo=apple" alt="Für macOS herunterladen"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.5.1/AI-Limits-v0.5.1-windows-setup.exe"><img src="https://shieldcn.dev/badge/Windows-v0.5.1-blue.svg?logo=ri:FaWindows" alt="Für Windows herunterladen"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.5.1/AI-Limits-v0.5.1-linux.AppImage"><img src="https://shieldcn.dev/badge/Linux-v0.5.1-yellow.svg?logo=linux" alt="Für Linux herunterladen"></a>
</p>

<p align="center"><a href="https://github.com/md2it/ai-limits/releases/tag/v0.5.1">Alle Downloads</a></p>

---

![ai-limits macOS](screenshots/macos.png)

<p align="center">
  <img src="screenshots/windows.png" alt="ai-limits unter Windows" width="24%">
  <img src="screenshots/linux.png" alt="ai-limits unter Linux" width="24%">
  <img src="screenshots/macos-light-settings.png" alt="Einstellungen von ai-limits" width="24%">
  <img src="screenshots/macos-help.png" alt="Hilfe zu ai-limits" width="24%">
</p>

<p align="center">
  <img src="screenshots/macos-popover-dark.png" alt="ai-limits macOS-Popover im dunklen Design" width="24%">
  <img src="screenshots/macos-popover-light.png" alt="ai-limits macOS-Popover im hellen Design" width="24%">
  <img src="screenshots/macos-notification-center-dark.png" alt="ai-limits-Benachrichtigungen in der macOS-Mitteilungszentrale" width="24%">
  <img src="screenshots/macos-notification-center-light.png" alt="helle ai-limits-Benachrichtigungen in der macOS-Mitteilungszentrale" width="24%">
</p>

## Vorteile

- Funktioniert ohne API-Abonnement,
- Kein separates AI-Limits-Konto: vorhandene Anbieter-Autorisierung wird verwendet,
- Alle Anbieter an einem Ort,
- Komplett kostenlos,
- Privat. Keine Drittanbieterdienste, Proxys oder Registrierungen,
- Leichtgewichtige Desktop-App für Mac, Windows und Linux,
- Benachrichtigungen zu Limits,
- Open Source.

## Funktionen

- Zeigt Limits, Reset-Datum und -Uhrzeit, verfügbare Tokens und verfügbare manuelle Resets,
- Arbeitet mit Codex, Claude und Cursor,
- Bezieht Daten aus lokalen Dateien, Anbieter-CLIs und APIs,
- Fallback-Logik: ist eine Quelle nicht verfügbar, wird eine andere geprüft,
- Leichtgewichtige Desktop-App (macOS, Windows, Linux),
- CLI-Schnittstelle mit mehreren Ausgabeformaten,
- Systembenachrichtigungen beim Erreichen von Limit-Schwellenwerten,
- Manuelle und flexible automatische Datenaktualisierung.

## Alternativen

| | **ai-limits** | [CodexBar](https://github.com/steipete/CodexBar) | [caut](https://github.com/Dicklesworthstone/coding_agent_usage_tracker) |
| --- | :---: | :---: | :---: |
| Desktop-App und CLI | ✅ | ✅ | ❌ |
| Codex, Claude und Cursor | ✅ | ✅ | ✅ |
| macOS, Windows und Linux | ✅ | ❌ | ❌ |

Der vollständige Vergleich über 18 Alternativen und 16 Kriterien steht im [Alternativenkatalog](../product/analogues.tsv).

## Plattformunterstützung und Einschränkungen

- macOS: unterstütztes Release; die App ist signiert, notarisiert und gestapelt; Benachrichtigungen funktionieren,
- Windows und Linux: unsignierte Vorab-Builds sind verfügbar; der Support entwickelt sich anhand von Nutzerfeedback weiter,
- Desktop-Benachrichtigungen sind derzeit nur unter macOS verfügbar,
- Einige lokale Quellen von Codex und Claude funktionieren unter Windows und Linux möglicherweise noch nicht (Quellen über die CLI funktionieren überall),
- CLI-Quellen für Codex und Claude erfordern die Autorisierung beim jeweiligen Anbieter; Cursor benötigt ein gültiges Token eines autorisierten Cursor Agent.

## Lizenz

[MIT-Lizenz](../../LICENSE)
