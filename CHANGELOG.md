# Histórico de atualizações

# BotLive 0.1.49

## O que mudou

- Resposta TXT no fio da pessoa: quando o gatilho vem de um arquivo de texto, a resposta sai presa à mensagem de quem falou (fio na Twitch, citação no Discord). Cada regra tem o interruptor **Responder no fio da pessoa**; regras antigas ganham o fio ligado sozinhas. No Kick e no YouTube não há fio e o Histórico avisa que saiu mensagem comum.
- Conferência de variáveis do TXT: ao vincular um arquivo, o BotLive analisa todas as linhas e lista até 20 avisos com número da linha e sugestão, ex.: `Linha 3: {{randomViewr}} não existe — quis dizer {{randomViewer}}?`. O botão **Conferir linhas** repete a conferência a qualquer momento. `R$100` e `$desconhecido` continuam texto, sem aviso.

## Como usar

- Respostas e sons → abra a regra e confira **Responder no fio da pessoa**. Salve.
- Vincule o TXT e leia os avisos logo abaixo da prévia; corrija o arquivo no seu editor e vincule de novo (ou confira de novo) até zerar.

## Validação

- `npm run check` (tsc), `npm test` (vitest 31/31), `npm run docs` (manual 21 capítulos) e `npm run test:updates` (8/8) executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); inclui testes de fio ligado por padrão em regra antiga, fio desligável e do analisador (acerto, sugestão e silêncio para texto comum); a CI compila e testa de verdade.
- Teste ao vivo do fio no chat e da conferência num TXT real ainda pendentes.

## Limitações

- O fio depende do ID da mensagem de origem; eventos sem ID (simulação, prévia) não prendem fio.
- A sugestão usa proximidade de nome (até 2–3 edições); variável muito diferente do catálogo sai como desconhecida sem palpite.
- Variáveis dinâmicas (`local.`, `global.`, `user.`, `session.`, `data.`, `random:`) não são conferidas pelo nome, só pelo escopo.

---

# BotLive 0.1.48

## O que mudou

- Corrige o teste de UI do timer (`timers.spec.ts`): usava o rótulo e os valores antigos em segundos. Agora cria o timer com 15 minutos e confere a célula `A cada 15 min`. Nenhuma mudança de comportamento além da 0.1.47.

## Como usar

- Vale o manual da 0.1.47: sorteio por ocorrência e intervalos em minutos.

## Validação

- `npx playwright test tests/ui/timers.spec.ts` executado localmente: 1 passed.
- `npm run test:updates` (8/8) executado antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); a CI testa de verdade.
- A 0.1.47 reprovou só neste teste de UI; esta 0.1.48 a substitui sem reescrever release, pois nenhuma release 0.1.47 chegou a existir.

## Limitações

- As mesmas da 0.1.47.

---

# BotLive 0.1.47

## O que mudou

- Sorteio de nome corrigido: `{{randomViewer}}` era sorteado uma vez por execução e repetia o mesmo nome em todas as ocorrências da mensagem (ex.: o Timer Donates citava `ruiva55` duas vezes). Agora cada ocorrência sorteia um nome novo na hora, do cadastro do momento. Sem ninguém no cadastro, continua valendo `|default:alguém`.
- Intervalos em minutos em toda a plataforma: Repetir a cada (timer), Intervalo entre usos, Intervalo por pessoa, intervalos das respostas TXT e dos sons, Pontos & loja, trivia (silêncio e intervalo), modo lento do Discord e tabelas de Comandos e Timers. O armazenamento e as APIs continuam em segundos; só a tela mudou, com conversão automática nos dois sentidos.

## Como usar

- Timer Donates: nenhuma mudança necessária na mensagem; cada `{{randomViewer}}` passa a variar sozinho. Para citar duas pessoas diferentes de propósito, use duas ocorrências.
- Timers existentes em segundos viram minutos sozinhos (600 → 10). Valores abaixo de meio minuto aparecem como fração (ex.: 5s → 0,08) e continuam funcionando até serem editados.
- Durações de punição (silêncio, timeout), modo lento da Twitch por voz e espera em milissegundos continuam como antes: o primeiro número da fala segue em segundos.

## Validação

- `npm run check` (tsc), `npm test` (vitest 31/31), `npm run docs` (manual 21 capítulos) e `npm run test:updates` (8/8) executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); inclui teste de regressão `random_viewer_draws_fresh_on_every_occurrence` (120 sorteios num único contexto, exige variação) e a CI compila e testa de verdade.
- Teste ao vivo do Timer Donates ainda pendente.

## Limitações

- Novos intervalos são digitados em minutos (passo livre); intervalos antigos abaixo de meio minuto só voltam a ser editáveis como fração ou arredondados na próxima gravação.
- O sorteio depende de quem já falou no chat (cadastro de nomes); com uma pessoa só no cadastro, o nome repetido é o comportamento correto.

---

# BotLive 0.1.46

## O que mudou

- Perfis de bot com cartão compacto: o espaço vazio grande sumiu. Cada perfil mostra avatar, nome, canal e uma pílula de estado (Conectado/Conectando/Reconectando/Desconectado) no topo, com as ações em fileira única abaixo. Cartões online ganham borda verde sutil.
- Botões de conexão mais claros: Conectar virou botão de destaque (verde) quando o bot está fora; Desconectar fica neutro quando está dentro. O estado também aparece na pílula com ponto pulsante, sem depender só da cor do botão.
- Discord com campos alinhados: fileiras de campo + botão (vínculo de identidade, busca de membro) agora alinham o botão à base do input, campos lado a lado dividem a largura por igual e textos auxiliares quebram em coluna própria. Vale para todas as telas que usam o mesmo padrão.
- Aniversário resgatado na entrada: novo interruptor **Pedir a data na entrada** no cartão Aniversariantes. Ligado, quem entra no servidor sem data registrada recebe mensagem no canal dos aniversários pedindo `/aniversario` com a data. Quem já tem registro não recebe de novo.

## Como usar

- Perfis: nada muda no fluxo; Conectar/Desconectar continuam no mesmo lugar, só mais visíveis.
- Discord → Aniversariantes: escolha o canal, ative Mensagens de aniversário e ligue **Pedir a data na entrada**. Salve a configuração.
- Limitação honesta: o Discord não entrega data de nascimento de ninguém via API; o bot pede e a pessoa informa pelo `/aniversario` ou pelo registro manual na tela.

## Validação

- `npm run check` (tsc) e `npm test` (vitest 31/31) executados antes do push.
- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); inclui teste `birthday_lookup_finds_only_registered_members` e a CI compila e testa de verdade.
- Teste visual das telas e teste ao vivo da mensagem de entrada ainda pendentes.

## Limitações

- O pedido na entrada exige canal dos aniversários configurado; sem canal, nada é enviado e nada quebra.
- A mensagem de entrada usa o canal dos aniversários (não o de boas-vindas) para não misturar os fluxos.

---

# BotLive 0.1.45

## O que mudou

- O editor limpa o texto do gatilho ao trocar para um tipo sem texto (seguidor, inscrição, raid e demais): antes, o padrão antigo (ex.: `!oi` herdado da criação) ficava gravado e aparecia na lista, confundindo. Funcionalmente era ignorado, agora nem aparece.
- Ao salvar, gatilhos sem texto sempre gravam padrão vazio, o que também limpa fluxos antigos na próxima gravação.

## Como usar

- Nada muda no uso. Fluxos existentes com texto fantasma se limpam sozinhos ao salvar de novo.

## Validação

- `npm run check` (tsc), `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); sem mudança em Rust e a CI compila e testa de verdade.
- Teste visual do editor ainda pendente.

## Limitações

- Nenhuma limitação nova.

---

# BotLive 0.1.44

## O que mudou

- Corrige o teste de variáveis dos alertas da live: os eventos de teste usavam `profile_id` e o modelo exige `profileId` em camelCase. Nenhuma mudança de comportamento.

## Como usar

- Vale o manual da 0.1.43: receitas de follow, sub, presente e raid com as variáveis documentadas.

## Validação

- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); a correção alinha o teste ao modelo e a CI testa de verdade.
- Teste ao vivo com follow, sub, presente e raid reais ainda pendente.

## Limitações

- As mesmas da 0.1.43.
- A 0.1.43 não gerou build (CI reprovou só neste teste novo); esta 0.1.44 a substitui sem reescrever release, pois nenhuma release 0.1.43 chegou a existir.

---

# BotLive 0.1.43

## O que mudou

- Alerta de follow com contadores ao vivo: gatilho Novo seguidor agora entrega `{{followerCount}}` e `{{subCount}}` com os totais da Twitch buscados na hora do evento. Ex.: `Obrigado por seguir a gente {{user}}! Agora estamos em {{followerCount}} seguidores e {{subCount}} subs!`.
- Novos gatilhos Nova re-inscrição e Sub de presente, com inscrições de resub (mensagem, meses acumulados e sequência) e de presente (quem presenteou, quantidade e nível) vindas da Twitch. O gatilho Nova inscrição segue para a primeira vez.
- Detalhes de sub e raid como variáveis: `{{subTier}}`, `{{subMonths}}`, `{{subStreak}}`, `{{subMessage}}`, `{{isGift}}`, `{{gifterName}}`, `{{giftTotal}}`, `{{giftTier}}`, `{{raidViewers}}` e `{{raiderLogin}}`, com entradas no catálogo.
- Raid com destaque automático por receita: gatilho Raid com ação de destaque usando `{{raiderLogin}}` mais mensagem com `{{user}}` e `{{raidViewers}}`. Resubs e presentes também avisam no Discord quando ligado.

## Como usar

- Crie uma automação por gatilho (Novo seguidor, Nova inscrição, Nova re-inscrição, Sub de presente, Raid) com uma ação Enviar mensagem e o texto das receitas no capítulo de exemplos. Contadores e detalhes chegam sozinhos; sem rede com a Twitch no instante, os contadores saem vazios sem quebrar a mensagem.
- Para raid: adicione antes a ação de destaque com `{{raiderLogin}}`; o login sai da Twitch na hora.

## Validação

- `npm run check` (tsc), `npm test` (vitest 31/31), `npx playwright test tests/ui/manual.spec.ts` (1 passed), varredura de tabelas, `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); inclui testes de enriquecimento de variáveis e de normalização de resub e presente, e a CI compila e testa de verdade.
- Teste ao vivo com follow, sub, presente e raid reais ainda pendente.

## Limitações

- Resubs sem mensagem e presentes anônimos chegam com texto e nome vazios ou Anônimo; a receita sugere texto alternativo.
- Níveis vêm como 1, 2 e 3; o detalhamento por benefícios fica para a mensagem customizada.
- Contadores dependem da API da Twitch no instante do alerta; em falha saem vazios.

---

# BotLive 0.1.42

## O que mudou

- Sessões EventSub independentes: a sessão do bot (mensagens do chat) e a do canal (follows, subs, categoria) agora se reconectam sozinhas, sem derrubar uma à outra. Antes, amarradas no mesmo `try_join`, qualquer soluço reiniciava as duas juntas e abria janelas cegas onde mensagens digitadas se perdiam sem rastro no Histórico.
- Timeout de leitura do WebSocket de 40 s para 90 s (o keepalive negociado é de 30 s): evita reconexões falsas por jitter que também abriam janelas cegas.

## Como usar

- Nada muda na configuração. Com a entrada estável, comandos digitados geram a linha do evento no Histórico em segundos; se alguma mensagem ainda não aparecer, o problema passa a ser entrega da Twitch, não o app.

## Validação

- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); mudança isolada no gerenciamento das sessões e a CI compila e testa de verdade.
- Teste ao vivo digitando `!setgame` com a entrada estável ainda pendente.

## Limitações

- Mensagens enviadas durante uma queda real de rede continuam irrecuperáveis: o EventSub não reenvia o que passou no silêncio. A correção reduz as quedas causadas pelo próprio app, não as da rede.

---

# BotLive 0.1.41

## O que mudou

- Mensagem da própria conta do bot no chat agora aparece no Histórico como `Mensagem própria ignorada (quem): texto`, em vez de sumir sem rastro. É proteção anti-loop: o bot nunca reage às próprias mensagens. Com a linha visível, dá para descobrir na hora se o comando foi digitado logado como o bot em vez do streamer.

## Como usar

- Nada muda na configuração. Se ao digitar um comando aparecer `Mensagem própria ignorada`, troque a conta logada no chat para a do streamer (ou outra conta que não seja o bot) e digite de novo.

## Validação

- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); mudança isolada em registro de log e a CI compila e testa de verdade.
- Teste ao vivo digitando como o bot e como o streamer ainda pendente.

## Limitações

- A linha nova aparece para qualquer mensagem da conta do bot, inclusive respostas normais dele quando o eco da Twitch chegar.

---

# BotLive 0.1.40

## O que mudou

- Corrige a compilação das variáveis de resultado: o auxiliar de gravação recebe a opção por empréstimo em vez de mover o valor no primeiro uso. Nenhuma mudança de comportamento além da 0.1.39.

## Como usar

- Vale o manual da 0.1.39: ação da Twitch guarda `local.twitchGame`, `local.twitchGameId`, `local.twitchTitle` e `local.twitchTarget` para as ações seguintes do mesmo fluxo.

## Validação

- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); a correção segue exatamente o erro apontado pela CI e a CI compila e testa de verdade.
- Teste ao vivo da mensagem variável ainda pendente.

## Limitações

- As mesmas da 0.1.39.
- A 0.1.39 não gerou build (CI reprovou na compilação); esta 0.1.40 a substitui sem reescrever release, pois nenhuma release 0.1.39 chegou a existir.

---

# BotLive 0.1.39

## O que mudou

- A ação da Twitch agora guarda o resultado em variáveis locais para as ações seguintes do mesmo fluxo: `local.twitchGame` e `local.twitchGameId` na troca de categoria, `local.twitchTitle` no título e `local.twitchTarget` nas operações com alvo. Com isso, uma automação só cobre todos os jogos com mensagem própria e variável.
- Catálogo de variáveis e aviso de variável ausente reconhecem as novas variáveis: o editor sugere `local.twitchGame` e não reclama quando a ação da Twitch vem antes no fluxo.

## Como usar

- Para mensagem própria e variável com uma automação só: gatilho comando `!setgame` com permissão de moderadores, ação 1 Ação na Twitch de categoria com conteúdo vazio, ação 2 Enviar mensagem como `@{{user}} agora é {{local.twitchGame}}!`. Vale para qualquer jogo falado ou digitado, sem uma automação por jogo.
- Sem automação própria, continuam valendo o `!setgame` nativo e a confirmação padrão em menção.

## Validação

- `npm run check` (tsc), `npm test` (vitest 31/31, inclui teste novo do mapeamento), `npx playwright test tests/ui/manual.spec.ts` (1 passed) e varredura de tabelas executados localmente.
- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); a gravação usa o mesmo caminho das respostas da IA e a CI compila e testa de verdade.
- Teste ao vivo da mensagem variável ainda pendente.

## Limitações

- As variáveis só existem depois da ação da Twitch executar: mensagem antes dela sai com erro de variável ausente, e o editor avisa.
- Demais limitações das versões anteriores seguem valendo.

---

# BotLive 0.1.38

## O que mudou

- Corrige a compilação da confirmação em formato de menção: faltavam as assinaturas de retorno em tupla na busca do jogo. Nenhuma mudança de comportamento além da 0.1.37.

## Como usar

- Vale o manual da 0.1.37: `!setgame` digitado ou falado confirma marcando quem pediu, com o nome oficial; automação própria continua tendo prioridade.

## Validação

- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); a correção segue exatamente o erro apontado pela CI e a CI compila e testa de verdade.
- Teste ao vivo da mensagem de confirmação ainda pendente.

## Limitações

- As mesmas da 0.1.37.
- A 0.1.37 não gerou build (CI reprovou na compilação); esta 0.1.38 a substitui sem reescrever release, pois nenhuma release 0.1.37 chegou a existir.

---

# BotLive 0.1.37

## O que mudou

- Mensagem de confirmação de categoria e título no formato de menção com o nome oficial do jogo: `@canal mudou o jogo para "VALORANT"!`. Em evento de voz, marca o canal; em comando digitado, marca quem digitou. O Histórico registra a mesma mensagem.
- Para mensagem totalmente própria, crie uma automação com comando `!setgame` ou `!settitle`: ela tem prioridade sobre o comando nativo.

## Como usar

- Digite `!setgame Valorant` ou fale o pedido: a confirmação sai marcando quem pediu, com o nome como está na Twitch.
- Se outro bot também responde ao mesmo comando, desative um dos dois para não duplicar.

## Validação

- `npm run check` (tsc), `npx playwright test tests/ui/manual.spec.ts` e varredura de tabelas executados localmente.
- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); mudança isolada em texto de confirmação e a CI compila e testa de verdade.
- Teste ao vivo da mensagem de confirmação ainda pendente.

## Limitações

- O nome oficial vem da busca da Twitch no momento da troca; se a busca falhar, a mensagem usa o nome falado ou digitado.

---

# BotLive 0.1.36

## O que mudou

- Correção de fundo na ação da Twitch: categoria, título e VIP agora executam com o token da **conta do canal**, não do bot. A documentação oficial da Twitch exige `broadcaster_id` igual ao usuário do token nesse endpoint, sem exceção para editores (diferente dos endpoints de moderação, que aceitam moderador). Minha premissa anterior de que bastava o bot editor estava errada e causava o 401 persistente.
- Escopos novos na conta do canal: `channel:manage:broadcast` e `channel:manage:vips`. É preciso reautorizar a conta do canal uma vez. Moderação e modos do chat continuam no bot (moderador), sem mudança.
- Editor e manual atualizados para a divisão correta: categoria, título e VIP pela conta do canal; o resto pela conta do bot.

## Como usar

- Reautorize a **conta do canal** em Perfis (para os escopos novos) e fale ou digite o comando como antes. As mensagens no chat continuam saindo pela conta do bot; só a chamada de categoria, título e VIP usa a credencial do canal, invisível para os viewers.

## Validação

- `npm run check` (tsc), `npm test` (vitest 30/30), `npx playwright test tests/ui/manual.spec.ts` (1 passed) e varredura de tabelas executados localmente.
- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); inclui teste do roteamento por conta e a CI compila e testa de verdade.
- Teste ao vivo da troca de categoria ainda pendente.

## Limitações

- Quem nunca autorizou a conta do canal precisa autorizá-la agora; quem já tinha, precisa reautorizar uma vez pelos escopos novos.
- VIP pela conta do canal segue a regra da Twitch para o dono do canal.

---

# BotLive 0.1.35

## O que mudou

- Erros da Twitch passam a trazer o motivo exato devolvido no corpo da resposta (ex.: escopo ausente ou autorização incorreta), anexado à mensagem no Histórico. Antes, só o código HTTP aparecia e 401 iguais tinham causas diferentes.

## Como usar

- Nada muda na configuração. Ao falar ou digitar o comando, se a Twitch recusar, leia o trecho `Resposta da Twitch:` na linha do Histórico e me mande o texto completo.

## Validação

- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); mudança isolada em montagem de mensagem de erro, e a CI compila e testa de verdade.
- Teste ao vivo da troca de categoria ainda pendente.

## Limitações

- O detalhe é cortado em 200 caracteres para não poluir o Histórico.

---

# BotLive 0.1.34

## O que mudou

- Corrige a compilação dos nativos `!setgame` e `!settitle`: a ação da Twitch agora devolve a mensagem de confirmação (antes devolvia vazio e o bloco do chat nativo não compilava). Nenhuma mudança de comportamento além disso.

## Como usar

- Digite no chat como moderador ou streamer: `!setgame Nome do Jogo` ou `!settitle Novo título`. Sem texto, o bot responde o modo de usar. Automação própria com o mesmo comando continua tendo prioridade sobre o nativo.
- Para voz, vale o manual anterior: gatilho com variações e ação Ação na Twitch.

## Validação

- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); a correção segue exatamente o erro apontado pela CI e a CI compila e testa de verdade.
- Teste ao vivo digitando `!setgame` e falando jogos variados ainda pendente.

## Limitações

- As mesmas da 0.1.33.
- A 0.1.33 não gerou build (CI reprovou na compilação); esta 0.1.34 a substitui sem reescrever release, pois nenhuma release 0.1.33 chegou a existir.

---

# BotLive 0.1.33

## O que mudou

- Comandos nativos `!setgame` e `!settitle` no chat: moderadores e streamer digitam `!setgame Nome do Jogo` ou `!settitle Novo título` e o bot executa pela própria conta, sem precisar montar automação. O nome do jogo sai do resto da mensagem; sem texto, o bot responde o modo de usar.
- Escolha de caminho: falar (gatilho Comando de voz com ação Ação na Twitch) ou digitar (nativos) fazem o mesmo. Quem criar uma automação própria com comando `!setgame` ou `!settitle` usa o fluxo em vez do nativo.
- A conferência do token do bot agora compara também o Client ID: se o campo Client ID do perfil mudou depois da autorização, o token pertence a outro aplicativo e a Twitch recusa tudo com 401 mesmo com escopos, identidade e cargos certos. O Histórico passa a dizer exatamente isso.

## Como usar

- Digite no chat como moderador ou streamer: `!setgame Minecraft` ou `!settitle Ranked com viewers`. O bot confirma no chat ou explica o erro no chat e no Histórico.
- Para voz, vale o manual anterior: gatilho com variações, conteúdo vazio usa o que você falou.
- Bot precisa ser moderador (moderação, VIP, modos) e editor (categoria, título); reautorize a conta do bot após atualizar.

## Validação

- `npm run check` (tsc), `npm test` (vitest), `npx playwright test tests/ui/manual.spec.ts` e varredura de tabelas do manual executados localmente.
- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); inclui testes dos nativos e a CI compila e testa de verdade.
- Teste ao vivo digitando `!setgame` e falando jogos variados ainda pendente.

## Limitações

- Nativos e voz usam a mesma API com o token do bot: um 401 persistente com escopos, identidade, cargos e Client ID certos deve ser investigado com a mensagem exata do Histórico.
- Sem texto após o comando, o bot responde o modo de usar em vez de adivinhar.
- Abreviações não resolvem para o nome oficial do jogo; números por extenso não valem como duração.

---

# BotLive 0.1.32

## O que mudou

- A ação na Twitch agora confere se o token guardado pertence mesmo à conta de bot registrada no perfil (compara o `user_id` da Twitch com o `bot_id`). Se for de outra conta — ex.: aprovou o código do bot logado como o canal — o Histórico diz exatamente qual conta é dona do token e manda reautorizar logado como o bot. Isso fecha o último caso de 401 com escopos em ordem.

## Como usar

- Nada muda na configuração. Ao falar o comando, se a conta estiver trocada, siga a mensagem do Histórico: entre no navegador como a conta do bot e reautorize a conta do bot em Perfis.

## Validação

- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); mudança pequena e isolada, e a CI compila e testa de verdade.
- Teste ao vivo da troca de categoria ainda pendente da conta certa.

## Limitações

- A checagem soma uma chamada à Twitch por execução da ação, junto da checagem de escopos já existente.
- Se o token for da conta certa, com escopos e cargos certos, e a Twitch ainda recusar, o erro original da Twitch é mantido no Histórico para diagnóstico.

---

# BotLive 0.1.31

## O que mudou

- Texto do erro HTTP 401 nas ações da Twitch passa a orientar a ordem certa: cargo do bot (editor para categoria e título, moderador para o resto) e ID do canal primeiro, reautorização por último. Antes, o texto mandava reautorizar sempre, o que levava a reautorizações repetidas quando a causa real era cargo. Nenhuma mudança de comportamento nas chamadas.

## Como usar

- Ao ver recusa da Twitch no Histórico, confira nesta ordem: bot como editor e moderador no Gestor de funções do canal, ID do canal no perfil e, por último, reautorização da conta do bot.

## Validação

- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); alteração só em texto de erro, sem lógica nova, e a CI compila e testa de verdade.
- Teste ao vivo da troca de categoria ainda pendente do cargo de editor.

## Limitações

- Mensagem de erro não distingue sozinha token sem escopo de chamador sem cargo quando a Twitch responde o mesmo HTTP; a checagem de escopos da 0.1.30 cobre o primeiro caso e este texto cobre o segundo.

---

# BotLive 0.1.30

## O que mudou

- A ação na Twitch agora confere os escopos reais do token do bot antes de executar (`GET https://id.twitch.tv/oauth2/validate`) e cobra os que faltam por operação (ex.: categoria e título exigem `channel:manage:broadcast`). Se faltar algum, o Histórico diz exatamente quais escopos faltam e manda reautorizar a conta do bot, em vez de um 401 genérico que levava a reautorizações no escuro.

## Como usar

- Nada muda na configuração. Ao falar o comando, se o token estiver incompleto, leia no Histórico quais escopos faltam e reautorize a CONTA DO BOT (botão Autorizar conta do bot, no perfil certo, com o programa atualizado) após aprovar cada permissão na página da Twitch.

## Validação

- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); inclui teste do mapa de escopos por operação e a CI compila e testa de verdade.
- Teste ao vivo falando jogos variados ainda pendente.

## Limitações

- A conferência soma uma chamada à Twitch por execução da ação; ações de voz são esporádicas, sem impacto relevante.
- Reautorização continua necessária uma única vez por mudança de escopos; tokens seguem no cofre do sistema com renovação silenciosa, sem reautorizar a cada build.

---

# BotLive 0.1.29

## O que mudou

- Corrige a expectativa do teste de remoção da ativação: o código apara o ponto final órfão (`Arroba. Troca...` vira `Troca o jogo para Minecraft`, sem ponto), e o teste esperava com ponto. Código de produção inalterado.

## Como usar

- Vale o manual da 0.1.25/0.1.26: conteúdo vazio na ação de categoria ou título usa o que você falou, sem a ativação e sem pontuação grudada.

## Validação

- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); na execução anterior da CI, 119 testes passaram e só este teste novo falhou pela expectativa — agora alinhada ao comportamento correto — e a CI testa de verdade.
- Teste ao vivo falando jogos variados ainda pendente.

## Limitações

- As mesmas da 0.1.25/0.1.26.
- As versões 0.1.22, 0.1.23, 0.1.26 e 0.1.27 não geraram build; esta 0.1.29 as substitui sem reescrever release, pois nenhuma release delas chegou a existir.

---

# BotLive 0.1.28

## O que mudou

- Corrige o teste da remoção da ativação: `strip_activation` agora apara pontuação das bordas após remover a palavra (`Arroba. Troca...` vira `Troca...`, sem o ponto órfão). Era só o teste que esperava o aparo; a extração usada em produção já descartava o ponto sozinho. Nenhuma mudança de comportamento além disso.

## Como usar

- Vale o manual da 0.1.25/0.1.26: conteúdo vazio na ação de categoria ou título usa o que você falou, sem a ativação e sem pontuação grudada.

## Validação

- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); a correção alinha o código ao valor que o próprio teste exigia e a CI testa de verdade.
- Teste ao vivo falando jogos variados ainda pendente.

## Limitações

- As mesmas da 0.1.25/0.1.26.
- As versões 0.1.22, 0.1.23 e 0.1.26 não geraram build; esta 0.1.28 as substitui sem reescrever release, pois nenhuma release delas chegou a existir.

---

# BotLive 0.1.27

## O que mudou

- Corrige a compilação da 0.1.26: concatenação `String + &String` não existe em Rust; trocado por `push_str`. Nenhuma mudança de comportamento.

## Como usar

- Vale o manual da 0.1.25/0.1.26: conteúdo vazio na ação de categoria ou título usa o que você falou, com ativação e pontuação removidas.

## Validação

- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); a correção segue exatamente o erro apontado pela CI e a CI compila e testa de verdade.
- Teste ao vivo falando jogos variados ainda pendente.

## Limitações

- As mesmas da 0.1.25/0.1.26.
- A 0.1.26 não gerou build (CI reprovou na compilação); esta 0.1.27 a substitui sem reescrever release, pois nenhuma release 0.1.26 chegou a existir.

---

# BotLive 0.1.26

## O que mudou

- Corrige a extração do nome falado na ação da Twitch: a palavra de ativação vazava para o nome (ex.: `Arroba. Troca o jogo para Minecraft.` virava a busca por `arroba. para minecraft.`) e a pontuação grudada nas palavras quebrava a busca. Agora o bot remove a ativação, remove o gatilho que casou, limpa a pontuação das bordas e mantém a caixa original do nome.
- Vale para categoria e título com conteúdo vazio: `Arroba troca o jogo para Valorant` vira `Valorant`; `Arroba muda o título para Ranked com viewers` mantém as maiúsculas no título.

## Como usar

- Nada muda na configuração: conteúdo vazio na ação de categoria ou título usa o que você falou. Fale a ativação, o pedido e o nome em uma frase só.
- Se a extração falhar, o Histórico mostra o nome tentado entre aspas na mensagem de erro, para ajustar a frase ou usar nome fixo.

## Validação

- `npm run check` (tsc), `npx playwright test tests/ui/manual.spec.ts` (1 passed) e varredura de tabelas do manual executados localmente.
- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); inclui testes novos de remoção da ativação e a CI compila e testa de verdade.
- Teste ao vivo falando jogos variados ainda pendente.

## Limitações

- Nomes mal transcritos continuam não achando o jogo; o erro mostra o nome tentado.
- A remoção da ativação é por frase configurada: ativações de várias palavras funcionam, mas a palavra precisa sair igual na transcrição.
- Demais limitações da 0.1.22 seguem valendo.

---

# BotLive 0.1.25

## O que mudou

- Rótulo do bloco de gatilho no canvas agora acompanha o tipo escolhido: ao trocar o Evento para Comando de voz (ou qualquer outro), o desenho atualiza na hora. Antes, o rótulo ficava preso no tipo anterior e confundia (o dado salvo sempre esteve correto).
- Categoria e título aceitam conteúdo vazio na ação da Twitch: com o campo vazio, o bot extrai o nome do que você falou, removendo a variação do gatilho que casou e as palavras de ligação. Ex.: `troca o jogo para Valorant` vira `Valorant`. Se a Twitch não reconhecer, tenta com iniciais maiúsculas antes de desistir com aviso no Histórico.

## Como usar

- Para jogo variável: gatilho Comando de voz (`troca o jogo, muda o jogo`) e ação de categoria com o conteúdo vazio. Fale o nome do jogo de forma clara após o pedido.
- Para jogo certo sempre: preencha o nome fixo no conteúdo (ex.: `Minecraft`). Nomes fixos não dependem da transcrição e são mais certeiros; abreviações como `lol` ou `cs` não resolvem para o nome oficial.
- Vale o mesmo para título: conteúdo vazio usa o que você falou (`troca o título para Ranked com viewers`), ou preencha o título fixo.

## Validação

- `npm run check` (tsc), `npm test` (vitest 30/30) e `npx playwright test tests/ui/manual.spec.ts` (1 passed) executados localmente.
- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); inclui testes novos de extração do assunto e cobre a compilação na CI.
- Teste ao vivo falando jogos variados ainda pendente.

## Limitações

- A extração depende da transcrição: nomes mal reconhecidos (`minecráfti`) não acham o jogo e o erro aparece no Histórico com o nome tentado.
- Abreviações e apelidos não mapeiam para o nome oficial da Twitch; use o nome como está na plataforma ou o modo fixo.
- As demais limitações da 0.1.22 seguem valendo (VIP por conta editora, sem reply em fio para voz, durações em dígitos).

---

# BotLive 0.1.24

## O que mudou

- Corrige a compilação da ação na Twitch (conta do bot): anotação explícita de tipo (`String`) na montagem do título, exigida pelo compilador. Nenhuma mudança de comportamento em relação à 0.1.23.

## Como usar

- Vale o manual da 0.1.22/0.1.23: bot como moderador e editor do canal, reautorização do bot em Perfis, automação com gatilho Comando de voz e ação Ação na Twitch.

## Validação

- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível nesta máquina); a compilação foi revisada por leitura contra o erro exato apontado pela CI e a CI compila e testa de verdade.
- Teste ao vivo com conta moderadora ainda pendente.

## Limitações

- As mesmas da 0.1.22: VIP por conta editora depende da Twitch aceitar; reply em fio é impossível em evento de voz; números por extenso não valem como duração; cache de nomes é melhor esforço.
- As versões 0.1.22 e 0.1.23 não geraram build (CI reprovou na compilação); esta 0.1.24 as substitui sem reescrever release, pois nenhuma release delas chegou a existir.

---

# BotLive 0.1.23

## O que mudou

- Corrige a compilação da ação na Twitch (conta do bot) da 0.1.22, que reprovou no `cargo test` da CI: faltava `async` na resolução de alvo (três `await` órfãos), faltava `=` na montagem do título e sobrava um `mut` no teste do gatilho de voz. Nenhuma mudança de comportamento em relação ao planejado na 0.1.22.
- Mantido o restante da 0.1.22: nova ação com 18 operações pela conta do bot, variações por vírgula no comando de voz, alvo por arroba, nome do chat ou alvo fixo, e escopos novos no token do bot.

## Como usar

- Vale o manual da 0.1.22: bot como moderador e editor do canal, reautorização do bot em Perfis, automação com gatilho Comando de voz e ação Ação na Twitch.

## Validação

- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível nesta máquina); a compilação foi revisada por leitura e a CI compila e testa de verdade.
- Teste ao vivo com conta moderadora ainda pendente.

## Limitações

- As mesmas da 0.1.22: VIP por conta editora depende da Twitch aceitar; reply em fio é impossível em evento de voz; números por extenso não valem como duração; cache de nomes é melhor esforço.
- A 0.1.22 não gerou build (CI reprovou na compilação); esta 0.1.23 a substitui sem reescrever release, pois nenhuma release 0.1.22 chegou a existir.

---

# BotLive 0.1.22

## O que mudou

- Nova ação **Ação na Twitch (conta do bot)** com 18 operações: trocar categoria (setgame), trocar título (settitle), silenciar (timeout), banir, desbanir, avisar, dar e tirar VIP, modo lento e desligar, só seguidores e liberar, só assinantes e liberar, só emotes e liberar, destaque de canal (shoutout) e responder marcando arroba (mention). Tudo executa com o token do bot, nunca com a conta do canal.
- Gatilho **Comando de voz** passa a aceitar variações separadas por vírgula, como `troca o jogo, muda o jogo, minecraft`. Basta uma delas aparecer na fala transcrita.
- Alvo em fala livre resolvido nesta ordem: arroba menção da frase, nome de quem passou pelo chat nas últimas 24 horas, alvo fixo configurado na automação. Sem nenhum, o bot avisa no Histórico e não executa. Durações usam o primeiro número da fala, ou o valor do editor.
- Escopos novos no token do bot: moderação de banidos e avisos, modos do chat, transmissão (categoria e título) e VIPs. É preciso reautorizar o bot em Perfis, e ele precisa ser moderador do canal (moderação, VIP, modos) e editor (categoria e título).

## Como usar

- No canal da Twitch, marque o bot como moderador e como editor, depois reautorize a conta do bot em Perfis para conceder os escopos novos.
- Crie uma automação com gatilho Comando de voz (ex.: `troca o jogo, muda o jogo, minecraft`) e adicione a ação Ação na Twitch com a operação desejada (ex.: trocar categoria com o nome do jogo no conteúdo).
- Para punir ou dar VIP por voz, fale a arroba (`bana o @troll123`) ou o nome de quem está no chat (`vip para a maria`); durações pedem dígitos (`timeout 300`).
- O Histórico registra cada execução com sucesso ou o motivo da recusa da Twitch.

## Validação

- `npm run check` (tsc), `npm test` (vitest 30/30) e `npx playwright test tests/ui/manual.spec.ts` (1 passed) executados localmente nesta máquina.
- `npm run update:check` e `npm run test:updates` executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível nesta máquina); coberto pelo job `windows` da CI, que compila o novo módulo e roda os testes de parser, alvo e gatilho de voz.
- Teste ao vivo com conta moderadora (falar e conferir categoria, timeout e VIP reais) ainda pendente.

## Limitações

- VIP por conta editora depende da Twitch aceitar a operação; se recusar, o erro aparece claro no Histórico.
- Responder em fio (reply) é impossível em evento de voz, que não tem mensagem de origem: a operação de menção posta `@alvo` mais o texto como mensagem normal.
- Números por extenso não valem como duração: fale os dígitos.
- O cache de nomes do chat é melhor esforço (24 horas, 2000 nomes) e só enxerga quem falou depois desta atualização.

---

# BotLive 0.1.21

## O que mudou

- Corrige a tabela do capítulo Variáveis do manual (`docs/VARIAVEIS.md`): a linha da variável `{{liveSpeech}}` continha um caractere `|` literal na descrição ("separadas por ` | `"), que o gerador do manual interpretava como divisória de coluna. A tabela era renderizada com 2 cabeçalhos e uma linha de 3 células, e o teste `tests/ui/manual.spec.ts` reprovava na CI, abortando a publicação antes das etapas de build e release. O pipe foi escapado (`\|`), formato que o parser de tabelas já previa; a célula continua exibindo `|` normalmente.

## Como usar

- Nenhuma mudança de uso. O manual volta a ser gerado íntegro com `npm run docs` (21 capítulos) e o teste do manual passa tanto localmente quanto na CI.

## Validação

- `npm run docs` executado localmente: manual gerado com 21 capítulos; varredura de todas as tabelas do `MANUAL-BOTLIVE.html`: nenhuma divergência entre `thead` e `tbody`, 21 artigos, 4 grupos de navegação, nenhum link interno quebrado.
- `npx playwright test tests/ui/manual.spec.ts` executado localmente: 1 passed.
- `npm run update:check` e `npm run test:updates` executados antes do push (resultado registrado no histórico do commit).
- `cargo test` não executado localmente (toolchain Rust indisponível nesta máquina); coberto pelo job `windows` da CI.

## Limitações

- As builds 0.1.19 e 0.1.20 não foram publicadas porque a execução da CI que as continha falhou neste teste do manual antes das etapas de release. Esta 0.1.21 carrega o mesmo código das duas mais a correção; a publicação da 0.1.21 pela CI libera os instaladores com Escuta contínua e Comandos via chat. Lançamentos retroativos de 0.1.19/0.1.20 exigiriam builds assinados via CI a partir dos commits antigos e ficam como decisão pendente.

---

# BotLive 0.1.20

# Versão 0.1.20 "Comandos via Chat"

## O que mudou

- **Comandos via chat da Twitch**: Moderadores e streamers podem criar automações usando `!cmd add/del/edit/list` no chat
- **Sintaxe**: `!cmd add !nome resposta`, `!cmd del !nome`, `!cmd edit !nova resposta`, `!cmd list`
- **Permissões**: Apenas usuários com papel `moderator` ou `broadcaster` da Twitch podem usar o recurso
- **Integração**: Comandos criados via chat aparecem como automações normais na página "Comandos"

## Sintaxe Detalhada

- `!cmd add !oi Olá {{user}}` - Cria comando !oi que responde "Olá {{user}}"
- `!cmd del !oi` - Remove o comando !oi
- `!cmd edit !oi Ola a todos` - Atualiza a resposta do comando !oi
- `!cmd list` - Lista todos os comandos criados via chat

## Como usar

1. Entre no chat da Twitch como moderador ou streamer
2. Digite `!cmd add !nome resposta` para criar um novo comando
3. Use `!cmd del !nome` para remover um comando
4. Use `!cmd edit !nome nova resposta` para atualizar a resposta
5. Use `!cmd list` para ver todos os comandos criados via chat
6. Os comandos aparecerão na página "Comandos" da interface desktop para edição avançada

## Limitações

- Comandos criados via chat são respostas de texto simples
- Automações complexas (com ações múltiplas, scripts, etc.) ainda devem ser criadas via interface desktop
- O comando `!cmd list` mostra todos os comandos de tipo "command" no perfil

## Testes

- Testes unitários do parser (`parse`) em `src-tauri/src/cmd_manager.rs`: casos add/del/edit/list, nome sem `!` e entrada inválida.
- As mensagens do próprio bot continuam ignoradas pela checagem existente em `src-tauri/src/engine.rs` (`e.user_id==p.bot_id`).

## Validação

- `npm run update:check` e `npm run test:updates` executados nesta máquina; registrar o resultado antes do push.
- Testes Rust (`cargo test`) NÃO executados aqui: toolchain Rust indisponível no ambiente. Não afirmar homologação Rust.
- Validação com conta moderadora na Twitch ao vivo ainda pendente: criar, editar, listar e remover um comando de teste e confirmar que ele aparece na página Comandos.

## Notas de Desenvolvimento

- Backend: `src-tauri/src/cmd_manager.rs` (novo), `src-tauri/src/engine.rs` (interceptação do `!cmd`) e `src-tauri/src/lib.rs` (registro do módulo)
- Frontend: sem alterações; a página Comandos já filtra por `trigger.kind === 'command'`
- Permissões via `crate::model::permitted("moderator", &e.role)`, que aceita `moderator` e `broadcaster`

---

# BotLive 0.1.19

# 0.1.19 "Escuta contínua"

## O que mudou

### Escuta contínua do microfone (novo)
- **Servidor local RealtimeSTT**: o BotLive agora se conecta a um `stt-server-production` rodando na mesma máquina (padrão `http://127.0.0.1:8010/transcribe-pcm16`). O áudio **nunca sai do computador** — tudo roda em HTTP local.
- **Captura em tempo real**: o webview abre um `AudioContext`, faz downmix para mono 16 kHz PCM16, agrupa em lotes de 100 ms e envia via operação `voice.frame`. O VAD (detecção de voz/silêncio) roda no servidor Rust com parâmetros ajustáveis.
- **Palavras de ativação**: campo **Palavras de ativação** (separadas por vírgula). Só falas que contenham uma delas disparam automações de **Comando de voz**. Vazio = toda fala reconhecida dispara.
- **Variáveis de contexto**:
  - `{{lastSpeech}}` — última fala finalizada reconhecida (vazia se nenhuma).
  - `{{liveSpeech}}` — todas as falas da sessão de escuta, unidas por ` | `.
- **Legenda no overlay**: opção **Legenda no overlay** no painel; o `examples/overlay.html` ganha um novo evento `captions` que mostra a fala finalizada em uma linha própria, some após 8 s.
- **Histórico e IA**: cada fala reconhecida vira uma linha no Histórico, alimenta o contexto da IA e atualiza as variáveis acima.
- **Painel de voz reformulado** (Comunidade → Abrir Controle por voz):
  - Servidor de transcrição, Idioma falado, Palavras de ativação, Legenda no overlay.
  - Ajuste fino: Silêncio que encerra a frase (200–3000 ms, padrão 600), Sensibilidade do microfone (0,002–0,5, padrão 0,02).
  - Interruptor **Escuta contínua** com `role="switch"`, `aria-label="Escuta contínua"`, polling de status a cada 2 s.
  - Aviso claro quando aberto na prévia do navegador: "Abra o aplicativo desktop para ligar a escuta contínua".
  - Religação automática ao trocar de perfil (efeito em `App.tsx` observando `profileId`).

### Contrato de rede corrigido
- O parâmetro `encoding` enviado ao RealtimeSTT agora é **`pcm16`** (antes `pcm_s16le`, que o servidor recusava com HTTP 400).
- Erro HTTP 503 do servidor ("modelo carregando") agora tem mensagem amigável: "Servidor de transcrição ainda carregando o modelo. Aguarde ele responder 200 em /health e tente de novo."

### Correção de ambiente (PyTorch CUDA)
- O `pip install torch` padrão traz a versão CPU e o `stt-server-production` falha com `Requested float16 compute type, but the target device or backend do not support efficient float16 computation`.
- Documentado e validado: `pip install torch==2.14.0+cu130 --index-url https://download.pytorch.org/whl/cu130` (ou versão compatível com o driver) + `nvidia-cublas-cu12` / `nvidia-cuda-runtime-cu12` para o `cublas64_12.dll` exigido pelo CTranslate2.

### Validação real executada
- Servidor RealtimeSTT 1.1.2 (`faster-whisper`, modelo `small`, CUDA `float16`, RTX 2070) rodando como `stt-server-production --host 127.0.0.1 --port 8010 ... --no-model-warmup` (warm-up travava no `cublas64_12.dll` ausente; com as libs NVIDIA instaladas, warm-up também funciona).
- Fala em português gerada via Windows SAPI (`Microsoft Maria Desktop`, 16 kHz mono), convertida para PCM16, enviada via `POST /transcribe-pcm16?encoding=pcm16&sample_rate=16000&language=pt` → **HTTP 200**, texto reconhecido, latência ~1,3 s (RTF 0,16).
- Teste ignorado no Rust (`listen::tests::fala_real_atravesa_o_nosso_cliente`) executa o mesmo caminho do código de produção contra o servidor real, guardado por `BOTLIVE_STT_PCM=<wav>`.
- Teste unitário garante que a URL montada contém `encoding=pcm16`, `sample_rate=16000` e `language=` (inclui `auto` — omitir faria o servidor cair no padrão inglês).

### Documentação
- `05-COMUNIDADE.md`: seção **Controle por voz** reescrita com dois caminhos (escuta contínua + captura por botão), passo a passo do painel, ajuste fino, legenda, religação automática.
- `01-INSTALACAO.md`: nova seção **Servidor de voz (RealtimeSTT)** com comandos exatos de instalação (Python, pip, torch CUDA, stt-server-production), opções CPU/GPU e solução do `cublas64_12.dll`.
- `08-SOLUCAO-DE-PROBLEMAS.md`: 8 novas linhas para voz (escuta não liga, para sozinha, transcrição errada, legenda não aparece, palavra de ativação não dispara).
- `VARIAVEIS.md`: `{{lastSpeech}}` e `{{liveSpeech}}` no catálogo; categoria **Voz** adicionada ao seletor.
- `EXEMPLOS-DE-USO.md`: receita "Controle por voz (escuta contínua)" com comando pronto para copiar.
- `MATRIZ-DE-ACEITE.md`: linha de voz atualizada com VAD, ativação, variáveis, legenda; dependência externa passa a citar RealtimeSTT.
- `API-LOCAL.md`: evento `captions` no protocolo do overlay.

### Testes
- **Rust**: 109 testes passam + 1 ignorado (o teste real). Novos: `url_for` contrato, `wav16` parser, `fala_real_atravesa_o_nosso_cliente` (ignorado).
- **Vitest**: 30 testes (1 novo: catálogo `lastSpeech`/`liveSpeech`).
- **Playwright**: 22 cenários (2 novos em `tests/ui/voice.spec.ts`: painel na prévia + escuta real com `--use-fake-device-for-media-stream`/`--use-fake-ui-for-media-stream`).
- `npm run check`, `npm run build`, `npm run docs`, `npm run update:check` aprovados.

## Como usar

1. **Instale o RealtimeSTT** na mesma máquina do BotLive:
   ```powershell
   winget install --id Python.Python.3.12 -e --accept-package-agreements --accept-source-agreements --disable-interactivity
   python -m pip install --upgrade pip
   python -m pip install "RealtimeSTT[server,faster-whisper]"
   # Para GPU NVIDIA (CUDA 12):
   python -m pip install torch==2.14.0+cu130 --index-url https://download.pytorch.org/whl/cu130
   python -m pip install nvidia-cublas-cu12 nvidia-cuda-runtime-cu12
   ```
2. **Inicie o servidor** (mantenha a janela aberta):
   ```powershell
   stt-server-production --host 127.0.0.1 --port 8010 --engine faster_whisper --model small --device cuda --compute-type float16 --language pt --no-model-warmup
   ```
   - Para CPU: troque `--device cpu --compute-type int8` e remova `--no-model-warmup`.
3. No BotLive desktop: **Comunidade → Controle por voz → Abrir Controle por voz**.
4. Confira **Servidor de transcrição** = `http://127.0.0.1:8010/transcribe-pcm16`.
5. Escreva **Palavras de ativação** (ex.: `arroba, botlive`) — ou deixe vazio para toda fala disparar.
6. Ligue **Escuta contínua**, autorize o microfone.
7. Crie um fluxo **Comando de voz** com trecho `boas-vindas` e ação de chat.
8. Diga "arroba, boas-vindas" → a ação dispara, a fala entra no Histórico e vira `{{lastSpeech}}`.

Para o overlay: use `examples/overlay.html` (agora com suporte a `captions`) como fonte de navegador no OBS.

## Validação

| Item | Resultado |
|------|-----------|
| `cargo test --locked --lib` | 109 passed, 1 ignored |
| `npx vitest run` | 30 passed |
| `npx playwright test` | 22 passed (2 novos voice) |
| `npm run check` | OK |
| `npm run build` | OK |
| `npm run docs` | OK (manual HTML regenerado) |
| `npm run update:check` | OK |
| Servidor real RealtimeSTT | HTTP 200, transcrição em ~1,3 s, CUDA OK |
| Teste Rust ignorado (`fala_real...`) | passa com WAV real via `BOTLIVE_STT_PCM` |

## Limitações

1. **Legenda mostra fala finalizada**, não parcial (o servidor só devolve o texto completo ao fim da utterance).
2. **Captura depende do webview aberto**: fechar a janela do BotLive encerra a escuta; minimizar mantém (o WebView2 continua rodando).
3. **Sem teste em live ao vivo** com chat real, ruído de microfone real, múltiplas pessoas falando ao mesmo tempo — validado só com fala sintética SAPI + servidor local.
4. **Tempo de timeout não aparece no Histórico nem em variável**: quando o servidor devolve 503 ("modelo carregando"), a falha aparece na linha de status do painel ("Escuta interrompida: ...") mas não gera entrada no Histórico nem variável específica.
5. **`encoding=pcm16` é fixo no código**: se o servidor futuro aceitar outro nome, será preciso ajustar o Rust.
6. **GPU NVIDIA recomendada**: CPU funciona (`--device cpu --compute-type int8`) mas latência sobe para 3–5 s no modelo `small`.
7. **Python 3.12 fixo no instalador do RealtimeSTT**; versões diferentes podem quebrar dependências (ctranslate2, faster-whisper).
8. **Torch CPU padrão quebra GPU**: `pip install torch` sem `--index-url` instala a versão CPU e o servidor falha ao tentar `float16` em CPU — documentado no guia de instalação.
9. **Warm-up do modelo `small` em CUDA travava sem `cublas64_12.dll`**; resolvido instalando `nvidia-cublas-cu12` e `nvidia-cuda-runtime-cu12` e colocando os diretórios `bin` no PATH do processo.

## Próximos passos (0.1.20)

Comandos criados por moderadores via chat da Twitch: `!cmd add/del/edit/list` (ex.: `!cmd add !oi Olá {{user}}`). Vira automação normal na página **Comandos**; só moderadores e streamer (badge Twitch).

---

# BotLive 0.1.18

## O que mudou

Seis correções e funções que não alteram o comportamento já usado:

- **Uma resposta por mensagem quando vários fluxos casam.** Quando mais de uma automação atende a mesma mensagem, apenas uma publica a resposta. A ordem de decisão é o tipo do gatilho — comando, depois chamada pelo nome, depois contém, depois toda mensagem — e, entre gatilhos do mesmo tipo, vence o mais específico: primeiro o de menos variações e mais letras, depois o que aparece antes na lista de Automações. Os demais fluxos continuam rodando por inteiro: contagem, pontos, memórias, áudio, webhook e as demais ações paralelas seguem normalmente; só a publicação de resposta fica de fora. O Histórico registra **Outra automação já respondeu esta mensagem; os demais efeitos continuam** no fluxo que ficou de fora. Implementado em `src-tauri/src/engine.rs` (pré-passagem de permissão e intervalo, escolha do vencedor pela chave composta e retenção das ações de resposta) e exposto no manual no capítulo 03.
- **`{{commandCount}}` deixou de sair zero.** O contador só aumentava com o interruptor **Contar usos deste comando** ligado, e um fluxo que usasse o marcador sem ligar o interruptor ficava preso em zero. Agora, gravar um comando cujo texto contenha `{{commandCount}}` liga o interruptor na mesma gravação (`src-tauri/src/command_counter.rs`, `auto_enable`, chamado no `flow.save` de `src-tauri/src/lib.rs`), e os comandos antigos já existentes tiveram o interruptor ligado na primeira abertura do aplicativo (`migrate_counters`, chamado na inicialização). Só comandos com gatilho **Comando de chat** recebem contador; timer e demais gatilhos continuam recusados com o aviso de sempre.
- **Backup diário em pasta escolhida, com retenção e extensão própria.** Novo cartão **Backup do bot** em Configurações, com **Escolher pasta**, **Backup automático diário**, **Guardar por dias** (1 a 365), **Semanas** (1 a 52), **Meses** (1 a 60), **Fazer backup agora** e **Importar backup…**. Cada arquivo tem extensão `.botlivebak` e nome `botlive-AAAA-MM-DD.botlivebak`, com sufixo `-1`, `-2` quando já existe um na mesma data, sem sobrescrever. O arquivo guarda as tabelas `profiles, flows, command_counters, logs, presets, kv, module_state, points, variables` e as pastas `knowledge, media, vaults`, em JSON único. O automático roda pelo agendador, no máximo um arquivo por dia, tenta de novo em uma hora quando falha e registra o resultado no Histórico (categoria `backup`). A retenção guarda as últimas cópias de cada dia do período, a mais recente de cada semana e a mais recente de cada mês, e só apaga arquivos com o nome do BotLive na pasta escolhida. A importação valida formato e versão, recusa arquivo de outra origem, grava uma cópia de segurança `backup-antes-da-importacao-AAAA-MM-DD-hhmmss.botlivebak` na pasta de dados antes de trocar, desconecta os perfis, encerra as tarefas do Discord e recarrega a tela. O cofre de credenciais nunca é lido por nenhum caminho do módulo: as senhas não entram no arquivo. As operações `backup.get`, `backup.save`, `backup.list`, `backup.now` e `backup.restore` são do proprietário. Implementado em `src-tauri/src/backup.rs` (novo), com chamadas em `src-tauri/src/lib.rs`, permissão em `src-tauri/src/access.rs`, agendador em `src-tauri/src/scheduler.rs` e interface em `src/BackupSettings.tsx` (novo) dentro de `src/Settings.tsx`.
- **Ação Punir na Twitch (silenciar, banir ou avisar).** Ação nova nos comandos, timers e automações, com **O que aplicar** (silenciar por um tempo, banir ou avisar), **Duração do silêncio (segundos)** de 1 segundo a 14 dias quando o modo usa tempo, **Quem leva a punição** (quem enviou a mensagem ou primeiro argumento do comando) e **Motivo**, que vai para a Twitch e para o Histórico. No alvo pelo primeiro argumento, o nome citado é convertido em ID pela API da Twitch antes da ação. A ação só executa em perfil da Twitch com a conta do canal autorizada; em perfil de outra plataforma ou em evento sem conta da Twitch, ela é recusada com aviso em português no Histórico, sem nenhuma chamada externa. Prévia e simulação nunca punem: mostram o plano (`Executaria punish: ...`). Erros de entrada são recusados na gravação, com frase própria para modo, alvo e duração. Implementado em `src-tauri/src/moderation.rs` (`punish`, `twitch_user_id`, `first_login`), campo `punish` e validações em `src-tauri/src/model.rs`, braço da ação em `src-tauri/src/engine.rs` e painel em `src/FlowEditor.tsx` com os rótulos em `src/types.ts`.
- **Aviso quando a mensagem usa uma variável local que nenhuma ação define.** O editor de mensagem mostra um aviso sob o quadro citando o nome da variável e explicando que a ação para com **Variável ausente** no Histórico sem publicar. Corrige o caso do timer **Donate**, que falhava com `Variável ausente: local.aiResponse` por citar a resposta da IA sem nenhuma ação de IA ao lado: acrescente a ação que gera o valor ou use `{{local.aiResponse|default:...}}`, e o aviso some. Implementado em `src/variableCatalog.ts` (`missingLocals`) e `src/MessageEditor.tsx`, com teste em `src/variableCatalog.test.ts`.
- **Variações de comando com erro de digitação.** **Texto que dispara** do gatilho Comando passa a aceitar uma lista separada por vírgula, como `!whislist, !whishlist, !wishlist`: quem erra o nome dispara o mesmo comando. Cada variação começa com `!`, não contém espaço e é comparada como palavra inteira, sem variação o comando não dispara nada e a prévia continua usando a primeira. O campo Comando do comando simples ganhou a mesma dica e o mesmo limite de entrada, e o editor visual explica a lista no campo do gatilho. Implementado em `src-tauri/src/model.rs` (`command_any`, validação e prévia) e em `src/FlowTools.tsx` e `src/FlowEditor.tsx`.

Arquivos: `src-tauri/src/backup.rs` (novo), `src-tauri/src/engine.rs`, `src-tauri/src/model.rs`, `src-tauri/src/moderation.rs`, `src-tauri/src/command_counter.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/access.rs`, `src-tauri/src/scheduler.rs`, `src-tauri/src/integration_tests.rs`, `src/types.ts`, `src/variableCatalog.ts`, `src/MessageEditor.tsx`, `src/FlowEditor.tsx`, `src/FlowTools.tsx`, `src/BackupSettings.tsx` (novo), `src/Settings.tsx`, `src/api.ts`, `src/styles.css`, `src/variableCatalog.test.ts`, `tests/ui/punish-and-backup.spec.ts` (novo) e os capítulos 03, 04, 07 e 08 do manual mais `docs/VARIAVEIS.md` e `docs/VALIDACAO.md`.

## Como usar

1. Para as variações de comando, abra **Comandos → Novo comando** e escreva no campo **Comando** as grafias separadas por vírgula, como `!whislist, !whishlist, !wishlist`. Salve e confira na lista: quem escreveu errado dispara a mesma resposta.
2. Para a contagem sair do número certo, escreva `{{commandCount}}` na resposta e salve: **Contar usos deste comando** é ligado junto. Em comandos antigos, a primeira abertura do aplicativo já liga o interruptor; confira na coluna Contador da lista.
3. Para o backup, abra **Configurações → Backup do bot**, escolha a pasta, ligue **Backup automático diário** e ajuste dias, semanas e meses. Use **Fazer backup agora** antes de fechar antes de uma live e confira o resultado no Histórico, categoria `backup`.
4. Para importar, clique em **Importar backup…**, escolha o arquivo `.botlivebak` e confirme: todas as áreas são substituídas pelas do arquivo. Depois da tela recarregar, reconecte os perfis e o Discord. A cópia `backup-antes-da-importacao-...botlivebak` fica na pasta de dados.
5. Para punir alguém, adicione a ação **Punir na Twitch**, escolha o modo, a duração e quem leva a punição, escreva o motivo e restrinja o gatilho em **Quem pode usar** para Moderadores ou Só o streamer. No alvo pelo primeiro argumento, escreva `!silenciar @alvo`. Confira o plano na simulação antes da live.
6. Para o timer Donate, abra o timer e observe o aviso sob o quadro de mensagem: acrescente **Gerar resposta da IA (variável)** antes de usar `{{local.aiResponse}}`, ou troque o marcador por `{{local.aiResponse|default:sem resposta}}`.

## Validação

Executado localmente nesta versão, em Windows x64 com Node.js 25 e Rust MSVC:

- `node .tools/run.cjs cargo test --locked --lib --manifest-path src-tauri/Cargo.toml`: 98 testes Rust aprovados, sendo 12 novos: em `model.rs`, `command_trigger_accepts_variations_separated_by_commas` (variações, maiúsculas, palavra inteira, lista vazia e opção vazia) e `command_variations_and_punish_rules_are_validated` (cada opção começa com `!`, sem espaço, e as regras de modo, alvo e duração da punição); em `engine.rs`, `only_the_most_specific_trigger_publishes_when_several_match` (tipo do gatilho, especificidade e desempate pela ordem) e `only_reply_actions_are_held_back_from_the_losing_flow` (só ações de resposta são retidas); em `command_counter.rs`, `counter_turns_on_when_the_answer_uses_command_count` e `migration_ligates_only_old_commands_that_use_the_counter`; em `backup.rs`, `only_relative_paths_inside_the_content_folders_are_accepted` (caminhos relativos só das três pastas de conteúdo), `backup_round_trip_covers_every_area_and_keeps_credentials_out` (criar, listar, importar, cópia de segurança, recusa de arquivo alheio, segundo arquivo do mesmo dia e ausência das chaves `bot_token`, `channel_token` e `refresh_token` no arquivo) e `retention_keeps_recent_days_and_a_sample_of_weeks_and_months` (datas fixadas, arquivos fora da retenção apagados, arquivo que não é nosso intocado e segunda passagem sem mudanças); e em `integration_tests.rs`, `only_one_automation_publishes_when_several_match` (ambos os fluxos rodam, o perdedor registra a mensagem e só o vencedor tenta publicar), `punish_action_is_previewed_validated_and_refused_without_a_twitch_account` (prévia sem punição, evento sem conta da Twitch recusado, perfil fora da Twitch recusado e entrada inválida na gravação) e `backup_is_owner_only_and_command_count_turns_on_when_it_is_used` (contador ligado na gravação com aviso no Histórico, retenção fora do intervalo e pasta automático sem pasta recusados, operações de backup recusadas para quem não é proprietário e arquivo de outra origem recusado).
- `npm run test` (Vitest): 29 testes aprovados, sendo 4 novos em `src/variableCatalog.test.ts` para `missingLocals` (aponta o local sem origem, some quando alguma ação gera o valor, respeita `|default:` e ignora o que não é local, e apagar ou incrementar não conta como origem).
- `npx playwright test`: 22 cenários aprovados, sendo 4 novos em `tests/ui/punish-and-backup.spec.ts` (painel da punição com modo, duração e alvo e os avisos de quem executa; comando com vírgulas aceito no comando simples e dica presente nos dois editores; aviso de variável local sem origem que some com `|default:`; e o cartão de backup com pasta, retenção, botão bloqueado sem pasta e recusa fora do desktop). Os 18 cenários existentes passaram sem alteração.
- `npm run check` (TypeScript) e `npm run build` (tsc + Vite) aprovados.
- `npm run test:updates`: 8 testes dos scripts de atualização aprovados.
- `npm run docs` aprovado; manual HTML regerado com 21 capítulos, após atualizar os capítulos 03, 04, 07 e 08, `docs/VARIAVEIS.md` e `docs/VALIDACAO.md`.
- `npm run update:check` aprovado com as notas 0.1.18.

Ocorrido durante a validação: na primeira execução da suíte completa, o cenário antigo `updates.spec.ts` falhou porque o cartão novo de backup tratava a resposta vazia da prévia de navegador como lista e derrubava a tela de Configurações (`Cannot read properties of null (reading 'length')`). O componente foi endurecido para aceitar resposta nula ou fora do formato, o cenário passou isolado e a suíte completa voltou a aprovar os 22 cenários, sem alteração de código entre essas duas execuções. Um erro de digitação no teste novo também foi corrigido antes de rodar: o tipo do objeto de ação exigia o campo `target`.

Também registrado: em uma das cinco execuções da suíte completa desta versão, o cenário antigo `tests/ui/app.spec.ts:105` (`catálogo de variáveis insere marcador e informa limite da prévia web`) falhou uma vez com a resposta contendo texto duplicado (`Olá, {{arg0}}{{arg0|default:amigo|upper}}Olá, `). Ele passou nas outras quatro execuções completas e em três execuções isoladas. O caminho de inserção (`src/MessageEditor.tsx` e `src/Variables.tsx`) não foi alterado nesta versão — a mudança no editor é só o aviso de variável local —, então a falha é uma corrida intermitente de temporização entre o preenchimento do campo e o clique de inserir, que não foi reproduzida de propósito nem corrigida nesta versão.

## Limitações

- Sem contas OAuth nesta validação não houve punição real na Twitch: o caminho foi comprovado pela recusa de eventos sem conta, pela prévia que não pune e pelas validações de entrada, o que não equivale à homologação de um timeout, ban ou aviso em canal ao vivo.
- A punição depende de uma pessoa: em timer ou em evento sem ID numérico da Twitch ela é recusada com aviso no Histórico, e em perfil que não é da Twitch ela não executa. A conversão de nome para ID usa a API da Twitch e falha com o nome citado inexistente.
- Quando vários fluxos casam, o vencedor é escolhido uma única vez entre os que passaram permissão e intervalo: se ele falhar ao publicar, nenhum outro fluxo assume a resposta nessa mesma mensagem. Os efeitos paralelos dos demais continuam acontecendo.
- O contador ligado sozinho vale só para gatilho **Comando de chat**; um timer que use `{{commandCount}}` continua sem contador e continua mostrando o aviso do editor.
- O backup automático só grava com o aplicativo aberto, no máximo um arquivo por dia, sem envio para nuvem e sem cópia em outro disco. A pasta precisa existir antes de ser escolhida.
- Limites do arquivo: 256 MB no total, até 64 MB por arquivo de `knowledge`, `media` ou `vaults` (maior que isso faz a gravação falhar com aviso), até 5000 arquivos e até 500 mil linhas por arquivo. Acima disso, o backup não é gravado.
- A importação substitui todas as áreas, sem mesclar. Se a gravação dos arquivos falhar no meio, as tabelas já trocadas permanecem; a cópia de segurança gravada antes é o caminho de volta, e ela só é gravada quando o estado atual cabe em 256 MB.
- As credenciais ficam fora do arquivo por construção: importar em outro computador exige reautorizar Twitch e Discord. O arquivo não é criptografado, portanto trate a pasta de backups como material sensível.
- O histórico de estatísticas entra no backup porque ele vem das tabelas de log; o arquivo também carrega operações e estados abertos no momento da gravação.
- No comando simples, cada variação precisa começar com `!` e não pode conter espaço dentro dela; `!whis list` é recusado na gravação, com o motivo em português.

---

# BotLive 0.1.17

## O que mudou

Quatro funções novas e independentes nenhuma das quais mexe no comportamento anterior:

- **Número sorteado em qualquer mensagem.** A variável `{{random:min,max}}` passa a existir junto de `{{randomViewer}}`. `{{random:1,50}}` sorteia um inteiro entre 1 e 50, ambos incluídos; `{{random:1,50.00}}` sorteia um decimal com vírgula brasileira, como `6,65`. As casas decimais são as do limite da faixa que tiver mais, até quatro. Na entrada os decimais usam ponto, porque a vírgula separa os dois valores; na saída o separador é sempre vírgula. A faixa vai de -1000000000 a 1000000000 e `min` não pode ser maior que `max`; erro de sintaxe ou faixa inválida interrompe a ação com aviso em português no Histórico, como qualquer variável ausente. O sorteio acontece a cada conferência da prévia e a cada execução real, uma vez por ocorrência escrita na mensagem: a mesma ação com `{{random:1,50}}` duas vezes dá dois números diferentes. O catálogo ganhou a ficha **Número sorteado** (grupo Execução), cujo painel **Opções** explica a faixa em vez de oferecer alternativa e formato, como fazem as variáveis de texto. Implementado em `src-tauri/src/variables.rs` (função `draw`, chamada na expansão da expressão), com teste em `src-tauri/src/variables.rs` e asserções em `src-tauri/src/integration_tests.rs`.
- **Responder a quem enviou.** Comando, timer e automação ganharam o interruptor **Responder à pessoa que enviou** logo abaixo de **Como enviar na Twitch**. Ligado, a mensagem sai no fio da pessoa que disparou, como um reply da Twitch; no Discord o BotLive cita a mensagem original. Nas plataformas sem essa forma, o Histórico registra o aviso e a mensagem sai comum, como já acontece com anúncio, fixação e destaque. Se você trocar a forma de envio para anúncio, mensagem fixada ou destaque, a mensagem sai sem o fio e o próprio aviso do campo avisa disso. O campo fica na prévia no Histórico. A escolha é gravada no fluxo (`reply_to` em `src-tauri/src/model.rs`), percorre o envio (`src-tauri/src/engine.rs`) e a plataforma (`src-tauri/src/platforms.rs`, `src-tauri/src/discord.rs`), e é editada em `src/FlowOptions.tsx` (componente `SendPicker`).
- **Gatilho Chamada pelo nome do bot.** Nova opção do campo **Evento** para o espectador escrever o nome do bot sem `@`, como "Arroba, vem aqui". **Texto que dispara** passa a aceitar uma lista de nomes separados por vírgula, e o gatilho só passa quando um deles aparece **como palavra inteira**: com `Arroba, ArrobaSrv`, a mensagem "ArrobaSrv mandou" não dispara pelo nome curto, porque `ArrobaSrv` é uma palavra só, e "arromba" não dispara dentro de "arrombado". A comparação ignora maiúsculas e minúsculas, aceita o nome com `@` na mensagem e respeita permissão, intervalo por fluxo e intervalo por pessoa dos demais gatilhos. A prévia usa o primeiro nome da lista. Salvar sem informar nenhum nome continua sendo recusado, com aviso em português. Implementado em `src-tauri/src/model.rs` (`mention_any`, `matches`, validação e prévia) e exposto em `src/types.ts` e `src/FlowEditor.tsx`.
- **Emotes da Twitch nas respostas da IA.** Quando o perfil conecta na Twitch, o BotLive busca os emotes do seu canal e os globais pela API da Twitch e guarda os nomes no perfil (`src-tauri/src/emotes.rs`, chamado na conexão por `src-tauri/src/platforms.rs`). A lista entra no pedido à IA com a regra: no máximo um emote por resposta, só onde combinar com a frase, maioria das respostas sem emote, nunca no meio de uma palavra e nunca citando o nome do emote como texto (`src-tauri/src/ai.rs`). A busca acontece só na conexão, não em cada resposta: a geração lê a lista guardada de forma síncrona. A lista vale 12 horas; quando a Twitch falha, a próxima tentativa é daqui a 5 minutos e as respostas seguem sem emote nesse meio-tempo, com aviso único por sessão no Histórico (categoria `ai`, informação). Entram no máximo 60 nomes, com os do canal antes dos globais, para a regra não pesar no prompt. As APIs usadas pedem só um token de usuário já existente, sem nenhuma autorização nova, e em perfil que não é da Twitch a lista fica vazia sem efeito nenhum.
- **Campo de instrução da IA maior.** O quadro **Como a IA deve responder** (e o demais campos de mensagem com a mesma classe) passa a crescer com o texto a cada tecla e a parar em 42vh, em vez de manter quatro linhas fixas com rolagem interna. Implementado no hook `src/autoGrow.ts`, aplicado em `src/MessageEditor.tsx` e `src/AIConversation.tsx`, com o teto em `src/styles.css`.

Arquivos: `src-tauri/src/variables.rs`, `src-tauri/src/model.rs`, `src-tauri/src/engine.rs`, `src-tauri/src/platforms.rs`, `src-tauri/src/discord.rs`, `src-tauri/src/ai.rs`, `src-tauri/src/emotes.rs` (novo), `src-tauri/src/lib.rs`, `src-tauri/src/integration_tests.rs`, `src/types.ts`, `src/FlowEditor.tsx`, `src/FlowOptions.tsx`, `src/FlowTools.tsx`, `src/Variables.tsx`, `src/variableCatalog.ts`, `src/MessageEditor.tsx`, `src/AIConversation.tsx`, `src/autoGrow.ts` (novo), `src/styles.css`, `src/variableCatalog.test.ts`, `tests/ui/random-reply-mention.spec.ts` (novo) e os capítulos 03, 04 e 08 do manual mais `docs/VARIAVEIS.md` e `docs/VALIDACAO.md`.

## Como usar

1. Para sortear um número, abra **Inserir variável e testar mensagem**, procure **Número sorteado** e clique na ficha; depois ajuste a faixa no próprio código, como `{{random:1,50.00}}`. Confira em **Testar como a mensagem vai ficar**: cada conferência sorteia de novo.
2. Para citar no fio de quem falou, ligue **Responder a quem enviou** em **Como enviar na Twitch**, salve e confira o Histórico na simulação. Nas plataformas sem resposta direcionada o aviso diz que a mensagem sai comum.
3. Para o bot atender pelo nome, crie uma automação, escolha **Evento: Chamada pelo nome do bot** e escreva em **Texto que dispara** os nomes separados por vírgula, como `Arroba, ArrobaSrv, arromba`.
4. Para a IA usar os emotes do seu canal, nada é preciso: conecte o perfil na Twitch e confira no Histórico a linha `Emotes da Twitch carregados para a IA: N.` Ela aparece na reconexão, a cada 12 horas no máximo. Se a lista não chegar, o aviso único `Sem emotes da Twitch agora` explica o motivo e as respostas seguem sem emote até a nova tentativa, 5 minutos depois.
5. Para escrever instruções longas de IA sem rolagem interna, digite normalmente: o quadro cresce com o texto e para em 42vh.

## Validação

Executado localmente nesta versão, em Windows x64 com Node.js 25 e Rust MSVC:

- `npm run check` (TypeScript) aprovado após as mudanças no editor de fluxos, no catálogo de variáveis e no hook de crescimento.
- `npm run test`: 25 testes Vitest aprovados, incluindo o do catálogo de variáveis para a ficha **Número sorteado**.
- `node .tools/run.cjs cargo test --locked --lib --manifest-path src-tauri/Cargo.toml`: 86 testes Rust aprovados, sendo novos os três de `emotes` (`keeps_channel_emotes_first_and_never_repeats`, `stops_at_the_prompt_limit` e `cache_roundtrip_reads_back_what_was_stored`), dois do sorteio em `variables.rs` (`random_stays_inside_the_range_and_writes_decimals_with_a_comma` e `random_explains_a_bad_range_instead_of_sending_a_hole`), dois do gatilho em `model.rs` (`mention_trigger_waits_for_one_of_the_bot_names` e `mention_preview_calls_the_bot_and_the_flow_needs_a_name`), a regra de emote em `ai.rs` (`emote_rule_reaches_the_prompt_only_when_there_are_emotes`), o corpo de resposta em `platforms.rs` (`chat_body_keeps_the_reply_thread_only_when_asked`) e a citação no Discord (`message_body_quotes_the_original_only_when_replying`), além das asserções de `{{random:3,3}}` e `{{random:3.0,3.0}}` dentro de `variable_templates_are_single_pass_bounded_and_typed`.
- `npx playwright test`: 18 cenários aprovados, sendo 4 novos em `tests/ui/random-reply-mention.spec.ts` (ficha de sorteio com faixa em **Opções**, gatilho `mention` com dica e persistência da resposta direcionada no editor visual e no comando simples, e o quadro de instrução da IA crescendo até 42vh). Os 14 cenários existentes passaram sem alteração. Em uma das três execuções da suíte completa o cenário antigo `app.spec.ts` "catálogo de variáveis insere marcador e informa limite da prévia web" falhou por corrida entre o preenchimento do campo **Resposta** e a inserção da ficha; ele passou isolado e nas outras duas execuções completas, sem alteração de código entre elas.
- `npm run test:updates`: 8 testes dos scripts de atualização aprovados.
- `npm run build` (tsc + Vite) e `npm run docs` aprovados; manual HTML regerado.
- `npm run update:check` aprovado com as notas 0.1.17.

## Limitações

- Sem contas OAuth nesta validação não houve envio real de chat, anúncio, fixação, destaque nem leitura da API de emotes da Twitch: o módulo de emotes foi comprovado pela composição da lista e pela persistência no perfil, o que não equivale à homologação em canal ao vivo.
- A resposta direcionada depende da autorização vigente: um fluxo com a autorização antiga continua publicando a mensagem comum, e a Twitch recusa o fio quando a forma escolhida não é mensagem normal, como já acontece com anúncio e fixação.
- A lista de emotes é composta por até 60 nomes com o canal antes dos globais; emotes ganhos depois disso entram na próxima busca de 12 horas, e um emote novo pode não aparecer se o limite já estiver cheio.
- `{{random:min,max}}` é um sorteio puro: não há semente, repetição nem exclusão de valores já sorteados. Faixas com casas decimais usam limite superior exclusivo, como manda a biblioteca de geração aleatória; no inteiro os dois extremos entram.
- O gatilho **Chamada pelo nome do bot** só observa mensagens de chat, não eventos, voz nem simulação, e a palavra inteira é definida por fronteira alfanumérica: `@Arroba` e `ArrobaSrv,` passam, `ArrobaSrv` sozinho não dispara pelo nome curto.
- O campo que cresce tem teto de 42vh; acima dele o texto continua rolando dentro do quadro, por escolha de leitura.
- O teste antigo `app.spec.ts` do catálogo de variáveis mostrou flakiness nesta rodada: falhou em uma das três execuções da suíte completa por corrida de preenchimento e passou isolado e nas demais, sem mudança de código. Não é sintoma de produto, mas convém observar se repetir.

---

# BotLive 0.1.16

## O que mudou

A IA das **Automações** passou por três correções independentes sobre o diagnóstico de quem relatou os sintomas. Nenhuma função existente mudou de lugar:

- **Endereço do provedor sem caminho duplicado.** O caminho da chamada agora é montado conforme o provedor escolhido (função `chat_url` em `src-tauri/src/ai.rs`). Sem caminho no endereço entra o do provedor: `/api/chat` no Ollama, `/v1/chat/completions` na API compatível e `/v1/messages` no Anthropic. Um endereço que já traz o caminho é aproveitado como está e nunca recebe o caminho duas vezes, e um endereço vindo de outro formato é convertido para o equivalente: `https://ollama.com/api/chat` com a API compatível selecionada vira `https://ollama.com/v1/chat/completions` em vez de `https://ollama.com/api/chat/chat/completions`, que devolve HTTP 404.
- **Cada erro diz qual é a ação certa.** HTTP 404 aponta o endereço exato usado na chamada, porque o problema é o endereço e não o modelo; 401 e 403 apontam a chave; 429 aponta limite ou quota; e os 5xx pedem nova tentativa.
- **Texto de análise não vai mais para o chat.** Quando o provedor devolve a leitura da mensagem em vez da frase pronta (o padrão que publicava "The user sent ..." em inglês), a ação falha com erro em português no Histórico, a variável de resposta fica vazia e nada é enviado ao público. O pedido ao modelo passou a carregar `replyFormat`, pedindo a frase final em português sem análise, e as regras do sistema reforçam que análise, tradução e raciocínio interno não vão para o chat.
- **Raciocínio sem resposta ganhou erro próprio.** Quando o conteúdo vem vazio porque o modelo gastou os tokens raciocinando, o aviso explica isso e sugere aumentar o tamanho da resposta ou escolher outro modelo; antes aparecia apenas "O provedor não retornou uma mensagem".
- **Mais espaço para o raciocínio do modelo.** O orçamento de tokens enviado ao provedor passa de 110/250/450 para 500/1000/1600 nos tamanhos curto, médio e livre. O texto publicado continua cortado em 120, 300 ou 450 caracteres, sem mudança do que o espectador vê.
- **Gatilho Mensagem contém aceita lista por vírgula.** Um gatilho escrito como `comprei, comprar, gastei` passa a disparar quando uma das partes aparece na mensagem. Antes a lista inteira precisava aparecer no texto, e por isso gatilhos que funcionavam na prévia nunca disparavam na live. Sem vírgula o comportamento continua o mesmo de antes. A prévia do editor usa a primeira opção da lista, o campo **Texto que dispara** ganhou uma dica no editor visual e a recusa de salvar continua igual.
- **Nome BotLive no topo esquerdo.** A marca na barra lateral passa de `botlive.` para `BotLive.`.

Arquivos: `src-tauri/src/ai.rs`, `src-tauri/src/model.rs`, `src/App.tsx`, `src/AIMemory.tsx`, `src/FlowEditor.tsx`, `tests/ui/app.spec.ts`, `tests/ui/flow-inspector.spec.ts`, os capítulos 03, 04 e 08 do manual e `docs/VARIAVEIS.md`.

## Como usar

1. Em **Inteligência artificial**, informe a base do provedor (por exemplo `https://ollama.com` ou `https://api.openai.com/v1`) e deixe o caminho para o BotLive montar; a dica do campo explica o comportamento. Endereços completos já existentes continuam valendo sem reconfigurar.
2. Em **Automações → gatilho Mensagem contém**, separe palavras ou frases com vírgula. Basta uma delas aparecer na mensagem; sem vírgula, o texto precisa aparecer inteiro.
3. Se a IA não responder, abra **Histórico**: o erro diz se é endereço (404, com o endereço usado), chave (401/403), limite (429), raciocínio sem resposta ou texto de análise recusado.
4. Em **Como esta ação responde**, aumente o tamanho da resposta quando o aviso indicar que o modelo gastou os tokens pensando; a troca de modelo continua sendo o caminho quando a resposta vier como análise.
5. O topo esquerdo do aplicativo agora mostra **BotLive.**; nada mais foi renomeado.

## Validação

Executado localmente nesta versão, em Windows x64 com Node.js 25 e Rust MSVC:

- `npm run check` (TypeScript) aprovado após as mudanças no App, no editor de fluxos e na tela de IA.
- `npm run test`: 24 testes Vitest aprovados.
- `node .tools/run.cjs cargo test --locked --lib --manifest-path src-tauri/Cargo.toml`: 77 testes Rust aprovados, sendo 3 novos: `chat_path_follows_the_provider_without_duplicating` (composição por provedor, conversão de `/api/chat`, caminho completo reaproveitado e recusas), `analysis_openers_never_pass_to_the_chat` (aberturas de análise em inglês e português barradas, respostas normais liberadas) e `contains_trigger_accepts_a_list_separated_by_commas` (lista com vírgula, trecho único e gatilho vazio).
- `npx playwright test`: 14 cenários aprovados, incluindo as asserções novas de marca `BotLive` na barra lateral e da dica de lista por vírgula no campo **Texto que dispara** (ausente em Comando, visível em Mensagem contém).
- `npm run test:updates`: 8 testes dos scripts de atualização aprovados.
- `npm run build` (tsc + Vite) aprovado.
- Sonda HTTP real sem chave: `POST https://ollama.com/api/chat/chat/completions` respondeu 404, que era o endereço montado antes da correção, e `POST https://ollama.com/v1/chat/completions` respondeu 401, confirmando que o caminho corrigido existe e pede chave.
- `npm run update:check` aprovado com as notas 0.1.16.

## Limitações

- Sem chave de API desta validação não houve geração real de um modelo pago: o endereço corrigido foi comprovado pela sonda HTTP (401) e por teste unitário, o que não equivale à homologação de uma resposta do provedor.
- O diagnóstico dos sintomas relatados — automação que não respondia, demora e resposta em inglês — foi feito sobre o banco e o log locais do aplicativo: caminho montado com 404, lista de gatilho que nunca casava na live, intervalo do gatilho e latência do próprio provedor.
- Modelos com raciocínio muito extenso ainda podem estourar o orçamento de tokens; o caso vira erro claro no Histórico, sem resposta publicada. O aumento de orçamento reduz a chance, não elimina.
- Quando a resposta já vem como análise, ela é descartada e não há nova tentativa automática; o fluxo segue em silêncio para o público, como já acontece em qualquer falha de geração.
- A conversão de endereço cobre os formatos conhecidos (`/api/chat`, `/v1`, `/chat/completions`, `/messages`). Um caminho desconhecido continua sendo apenas completado com o sufixo do provedor, e nesse caso um erro 404 mostra o endereço final para correção.
- Esta versão não reduz o tempo de resposta: a latência continua vindo do provedor, do modelo e da rede.

---

# BotLive 0.1.15

## O que mudou

A tela de **Automações** e o editor visual aberto a partir de **Comandos** e **Timers** ficaram mais fáceis de ler, sem perder nenhuma função:

- **Seções recolhíveis no painel do bloco.** O painel "CONFIGURAR BLOCO" deixa de ser uma lista longa e passa a agrupar os campos: no gatilho, **Quando** (evento, texto que dispara, quem pode usar, intervalos), **Como sai** (forma de envio e áudio do disparo) e **Comportamento** (contagem de usos e intervalo do timer, que começa recolhido); em cada ação, **Como sai** (tipo de ação, conteúdo, alvos, opções de IA e teste) e **Comportamento** (a condição opcional, que antes era o bloco "Condição opcional"). Todas as seções abrem e fecham com um clique e o que estava visível continua visível por padrão.
- **Ajuda longa recolhida.** O parágrafo "A mensagem atual, até 12 falas recentes…" virou o resumo recolhível **Como a IA monta a resposta**, dentro da ação de IA.
- **Erros técnicos traduzidos.** O novo módulo `src/errors.ts` converte falhas de rede, de leitura de dados, de permissão, de disco, de memória e panics em frases em português. Mensagens que já estão em português são mantidas sem alteração. Quando o texto original é técnico, ele continua disponível em **Detalhes técnicos** dentro do aviso de erro (`role="alert"`) do editor. A tradução vale para a lista e os diálogos de Comandos, Timers, Contadores e Automações, incluindo o áudio do disparo.
- **Ordem por conexão explicada em dois lugares.** O rodapé do painel agora diz que a ordem é a das setas entre os blocos, não a posição deles na tela, e que blocos desconectados ficam de fora; a dica entrou também no bloco de ajuda da página Automações.
- **Aviso na troca simples → visual.** O modo simples ganhou o botão **Abrir no editor visual**. Sempre que um comando ou timer abre no editor visual (por ter mais de uma ação ou pelo botão), um aviso no topo explica a troca e garante que nome, gatilho e ativação continuam os mesmos.
- **Ajustes de estilo** (`.flow-intro`, `.node-inspector > details`, detalhes do aviso de erro) preservam o visual premium: sem texto vazando das caixas e sem rolagem horizontal em 720 px.

Arquivos: `src/errors.ts` e `src/errors.test.ts` (novos), `src/FlowEditor.tsx`, `src/FlowTools.tsx`, `src/FlowOptions.tsx`, `src/Commands.tsx`, `src/styles.css`, `tests/ui/flow-inspector.spec.ts` (novo) e os capítulos 03 e 08 do manual.

## Como usar

1. Abra **Automações → Novo fluxo** (ou clique em **Editar** de um fluxo existente) e clique no gatilho: configure o essencial em **Quando** e **Como sai**; **Comportamento** fica recolhido e guarda a contagem de usos ou o intervalo do timer.
2. Clique em um bloco de ação: em **Como sai** escolha o tipo de ação, o conteúdo, as opções de IA e o teste; em **Comportamento** preencha **Executar só se a mensagem contiver** quando quiser condicionar a execução.
3. Para reordenar, arraste de um ponto de conexão ao outro e mantenha uma única sequência a partir do gatilho; a posição dos blocos na tela não altera a execução.
4. Em **Comandos**, um comando simples pode ser aberto no editor visual pelo botão **Abrir no editor visual** do rodapé; o aviso no topo explica o que muda. Com uma ou duas ações, ele continua abrindo pelo modo simples.
5. Se um salvamento for recusado, leia o motivo em português no topo do editor; quando houver, abra **Detalhes técnicos** para ver o texto original.

## Validação

Executado localmente nesta versão, em Windows x64 com Node.js 25 e Rust MSVC:

- `npm run check` (TypeScript) aprovado.
- `npm run test`: 24 testes Vitest aprovados, sendo 7 novos do tradutor de erros (mensagem em português preservada, falha de rede, leitura de dados, permissão, texto vazio/objeto genérico, técnico desconhecido com detalhe preservado e acentuação).
- `node .tools/run.cjs cargo test --locked --lib`: 74 testes Rust aprovados.
- `npx playwright test`: 14 cenários aprovados (os 13 existentes e o novo `tests/ui/flow-inspector.spec.ts`, que cobre seções abertas/recolhidas, permanência do estado recolhido após editar campos, dica de ordem por conexão, recusa de salvar com mensagem em português sem detalhe técnico, aviso da troca simples → visual, contagem de blocos e o retorno ao modo simples depois de salvar no visual).
- `npm run test:updates`: 8 testes dos scripts de atualização aprovados.
- Inspeção visual das telas do editor em 1440 px e 720 px (print em `artifacts/`), sem estouro horizontal e sem texto vazando das caixas.

## Limitações

- Os erros técnicos foram exercitados por teste unitário do tradutor; não houve falha real de rede, de disco ou de permissão para observar na tela durante a validação.
- Padrões técnicos desconhecidos caem na mensagem genérica "Não foi possível concluir", mantendo o texto original em **Detalhes técnicos**.
- Sem contas OAuth configuradas não houve envio real de chat nem homologação de eventos nas plataformas; a prévia de navegador não valida IPC, cofre, OAuth ou módulos nativos.
- O painel do gatilho continua com rolagem interna: em janelas muito baixas é preciso rolar para chegar em **Como sai**.

---

# BotLive 0.1.14

## O que mudou

- Resenha contextual: instruções passam a exigir resposta direta, sem repetir a fala na abertura e sem travessão. A limpeza local remove cópia literal inicial da mensagem com 12 ou mais caracteres, substitui travessões e limita o texto sem cortar palavras quando possível. Evitar repetição usa as seis respostas mais recentes do bot, em vez das primeiras da janela.
- Respostas automáticas sem tamanho explícito passam a até 120 caracteres e orçamento de 110 tokens. Escolhas explícitas de 300/450 caracteres continuam valendo. Trechos de conhecimento dividem o orçamento e têm teto de 2.200 caracteres por arquivo. Arquivos de outros nichos, canais e eventos não são abertos durante a seleção.
- Knowledge reconhece também canais/<canal>-exemplos-reais.md e prioriza o canal antes das gírias gerais. A opção Usar base no bloco funciona mesmo se o padrão do perfil estiver desligado, respeitando arquivos individualmente desativados.
- Memórias automáticas usam ID e plataforma, preservam notas manuais e antigas, guardam até 100 falas distintas de até 500 caracteres e evitam registros duplicados. Recuperação usa a mensagem atual, pontuação normalizada, relevância e linhas recentes, até quatro notas e 6.000 caracteres. Históricos automáticos de outras identidades não são lidos na recuperação.
- Em erro da IA, a variável é limpa, aiSuccess fica false e o fluxo para. Não há fallback para chat, voz ou overlay; o erro fica no Histórico. A opção antiga de resposta de indisponibilidade saiu da interface e valores legados são ignorados.
- Timers aparecem apenas na área Timers. O editor de automações não oferece mais gatilho periódico; timers existentes continuam preservados.
- Convite do Discord abre pelo navegador do sistema. A tela e o manual mostram a ordem correta: token, convite/autorização, descoberta, servidor salvo, canais/cargos, ativação e conexão. Explicam intents, hierarquia e permissões para moderação.
- Manual atualizado com fluxo completo da resenha, inventário dos 22 arquivos knowledge, diferença entre referências de projeto e funções implementadas, manutenção das memórias, exemplos, limites e silêncio em falhas. Fontes de variáveis, presets e timers também foram ajustadas.

## Como usar

1. Em Inteligência artificial, importe E:\StudioCactus\Arroba Chatbot\documentação\knowledge; escolha o nicho, a profundidade e salve. Após editar os originais, use Atualizar da pasta original.
2. Use Resenha com IA, ancoragem na mensagem atual, Uma frase e Evitar repetição. O tamanho automático agora também é curto. As opções por bloco sobrescrevem as do perfil.
3. Ative Registrar interações para guardar falas bem-sucedidas. Revise ou apague notas em Memórias. Falas muito curtas, comandos com ! e eventos sem ID não entram no registro automático.
4. Crie e edite mensagens periódicas em Timers. Automações fica para eventos e sequências de ações.
5. No Discord, salve o token de bot e abra Convite do bot antes de descobrir servidores. Autorize no servidor, volte, escolha e salve o servidor, descubra os canais novamente, ative e conecte. Posicione o cargo do bot acima dos cargos administrados. Consulte o capítulo Discord do manual.

## Validação

- Compilação TypeScript/Vite concluída; 17 testes Vitest e 13 cenários Playwright aprovados, incluindo a ausência de timers em Automações, gatilhos separados e instruções de instalação Discord.
- 74 testes Rust aprovados, incluindo limpeza de resposta, seleção dos últimos seis textos, orçamento e corpus do canal, deduplicação/limite/isolamento de memórias e interrupção de ai/ai.generate para respostas HTTP 401, 429 e 500 de um provedor local simulado.
- Oito testes dos scripts de atualização/publicação aprovados. Manual offline regenerado com 21 capítulos e links validados pelo gerador.
- A primeira compilação Vite e os testes dos scripts encontraram restrição de subprocessos EPERM; foram executados novamente com a permissão necessária e passaram. Falhas intermediárias de compilação/testes foram corrigidas antes do fechamento desta atualização.

## Limitações

- Não houve homologação com provedor de IA pago, chat ao vivo ou servidor Discord real. A latência real depende do modelo, rede e fila de envio; redução de contexto e tokens não é medição de tempo em produção.
- Memória continua lexical e registra falas, sem extração semântica ou verificação automática de fatos. Não há migração destrutiva das notas antigas; históricos legados continuam disponíveis como notas manuais.
- Paráfrases da pergunta e repetição semântica ainda dependem do modelo. O bloqueio de assuntos continua fazendo uma segunda chamada quando configurado.
- A autorização do convite Discord precisa ser concluída pelo responsável pelo servidor no navegador. A integração exige intents/permissões e o aplicativo aberto. Nenhuma punição real foi aplicada nesta atualização.
- A pasta original knowledge não foi modificada; referências de projeto nela não representam automaticamente recursos implementados. Testes de prévia usam configurações salvas do perfil, sem homologar ajustes por ação no provedor real.
- Não foi feita instalação em máquina limpa nem substituição do aplicativo que estava em execução no computador.

---

# BotLive 0.1.13

## O que mudou

Três frentes na mesma versão: os marcadores de variável passam a ser `{{nome}}`, a IA ganha uma base de conhecimento importada para dentro do aplicativo e cada ação de IA ganha controles próprios, além de receber um resumo do que a live está fazendo agora.

**Marcadores de variável.** O formato `$nome` saiu do motor. O editor, a prévia, a importação de preset e a leitura de TXT convertem sozinhos `$nome` em `{{nome}}`, apenas para nomes conhecidos do catálogo. `$desconhecido` continua texto puro, e valores como `$5`, `R$100` e o escape `\$` nunca são tocados. Gatilho, condição e script Rhai não são migrados automaticamente, e `%nome%` continua aceito. Os capítulos de variáveis, comandos, guia de uso e integrações foram atualizados.

**Base de conhecimento.** Em **Inteligência artificial**, o cartão **Base de conhecimento** importa uma pasta de arquivos `.md` (por exemplo `documentação\knowledge`) para os dados do perfil. Depois da importação a pasta original pode ser movida ou apagada sem quebrar nada, e **Atualizar da pasta original** repete a cópia. O cartão traz **Usar a base nas respostas**, **Nicho do canal**, **Profundidade** (Leve ~5 mil, Padrão ~13 mil, Completa ~20 mil caracteres por resposta) e a lista dos arquivos, cada um com interruptor para desligar sem apagar e lixeira para remover. A ordem de leitura segue a prioridade tom → nicho escolhido → evento da hora → gírias → canal → demais arquivos mais parecidos com a pergunta, sempre dentro do orçamento escolhido, e arquivos com `tipo: documento_de_logica` ficam de fora. A importação aceita até 400 arquivos `.md` em texto UTF-8, 200 kB por arquivo e 12 MB no total, sem atalhos e fora dos dados do aplicativo.

**Como a IA responde.** A tela de IA ganhou o cartão **Como responder**, com os padrões de ancoragem, tamanho da resposta e evitar repetição. No editor de fluxo, os blocos **Responder com IA** e **Gerar resposta da IA (variável)** abrem **Como esta ação responde**: a linha **Hoje vale** mostra o efeito depois das escolhas, cada campo em **Padrão do perfil** herda a tela de IA e um valor escolhido sobrescreve só aquele bloco. O mesmo painel tem **Tom deste bloco**, até 600 caracteres, aplicado depois da personalidade.

**O que a live está fazendo agora.** O pedido à IA passa a incluir um resumo em memória com a categoria atual (lida da Twitch na conexão e atualizada quando o canal troca de jogo), os eventos dos últimos 90 segundos (assinatura, bits, resgate por pontos, presente, doação, clipe e troca de categoria, com quem fez), o calor do chat e as falas recentes. Esse estado existe só em RAM, é limpo ao fechar o aplicativo, trocar de canal ou excluir o perfil, e simulação e prévia nunca o alteram.

**Verificação de commits.** A checagem por intervalo usada pela CI passou a tratar o commit de merge sintético que o GitHub cria ao validar um pull request. Antes, o diff daquele merge saía vazio e a validação falhava mesmo com o registro de atualização correto; agora o merge é comparado com o primeiro pai e um teste automatizado cobre esse formato.

Foram atualizados os capítulos de comandos e automações (03), IA e memória (04), solução de problemas (08) e a matriz de aceite.

## Como usar

1. **Variáveis:** abra um comando existente e salve. `$user` vira `{{user}}` automaticamente; `{{nome}}` é o formato que vale no editor, na prévia, no catálogo e nos presets.
2. **Base de conhecimento:** abra **Inteligência artificial**, clique em **Importar pasta** no cartão **Base de conhecimento** e escolha `documentação\knowledge`. Informe o nicho do canal, ajuste a profundidade, desligue o que não quiser usar e clique em **Salvar personalidade**.
3. **Padrões:** no cartão **Como responder**, escolha ancoragem, tamanho e repetição que todos os blocos novos de IA vão herdar.
4. **Por bloco:** em **Automações**, clique no bloco de IA e abra **Como esta ação responde** para sobrescrever qualquer valor ou escrever o tom daquele bloco, depois clique em **Salvar fluxo**.
5. **Estado da live:** conecte o perfil à Twitch. A categoria registrada aparece no **Histórico** como `Categoria atual: ...`, e os eventos entram conforme acontecem na transmissão.

## Validação

Executada nesta máquina antes de publicar:

- `cargo test --lib` (Rust): 69 testes aprovados, 0 falhas. Inclui os testes da base de conhecimento (importação, lista e seleção por prioridade), do estado da live, da normalização do EventSub `channel.update`, da herança dos controles por ação e de um resultado contextual comprovado com um provedor falso: calor, categoria, evento, ancoragem, estilo, tamanho e bloco de conhecimento presentes no pedido, mais a checagem da ação com a base desligada.
- `npm run check` (TypeScript): sem erros.
- `npm run test` (Vitest): 17 testes aprovados em 4 arquivos, sendo 6 novos sobre herança/sobreposição dos controles por ação e agrupamento da base.
- `npx playwright test`: 13 testes aprovados. O novo teste cria o perfil, abre a tela de IA, confirma os cartões e a lista da base, altera os padrões, salva, reabre, desliga um arquivo, confere a janela de 720 px sem rolagem horizontal e sem conteúdo vazando das caixas, e edita o bloco de IA verificando a linha **Hoje vale**, a sobreposição e a persistência depois de salvar o fluxo.
- `npm run docs`: manual regenerado com 21 capítulos.
- `npm run update:check` e `npm run test:updates`: executados na preparação desta versão, com 8 testes de scripts; o novo cobre a validação do merge de um pull request, que falhava antes da correção.

Não executados nesta atualização: conexão real com a Twitch, chamada a um provedor de IA com modelo pago, importação da pasta de conhecimento pelo aplicativo desktop (a prévia do navegador usa uma lista simulada) e medição de custo ou latência com a base ligada.

## Limitações

- O estado da live fica só em memória: fechar o aplicativo, trocar de canal ou excluir o perfil zera, e os eventos valem por 90 segundos. Não há geração de tópico por IA nem persistência desse estado entre sessões.
- A categoria vem da Twitch. Nas demais plataformas o resumo fica com os eventos e o calor do chat que o adaptador normalizar.
- A base aceita apenas arquivos `.md` em texto UTF-8, sem atalhos, com até 400 arquivos, 200 kB por arquivo e 12 MB no total; pastas dentro dos dados do aplicativo são recusadas.
- A seleção usa orçamento de caracteres: o que não couber fica de fora daquela resposta e pode entrar na tentativa seguinte. A recuperação é lexical, não semântica.
- Com a base ligada o pedido ao provedor fica maior; confira o consumo e os limites do seu provedor.
- Controles por ação valem apenas para as ações de IA. Marcadores `$nome` com nome desconhecido não são convertidos, para não quebrar textos que são dinheiro, data ou citação.

---

# BotLive 0.1.12

## O que mudou

Duas funções novas, disponíveis em comandos, timers e automações: anexar um áudio já carregado na plataforma e escolher a forma como a mensagem sai na Twitch.

**Tocar áudio ao disparar.** O campo abre a biblioteca de **Respostas e sons**: escolha um som já importado ou use **Escolher som** para importar WAV, MP3 ou OGG, ajuste **Volume** e ouça com **Testar som**. O som é solicitado antes das ações, na mesma execução da contagem do comando, então um comando `!ifood` com contador publica a mensagem com o número certo enquanto a vinheta toca. O áudio sai na saída do aplicativo, a mesma que o OBS captura. A simulação não toca nada e escreve `[Simulação] Tocaria o áudio do fluxo` no Histórico; se o arquivo for apagado da biblioteca, o fluxo continua sem som e o Histórico pede para escolher outro. O áudio do fluxo não depende de **Ativar sons por espectador** e não usa os intervalos daqueles sons. Ao importar preset, um áudio inexistente no perfil de destino é descartado em vez de quebrar o fluxo.

**Como enviar na Twitch.** Um campo por fluxo escolhe a forma das mensagens daquele fluxo: **Mensagem normal**, **Anúncio** com **Cor do anúncio** (cor do canal, azul, verde, laranja ou roxo), **Mensagem fixada** (comunicado no topo do chat por cerca de 20 minutos) e **Destaque de canal** (a mensagem é o nome do canal de destino, como `outrocanal` ou `$user`). As três formas novas usam as rotas oficiais da API da Twitch — anúncio, envio com fixação e shoutout —, com o bot como remetente e a conta do canal como alternativa quando o bot não puder. Nas demais plataformas a mensagem sai como mensagem comum e o Histórico avisa. A prévia registra a forma escolhida, por exemplo `[Simulação] [Anúncio] ...`, antes de publicar.

Nenhuma função antiga mudou de nome nem de comportamento: permissões, intervalos, contador, variáveis, prévia, editor visual, presets e simulação continuam iguais, e um fluxo salvo antes desta versão continua válido com os padrões **Mensagem normal** e nenhum áudio.

**Permissões novas.** **Autorizar conta do bot** e **Autorizar conta do canal** agora pedem também `moderator:manage:announcements`, `moderator:manage:chat_messages` e `moderator:manage:shoutouts`. Autorizações antigas não têm esses escopos: as três formas novas só funcionam depois de autorizar de novo e salvar o perfil. **Mensagem normal** não foi afetada.

## Como usar

1. Abra **Comandos → Novo comando**, **Timers → Novo timer** ou **Automações → Editar**.
2. Em **Tocar áudio ao disparar**, escolha **Escolher som** e importe o arquivo, ou selecione um som já cadastrado em **Respostas e sons**. Ajuste o volume e clique em **Testar som**.
3. Em **Como enviar na Twitch**, escolha a forma. Em **Anúncio**, escolha também a **Cor do anúncio**. Em **Destaque de canal**, escreva na resposta apenas o nome do canal de destino.
4. Salve, rode a simulação e confira no **Histórico** as linhas `[Simulação] [Anúncio] ...` e `[Simulação] Tocaria o áudio do fluxo`.
5. Para publicar de verdade, autorize as duas contas novamente, confira que o bot é moderador do canal e conecte o perfil.

Receita do exemplo: comando **!ifood** com **Contar usos deste comando** ativo, resposta `O ifood já passou a milhão na rua {{commandCount}} vezes!`, um som de caixa em **Tocar áudio ao disparar** e **Como enviar na Twitch** em **Anúncio**.

Capítulos atualizados: **Comandos e automações** (as duas seções novas), **Timers e contadores**, **Respostas e sons**, **Perfis, contas e conexões**, **Solução de problemas**, **Exemplos práticos** e **Matriz de aceite**.

## Validação

Executado localmente, antes e depois da preparação desta versão: `npm run check` (TypeScript), `npm test` (11 testes Vitest), `npm run test:updates` (7 testes), `npx playwright test` (12 testes), `cargo test --locked --lib -j 1` (56 testes Rust), `npm run build` e `npm run docs` (manual de 21 capítulos). `npm run update:check` confere as versões e os registros.

Dois testes novos cobrem as funções. No Rust, `model::tests::delivery_type_color_and_flow_audio_rules` valida as quatro formas de envio, as cinco cores, o volume entre 0% e 100%, o áudio com espaços rejeitado e a exigência de uma ação de mensagem para o destaque. `integration_tests::flow_audio_and_twitch_delivery_type` confere que o salvamento recusa áudio fora da biblioteca e cor inválida, que o evento `viewer-sound` sai com o áudio e o volume escolhidos, que a ação sem autorização registra o erro no Histórico sem derrubar o processo, que a simulação não emite áudio e que a prévia registra `[Simulação] [Anúncio]`.

Em Playwright, o teste **comando escolhe como enviar na Twitch e oferece o áudio do disparo** cria um comando, troca a forma para Anúncio, escolhe a cor, confirma os controles de áudio com **Testar som** e **Escolher som** indisponíveis fora do desktop, verifica ausência de rolagem horizontal em 720x800, salva, reabre o fluxo no editor com as escolhas preservadas e salva o fluxo. A interface foi revisada nas capturas `artifacts/command-send-audio.png` e `artifacts/flow-send-audio.png`.

## Limitações

Anúncio, mensagem fixada e destaque existem só na Twitch. Nas demais plataformas a mensagem sai como mensagem comum, com aviso no Histórico, e o campo fica desabilitado para não criar expectativa.

As três formas novas dependem de permissões que autorizações antigas não têm. O bot precisa ser moderador do canal: sem isso a Twitch responde 403 e o Histórico traduz como "a conta precisa ser moderadora do canal". Mensagem fixada usa a conta do bot como remetente, sem alternativa; se o bot não for moderador, o envio falha com esse mesmo aviso.

Destaque de canal só funciona com o canal ao vivo, pede um login válido como destino e respeita os limites de horários da Twitch. Nesse modo a mensagem não é publicada, porque o próprio destaque é a ação.

O áudio sai pela saída do aplicativo: sem capturar essa saída no OBS a live não ouve o som. A prévia web mostra a escolha, mas importar e testar arquivos fica no aplicativo desktop. A validação visual foi feita em capturas 1440x1000 e 720x800, sem rolagem horizontal; a conferência manual no aplicativo desktop não foi executada.

---

# BotLive 0.1.11

## O que mudou

O painel **Inserir variável e testar mensagem** foi reconstruído. Três defeitos o tornavam imprestável: o texto vazava das fichas, clicar não parecia fazer nada e o painel não deixava claro o que ia parar no chat.

O transbordamento tinha uma causa única e escondida. A regra global de botões aplica `height: var(--h-md)` e `white-space: nowrap` a qualquer botão fora de uma lista de exceções, e a especificidade dessa regra (nove classes) é maior do que a do catálogo, que pedia `white-space: normal`. Toda ficha ficava presa a uma linha só: rótulos e exemplos longos atravessavam a caixa vizinha e o catálogo ganhava uma barra de rolagem horizontal. As fichas agora se chamam `.variable-card`, com um botão principal e um rodapé, e `.variable-card-btn` entrou na lista de exceções da regra global. Dentro da caixa, o nome é cortado em duas linhas e o exemplo em uma, ambos com reticências; o texto completo aparece ao passar o mouse. Todas as fichas têm a mesma altura, com a linha de exemplo reservada mesmo quando não há exemplo.

O clique que não fazia nada era um problema de posicionamento, não de função. A ficha apenas registrava a escolha e o formulário com **Se não houver valor, mostrar**, **Como mostrar** e **Inserir na mensagem** era renderizado depois do catálogo e do bloco **Usar uma variável personalizada**, fora da área visível do diálogo. O clique na ficha agora insere o marcador na hora, no cursor, e uma confirmação logo abaixo do catálogo mostra o código que entrou. As opções passaram a ser abertas pelo botão **Opções** do rodapé da ficha e aparecem dentro do catálogo, fixas no topo, com a rolagem da página e o foco do teclado movidos automaticamente — inclusive quando vêm de **Escolher este valor**, na variável personalizada.

O painel também ficou mais curto e legível: busca e categoria em duas colunas na mesma linha, **Mostrar códigos técnicos** e **Atualizar minhas variáveis** na mesma barra, e o quadro **Como será enviado ao chat** mostra a mensagem com etiquetas logo ao abrir, antes de qualquer clique, respondendo na tela o que a ferramenta vai publicar. Cada ficha passou a ter um formato só: nome, exemplo, a categoria em etiqueta e o botão **Opções**.

Nenhuma função foi removida: busca, filtro por categoria, códigos técnicos, variáveis salvas com a atualização, variável personalizada, texto alternativo, todos os formatos, a seção **Testar como a mensagem vai ficar** com o aviso de timer, a exibição de erros e a inserção no cursor ou no trecho selecionado continuam com os mesmos rótulos, então os fluxos já cobertos por teste continuam valendo.

## Como usar

Abra **Inserir variável e testar mensagem** e clique na ficha da informação: o marcador entra na mensagem na hora e a confirmação traz o código inserido, por exemplo `{{arg0}}`. O quadro **Como será enviado ao chat**, no topo do painel, acompanha a mensagem enquanto você trabalha; os nomes das informações viram etiquetas e os valores reais só entram no envio.

Quando o valor puder faltar, ou precisar de outro formato, clique em **Opções** no rodapé da mesma ficha. Preencha **Se não houver valor, mostrar** com o texto de reserva, escolha **Como mostrar** e clique em **Inserir na mensagem**. O painel de opções fica fixo no topo do catálogo, então nunca aparece fora da tela. Em **Mostrar códigos técnicos**, cada ficha mostra o marcador completo, como `{{local.aiResponse}}`.

Os capítulos atualizados foram **Variáveis**, com os passos do clique direto, do **Opções** e o corte de texto das fichas, e **Comandos e automações**, com o mesmo caminho resumido.

## Validação

Executado localmente: `npm run check` (TypeScript), `npm test` (11 testes Vitest), `npm run test:updates` (7 testes), `npx playwright test` (11 testes), `cargo test --locked --lib -j 1` (54 testes Rust), `npm run build` e `npm run docs` (manual de 21 capítulos), com o Playwright reexecutado depois da geração do manual. `npm run update:check` confere as versões e os registros antes do commit.

Dois testes de `tests/ui/app.spec.ts` cobrem a reconstrução. O teste **catálogo de variáveis insere marcador e informa limite da prévia web** agora verifica o clique direto (entra `{{arg0}}` e a confirmação aparece) e o caminho de opções, com asserções de que o painel está visível na janela e contido na caixa do catálogo, antes de conferir `{{arg0|default:amigo|upper}}`. O teste novo **fichas do catálogo têm tamanho padrão, nada vaza da caixa e o clique insere na hora** abre o catálogo inteiro, com mais de 25 fichas, e verifica que nenhuma ficha e nenhum filho estoura a caixa (exceto os elementos cortados por reticências, que são cortados de propósito), que a altura de todas as fichas varia no máximo um pixel, que o catálogo não tem rolagem horizontal e que o clique insere `{{randomViewer}}` com a leitura atualizada no quadro de envio.

## Limitações

O texto que não cabe é cortado com reticências e o completo só aparece ao passar o mouse; não há abertura da ficha para leitura integral, o que pesa em leitores de tela que não disparam `title`.

O quadro **Como será enviado ao chat** é leitura com etiquetas, não execução: ele mostra a composição da mensagem, e os valores resolvidos continuam vindo de **Conferir variáveis** dentro de **Testar como a mensagem vai ficar**, botão que segue desabilitado fora do aplicativo desktop. A validação de interface foi feita pela captura automatizada em 1440x1000 e pela janela estreita de 720x800, sem rolagem horizontal; a conferência manual no aplicativo desktop não foi executada.

A confirmação de inserção fica até o painel ser fechado ou outra ficha ser escolhida: se a mensagem for editada à mão logo depois, o aviso continua mostrando o código anterior, sem acompanhar a edição.

---

# BotLive 0.1.10

## O que mudou

A prévia de variáveis deixou de mostrar um resultado que a produção não entrega. O editor enviava sempre o mesmo evento de teste — `kind:'chat'`, nome "Ana" e ID "test-user" — e o núcleo o usava como estava, sem olhar o gatilho do fluxo. Um timer conferido na prévia mostrava `{{user}}` como Ana e `{{userId}}` como `test-user`, enquanto o disparo real monta `kind:'timer'`, remetente `BotLive`, papel `broadcaster`, ID vazio e mensagem vazia. Quem testava antes da live recebia exatamente o oposto do que seria publicado. O endpoint `variables.preview` agora passa o evento por `model::preview_event`, que entrega o que o gatilho produz: para **Timer periódico**, o mesmo construtor de `timers.rs`, incluindo `timerId` e `timerRevision` em `data`; para **Comando de voz**, a voz local do streamer (`local-streamer`); para **Comando de chat** e **Mensagem contém**, um evento de chat cuja mensagem começa pelo texto que dispara, para `{{command}}`, `{{rawInput}}` e `{{argN}}` calcularem sobre o comando de verdade; para os demais gatilhos, o tipo de evento escolhido, como `follow` ou `redemption`. Os campos de teste continuam valendo nos gatilhos em que a produção não os determina.

O botão **Simular timer** refez o evento à mão em um terceiro lugar e ficava sem `timerRevision`; ele e a prévia agora usam a mesma construção, `timers::event`.

Os campos de teste de mensagem, nome, ID e JSON deixaram de aparecer quando o gatilho é timer, porque aquele evento não tem esses valores: no lugar entra uma explicação do que a prévia está usando. Para isso, o diálogo de comando, o de timer e o editor visual passaram a mandar o fluxo aberto para o editor de mensagem — antes só o editor visual fazia isso. A prévia do editor simples calcula, portanto, as ações na ordem, as condições e as variáveis locais do fluxo, como o editor visual já fazia.

Também entrou `{{randomViewer}}`, um marcador que sorteia um nome entre quem já falou no chat do perfil. Ele existe justamente para mensagens que não têm pessoa associada, como o timer que quer citar um espectador. A fonte é o mesmo cadastro que os comandos de pontos, fila e transferência já usam (`community.names`, gravado a cada mensagem recebida no SQLite e apagado junto com o perfil), exposto por `modules::random_chatter`. O sorteio acontece uma vez no início de cada execução, então todas as ações do mesmo disparo citam a mesma pessoa e o próximo disparo escolhe outra. Quando ninguém falou ainda, o marcador é variável ausente e interrompe a ação com a mensagem no Histórico, em vez de publicar um texto com um buraco no meio — é a regra já documentada para variáveis ausentes.

## Como usar

Para um timer que cita alguém, escreva na resposta `Minecraft — {{randomViewer|default:alguém}}, quer jogar com a gente? Nosso servidor funciona 24/7.` O `|default:alguém` cobre os primeiros minutos da live, quando ainda não há ninguém no cadastro; sem ele, a ação é interrompida e o motivo aparece no Histórico. O marcador está no catálogo, categoria **Pessoa**, com o rótulo **Nome sorteado no chat**. Em mensagens disparadas por uma pessoa, continue usando `{{user}}`, que é o nome de quem falou.

Para conferir antes da live, abra o timer ou o comando, expanda **Inserir variável e testar mensagem**, abra **Testar como a mensagem vai ficar** e clique em **Conferir variáveis** no aplicativo desktop. Num timer não há campos de teste: o aviso explica que a prévia usa o evento do disparo real, e o resultado mostra `BotLive` em `{{user}}`, vazio em `{{userId}}` e `timer` em `{{eventType}}`. Num comando, a mensagem de teste passa a começar pelo comando do fluxo, e o resultado é apresentado como a lista de ações na ordem, igual ao editor visual.

Os capítulos atualizados foram **Variáveis**, com o catálogo e a receita completa, **Timers e contadores**, com o comportamento do gatilho, e **Exemplos de uso**, com a linha pronta do timer Minecraft.

## Validação

Executado localmente: `npm run check` (TypeScript), `npm test` (11 testes Vitest), `npm run test:updates` (7 testes), `npx playwright test` (10 testes), `cargo test --locked --lib -j 1` (54 testes Rust), `npm run build` e `npm run docs` (manual de 21 capítulos), com o Playwright reexecutado depois da geração do manual. `npm run update:check` confere as versões e os registros antes do commit.

Três testes novos cobrem a correção. No Rust, `model::tests::preview_event_follows_the_trigger` verifica o evento de timer (BotLive, sem ID, mensagem vazia, `simulated` preservado), de comando (`!minecraft` no lugar do `!oi` de teste), de contém, de voz e de seguidor. `integration_tests::timer_preview_delivers_the_real_event_and_sorts_a_chatter` verifica o fim a fim: com o cadastro vazio, `{{randomViewer}}` é ausente e `{{randomViewer|default:alguém}}` publica "alguém"; depois de uma mensagem recebida, o sorteio devolve "Ana"; e a prévia do fluxo com os dados de teste de chat devolve `BotLive||timer`. No Vitest, `variableCatalog.test.ts` confirma que `{{randomViewer}}` é rotulado no catálogo. No Playwright, `tests/ui/timers.spec.ts` abre a prévia de um timer e confere que a explicação do evento real está visível, que os campos de teste não existem e que **Conferir variáveis** continua desabilitado na versão web.

## Limitações

O cadastro é de quem **escreveu** no chat: quem só assiste sem mandar mensagem não entra nele, e mensagens simuladas também não entram. O sorteio é entre todos os nomes do cadastro do perfil, sem limite de idade nem peso — quem falou há três lives tem a mesma chance de quem falou agora — e não há garantia de não repetir a mesma pessoa em disparos consecutivos. Apagar o perfil apaga o cadastro junto, porque a tabela é apagada em cascata.

O vazio de `{{userId}}` num timer continua não interrompendo a ação: a variável existe no contexto, só chega como string vazia, e é isso que a prévia agora mostra. Quem quiser falhar alto em mensagem automática deve usar `{{randomViewer}}`, que é ausente de verdade quando não há valor, ou informar `|default:`. Nenhuma mensagem real foi publicada em chat durante a validação: os testes de integração usam o motor com publicação desligada e a prévia não envia nada, e a conferência no aplicativo desktop não foi executada porque exige um perfil conectado.

A prévia continua sendo cálculo, não execução: scripts e serviços externos não rodam, e o resultado depende do estado atual do cadastro, então o nome sorteado muda a cada conferência. O botão **Conferir variáveis** segue desabilitado fora do aplicativo desktop, e o teste automatizado cobre a interface nesse estado, não o cálculo do motor no navegador.

---

# BotLive 0.1.9

## O que mudou

O box do logo na barra lateral deixou de ser espremido. `.brand-mark` declarava 35×35, mas era item de flex sem `flex-shrink`, e a largura colapsava para 25px para caber "botlive." junto com "DESKTOP". O box agora é 35×35 (34×34 abaixo de 920px) com o canto arredondado preservado; o rótulo "DESKTOP" deixa de aparecer abaixo de 1150px, faixa em que a barra cai para 205px e não comporta os três elementos.

A atualização pelo instalador deixou de tirar o aplicativo da barra de tarefas. O instalador padrão da Tauri desinstala a versão anterior sempre que o `.exe` é rodado por cima dela: a página de reinstalação marca "desinstalar antes de instalar" como primeira opção, e a desinstalação apaga `botlive.exe`, os atalhos do Menu Iniciar e da Área de Trabalho e chama `UnpinShortcut` (`IStartMenuPinnedList::RemoveFromList`). A instalação seguinte só recria os atalhos do Menu Iniciar e da Área de Trabalho, nunca a fixação na taskbar. O BotLive passa a usar um template NSIS próprio, `src-tauri/installer.nsi` derivado da tauri-bundler 2.11.5, em que a opção padrão durante uma atualização é "Não desinstalar": o executável é sobrescrito no lugar e os atalhos existentes apenas têm o alvo atualizado. O caminho silencioso e automático (flag `/P`, atualizador interno) segue exatamente o comportamento da Tauri. Um hook de pós-instalação, `src-tauri/installer-hooks.nsh`, executa `ie4uinit.exe -show` para regenerar o cache de ícones do Explorer.

Sprint A de geometria. `:root` recebeu a escala `--s1..--s7` (4/8/12/16/24/32/48px), `--r-sm/md/lg/pill` (6/8/12/999px), `--h-sm/md/lg` (32/40/48px) e `--t-meta..--t-display`. Ações e campos passaram a ter altura fixa de 40px, com `white-space:nowrap` para o rótulo nunca quebrar em duas linhas; `.icon-button` ficou 32×32; o ícone de botão passou a 16px; `.row` ganhou `gap:8px` e `flex-wrap:wrap`. Raios e gaps de grid foram unificados: `.card` em 12px e 24px de padding, `.metrics`, `.two-columns`, `.dashboard-columns`, `.form-grid` e `.list-row` em 16px.

As duas listas do painel **Respostas e sons** deixaram de ser um cartão por item. `ChatExtras.tsx` renderizava `<section class="card">` dentro de um `<Card>` — moldura dentro de moldura, 24px de padding e 24px de margem, sempre totalmente expandido —, o que custava 510px por regra de TXT, 719px por pessoa e deixava 72px de espaço morto entre o conteúdo de um item e do próximo. Cada regra e cada pessoa agora viram uma linha-resumo de 56px com ponto de situação, nome, pastilhas de metadados (reconhecimento, seleção, arquivo e intervalo), seta e controle de ativar; o formulário abre no mesmo lugar por `grid-template-rows` de `0fr` a `1fr`, sem medição em JavaScript. O conteúdo fechado recebe `inert`, sai da ordem de tabulação do teclado e a linha declara `aria-expanded` e `aria-controls`. Uma barra acima da lista traz o contador, **Expandir todos**, **Recolher todos** e um campo **Filtrar** que só aparece acima de cinco itens. Sem margem entre itens, a separação passou a ser feita por `border-top`, e a linha aberta ganha um filete de destaque na cor de destaque. A regra ou a pessoa recém-adicionada já abre sozinha. O padrão é genérico — `ItemStack`, `AccordionItem` e `ListTools` saíram de `components.tsx` — e ficou pronto para outras listas.

## Como usar

Nenhum fluxo mudou para usar o BotLive. O logo quadrado aparece na barra lateral em qualquer largura de janela. Para atualizar, use Configurações → Verificar atualização → Instalar atualização, ou baixe o instalador da release e rode por cima da versão instalada; nas duas formas o app continua fixado na barra de tarefas. Quem quiser mesmo assim apagar tudo antes de instalar ainda pode escolher a segunda opção na página de reinstalação, e a desinstalação manual continua removendo a fixação.

Em **Respostas e sons**, cada regra e cada pessoa aparece como uma linha com o nome do cadastro, as pastilhas de resumo, o controle de ativar e uma seta. Clique no título da linha para abrir ou fechar o formulário, ou use **Expandir todos** e **Recolher todos** para fazer o mesmo com a lista inteira. O contador ao lado informa quantos cadastros existem; com mais de cinco, o campo **Filtrar** localiza um item pelo nome, pela palavra-chave ou pelo arquivo. Uma regra ou pessoa nova já abre sozinha. O comportamento das regras, dos sons e do salvamento não mudou: as alterações continuam valendo só depois de **Salvar respostas e sons**.

## Validação

Executado localmente: `npm run check` (TypeScript), `npm test` (10 testes Vitest), `npm run test:updates` (7 testes), `npx playwright test` (10 testes), `cargo test --locked --lib -j 1` (52 testes Rust) e `npm run build`. Medições na prévia na porta 1421: ações de botão todas em 40px (antes 37, 38, 39, 40 e 52), campos `input` e `select` todos em 40px (antes `select` em 40 e 42, `input` em 27, 37 e 40), diferença de altura entre `select` e `input` na mesma linha igual a zero em todos os pares medidos, `.icon-button` em 32px, logo em 35×35 em 1440, 1280 e 1100px e 34×34 em 900 e 760px, e overflow horizontal de 0px a 390px em Visão geral, Discord, Comandos, Comunidade e Configurações, contra 147px no Discord antes da correção.

Listas, com três regras e três pessoas na mesma tela: custo por item fechado de 56px (antes 510px por regra e 719px por pessoa), espaço entre itens de 0px (antes 24px de margem mais 48px de padding), 1.715px → 425px na lista de regras e 2.751px → 834px na de pessoas com tudo recolhido, e altura do documento de 5.082px → 3.282px. A mesma tela medida em 1440px e 390px: overflow horizontal 0, nenhum elemento para fora da janela, nenhum botão sem nome acessível e nenhum alvo abaixo de 30px; nas linhas fechadas `aria-expanded="false"` e `inert` presentes, e o foco na linha mede 953×40px em 1440px e 197×40px em 390px. `npm run docs` gerou o manual de 21 capítulos e o Playwright foi reexecutado em seguida, ainda com 10 testes passando; o teste do painel agora também cobre recolher (linha abaixo de 120px, `inert` presente e `aria-expanded="false"`), expandir (linha acima de 300px, `inert` removido e o campo visível de novo) e remover pessoa. O `toBeHidden` do Playwright não serve para esse caso: ele mede a caixa do próprio elemento e ignora o corte por `overflow` do ancestral. A asserção útil é a altura da linha, medida com `expect.poll` porque a transição de `grid-template-rows` leva 220ms enquanto os atributos já mudaram no primeiro render.

## Limitações

O template NSIS próprio é uma cópia da `installer.nsi` da tauri-bundler 2.11.5 e precisa ser baixado de novo e ter os patches reaplicados quando o `@tauri-apps/cli` mudar; o procedimento está escrito no cabeçalho do próprio arquivo e os trechos alterados estão marcados como "BotLive patch". A correção da barra de tarefas foi verificada pela leitura do script NSIS gerado e do código do instalador, não por uma atualização executada e conferida no Windows nesta máquina; não há homologação de instalação real. A desinstalação manual continua removendo a fixação na taskbar, o que é o comportamento esperado de uma desinstalação. A correção depende de o usuário não escolher explicitamente a opção de desinstalar.

O accordion foi aplicado às duas listas que tinham o problema de altura — Respostas por TXT e Sons por espectador; as demais listas do aplicativo já usavam linhas compactas (`.list-row`) e não foram tocadas. Os controles de ativar dentro das linhas continuam com 33×19px, abaixo de 30px na altura: é o achado P1 de alvos pequenos já conhecido, não introduzido aqui. A busca aparece somente acima de cinco itens e filtra pelo texto da própria linha; não há ordenação nem arrastar-e-soltar. Continuam pendentes o restante do plano de refinamento: posição do toast sobrepondo campos, overflow da barra lateral, redistribuição das colunas de Configurações, redução de 20 para 7 tamanhos tipográficos, contraste AA de rótulos secundários e o plano de testes de layout automatizado.

---

# BotLive 0.1.8

## O que mudou

Esta versão conecta o BotLive diretamente ao Discord, sem nenhuma dependência de terceiros como Loritta ou pontes de webhook. O bot usa os dois caminhos oficiais: o Gateway (`wss://gateway.discord.gg/?v=10`) para receber eventos e a API REST (`https://discord.com/api/v10`) para executar ações, sempre com o token de um bot criado pelo usuário no portal do desenvolvedor do Discord.

**Tela nova.** `src/Discord.tsx` abre em **SEU ESPAÇO** com navegação, subtítulo e roteamento próprios em `src/App.tsx`. A tela tem nove blocos: conexão e token, servidor/canais/cargos, entrada saída e espelho, moderação em dupla, ranking de XP, sorteios, aniversários, identidade unificada, comandos slash, moderação no servidor e auditoria com exportação e desfazer. A prévia de navegador recebe dados de demonstração pelos novos casos de `discord.*` e `secret.save` em `src/api.ts`, então a tela abre mesmo sem aplicativo desktop.

**Núcleo Rust.** Três módulos novos em `src-tauri/src`: `discord.rs` (conexão Gateway, heartbeat e resume, REST com retentativa em 429, cache de servidores canais e cargos, envio de mensagens, espelho de chat e contador de membros), `discord_admin.rs` (roteador único `discord.action`, punição nos dois lados, auditoria única com `platforms`, desfazer e exportação CSV) e `discord_engage.rs` (XP e nível, aniversários, sorteios, vínculo de identidade e catálogo de comandos slash). `lib.rs` registra os módulos, delega o namespace `discord.*`, guarda o token no cofre com `discord_token` e valida a chave `discordBot` em `module.config`. A chave antiga `discord`, usada pelo convite `!discord`, continua intacta.

**Funções entregues.** Moderação completa (aviso, timeout, expulsão, banimento, desbanimento, apagar mensagem, cargo e modo lento) com auditoria única e desfazer; entrada e saída com cargo automático e contador de membros por renomeação de canal; auto-moderação reaproveitando a lista de palavras bloqueadas, o bloqueio de links e o anti-repetição já usados na Twitch; logs de auditoria em canal; sorteios pela reação 🎉 com encerramento e re sorteio; XP e nível por mensagem com ranking e quanto falta para o próximo nível; mensagens de aniversário com idade calculada; identidade unificada Twitch e Discord pelo código de seis dígitos de `!vincular`; doze comandos slash próprios; espelho de chat nos dois sentidos com interruptores separados; e notificações da Twitch para o Discord.

**Ganchos e testes.** `engine.rs` passou a consultar o comando de vínculo antes do espelho e `scheduler.rs` roda o ciclo do Discord a cada volta do loop. Foram criados `tests/ui/discord.spec.ts` e a cobertura da nova navegação em `tests/ui/app.spec.ts`. Os módulos Rust têm testes de unidade para contador de memória, curva de nível, comandos slash, permissões, espelho e retenção de auditoria.

**Documentação.** Novo capítulo `docs/09-DISCORD.md` com requisitos, conexão, campo a campo, receitas e limitações; o manual offline passou de 20 para 21 capítulos e continua com quatro grupos. `docs/05-COMUNIDADE.md`, `docs/INTEGRACOES.md`, `docs/08-SOLUCAO-DE-PROBLEMAS.md`, `docs/EXEMPLOS-DE-USO.md` e `docs/README.md` apontam para o capítulo novo, e `docs/MATRIZ-DE-ACEITE.md` ganhou a linha "Discord nativo".

## Como usar

1. Crie um aplicativo no portal do desenvolvedor do Discord e copie o token do bot. Ative **Server Members Intent** e **Message Content Intent** na página do aplicativo.
2. Abra **Discord** no perfil desejado, cole o token em **Token do bot** e clique em **Salvar token**. O token vai para o cofre do sistema e não volta para a tela.
3. Clique em **Descobrir servidor e canais**, escolha o **Servidor do Discord** e os canais de logs, boas-vindas, despedida, espelho e notificações, e escolha os cargos do contador, do mapa de permissões e do cargo automático.
4. Ligue **Ativar o bot do Discord** e clique em **Salvar configuração**. Use **Convite do bot** para autorizar o bot no servidor, se ainda não tiver convidado.
5. Clique em **Conectar**. O rótulo deve passar a **Conectado**; motivos de falha aparecem no **Histórico** com a categoria `discord`.
6. Ative **Auto-moderação do Discord** para reusar as regras da Twitch, ative **Espelho do chat** com o canal escolhido e **Notificações da Twitch** para seguidores, inscrições, bits e raids.
7. Em **Moderação no servidor**, busque o membro, informe o motivo e use **Timeout**, **Expulsar**, **Banir**, **Desbanir**, **Avisar nos dois chats** ou **Ver avisos**. Cada ação entra em **Auditoria única**, com **Desfazer** quando a API do Discord tem reversão e **Exportar CSV** para conferência.
8. Para unificar as identidades, gere um código em **Identidade unificada Twitch e Discord** e peça para digitar `!vincular CODIGO` no chat da Twitch. Depois disso, aviso e punição valem nas duas casas.
9. Clique em **Registrar comandos slash** depois de conectar para publicar `/painel`, `/convite`, `/ranking`, `/xp`, `/avisos`, `/vincular`, `/aniversario`, `/limpar`, `/ban`, `/mute`, `/warn` e `/sorteio` no servidor.

O passo a passo completo, campo a campo, está em `docs/09-DISCORD.md` e no capítulo **Discord** do manual offline.

## Validação

Executados nesta revisão, todos aprovados:

- `npm run check` (compilação TypeScript com `tsc -b`) sem erros.
- `npm test` (Vitest): 10 testes em 3 arquivos.
- `npm run test:updates` (Node): 7 testes dos scripts de atualização e do manifesto de release.
- `npx playwright test` (Edge, 10 cenários): inclui `tests/ui/discord.spec.ts`, que cria um perfil, abre a tela Discord, salva token, descobre servidor e canais, escolhe guild e canais, liga sete interruptores, salva, recarrega e confirma a persistência, busca um membro, registra aviso, abre o ranking, os aniversários, os sorteios e a auditoria, desfazer um registro, gera um código de vínculo e registra os comandos slash, sempre sem erros de página. Também foram atualizados `tests/ui/app.spec.ts` (navegação inclui **Discord**, com escopo na barra lateral para não colidir com o módulo de webhook) e `tests/ui/manual.spec.ts` (21 capítulos, 4 grupos).
- `npm run docs`: manual regenerado com 21 capítulos e 386 KiB; os links internos são validados pelo teste do manual.
- `cargo test --manifest-path src-tauri/Cargo.toml --locked --lib -j 1`: 52 testes, 0 falhas.

O que **não** foi executado: nenhuma conexão real com um servidor Discord. A validação acima cobre compilação, regras do núcleo e a interface, não a homologação com o Gateway e a API em produção.

## Limitações

- **Sem homologação com conta Discord real nesta versão.** O Gateway e a API REST foram implementados e cobertos por testes de regra, mas o primeiro contato com um servidor de verdade depende do token e do servidor de quem for usar. Esse é o principal item pendente, registrado em `docs/MATRIZ-DE-ACEITE.md`.
- A prévia de navegador não abre conexões: ela mostra a tela com dados de demonstração. Gateway e API só funcionam no aplicativo desktop.
- O token é de bot criado no portal do desenvolvedor. Conta pessoal (selfbot) viola os termos do Discord e não é suportado.
- As intents de membros e de conteúdo de mensagens são privilegiadas e precisam ser ligadas manualmente na página do aplicativo; sem elas o contador de membros não acompanha o total e o bot não lê o texto para moderar.
- Limites de requisição do Discord valem. Em 429 o BotLive espera o tempo indicado e tenta de novo até três vezes por chamada.
- O espelho corta mensagem em 2000 caracteres, o limite do Discord; não há tradução nem formatação especial.
- O contador de membros depende de um canal criado só para isso e de um rótulo com letras, números e hífen; canais apagados geram erro no Histórico sem derrubar o resto.
- Sorteio no Discord é por reação 🎉; peso de assinantes e bilhetes, existentes na Twitch, não existem aqui.
- A moderação nativa continua pendente para Kick e YouTube; o webhook do Discord e o convite `!discord` continuam existindo em **Comunidade** como funções separadas.

---

# BotLive 0.1.7

## O que mudou

- O grupo Seu espaço passa a separar as tarefas em telas próprias: Comandos, Timers, Respostas e sons, Contadores e Automações, cada uma com título, resumo, busca e ações próprios na barra lateral e na paleta Ctrl+K.
- A tela Comandos deixou de usar as abas "Comandos e contadores" e "Timers". Ela lista apenas comandos e fluxos de evento, mantém Resenha com IA, Variáveis, Simular evento, a coluna Contador e o atalho para Automações.
- Novo painel Timers: lista os lembretes periódicos do perfil com busca, ativação, simulação sem publicação, atalho para presets, edição e exclusão, além de estado vazio com atalho para criar o primeiro timer.
- Novo painel Contadores: reúne o total de cada comando com contagem ativa, soma dos usos do perfil, busca, ajuste do total e atalho para abrir o comando correspondente.
- O conteúdo de Respostas TXT e sons deixou de abrir em modal e virou a tela Respostas e sons. O botão em Comunidade virou um atalho para essa tela, eliminando a configuração duplicada que existia nos dois lugares.
- Diálogos de criação e edição de fluxo, simulação de evento, ajuste de contador, ativação e ações de linha foram extraídos para src/FlowTools.tsx e agora são compartilhados pelas telas novas e existentes, em vez de duplicados dentro de Commands.tsx.
- A barra lateral ganhou quatro entradas sem corte: a navegação já comporta rolagem própria, e os títulos, resumos e itens da paleta acompanharam a nova estrutura.
- Documentação ajustada em Guia de uso, Comandos e automações, Timers e contadores, Respostas e sons, Comunidade, Exemplos práticos, Variáveis e Arquitetura; o manual offline foi regerado.

## Como usar

Abra Seu espaço no menu lateral. Para um lembrete, clique em Timers e depois em Novo timer: nome Água, resposta Hora de beber água!, Repetir a cada (segundos) 900, Salvar timer e mantenha o controle de Ativar ligado. Conecte o perfil e aguarde o intervalo completo para o primeiro envio real.

Para contar usos, abra Comandos, crie ou edite um comando, ative Contar usos deste comando e use + Contagem do comando na resposta. Os totais aparecem em Contadores, onde Ajustar define um novo total ou zero para reiniciar. O mesmo valor continua visível na coluna Contador da lista de comandos.

Para respostas por palavra ou som por espectador, abra Respostas e sons no menu lateral ou o botão Respostas TXT e sons em Comunidade, configure as regras e clique em Salvar respostas e sons. As demais telas permanecem nos mesmos grupos de antes: IA e Memórias em Personalidade e memória; Comunidade, Presets, Histórico e Estatísticas em Ferramentas.

## Validação

Executados nesta revisão: compilação TypeScript (`npm run check`), dez testes Vitest, sete testes dos scripts de atualização (`npm run test:updates`) e nove cenários Playwright no Edge, todos aprovados. O manual offline foi regerado pelo gerador, que validou os vinte capítulos e os links internos.

Os testes de interface foram adaptados à nova navegação: o painel de respostas agora é aberto pela entrada do menu lateral, o fluxo de timer parte da tela Timers e o comando com contador é criado direto em Comandos. O cenário de timers também passou a abrir Contadores e conferir que o comando aparece listado, com captura própria em artifacts.

Nenhum arquivo Rust foi alterado nesta revisão, portanto a suíte de testes Rust não foi reexecutada. Verificou-se ainda, em execução pontual da interface, que apenas um item da barra lateral permanece em estado ativo por vez.

## Limitações

A alteração é de interface: motor, banco e formatos gravados não mudaram, e timers, comandos e contadores existentes continuam válidos sem migração. Timers seguem exigindo aplicativo aberto e perfil conectado, e contadores continuam medindo usos aceitos antes das ações, não entregas confirmadas.

Ajuste de contador e vinculação de arquivos continuam disponíveis somente no aplicativo desktop; na prévia de navegador o total é exibido para leitura. Não houve nesta revisão execução de testes Rust, compilação do pacote Windows, instalação sobre a versão anterior nem homologação com contas reais de Twitch, YouTube ou Kick. O registro detalhado está em docs/VALIDACAO.md.

---

# BotLive 0.1.6

# BotLive 0.1.6 — manual prático, timers e contadores individuais

## O que mudou

- Manual organizado em Comece aqui, Configure o bot, Cuide da sua live e Referência técnica. Vinte capítulos, sumários locais, atalhos por tarefa, busca por capítulo e botões para copiar exemplos. Âncoras antigas preservadas.
- Novo catálogo de receitas cobre comandos, timers, contadores, variáveis, IA contextual, memória, TXT, sons, saída de áudio, pontos, loja, sorteios, previsões, fila, música, jogos, voz, Discord, moderação, ações, overlay, presets, acesso, backup e atualização. Cada receita explica configuração, entrada e resultado esperado, distinguindo simulação e uso real.
- Aba Timers e formulário simples de mensagem periódica por perfil, com nome, texto, intervalo de 30 segundos a 24 horas, ativação e simulação. Disponível também no editor visual como Timer periódico.
- Agendador aguarda o intervalo completo, executa somente em perfil conectado, mantém períodos independentes, pula sobreposições e não acumula disparos perdidos. Pausa/desconexão/edição reiniciam a espera; eventos antigos de uma configuração editada são descartados. Eventos externos reais não podem forçar timers pelo mesmo gatilho.
- Contar usos deste comando ativa um total persistente individual. A variável commandCount e o atalho Contagem do comando inserem o valor nas respostas. A lista mostra o total e permite ajustar ou zerar com confirmação.
- Contagem transacional respeita permissão, cooldown e processamento anterior. Simulações mostram o próximo valor sem gravar. O total é incrementado antes das ações e sobrevive a falhas posteriores. Contadores não compartilham dados entre perfis ou comandos.
- Banco cria tabela de contadores com remoção em cascata; comandos antigos continuam com contador desativado. Presets carregam a opção e os intervalos, sem exportar totais.
- Barra de ações adaptada para janelas menores, sem rolagem horizontal causada pelos novos botões.

## Como usar

Abra o manual e comece por Exemplos práticos. Para um lembrete, Comandos → Novo timer: nome Água, resposta Hora de beber água!, intervalo 900. Salve, ative e conecte o perfil. Aguarde quinze minutos para o primeiro envio real. Simular timer registra a prévia no Histórico sem publicar. Pause ou desconecte ao terminar a live.

Para um contador: Novo comando, nome Mortes, gatilho !mortes. Ative Contar usos deste comando e escreva Mortes registradas: {{commandCount}}. Escolha Moderadores ou Só o streamer para limitar quem incrementa. Crie !vitorias da mesma forma para outro total. Na coluna Contador, Ajustar permite definir zero ou corrigir o valor; confirme o novo total. Cada uso aceito soma um. O bot não detecta mortes ou vitórias no jogo automaticamente.

Variáveis por pessoa não se aplicam a timers sem usuário. Use canal, data e horário. Configurações de timer também podem ser combinadas com ações no editor visual; o agendamento não depende de uma mensagem no chat.

## Validação

Compilação TypeScript/Vite aprovada. Executados dez testes Vitest, 36 testes Rust, nove testes Playwright e sete testes de scripts de atualização. Os testes novos cobrem concorrência/persistência/isolamento de contadores, permissões, cooldown, simulação, timer por destino, bloqueio offline/externo, descarte após edição e intervalos sem recuperação acumulada. Após ajustes finais, os testes Rust e os dois testes específicos de interface/manual foram repetidos e passaram.

Manual gerado com vinte capítulos; verificadas âncoras, grupos, busca, tabelas, impressão, tela de 390 pixels e aparência desktop. A suíte detectou inicialmente overflow de três pixels na barra em 720 pixels; corrigido e validado. A compilação inicialmente bloqueada pelo sandbox foi repetida com permissão e passou. Não restaram falhas de teste conhecidas.

## Limitações

Timers exigem aplicativo aberto e perfil online; online não equivale a transmissão ao vivo. O Kick por ponte externa não inicia timers nativos. O agendador verifica a cada segundo; filas, limites de envio e serviços podem atrasar execução. Mensagens já enviadas não são desfeitas e uma chamada externa iniciada pode terminar após pausa. Não há calendário, horário fixo, contagem regressiva ou requisito de número mínimo de mensagens.

Contadores medem usos aceitos antes das ações, não entregas confirmadas nem eventos do jogo. Simulação não altera o total. Ajuste pelo painel é permitido a acessos autorizados ao perfil; não há comando de chat para definir ou subtrair o total. Excluir o comando exclui seu contador. Ao substituir configuração via preset, um comando existente conserva o contador associado ao seu identificador.

Os testes locais não comprovam envio periódico com contas reais nem homologação sobre a instalação aberta do usuário. Funções que dependem de IA, Twitch, OBS, áudio e serviços externos mantêm as limitações documentadas. A build instalada só recebe as mudanças depois de atualizar e reabrir.

---

# BotLive 0.1.5

# BotLive 0.1.5 — atualização visível, entrada silenciosa e saída de áudio

## O que mudou

- A lateral e o rodapé exibem a versão completa do pacote, eliminando o texto fixo v0.1.
- Configurações mostra a versão instalada, andamento da consulta, resultado persistente, erros e notas da versão. Só habilita instalação após encontrar atualização. A consulta aguarda configurações carregadas e não afirma que instalou quando nenhuma atualização foi encontrada.
- Consulta do manifesto limitada a 30 segundos e download a dez minutos. Assinatura e canal oficial preservados.
- Sons por pessoa oferecem gatilhos separados: mensagem, entrada silenciosa ou ambos; frequência uma vez por sessão ou com intervalo. Configurações antigas assumem mensagem, sem ativar monitoramento automaticamente.
- Monitor opcional Twitch IRC recebe entradas e saídas, responde keepalive, reconecta e cancela junto com o perfil. Não duplica mensagens EventSub nem executa pontos ou automações gerais. Ignora a lista inicial e deduplica entradas observadas. Solicita chat:read na autorização do bot.
- Saída de áudio dos sons selecionável por perfil, com atualização da lista e aplicação antes da reprodução. Saída indisponível gera erro sem desviar silenciosamente o som. Ambiente sem suporte orienta o Mixer do Windows.
- Manual atualizado nos capítulos de conexões, atualização e respostas/sons.

## Como usar

Em Configurações, clique em Verificar atualização, leia o resultado e use Instalar atualização quando disponível. Faça isso fora da live; o instalador pode fechar o aplicativo. Se uma versão antiga não atualizar internamente, use o instalador oficial na mesma pasta, preservando os dados.

Abra Comandos ou Comunidade → Respostas TXT e sons. Cadastre a pessoa, seu login, apelido e áudio. Escolha Disparar som e Quando tocar. Para entrada silenciosa na Twitch, ative Monitorar entradas silenciosas, salve, autorize novamente a conta do bot e conecte o perfil. Confira Monitor de entradas ativo no Histórico. O login atual é necessário mesmo quando o ID está preenchido.

Escolha Dispositivo de saída dos sons, salve e use Testar som. Capture essa saída no OBS. A seleção não altera o texto para fala. Reiniciar sessão de sons libera uma pessoa cujo primeiro disparo já foi consumido.

## Validação

Executados localmente: compilação TypeScript/Vite; dez testes Vitest, incluindo ordem de seleção/reprodução e dispositivo removido; 32 testes Rust, incluindo parsing IRC, gatilhos e migração de configuração antiga; sete testes Playwright, incluindo consulta com erro, andamento, notas, versão sem atualização e controles de som; sete testes dos scripts de atualização/publicação. Compilação e testes Node inicialmente encontraram bloqueio de criação de processos no sandbox; repetidos com permissão, passaram. Nenhuma falha de teste permaneceu.

Confirmado que o aplicativo aberto no PC estava em 0.1.3 e que a release anterior 0.1.4 tinha manifesto e instaladores públicos acessíveis. A consulta na interface foi testada com transporte simulado; não foi realizada instalação sobre o aplicativo em execução.

## Limitações

Entrada silenciosa representa conexão ao chat, não audiência do vídeo. A Twitch pode atrasar ou omitir eventos. O monitor requer nova autorização chat:read e ainda precisa de homologação com conta Twitch real. No YouTube, use mensagem; outras plataformas dependem de uma ponte join.

Dispositivos disponíveis e seleção dependem do Windows/WebView e das permissões. Reprodução física e captura no OBS não foram homologadas nesta execução. Dispositivo salvo é local ao computador. Sons mantêm limite de quinze segundos, fila de cinco, intervalo geral de cinco segundos e exigem app aberto/desbloqueado. Falha de reprodução pode consumir a primeira participação; reinicie a sessão após corrigir.

A versão instalada só muda após concluir a instalação. O erro original do atualizador instalado não foi reproduzido diretamente; foram corrigidos ausência de feedback persistente, versão fixa, falta de limite de espera e confirmação indevida de instalação. As verificações locais não substituem um teste real de atualização e áudio no ambiente da live.

---

# BotLive 0.1.4

## O que mudou

- Novo painel Respostas TXT e sons em Comandos e Comunidade, com controles de ativação geral e individual.
- Respostas automáticas por palavra ou expressão inteira ou por trecho. Arquivos TXT UTF-8 continuam vinculados externamente e são relidos no disparo; escolha aleatória sem repetição imediata ou sequencial, variáveis e intervalos por regra/pessoa.
- Cadastro de espectadores por nome ou ID, apelido, áudio WAV/MP3/OGG, volume e modo primeira participação da sessão ou mensagens com intervalo. Reinício manual da sessão de sons por perfil.
- Áudios copiados para a pasta de dados, com fila de reprodução no aplicativo, intervalo global, limite de tamanho e duração. Teste individual sem depender de uma mensagem real.
- Simulação sem publicação, reprodução ou consumo dos intervalos reais. Regras isoladas por perfil, validação de arquivos e restrições de acesso para importação e configuração.
- Manual agora com 18 capítulos, incluindo configuração dos recursos, exemplo TXT, captura de áudio no OBS, backup e limitações de presença.

## Como usar

Abra Comandos ou Comunidade > Respostas TXT e sons no desktop. Adicione uma regra, preencha a palavra, vincule o TXT, escolha o modo e ative o controle geral. Para sons, adicione a pessoa com nome do chat ou ID, apelido e áudio, ajuste volume e modo e use Testar som. Ative o controle individual e geral e clique em Salvar respostas e sons. Para a live, capture o áudio do BotLive ou da saída do desktop no OBS. O arquivo examples/respostas-cafe.txt serve como modelo inicial.

## Validação

Build TypeScript/Vite aprovado, 30 testes Rust, nove testes Vitest e seis cenários Playwright passaram localmente. Cobertura inclui leitura UTF-8 e limites, arquivo editado/apagado, seleção sequencial/aleatória, cooldown, identidade por ID/nome, cópia de áudio, controles de ativação, simulação, isolamento e permissões. Player testado com áudio controlado em memória para fila, volume, timeout e falha. O primeiro teste de interface falhou por seletor de label do select; corrigido para nome acessível, com suíte completa aprovada. Sem mensagens enviadas a contas reais durante a validação.

## Limitações

Entrada silenciosa não é detectada pelos adaptadores atuais: usa-se a primeira mensagem; evento join depende de ponte autorizada. Reprodução exige aplicativo aberto e desbloqueado. Captura e mixagem no OBS precisam ser configuradas e homologadas pelo operador; não foram testadas numa transmissão real. TXT externo precisa de backup separado e caminho válido; áudios copiados ficam em media. Recursos não são transportados pelos presets. Remover cadastros não apaga arquivos originais ou cópias de mídia. A build Windows desta versão será gerada pelo workflow após o push.

---

# BotLive 0.1.3

## O que mudou

- IA agora recebe a mensagem atual, autor e até 12 falas recentes dos últimos cinco minutos no mesmo perfil, além da personalidade e das memórias recuperadas.
- Nova ação Gerar resposta da IA (variável), sem envio automático, com nome local personalizável e marcadores local.aiResponse e local.aiSuccess. Respostas não são reinterpretadas como comandos ou variáveis.
- Botão Resenha com IA cria uma automação pronta com gatilho por trecho, orientação de humor e intervalos de 60 segundos por fluxo e 120 por pessoa.
- Campos de mensagem incluem atalhos Resposta da IA e Mensagem do chat; o catálogo tem categoria IA e nomes legíveis.
- Teste contextual permite informar mensagem e conversa anterior e chamar o provedor sem enviar ao chat ou gravar memórias. Simulação gratuita usa um marcador explícito, sem chamar o provedor.
- Contexto isolado por perfil e execução, limitado em tamanho e idade; eventos do próprio bot são ignorados quando o ID está configurado. Manual atualizado com receita, variáveis, exemplos e limites.

## Como usar

Configure e salve o provedor e a personalidade em Inteligência artificial. Abra Comandos > Resenha com IA, escolha o trecho que dispara (exemplo: amassando), ajuste o humor e use Testar resposta contextual no desktop. Salve para criar a geração seguida do envio. No editor visual, reutilize a resposta nas próximas ações clicando em + Resposta da IA; também pode encaminhar para voz ou overlay. Para citar partidas ou acontecimentos anteriores, registre esse contexto nas memórias ou forneça falas reais do chat.

## Validação

Build TypeScript/Vite aprovado; sete testes Vitest e cinco cenários Playwright passaram. Os testes Rust cobrem 26 casos, incluindo provedor HTTP local controlado, contexto por perfil, expiração, limite de histórico, variável local, texto sem expansão recursiva, fallback, simulação, prévia e autorização. O primeiro cenário novo de interface falhou por seletor de rótulo do textarea; corrigido para consultar o nome acessível do campo, com suíte completa aprovada. Não houve chamada a um modelo pago nem envio a um chat real nesta validação.

## Limitações

A qualidade, coerência e humor dependem do modelo e da personalidade; instruções não garantem ausência de alucinações. Testes com provedor controlado validam integração, não qualidade linguística de um modelo real. Requer provedor configurado no desktop. Provedores remotos recebem a conversa enviada na geração e podem cobrar por uso. Contexto recente não sobrevive ao fechamento do app; histórico normal e memórias seguem as regras existentes. A nova compilação Windows é produzida pelo workflow após o push; não há homologação de instalação em máquina limpa.

---

# BotLive 0.1.2

## O que mudou

- Corrigida a etapa final de publicação: o empacotador forneceu URLs da API do GitHub e a validação anterior recusou o manifesto da versão 0.1.1.
- O script agora consulta os arquivos reais do rascunho e converte as URLs reconhecidas para downloads públicos da mesma release, preservando as assinaturas.
- Publicação rejeita arquivos desconhecidos, assinaturas ausentes, versão/tag divergente e releases já publicadas.
- Incluídos testes de regressão no comando test:updates e explicação no manual de atualizações.

## Como usar

Baixe o instalador Windows x64 na página Releases do repositório. Nas versões com canal oficial configurado, use Configurações > Verificar atualização. Para manter o projeto, continue preparando versão e notas com npm run update:prepare antes de cada commit; a conversão dos links ocorre automaticamente na publicação.

## Validação

Os sete testes do sistema de atualização passaram localmente, incluindo links da API e URLs temporárias de rascunho, preservação da assinatura, rejeição de destinos desconhecidos e proteção de releases publicadas. Na execução anterior, os testes de aplicação e a geração dos instaladores assinados 0.1.1 passaram; a falha ocorreu somente na validação final do endereço do manifesto. O rascunho 0.1.1 foi recuperado com este script, publicado e conferido sem autenticação: manifesto e downloads MSI/NSIS acessíveis. A nova versão passará novamente pelo workflow completo após o push.

## Limitações

Canal automático disponível para Windows x64. Assinaturas do atualizador não são certificados Authenticode. Instalação em máquina limpa e atualização completa em uma instalação real ainda precisam de homologação. Esta correção valida a correspondência dos arquivos e mantém a verificação criptográfica de instalação a cargo do atualizador Tauri.

---

# BotLive 0.1.1

## O que mudou

- Publicação inicial do projeto BotLive no repositório studiocactus/arrobabot, com interface React, núcleo Rust/Tauri, perfis, automações, comunidade, IA e manual offline.
- Sistema de variáveis com escopos local, perfil, pessoa e sessão; filtros, argumentos do comando, dados de eventos e incrementos atômicos.
- Editor simplificado com inserção no cursor, etiquetas, categorias, opções de formato e tipos de valor no gerenciador.
- Canal oficial de atualização pelo GitHub, chave pública embutida e configuração automática do manifesto para novas instalações.
- Workflow Windows para verificar commits, executar testes e gerar releases assinadas com instaladores, manual e notas detalhadas.
- Scripts de preparação e validação de versões, hook de pre-commit e registro obrigatório de cada atualização.

## Como usar

Instale a primeira versão 0.1.1 pela página Releases quando o workflow terminar. Abra Configurações e use Verificar atualização; o endereço e a chave pública oficial já vêm preenchidos. Para variáveis, abra Comandos e use os botões abaixo da resposta. Para desenvolvimento, execute npm run repo:setup e prepare cada versão com npm run update:prepare. Consulte docs/ATUALIZACOES.md para o procedimento completo.

## Validação

Validação local aprovada: 22 testes Rust, 7 Vitest, 4 cenários Playwright e 3 testes do script de atualização. TypeScript e build Vite aprovados. A revisão do stage excluiu chaves privadas, instaladores e dados de execução. Os testes de interface agora usam um servidor de produção isolado na porta 1421; o servidor de desenvolvimento anterior não respondeu no primeiro ensaio. A publicação remota continua condicionada aos testes do workflow.

## Limitações

O canal automático inicial é Windows x64. Instalação em máquina limpa, homologação de contas reais, certificados Authenticode e pacotes macOS/Linux não foram concluídos. Clientes anteriores sem canal configurado precisam da primeira instalação manual. O endpoint fica disponível após a publicação bem-sucedida da primeira release. Assinatura do atualizador não substitui certificado Windows. As funções ainda não equivalentes ao Streamer.bot estão listadas na matriz de aceite.
