# Comandos simples — criação em uma tela e tempo de follow

Este documento reúne o estudo comparativo das referências do mercado, o diagnóstico da complexidade anterior do BotLive, a nova experiência de criação de comandos e o funcionamento do comando **Tempo de follow**, consultado com dados reais da Twitch.

## 1. Estudo comparativo (fontes oficiais)

| Produto | Criação de comando | Origem do "tempo de follow" | Autorização exigida | Vale para follow antigo? |
|---|---|---|---|---|
| **StreamElements** | Uma tela: *Bot Commands → Custom commands → Add new command* (nome + resposta + opções); sem "ação" separada | Comando padrão embutido `!followage` (alias `!howlong`), processado pelo backend da SE — **não** existe variável de follow age para comandos custom | Conta SE conectada ao canal; `!followage` é Twitch-only | Sim — responde com duração do tipo "6 years 1 month 10 days" |
| **Streamer.bot** | Duas peças: *Command* (gatilho) + *Action* (sub-actions) ligadas pelo trigger *Command Triggered* | Sub-action **Get Follow Age Info for Target** (Twitch → Followers), que popula `%followAge…%`, `%followDate%`, `%isFollowing%` | Conta Twitch via OAuth (Broadcaster obrigatório; Bot opcional) | Sim — retorna anos/meses/dias e a data |
| **Streamlabs Cloudbot** | Uma tela: *Cloudbot → Commands → Custom → Add Command* (modal único com Template, Reply, Permissions) | Template pronto `!followage` com a variável embutida `{touser.followage}` | Cloudbot habilitado + conta Twitch; followage é Twitch-only | Sim — exemplos documentados: "2 years 1 month" |
| **Twitch Helix** | — | `GET /helix/channels/followers` devolve `followed_at` (data real do vínculo) | Token de usuário com **`moderator:read:followers`**, sendo o chamador o broadcaster ou moderador | Sim — sem restrição de idade |

Fontes: [SE – Custom commands](https://docs.streamelements.com/chatbot/commands/custom) · [SE – !followage](https://docs.streamelements.com/chatbot/commands/default/followage) · [SE – Variáveis](https://docs.streamelements.com/chatbot/variables/cheat-sheet) · [Streamer.bot – Commands](https://docs.streamer.bot/guide/core/commands) · [Streamer.bot – Get Follow Age Info](https://docs.streamer.bot/api/sub-actions/twitch/followers/get-follow-age-info-for-target) · [Streamer.bot – Twitch Accounts](https://docs.streamer.bot/guide/platforms/twitch) · [Streamlabs – Cloudbot 101](https://support.streamlabs.com/hc/en-us/articles/44209470975515-Cloudbot-101-Custom-Commands-and-Variables-Part-One) · [Streamlabs – Followage](https://streamlabs.com/content-hub/post/how-to-setup-a-followage-command-on-twitch) · [Twitch – Get Channel Followers](https://dev.twitch.tv/docs/api/reference/#get-channel-followers) · [Twitch – Escopos](https://dev.twitch.tv/docs/authentication/scopes/) · [Twitch – Get Users](https://dev.twitch.tv/docs/api/reference/#get-users)

### Contrato da Twitch (o que o dado é, de verdade)

- `GET /helix/channels/followers?broadcaster_id=…&user_id=…` retorna `data[].followed_at` — a data em que o follow **atual** começou. Funciona para follows de anos atrás; não há restrição de idade.
- Exige user token com `moderator:read:followers` e chamador broadcaster ou moderador. **Sem isso, a resposta é uma lista vazia** — igualzinha à de quem não segue. Por isso permissão precisa ser verificada **antes** de concluir "não segue".
- A API informa o vínculo atual: não há histórico de unfollow/refollow. Não existe como reconstruir períodos que a Twitch não devolve.
- Resolver `@nome` → ID usa `GET /helix/users`; para o próprio autor da mensagem, o ID já vem no evento de chat (chamada a menos).
- Rate limit global (429) com headers `Ratelimit-*`; consultas repetidas devem ser cacheadas por canal e usuário.

### Lições (dado da API × dado coletado localmente)

1. `followed_at` vem da Twitch. Primeira mensagem, `firstSeen`, contadores locais ou a data de criação do comando medem **outra coisa** (presença, não follow) e nunca podem substituí-lo.
2. Autorização é estrutural: lista vazia pode significar "não segue" **ou** "sem permissão" — trate separadamente.
3. Não prometa histórico que a fonte não tem (primeiro follow histórico de antes de unfollows, tempo assistido retroativo etc.).

## 2. Diagnóstico do BotLive (o que exigia passos demais)

Antes desta versão, um comando como "tempo de follow" exigia **três entidades** e sintaxe de variáveis:

1. Uma automação para guardar dados (ex.: `Definir variável` com escopo por pessoa);
2. Outra automação/comando para acionar;
3. Mensagens com marcadores escritos à mão (`{{user.seguindoDesde}}`).

Problemas encontrados: campos essenciais da etapa de variável fora da área visível (já corrigido em v0.20.4), rótulos internos ("Automação salva.") vazando para o usuário, e risco real de confundir dado local com dado da Twitch (o `!seguindo` guardado manualmente nasce do zero e não serve para follows antigos). Não havia consulta à Twitch para dados de follow — agora existe.

## 3. A nova experiência (uma tela)

Em **Comandos**:

- **Receitas prontas** — substitui a criação antiga de Comandos prontos e reúne Resenha com IA. Escolha o resultado, confira os requisitos e abra o editor da própria função. Configurações novas começam desativadas; as equivalentes já existentes são abertas para edição, preservando os ajustes.
- **Criar comando personalizado** — o editor simples (nome, comando+alias, resposta, permissão, intervalos, áudio, contador) com **Testar fluxo** (resultado visível) e **Abrir no editor visual** quando precisar de mais.
- **Edição avançada opcional** — o editor visual continua existindo para etapas extras; nada do que já existia é descartado.

Cada comando reúne nome, alias (variações por vírgula), função, mensagem, permissão, intervalos, ativação e teste. Internamente o motor de fluxos continua sendo a **única fonte de verdade**: editar, duplicar, desativar ou excluir afeta exatamente uma execução, sem fluxos órfãos.

Conflitos: ao salvar um comando pronto, nomes e alias repetidos são **recusados com o nome do conflito** — nada é sobrescrito automaticamente.

## 4. Tempo de follow (`!followers`)

Modelo: **Receitas prontas → Tempo de follow → Configurar receita**. Padrão: `!followers` com alias `!followage`; sem argumento consulta quem enviou; com `@usuario` (quando a opção está ligada) consulta a pessoa citada.

Como funciona por dentro: a etapa **Consultar tempo de follow** chama `GET /helix/channels/followers` com o `broadcaster_id` do perfil e o `user_id` consultado, usando o token da **conta do canal** (com `moderator:read:followers`). A resposta guarda quatro campos:

| Campo | Conteúdo | Exemplo |
|---|---|---|
| `{{local.followAge}}` | Tempo decorrido, em português | `2 anos, 3 meses e 5 dias` |
| `{{local.followSince}}` | Data da Twitch, falada | `14 de março de 2026, às 21:05:09` |
| `{{local.followNote}}` | Aviso quando não der | `não segue o canal.` |
| `{{local.followResult}}` | `ok`, `not_following`, `not_found`, `unauthorized`, `rate_limited`, `unavailable`, `invalid` | `ok` |

Regras cumpridas: serve para follows anteriores à instalação (a data vem da Twitch); não usa primeira mensagem, não começa contador agora, não usa IA para inventar nada e não promete histórico de refollows. Falha de autorização **nunca** vira "você não segue o canal" — cada caso tem sua mensagem e o Histórico mostra o motivo real.

**A fala é do ArrobaSrv.** Os dados entram exatamente como a Twitch informou; a frase que os apresenta é escrita pela IA, no jeito do bot e falando com quem pediu — inclusive quando o assunto é um aviso (ex.: "reautorize a conta do canal em Perfis"), que é repassado fiel e com humor. A instrução fica em **Como o ArrobaSrv deve responder**, dentro do comando pronto, para ajustar o tom.

## 5. Catálogo

| Modelo | Status | Origem do dado |
|---|---|---|
| Tempo de follow | ✅ implementado | Twitch (`/helix/channels/followers`) |
| Total de seguidores | ✅ implementado | Twitch (contagem do canal) |
| Música atual | ✅ implementado | Integração do Spotify (quando conectada) |
| Link de doação (IA) | ✅ implementado | IA + seu link |
| Clipadores (clipe + contagem) | ✅ implementado | Twitch (clipe) + contador por pessoa |
| Tempo da live | 📋 pendente | Exige consulta `stream.online`/`uptime` — próximo passo |
| Categoria/título atual | 📋 pendente | `GET /helix/channels` — próximo passo |
| Links sociais | 📋 pendente | Configuração do perfil + resposta fixa |
| Lista de comandos | 📋 pendente | Leitura dos próprios comandos ativos |
| Contadores (existentes) | ✅ disponível | Contadores/`{{commandCount}}` já existem em Automações |
| Shoutout / clipes como ação | ✅ disponível | Já existem nas automações (Ação na Twitch) |

Nada é exibido como pronto sem executar: os itens pendentes não têm cartão no app.

## 6. Personalização sem sintaxe

A resposta padrão de cada modelo já funciona. Para personalizar, o editor oferece campos legíveis — **Nome do usuário**, **Nome do canal**, **Tempo seguindo**, **Data do follow**, **Aviso quando não der** — e por baixo eles são os mesmos campos tipados do sistema (inclusive `{{followerCount}}`, `{{nowplaying.*}}`). Consultas redundantes na mesma execução são evitadas (o ID do chat já resolve o autor) e consultas repetidas usam cache curto por canal e usuário (60 s), sem esconder mudanças por muito tempo.

## 7. Migração e compatibilidade

- Comandos e automações existentes continuam intocados.
- Nomes/alias repetidos são recusados antes de criar, com aviso do conflito — o `!followers` que você já tiver **não** é sobrescrito.
- Comandos antigos que tentavam "seguindo desde" com data local não são alterados: crie o novo modelo e revise; a edição avançada preserva qualquer ação extra do fluxo antigo.
- Textos corrigidos: salvar no editor simples confirma **"Comando salvo"** (e "Timer salvo"/"Automação salva" conforme a tela).

## 8. Testes e limitações

Executado nesta entrega: 386 testes Rust (formatação de data/duração em português com pluralização, interpretação dos desfechos da Twitch — segue, não segue, sem data, resposta ilegível — e o restante da suíte), 159 de interface (modelos, respostas padrão e detecção de conflitos) e 93 de tela, incluindo o percurso de aceitação **Comandos → Comandos prontos → Tempo de follow → Salvar** sem abrir o editor de automações, com capturas em 1100 px e janela larga (`%TEMP%\botlive-ux\comandos-prontos-*.png`).

Limitações reais:
- As respostas controladas de API validam o formato e os desfechos; a consulta real com as suas credenciais só acontece na sua máquina (e é somente leitura).
- O tempo é o do follow **atual** — histórico de unfollow/refollow não existe na API e não é reconstruído.
- O cache é de 60 s por canal e usuário; um follow novo pode demorar até isso para aparecer em consultas repetidas.

## 9. Próximos passos (pendências)

1. Tempo da live, categoria e título (consultas `stream.online` e `GET /helix/channels`).
2. Lista de comandos (leitura dos comandos ativos do perfil).
3. Social/doação como campos do perfil, sem texto solto.
4. Teste de leitura real na sua conta (só leitura) como evidência separada dos mocks.


## Receitas prontas — criação unificada

O catálogo em **Comandos** e **Automações** substitui o formulário antigo de Comandos prontos e o botão separado de Resenha com IA. Os comandos antigos continuam editáveis; as funções conhecidas usam o mesmo formulário ao criar e editar.

Receitas disponíveis: boas-vindas por comando, tempo de follow, total de seguidores, música atual, doação, clipadores, contador de mortes, resenha com IA, lembrete de água, som por resgate, agradecimento de raid e voz por resgate.

Escolher uma receita não salva nem executa ações. Revise o gatilho, resultado e requisitos. Ao salvar uma configuração nova, ela fica desativada até você ativá-la na lista. Configurações existentes mantêm seu estado de ativação, nome e passos personalizados.

Tempo de follow vem sem IA por padrão, com a opção preservada no formulário. Doação exige seu link. O som por resgate exige uma recompensa específica e um som; a receita não permite salvar como Qualquer resgate. Seus sons continuam na mesma fonte Áudio do BotLive.

Raid abre **Comunidade → Shoutout e ganks**; voz abre **Voicemod → Etapa 3**. Não há outro editor nem outra fila para essas funções. Lembrete de água usa o formulário de timer e aparece em Timers após salvar.

Receitas novas guardam identidade para serem reconhecidas mesmo após renomear. Para configurações antigas, funções com identidade e comandos com aliases iguais são reaproveitados; um timer antigo sem identidade só é identificado pelo nome e tipo. Não há migração ou exclusão automática dos seus dados. Comandos em conflito são recusados no salvamento.

Biblioteca de presets continua destinada aos pacotes que você salva, importa ou compartilha; não foi criado outro gerenciador de pacotes.
