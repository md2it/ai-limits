# ai-limits

<p align="center">| <a href="DE.md">DE</a> | <a href="../../README.md">EN</a> | <a href="ES.md">ES</a> | <a href="FR.md">FR</a> | <a href="PT.md">PT</a> | RU | <a href="ZH.md">中文</a> | <a href="AR.md">عربي</a> |</p>

<p align="center">
   Локальное приложение для контроля лимитов и использования AI-подписок в Codex, Claude и Cursor.
</p>

<p align="center">
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.4.0/AI-Limits-v0.4.0-macos-arm64.dmg"><img src="https://shieldcn.dev/badge/macOS-v0.4.0-grey.svg?logo=apple" alt="Скачать для macOS"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.4.0/AI-Limits-v0.4.0-windows-setup.exe"><img src="https://shieldcn.dev/badge/Windows-v0.4.0-blue.svg?logo=ri:FaWindows" alt="Скачать для Windows"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.4.0/AI-Limits-v0.4.0-linux.AppImage"><img src="https://shieldcn.dev/badge/Linux-v0.4.0-yellow.svg?logo=linux" alt="Скачать для Linux"></a>
</p>

<p align="center"><a href="https://github.com/md2it/ai-limits/releases/tag/v0.4.0">Все сборки</a></p>

---

![ai-limits macOS](screenshots/macos.png)

<p align="center">
  <img src="screenshots/windows.png" alt="ai-limits на Windows" width="24%">
  <img src="screenshots/linux.png" alt="ai-limits на Linux" width="24%">
  <img src="screenshots/macos-light-settings.png" alt="Настройки ai-limits" width="24%">
  <img src="screenshots/macos-help.png" alt="Справка ai-limits" width="24%">
</p>

<p align="center">
  <img src="screenshots/macos-popover-dark.png" alt="Popover ai-limits для macOS в тёмном оформлении" width="24%">
  <img src="screenshots/macos-popover-light.png" alt="Popover ai-limits для macOS в светлом оформлении" width="24%">
  <img src="screenshots/macos-notification-center-dark.png" alt="Уведомления ai-limits в Центре уведомлений macOS" width="24%">
  <img src="screenshots/macos-notification-center-light.png" alt="Светлые уведомления ai-limits в Центре уведомлений macOS" width="24%">
</p>

## Преимущества

- Работает без подписки на API,
- Без отдельной регистрации в AI Limits: используются существующие авторизации провайдеров,
- Все провайдеры в одном месте,
- Полностью бесплатно,
- Приватно. Никаких сторонних сервисов, прокси, регистраций,
- Легковестное desktop приложение для Mac, Windows, Linux,
- Уведомления о лимитах,
- Open Source.

## Возможности

- Показывает лимиты, дату-время reset, доступные токены, достуаные ручные reset,
- Работает с Codex, Claude и Cursor,
- Получает данные из локальных файлов, CLI провайдеров и API,
- Логика fallback: если один источник недоступен, проверяет другой,
- Легковесное desktop-приложение (macOS, Windows, Linux)
- CLI интерфейс с несколькими вариантами вывода,
- Системные уведомления при достижении порогов лимита,
- Ручное обновление всех данных и единая частота автоматического обновления.

## Аналоги

| | **ai-limits** | [CodexBar](https://github.com/steipete/CodexBar) | [caut](https://github.com/Dicklesworthstone/coding_agent_usage_tracker) |
| --- | :---: | :---: | :---: |
| Desktop-приложение и CLI | ✅ | ✅ | ❌ |
| Codex, Claude и Cursor | ✅ | ✅ | ✅ |
| macOS, Windows и Linux | ✅ | ❌ | ❌ |

Полное сравнение по 18 аналогам и 16 критериям — в [каталоге аналогов](../product/analogues.tsv).

## Поддержка платформ и ограничения

- macOS: поддерживаемый релиз; приложение подписано, нотарифицировано и имеет stapling; уведомления работают,
- Windows и Linux: доступны неподписанные pre-release сборки; поддержка развивается на основе обратной связи пользователей,
- Уведомления desktop-приложения пока доступны только на macOS,
- Некоторые локальные источники Codex и Claude пока могут не работать в Windows и Linux (источники через CLI работают везде),
- Для источников Codex и Claude через CLI требуется авторизация у соответствующего провайдера; для Cursor нужен действующий токен авторизованного Cursor Agent.

## Лицензия

[Лицензия MIT](../../LICENSE)
