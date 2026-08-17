# ai-limits

<p align="center">| <a href="DE.md">DE</a> | <a href="../../README.md">EN</a> | <a href="ES.md">ES</a> | <a href="FR.md">FR</a> | <a href="PT.md">PT</a> | <a href="RU.md">RU</a> | 中文 | <a href="AR.md">عربي</a> |</p>

<p align="center">
   本地应用，用于查看与管控 Codex、Claude 和 Cursor 中 AI 订阅的用量与限额。
</p>

<p align="center">
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.5.1/AI-Limits-v0.5.1-macos-arm64.dmg"><img src="https://shieldcn.dev/badge/macOS-v0.5.1-grey.svg?logo=apple" alt="下载 macOS 版本"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.5.1/AI-Limits-v0.5.1-windows-setup.exe"><img src="https://shieldcn.dev/badge/Windows-v0.5.1-blue.svg?logo=ri:FaWindows" alt="下载 Windows 版本"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.5.1/AI-Limits-v0.5.1-linux.AppImage"><img src="https://shieldcn.dev/badge/Linux-v0.5.1-yellow.svg?logo=linux" alt="下载 Linux 版本"></a>
</p>

<p align="center"><a href="https://github.com/md2it/ai-limits/releases/tag/v0.5.1">所有下载</a></p>

---

![ai-limits macOS](screenshots/macos.png)

<p align="center">
  <img src="screenshots/windows.png" alt="Windows 上的 ai-limits" width="24%">
  <img src="screenshots/linux.png" alt="Linux 上的 ai-limits" width="24%">
  <img src="screenshots/macos-light-settings.png" alt="ai-limits 设置" width="24%">
  <img src="screenshots/macos-help.png" alt="ai-limits 帮助" width="24%">
</p>

<p align="center">
  <img src="screenshots/macos-popover-dark.png" alt="深色模式下的 ai-limits macOS 浮窗" width="24%">
  <img src="screenshots/macos-popover-light.png" alt="浅色模式下的 ai-limits macOS 浮窗" width="24%">
  <img src="screenshots/macos-notification-center-dark.png" alt="ai-limits 在 macOS 通知中心的通知" width="24%">
  <img src="screenshots/macos-notification-center-light.png" alt="ai-limits 在 macOS 通知中心的浅色通知" width="24%">
</p>

## 优势

- 无需 API 订阅即可使用，
- 无需单独注册 AI Limits 帐户：使用现有服务商授权，
- 所有服务商集中一处，
- 完全免费，
- 保护隐私：无第三方服务、代理或注册，
- 轻量级桌面应用，支持 Mac、Windows、Linux，
- 限额通知，
- 开源。

## 功能

- 显示限额、重置日期与时间、可用 token，以及可用的手动重置次数，
- 支持 Codex、Claude 和 Cursor，
- 从本地文件、服务商 CLI 与 API 获取数据，
- 回退逻辑：某一数据源不可用时，自动检查其他来源，
- 轻量级桌面应用（macOS、Windows、Linux），
- 提供多种输出格式的 CLI，
- 达到限额阈值时发送系统通知，
- 支持手动刷新所有数据和统一的自动刷新频率。

## 同类产品

| | **ai-limits** | [CodexBar](https://github.com/steipete/CodexBar) | [caut](https://github.com/Dicklesworthstone/coding_agent_usage_tracker) |
| --- | :---: | :---: | :---: |
| 桌面应用与 CLI | ✅ | ✅ | ❌ |
| Codex、Claude 与 Cursor | ✅ | ✅ | ✅ |
| macOS、Windows 与 Linux | ✅ | ❌ | ❌ |

完整对比涵盖 18 款同类产品与 16 项标准，见[同类产品目录](../product/analogues.tsv)。

## 平台支持与限制

- macOS：受支持的正式版本；应用已签名、完成公证并附加票据；通知可用，
- Windows 与 Linux：提供未签名的预发布构建版本；支持将根据用户反馈持续完善，
- 桌面应用通知目前仅在 macOS 上可用，
- 部分 Codex 与 Claude 的本地数据源在 Windows 与 Linux 上可能尚不可用（通过 CLI 的数据源在各平台均可使用），
- Codex 和 Claude 的 CLI 数据源需要相应服务商授权；Cursor 需要已授权 Cursor Agent 提供的有效令牌。

## 许可证

[MIT 许可证](../../LICENSE)
