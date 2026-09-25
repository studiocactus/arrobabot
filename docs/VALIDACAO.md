# Registro de validação

## Verificações realizadas

- TypeScript: `npm run check` aprovado após as alterações de comunidade e onboarding.
- Editor de fluxos: 4 testes Vitest aprovados.
- Interface: 3 cenários Playwright aprovados (navegação/CRUD/persistência, janela estreita e personalização/restauração da cor).
- Build Vite de produção aprovado com os módulos e temas atualizados.
- Núcleo Rust: 17 testes aprovados (16 da suíte funcional e 1 de regressão da configuração do atualizador). A suíte inclui execução ordenada, cooldown, isolamento, presets, reembolsos, transações, reabertura SQLite, autenticação WebSocket, permissões e filtro de IA. Inclui também limites de músicas, emissão de overlay e transporte de configurações de pontos e tema em presets.
- Compilação Windows concluída: executável, instalador NSIS e MSI gerados. Abertura nativa verificada: painel renderizado, sem o banner de prévia. Corrigidos o manifesto Common Controls e a configuração de inicialização do atualizador. A instalação em máquina limpa não foi testada.

## Ambiente

Windows x64, Node.js 25, Rust MSVC 1.98.1 e Visual Studio Build Tools 2022. A compilação usa um job e não gera símbolos de depuração para reduzir consumo de memória. O ambiente apresentou limitação de paginação em builds paralelos.

## Limite de verificação

Sem contas OAuth do usuário, não houve envio real de chat nem homologação de eventos nas plataformas. A prévia de navegador não valida IPC, cofre, OAuth ou módulos nativos. Os screenshots em artifacts documentam somente a interface testada.

Houve falha temporária da revisão automática de aprovação. A revisão voltou a funcionar e os testes de interface foram executados normalmente.

## Manual de uso

Manual HTML offline verificado em 22/09/2026 no Microsoft Edge: 15 capítulos, destinos dos links internos, busca por conteúdo e sem acentos, limpeza da busca, estado sem resultados, impressão com todos os capítulos, imagem incorporada e leitura em janela de 390 px sem rolagem horizontal. Nenhum erro JavaScript ou requisição de rede ocorreu durante a leitura. Os links externos não fazem parte dessa validação.

## Sistema de variáveis — 23/09/2026

- 21 testes Rust aprovados: inclui quatro novos cenários para expansão sem recursão, filtros, erros, persistência, isolamento por perfil/plataforma/pessoa, sessão temporária, 400 incrementos concorrentes, simulação sem gravação, prévia e autorização. A execução real de uma sequência foi verificada por overlay local, sem enviar chat externo.
- 4 testes Vitest aprovados; TypeScript e build Vite aprovados.
- 4 cenários de interface aprovados no Edge (3 existentes e o novo catálogo/gerenciador). O novo cenário passou após corrigir um seletor do teste; a inserção de marcadores já estava funcionando. A prévia nativa é coberta pelo teste de RPC Rust; o navegador verifica a indicação de que essa execução exige o desktop.
- O manual passou a ter 16 capítulos, incluindo a referência de variáveis.

- Entrega Windows atualizada em 23/09/2026: executável, NSIS e MSI compilados. Inicialização da release e RPC snapshot verificados. O Windows não aceitou isolar a pasta de dados via APPDATA, então a edição pela janela nativa foi omitida para preservar os perfis existentes; a gravação e a prévia foram verificadas por RPC em banco temporário nos testes Rust.
- Manual de 16 capítulos verificado: busca, links internos, tabelas de variáveis, impressão, tela de 390 px e ausência de requisições de rede. Hashes da entrega conferidos após a cópia.

## Usabilidade das variáveis — 23/09/2026

Editor com inserção na posição do cursor e substituição da seleção; atalhos por nome; leitura visual por etiquetas; catálogo por categorias; alternativa e formato por seletores. Gerenciador com tipos Texto, Número, Sim/Não e JSON avançado. Sete testes Vitest aprovados (incluem seleção, Unicode, filtros e preservação dos marcadores), verificação TypeScript aprovada e quatro cenários Playwright aprovados no Edge. A comparação com funções do Streamer.bot foi registrada na matriz, separando funções implementadas e pendências.

A versão com o editor simplificado foi empacotada em executável, NSIS e MSI. Inicialização nativa e RPC snapshot aprovados; hashes da pasta entrega atualizados e conferidos. O cálculo de variáveis não mudou nesta revisão: os 21 testes Rust registrados acima continuam como evidência do motor; os testes desta revisão cobrem a interface e os auxiliares de edição. A última correção descarta prévias antigas quando o conteúdo é editado durante o cálculo.

## Canal GitHub e versão 0.1.1

Configuração do atualizador com endpoint oficial e chave pública embutidos, assinatura em secrets, script de versão e hook de commit. Validação local: 22 testes Rust, 7 Vitest, 4 Playwright e 3 testes Node do script de atualização aprovados; TypeScript e build Vite aprovados. O primeiro ensaio de interface esgotou o tempo no servidor de desenvolvimento; a suíte passou com a configuração final, que inicia uma prévia de produção isolada na porta 1421. O stage foi revisado sem chaves privadas, binários ou dados locais. A primeira release remota será validada pelo workflow após o push; não confundir configuração pronta com publicação já concluída.
# Verificação de timers, contadores e manual — 0.1.6

Executados: 36 testes Rust, dez Vitest, nove Playwright e sete testes de scripts de atualização. Compilação TypeScript/Vite aprovada. Após ajustes finais de navegação e editor, os dois testes Playwright de timers e manual foram repetidos e passaram; os 36 testes Rust foram repetidos após a proteção contra disparo de configuração antiga e passaram.

Os novos testes verificam: contagem atômica com eventos concorrentes, persistência, remoção com comando, isolamento entre perfis, permissões, cooldown, simulação sem gravação, timer direcionado, bloqueio offline e de eventos externos reais, descarte após edição, intervalos independentes e ausência de recuperação acumulada. A interface verifica criação/reabertura de timer e contador. O manual verifica vinte capítulos, grupos, links e âncoras antigas, busca, tabelas, impressão e largura de 390 pixels. Captura desktop do manual inspecionada visualmente.

Uma verificação inicial detectou rolagem horizontal de três pixels na barra com novos botões em 720 pixels; o layout foi corrigido e a suíte passou. Uma compilação inicialmente esbarrou na restrição de criação de processos do sandbox; a execução autorizada passou. Não houve teste de envio periódico com conta real de Twitch/YouTube nem migração sobre o aplicativo do usuário em execução.

## Verificação da reorganização de telas — 0.1.7

Executados após a separação em Comandos, Timers, Respostas e sons e Contadores: `npm run check` (TypeScript), dez testes Vitest, sete testes dos scripts de atualização (`npm run test:updates`) e nove cenários Playwright no Edge, todos aprovados. O manual offline foi regerado pelo gerador, que validou os vinte capítulos e os links internos.

Os testes de interface foram ajustados à nova navegação: a configuração de respostas é aberta pela entrada do menu lateral, o timer é criado a partir da tela Timers e o comando com contador é criado em Comandos. O cenário de timers passou também a abrir Contadores, confirmar que o comando aparece listado e gerar a captura correspondente.

Verificado em execução pontual da interface que apenas um item da barra lateral permanece em estado ativo por vez e que a navegação com quatro entradas a mais não produz rolagem na barra lateral nem rolagem horizontal em janela estreita. Nenhum arquivo Rust foi alterado nesta revisão, então a suíte de 36 testes Rust não foi reexecutada; a compilação do pacote Windows, a instalação sobre a versão anterior e a homologação com contas reais também não foram realizadas aqui.

## Geometria, listas do painel, ícone da barra lateral e atualização por instalador — 0.1.9

Executados após o Sprint A de geometria, a correção do logo espremido, a correção da barra de tarefas e a conversão das duas listas do painel em accordion: `npm run check` (TypeScript), dez testes Vitest, sete testes dos scripts de atualização (`npm run test:updates`), dez cenários Playwright no Edge e 52 testes Rust (`cargo test --locked --lib -j 1`), todos aprovados. O manual foi regerado pelo gerador com vinte e um capítulos e o Playwright foi reexecutado logo em seguida, ainda com dez cenários aprovados.

Medições na prévia de produção na porta 1421: ações de botão todas em 40 px (antes 37, 38, 39, 40 e 52), campos `input` e `select` todos em 40 px (antes `select` em 40 e 42 e `input` em 27, 37 e 40), diferença de altura entre `select` e `input` na mesma linha igual a zero em todos os pares medidos, `.icon-button` em 32 px, logo em 35×35 em 1440, 1280 e 1100 px e 34×34 em 900 e 760 px, e overflow horizontal de 0 px a 390 px nas telas de Visão geral, Discord, Comandos, Comunidade e Configurações, contra 147 px no Discord antes da correção do `flex-wrap`.

As listas de Respostas por TXT e Sons por espectador deixaram de renderizar `<section class="card">` dentro de um `<Card>`, que custava 510 px por regra e 719 px por pessoa, sempre expandidos. Medições na mesma prévia com três regras e três pessoas: custo por item fechado de 56 px, espaço entre itens de 0 px (antes 24 px de margem mais 48 px de padding), 1.715 px → 425 px na lista de regras e 2.751 px → 834 px na de pessoas com tudo recolhido, e altura do documento de 5.082 px → 3.282 px. A mesma tela em 1440 px e 390 px: overflow horizontal 0, nenhum elemento para fora da janela, nenhum botão sem nome acessível e nenhum alvo abaixo de 30 px (só os controles de ativar em 33×19 px, achado P1 anterior); nas linhas fechadas `aria-expanded="false"` e `inert` presentes, e o foco na linha mede 953×40 px em 1440 px e 197×40 px em 390 px. O teste do painel foi ampliado para cobrir recolher (linha abaixo de 120 px, `inert` e `aria-expanded="false"`), expandir (linha acima de 300 px, `inert` removido e campo visível de novo) e remover pessoa.

A correção do ícone sumindo da barra de tarefas foi verificada por leitura do script NSIS e do instalador gerado, não por uma atualização executada: o template `src-tauri/installer.nsi` derivado da tauri-bundler 2.11.5 marca a opção "Não desinstalar" como padrão durante a atualização, o arquivo `src-tauri/installer-hooks.nsh` é incluído na linha 52 do instalador gerado e o patch aparece na linha 309. O caminho silencioso/automático (`/P`) continua idêntico ao da Tauri. Nenhuma instalação ou atualização real foi executada nesta máquina: a instalação em máquina limpa não foi testada e a instalação sobre a versão anterior, que é a causa original do problema, também não foi executada e conferida.

Abertura nativa verificada em 25/09/2026: `botlive.exe` da release renderizou o painel com a janela intitulada "BotLive", respondeu ao fechamento normal e a instância de teste foi encerrada sem afetar o aplicativo do usuário em execução em outro caminho. A entrega foi regerada no mesmo dia com `npm run package` para incluir as listas em accordion — executável, NSIS, MSI e as duas assinaturas do atualizador — e reaberta para conferir a inicialização; os três hashes foram conferidos individualmente por `certutil` contra `entrega/SHA256SUMS.txt`. Os binários 0.1.0 anteriores permanecem preservados em `entrega/0.1.0/`.

Para conferir o recolhimento das listas não serve o `toBeHidden` do Playwright: ele mede a caixa do próprio elemento e ignora o corte por `overflow` do ancestral. A transição de `grid-template-rows` leva 220 ms enquanto os atributos já mudam no primeiro render, então a asserção usa `expect.poll` sobre a altura da linha.
