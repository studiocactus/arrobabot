# Histórico de atualizações

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
