# BotLive

Aplicativo desktop de automação para transmissões, desenvolvido em Tauri 2, Rust, React e TypeScript.

## Abrir no Windows

Baixe o instalador na [página oficial de releases](https://github.com/studiocactus/arrobabot/releases). Os binários não são versionados no Git. Na entrega local já preparada, INICIAR-BOTLIVE.cmd abre entrega/BotLive.exe. A primeira release aparecerá quando o workflow de publicação terminar com sucesso.

## Manual completo

Use a [central de documentação](docs/README.md) ou baixe MANUAL.html junto à release. Após clonar o código, npm run docs gera docs/MANUAL.html e a cópia independente entrega/MANUAL-BOTLIVE.html; ABRIR-MANUAL.cmd abre a versão local. O manual tem busca offline e impressão.

## Ordem de uso

1. Leia [Guia de uso](docs/GUIA-DE-USO.md).
2. Cadastre um perfil e configure as contas em [Integrações](docs/INTEGRACOES.md).
3. Crie um comando, simule seu fluxo e confira o histórico.
4. Configure IA, memória e os módulos desejados.
5. Consulte [Distribuição](docs/DISTRIBUICAO.md) para compilar e instalar.

## Desenvolvimento

Após clonar, execute `npm run repo:setup` para instalar a verificação local de commits. Todo commit precisa de uma nova versão e notas detalhadas: consulte [Atualizações e commits](docs/ATUALIZACOES.md) e [Histórico de atualizações](CHANGELOG.md). Um push válido em main inicia os testes e a publicação Windows assinada no GitHub.

Requisitos: Node.js, Rust estável, ferramentas C++ do Visual Studio e WebView2 no Windows. Execute `npm ci`, `npm run desktop`. A preparação local deste ambiente também oferece `node .tools/run.cjs run desktop`.

Verificações: `npm run check`, `npm test`, `cargo test --manifest-path src-tauri/Cargo.toml -j 1`. Interface: servidor Vite ativo na porta 1420 e `npx playwright test`. O modo navegador é uma prévia de interface; conexões, cofre e motor dependem do aplicativo nativo.

## Documentação técnica

- [Arquitetura](docs/ARQUITETURA.md)
- [API local e OBS](docs/API-LOCAL.md)
- [Matriz de aceite](docs/MATRIZ-DE-ACEITE.md)
- [Validação](docs/VALIDACAO.md)

As credenciais não acompanham o código. A homologação das plataformas depende das contas e permissões reais. Consulte a matriz para distinguir implementação de validação externa.
