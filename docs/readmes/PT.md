# ai-limits

<p align="center">| <a href="DE.md">DE</a> | <a href="../../README.md">EN</a> | <a href="ES.md">ES</a> | <a href="FR.md">FR</a> | PT | <a href="RU.md">RU</a> | <a href="ZH.md">中文</a> | <a href="AR.md">عربي</a> |</p>

<p align="center">
   Aplicação local para controlar os limites e a utilização das subscrições de IA no Codex, Claude e Cursor.
</p>

<p align="center">
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.5.2/AI-Limits-v0.5.2-macos-arm64.dmg"><img src="https://shieldcn.dev/badge/macOS-v0.5.2-grey.svg?logo=apple" alt="Descarregar para macOS"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.5.2/AI-Limits-v0.5.2-windows-setup.exe"><img src="https://shieldcn.dev/badge/Windows-v0.5.2-blue.svg?logo=ri:FaWindows" alt="Descarregar para Windows"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.5.2/AI-Limits-v0.5.2-linux.AppImage"><img src="https://shieldcn.dev/badge/Linux-v0.5.2-yellow.svg?logo=linux" alt="Descarregar para Linux"></a>
</p>

<p align="center"><a href="https://github.com/md2it/ai-limits/releases/tag/v0.5.2">Todos os downloads</a></p>

---

![ai-limits macOS](screenshots/macos.png)

<p align="center">
  <img src="screenshots/windows.png" alt="ai-limits no Windows" width="24%">
  <img src="screenshots/linux.png" alt="ai-limits no Linux" width="24%">
  <img src="screenshots/macos-light-settings.png" alt="Definições do ai-limits" width="24%">
  <img src="screenshots/macos-help.png" alt="Ajuda do ai-limits" width="24%">
</p>

<p align="center">
  <img src="screenshots/macos-popover-dark.png" alt="popover do ai-limits para macOS no modo escuro" width="24%">
  <img src="screenshots/macos-popover-light.png" alt="popover do ai-limits para macOS no modo claro" width="24%">
  <img src="screenshots/macos-notification-center-dark.png" alt="notificações do ai-limits na Central de Notificações do macOS" width="24%">
  <img src="screenshots/macos-notification-center-light.png" alt="notificações claras do ai-limits na Central de Notificações do macOS" width="24%">
</p>

## Vantagens

- Funciona sem subscrição de API,
- Sem conta AI Limits separada: utiliza a autorização existente do fornecedor,
- Todos os fornecedores num só lugar,
- Completamente gratuita,
- Privada. Sem serviços de terceiros, proxies ou registos,
- Aplicação de ambiente de trabalho leve para Mac, Windows e Linux,
- Notificações de limites,
- Código aberto.

## Funcionalidades

- Mostra os limites, a data e hora de reinício, os tokens disponíveis e os reinícios manuais disponíveis,
- Funciona com Codex, Claude e Cursor,
- Obtém dados a partir de ficheiros locais, CLIs dos fornecedores e APIs,
- Lógica de fallback: se uma fonte estiver indisponível, verifica outra,
- Aplicação de ambiente de trabalho leve (macOS, Windows, Linux),
- Interface CLI com vários formatos de saída,
- Notificações do sistema ao atingir limiares de limite,
- Atualização manual de todos os dados e uma frequência automática compartilhada.

## Alternativas

| | **ai-limits** | [CodexBar](https://github.com/steipete/CodexBar) | [caut](https://github.com/Dicklesworthstone/coding_agent_usage_tracker) |
| --- | :---: | :---: | :---: |
| Aplicação de ambiente de trabalho e CLI | ✅ | ✅ | ❌ |
| Codex, Claude e Cursor | ✅ | ✅ | ✅ |
| macOS, Windows e Linux | ✅ | ❌ | ❌ |

A comparação completa com 18 alternativas e 16 critérios está no [catálogo de alternativas](../product/analogues.tsv).

## Suporte de plataformas e limitações

- macOS: versão suportada; a aplicação está assinada, notariada e com ticket anexado; as notificações funcionam,
- Windows e Linux: estão disponíveis builds de pré-lançamento não assinadas; o suporte evolui com base no feedback dos utilizadores,
- As notificações da aplicação de ambiente de trabalho estão por enquanto disponíveis apenas no macOS,
- Algumas fontes locais do Codex e do Claude podem ainda não funcionar no Windows e no Linux (as fontes via CLI funcionam em todo o lado),
- As fontes CLI do Codex e do Claude requerem autorização junto do respetivo fornecedor; o Cursor requer um token válido de um Cursor Agent autorizado.

## Licença

[Licença MIT](../../LICENSE)
