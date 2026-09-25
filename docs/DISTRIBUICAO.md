# Desenvolvimento e distribuição

## Estrutura do repositório

| Caminho | Conteúdo |
|---|---|
| src | Interface React e estilos |
| src-tauri/src | Núcleo Rust, integrações, persistência e testes |
| src-tauri/tauri.conf.json | Janela, política de conteúdo, plugins e empacotamento |
| src-tauri/capabilities | Permissões da janela Tauri |
| tests/ui | Cenários Playwright |
| examples | Presets e overlay |
| docs | Fontes Markdown e manual HTML gerado |
| scripts | Auxiliares de desenvolvimento e geração do manual |
| entrega | Binários e instruções da entrega local |
| .github/workflows/build.yml | Pipeline de verificação e builds por sistema |

## Preparar desenvolvimento no Windows

Instale Node.js/npm, Rust MSVC estável, ferramentas C++ do Visual Studio e WebView2 conforme os [pré-requisitos oficiais do Tauri](https://v2.tauri.app/start/prerequisites/).

O ambiente usado na entrega tinha Node.js 25, Rust MSVC 1.98.1 e Visual Studio Build Tools 2022. O workflow usa Node.js 22 e Rust estável. Consulte package-lock.json e Cargo.lock para versões efetivamente resolvidas; use os arquivos de lock em verificações reproduzíveis.

No diretório arquivos:

```text
npm ci
npm run desktop
```

npm run desktop inicia Vite e o aplicativo Tauri em desenvolvimento. npm run dev inicia só a prévia de navegador, na porta 1420. O modo navegador usa armazenamento separado e não substitui testes do backend.

scripts/dev.ps1 configura o Rust local quando a pasta .tools existe. A pasta .tools pertence à preparação deste ambiente e não é requisito para outra máquina. Com ferramentas instaladas normalmente, use os comandos padrão acima.

## Comandos de manutenção

| Comando | Finalidade |
|---|---|
| npm run check | Verificação TypeScript |
| npm test | Testes de ordenação/validação dos fluxos |
| cargo test --lib --manifest-path src-tauri/Cargo.toml -j 1 | Testes do núcleo |
| npm run build | Build da interface |
| npm run desktop | Aplicativo em desenvolvimento |
| npm run dev | Prévia de navegador |
| npx playwright test | Testes de interface com servidor de produção iniciado automaticamente |
| npm run package | Executável e instaladores Tauri |
| npm run docs | Regenerar docs/MANUAL.html a partir dos Markdown |

A configuração Playwright usa Microsoft Edge. O teste de interface gera o build e inicia automaticamente uma prévia na porta 1421; requer o navegador disponível. Os testes Rust usam diretórios temporários e serviços locais simulados; não são homologação de plataformas.

No ambiente preparado desta entrega, node .tools/run.cjs run package configura os caminhos do Rust local. O auxiliar força modo offline para o Cargo; se faltarem dependências nessa instalação, use uma configuração de desenvolvimento apropriada para obtê-las.

## Gerar uma entrega

1. Verifique tipos e testes relevantes.
2. Gere o manual com npm run docs.
3. Execute npm run package.
4. Abra o executável gerado e confira a inicialização.
5. Teste a instalação em um ambiente limpo antes de distribuir publicamente.
6. Copie os artefatos finais para entrega e atualize os hashes.
7. Registre o resultado em VALIDACAO.md, sem transformar testes simulados em aceite real.

No Windows, os caminhos padrão são src-tauri/target/release/botlive.exe e src-tauri/target/release/bundle. A configuração atual gera MSI e NSIS.

O NSIS usa um template próprio, `src-tauri/installer.nsi`, apontado por `bundle.windows.nsi.template`, e um arquivo de hooks, `src-tauri/installer-hooks.nsh`, apontado por `bundle.windows.nsi.installerHooks`. O template é uma cópia da `installer.nsi` da tauri-bundler 2.11.5 com o patch marcado como "BotLive patch": durante uma atualização a opção padrão da página de reinstalação passou a ser não desinstalar, porque a desinstalação apaga o executável, remove os atalhos e chama `UnpinShortcut` (`IStartMenuPinnedList::RemoveFromList`), o que tira o aplicativo da barra de tarefas do Windows. Ao atualizar o `@tauri-apps/cli`, baixe a `installer.nsi` da tag correspondente, reaplique cada trecho marcado e compile com `npm run tauri build -- --bundles nsis`. Os hooks são inseridos depois da página de reinstalação e não conseguem impedir a desinstalação: a correção está no template, e o hook existe só para regenerar o cache de ícones com `ie4uinit.exe -show`.

A configuração usa um job para reduzir consumo de memória. Limitações de paginação já afetaram builds paralelos neste computador; uma primeira compilação nativa pode levar bastante tempo.

## Cuidados ao alterar o núcleo

A configuração do plugin updater precisa ser um objeto válido com pubkey e endpoints. Valores iniciais vazios permitem abrir o app; o endereço e a chave operacional são informados nas Configurações. Remover o objeto pode causar falha de inicialização, coberta por teste de regressão.

O build.rs trata o manifesto de Common Controls do Windows para evitar conflitos nos testes nativos. Verifique testes e abertura do executável depois de alterar esse arquivo, dependências de diálogo ou configuração de empacotamento.

Não execute builds finais com uma cópia antiga do dist. npm run package já executa o build da interface antes da compilação nativa.

## macOS e Linux

O workflow oficial atual publica Windows x64. A estrutura Tauri pode ser expandida para outros sistemas, mas não há release automática macOS/Linux habilitada. Compilação cruzada não substitui build e teste no sistema de destino. Linux precisa das dependências de WebKit, bibliotecas de interface e serviço de segredos.

Notarização Apple, assinaturas de distribuição e credenciais de publicação não estão configuradas. Pacotes macOS/Linux não foram gerados nem homologados neste computador Windows.

## Atualizações verificadas

O app aceita endpoint HTTPS e chave pública do distribuidor, consulta o manifesto e usa o plugin Tauri para verificar/instalar atualizações. O distribuidor precisa gerar artefatos assinados e hospedar um manifesto compatível. A chave privada deve ficar fora do repositório.

O canal oficial está configurado para studiocactus/arrobabot no GitHub. O workflow publica pacotes assinados e latest.json depois dos testes; os clientes novos já incluem o endereço e a chave pública. O feed passa a existir após a primeira publicação bem-sucedida. Consulte [Atualizações e commits](ATUALIZACOES.md) para preparar cada versão e preservar a chave de assinatura.

## Atualizar a documentação

Edite os Markdown em docs, depois execute npm run docs. O gerador inclui os capítulos em uma página offline, transforma os links internos para a navegação dessa página e mantém os arquivos Markdown como fonte editável.

Confira links, busca, impressão e leitura em janela estreita. Ao alterar comportamento do produto, atualize o capítulo de uso, a matriz de aceite e o registro de validação quando aplicável.
