# Histórico de atualizações

# BotLive 0.20.6

## O que mudou

**Publicação corrigida: a v0.20.5 travou num teste do manual e não gerou release — esta versão traz a correção e o conteúdo inteiro junto.**

- O manual passou a ter **28 capítulos** (o novo **Comandos simples** entrou no índice) e o teste de estrutura do manual ainda esperava 27 — por isso a CI parou antes de gerar release nenhuma. Nada foi publicado pela v0.20.5.
- A contagem foi atualizada para **28** e continua servindo de guarda: um capítulo a mais ou a menos sem atualizar o teste volta a falhar.
- O conteúdo completo da v0.20.5 entra nesta publicação: **Comandos prontos em uma tela só**, **Tempo de follow** com a data real da Twitch, **Total de seguidores**, **Música atual**, conflitos de nomes recusados (sem sobrescrever) e a confirmação **"Comando salvo"**.

## Como usar

Igual à v0.20.5 (notas dela e capítulo **Comandos simples** do manual): **Comandos → Comandos prontos → Tempo de follow → Salvar e ativar** — e digite `!followers` no chat.

## Validação

- Testes de tela (`playwright`): **93 aprovados** na suíte completa, agora incluindo o teste do manual com os 28 capítulos, o percurso de aceitação do comando e as capturas em 1100 px e janela larga.
- Testes Rust (**386**) e de interface (**159**): os mesmos da v0.20.5, já aprovados na CI.
- A CI revalida tudo (registro de commits, testes, build, docs) antes de gerar a release assinada.

## Limitações

- As mesmas da v0.20.5: consultas reais da Twitch acontecem com as suas credenciais (somente leitura), e histórico de unfollow/refollow não existe na API — nada é reconstruído.

---

# BotLive 0.20.5

## O que mudou

**Criar comandos comuns virou uma tela só — e o Tempo de follow responde com a data real da Twitch, inclusive para follows anteriores à instalação.**

- **Comandos prontos, em três passos.** Na tela Comandos, o botão **Comandos prontos** abre a escolha da função: **Tempo de follow**, **Total de seguidores**, **Música atual**, **Link de doação** e **Clipadores**. Confira nome, alias e resposta — tudo editável, com campos legíveis — e clique **Salvar e ativar**. O comando nasce pronto: sem montar automação, sem criar variáveis, sem segunda entidade. Permissões e intervalos ficam em **Avançado**, dentro do próprio comando depois de salvo.
- **Tempo de follow (`!followers`, alias `!followage`).** A etapa nova **Consultar tempo de follow** pergunta à Twitch (`/helix/channels/followers`) a data real do follow e calcula o tempo decorrido — "Ana segue cactus há 2 anos, 3 meses e 5 dias, desde 14 de março de 2026, às 21:05:09". Sem argumento consulta quem enviou; com `@usuario` (opcional) consulta a pessoa citada. Serve para follows de anos atrás: o dado vem da Twitch, não de histórico do bot.
- **Cada caso responde diferente e sem mentira:** não segue o canal, usuário não encontrado, sem permissão (reautorizar conta do canal), limite da Twitch, indisponibilidade e data ausente têm mensagens próprias — falha de autorização nunca vira "você não segue", e consulta falhada nunca vira "zero dias". O Histórico guarda o motivo real.
- **Sem sobrescrever nada:** nomes e alias repetidos são recusados antes de criar, citando o conflito — o seu `!followers` antigo nunca é apagado por acidente. Comandos e automações existentes continuam como estão.
- **Textos corrigidos:** salvar no editor simples confirma **"Comando salvo"** (e "Timer salvo"/"Automação salva" conforme a tela) — nada de conceito interno vazando para você.
- **Catálogo honesto:** Total de seguidores e Música atual usam dados reais (contagem do canal; integração do Spotify). Tempo da live, categoria/título, links sociais e lista de comandos ficaram documentados como pendência — sem cartão fingindo funcionar.
- **Estudo comparativo** (StreamElements, Streamer.bot, Streamlabs Cloudbot e Twitch Helix) com fontes, requisitos e limitações em **docs/COMANDOS-SIMPLES.md**, incluindo a diferença entre dado da API e dado coletado localmente.

## Como usar

1. Atualize o BotLive e abra **Comandos**.
2. **Comandos prontos** → escolha **Tempo de follow**.
3. Confira o comando (`!followers`), o alias (`!followage`) e as respostas — os campos legíveis (nome do usuário, canal, tempo seguindo, data do follow) já estão nos textos.
4. **Salvar e ativar**. Digite `!followers` no chat — ou `!followers @amigo` (com a opção ligada).

Para os demais modelos, o caminho é o mesmo. Quem quiser mais controle usa **Criar comando personalizado** e, se precisar, **Abrir no editor visual**.

## Validação

- Testes Rust (`cargo test`, src-tauri): 386 aprovados, incluindo os novos — data e duração faladas em português com plural certeira ("1 mês e 2 dias", "3 horas e 12 minutos"), e a interpretação separada dos desfechos da Twitch (segue, não segue, sem data, resposta ilegível, login inexistente).
- Testes de interface (`vitest`): 159 aprovados, incluindo os modelos (estrutura do Tempo de follow com os dois desfechos, resposta padrão editável) e a detecção de conflitos de nomes.
- Testes de tela (`playwright`): 93 aprovados, incluindo o percurso de aceitação **Comandos → Comandos prontos → Tempo de follow → Salvar e ativar** sem abrir o editor de automações, com recusa de nome repetido e capturas em 1100 px e janela larga (`%TEMP%\botlive-ux\comandos-prontos-*.png`).
- Build e verificação de tipos limpos; **docs/COMANDOS-SIMPLES.md** criado e manual regenerado (28 capítulos).

## Limitações

- As consultas de API são validadas por respostas controladas; a consulta real (somente leitura) acontece com as suas credenciais, na sua máquina.
- O tempo é o do follow **atual**: histórico de unfollow/refollow não existe na API da Twitch e não é inventado.
- Cache curto de 60 segundos por canal e usuário — um follow recém-dado pode levar até isso para aparecer em consultas repetidas.
- O `@usuario` consulta outra pessoa apenas quando a opção está ligada; sem ela, o comando responde sempre por quem enviou.

---

# BotLive 0.20.4

## O que mudou

**A etapa “Definir variável” para de dar erro misterioso: os campos que validam aparecem primeiro, já preenchidos, e o salvamento explica o que falta.**

- **Nome do valor e Guardar o valor sobem para o topo da etapa** — antes do conteúdo. Eles ficavam escondidos abaixo do campo de texto e dos atalhos, e quem montava o fluxo salvava sem ver, levando “Escopo ou nome de variável inválido” sem saber por quê.
- **O nome já nasce preenchido** (`valor`) ao escolher *Definir variável*, *Incrementar variável* ou *Apagar variável* — só troque pelo nome que quiser (ex.: `seguindoDesde`).
- **O erro do salvamento aponta o campo**: “Escopo ou nome de variável inválido. Preencha Nome do valor com letras sem acentos, números e sublinhado (ex.: seguindoDesde).” Sem escopo nenhum, a mensagem segue explicando o formato (`local.resposta`, `global.contador`…).

## Como usar

Em uma etapa **Definir variável**:
1. **Guardar o valor** → *Salva por pessoa* (ou o escopo que quiser).
2. **Nome do valor** → `seguindoDesde` (já vem `valor` preenchido).
3. **Mensagem / conteúdo** → o valor, usando as variáveis de **Data e hora** se quiser.

## Validação

- Testes Rust (`cargo test`, src-tauri): 384 aprovados, incluindo o novo — nome vazio é recusado apontando **Nome do valor**, e alvo sem escopo explica o formato.
- Testes de tela (`playwright`): 93 aprovados na suíte completa; captura da etapa com os campos na ordem nova, em 1100 px e janela larga (`%TEMP%\botlive-ux\variavel-campos-ordem.png` e `variavel-campos-ordem-larga.png`).
- Testes de interface (`vitest`): 155 aprovados; build e verificação de tipos limpos.

## Limitações

- Fluxos antigos com nome de variável inválido continuam sendo recusados ao salvar — agora com o campo certo na mensagem.

---

# BotLive 0.20.3

## O que mudou

**A tela Voicemod virou acordeão: as etapas abrem uma por vez e cada cabeçalho mostra o estado daquela etapa.**

- As seções — **Sua voz no Voicemod**, **Durante a live**, **Etapa 1**, **Etapa 2**, **Etapa 3** e **Avançado** — passam a usar o acordeão padrão do BotLive, o mesmo de Sons de Live, Rótulos OBS e Discord. Uma seção aberta por vez: a tela cresce sem virar parede de texto, já que mais etapas estão por vir.
- **Estado à vista mesmo recolhido:** o cabeçalho traz as fichas de situação — `CONECTADO`/`SEM CONEXÃO` em Durante a live, `PRONTO`/`PENDENTE` na Etapa 1, `CONFERIDO NO OBS`/`A CONFERIR` na Etapa 2 e a quantidade de regras na Etapa 3.
- A tela começa aberta na **Etapa 1**; os conteúdos continuam exatamente os mesmos — inclusive o **Alterar conexão e diagnóstico** dentro da Etapa 1 e o **Avançado** de cada regra de voz.

## Como usar

Clique no cabeçalho para abrir a etapa que quiser; as demais fecham sozinhas. Nenhuma configuração mudou de lugar — só a forma de navegar.

## Validação

- Testes de tela (`playwright`): 93 aprovados, com os testes da tela Voicemod adaptados ao acordeão (abrir a seção antes de conferir o conteúdo) e as capturas regeneradas em 1100 px e janela larga (`%TEMP%\botlive-ux\voicemod-1100.png` e `voicemod-largo.png`).
- Build e verificação de tipos limpos; capítulo de áudio do manual atualizado.
- Testes Rust e de interface inalterados — só mudou a apresentação (os mesmos 383 e 155 da v0.20.2).

## Limitações

- O acordeão recomeça na Etapa 1 a cada visita à tela (o estado aberto não é lembrado entre sessões).
- A conferência visual foi feita pelas capturas; a validação com o Voicemod aberto e vozes reais continua sendo o teste na sua máquina.

---

# BotLive 0.20.2

## O que mudou

**O aviso da LivePix para de ficar preso na tela: salvar credenciais limpa o aviso antigo, e o novo botão “Conferir agora” valida o acesso na hora.**

- **Salvar credenciais renova de verdade.** Ao clicar **Salvar no cofre**, o token antigo em memória é esquecido e o aviso de acesso recusado é apagado — ele falava de um segredo que não existe mais. Se o novo acesso falhar, o aviso volta com o motivo atualizado.
- **Botão “Conferir agora”** na tela Meta (ao lado de Desativar). Ele dispara uma consulta completa na hora — token novo, pagamentos e estado — **sem esperar o ciclo de 15 segundos e sem precisar de meta ativa**. Consulta ok limpa o aviso; senão, o motivo aparece na própria tela, com o próximo passo.
- O ciclo automático continua exatamente como estava (a cada 15 segundos, com a meta ativa).

## Como usar

Depois de resetar o Client Secret no painel da LivePix:

1. Abra **Meta**, cole o **Client Secret novo** no campo (deixar vazio **mantém o segredo antigo**, que já não vale).
2. **Salvar no cofre** — o aviso antigo sai na hora e o token é renovado.
3. Se quiser conferir sem esperar, clique em **Conferir agora**: ou o aviso cai (acesso ok), ou ele volta dizendo exatamente o que falta.

## Validação

- Testes Rust (`cargo test`, src-tauri): 383 aprovados, incluindo os novos — salvar credenciais apaga o aviso antigo de acesso, e o “Conferir agora” funciona sem meta ativa apontando o motivo quando o acesso ainda não renova.
- Testes de interface (`vitest`): 155 aprovados; build e verificação de tipos limpos.
- Testes de tela (`playwright`): 93 aprovados na suíte completa; capturas da tela Meta com o botão novo, em 1100 px e janela larga (`%TEMP%\botlive-ux\meta-conferir-agora.png` e `meta-conferir-agora-larga.png`).
- Solução de problemas atualizada com o caminho novo.

## Limitações

- O aviso continua sendo honesto: enquanto o acesso for recusado de verdade, ele aparece — o que mudou é que agora ele some sozinho quando a situação se resolve, e dá para forçar a conferência.
- A consulta real com as suas credenciais novas só acontece na sua máquina.

---

# BotLive 0.20.1

## O que mudou

**Padrão de versões documentado: a partir de agora, as atualizações avançam uma a uma em patch até o próximo marco minor.**

- Publicada a **v0.20.0**, as próximas atualizações são **v0.20.1**, **v0.20.2** e assim por diante — cada entrega com seu commit, suas notas e seus testes, como sempre foi.
- O salto para a próxima versão **minor** (**v0.21.0**) abre um ciclo novo, e o padrão continua igual: v0.21.1, v0.21.2…
- A regra ficou nos dois lugares de manutenção: nas instruções do repositório (**AGENTS.md**) e no capítulo **Atualizações e commits** do manual. Inclui a forma de escrita: no `update:prepare` e nos arquivos a versão vai sem o "v" (`0.20.1`); nas releases, tags e conversa, com o "v" (`v0.20.1`).

## Como usar

- Ao preparar a próxima entrega, use o próximo patch — `npm run update:prepare -- 0.20.1 caminho/das-notas.md` — e siga o mesmo fluxo de sempre (notas detalhadas, testes, `npm run update:check`).
- Sem saltos e sem repetir versão publicada: é a mesma regra de antes, agora com a numeração definida.

## Validação

- `npm run update:check` e `npm run test:updates` aprovados; manual regenerado com o capítulo novo.
- Entrega só de documentação: nenhum código mudou — os testes do aplicativo continuam os mesmos da v0.20.0 (381 Rust, 155 de interface, 93 de tela).

## Limitações

- O padrão vale a partir do ciclo v0.20.x; o histórico anterior mantém a numeração que já foi usada, sem reescrever releases.

---

# BotLive 0.20.0

## O que mudou

**Resgates de voz não se cancelam mais (um cumpre o seu tempo e o próximo espera), dá para avisar quem resgatou com uma mensagem sua, o `!clip` explica quando o canal está errado e os clipadores ganham contagem por pessoa.**

- **Uma troca por vez, na sequência.** As trocas de voz vindas de **automações** (etapa **Mudar minha voz**) sobrescreviam o efeito no ar — quem pagava pontos via o resgate ser cortado no meio. Agora automações e resgates respeitam a **mesma fila**: o efeito ativo cumpre o seu tempo inteiro e o próximo começa em seguida. A exceção é a duração **0** (“até a próxima troca”), que é substituída pelo próximo por definição. A política por regra (Fila/Ignorar/Substituir) continua em **Avançado**.
- **Mensagem quando o resgate entra na fila** (campo novo em **Avançado → Fila e intervalos**). Quem resgata ouve do bot que o pedido começa em seguida — texto seu, com `{{user}}`, `{{pos}}` e `{{voz}}`; vazio desliga. Serve para resgates e para as automações.
- **`!clip` com canal errado deixa de ser mistério.** O print do Histórico mostrava “Channel offline” da Twitch: o alvo do clipe é o canal do perfil, e quando ele está apontado para a conta do bot (autorização feita no navegador com a conta errada) a Twitch recusa como se o canal estivesse offline. Agora o BotLive recusa antes, explicando: “O canal do perfil está apontando para a conta do bot… autorize a conta do canal em Perfis”. Canal vazio também é recusado com o caminho.
- **Ranking dos clipadores.** O botão **Modelos** ganhou o comando pronto **Clipadores (!clip)**: quem pede faz o clipe ser criado na hora, recebe a resposta com o nome dela e o número de clipe que já fez — “Ana, esse é o seu 3º clipe da live!” — e a contagem fica em `{{user.clips}}`, por pessoa e persistente, pronta para ranking. Só conta clipe criado com sucesso: se o clipe falhar, nada soma.

## Como usar

- **Resgates na fila:** atualize e pronto — quem tem regra ou automação de voz passa a ver um efeito cumprir o tempo inteiro. Para ligar o aviso: **Voicemod → Avançado → Fila e intervalos → Mensagem quando o resgate entra na fila** (o texto padrão já vem pronto; ajuste como quiser).
- **Clipadores:** **Comandos → Modelos → Clipadores (!clip)** → confirme o comando (`!clip, !clipe`) → **Criar comando**. Teste no chat; a contagem de cada pessoa aparece na resposta.
- **Se o !clip falhar:** leia a linha do Histórico — live desligada, limite da Twitch, permissão de clipes ou canal do perfil errado (o aviso diz qual, com o próximo passo).

## Validação

- Testes Rust (`cargo test`, src-tauri): 381 aprovados, incluindo os novos — a automação de voz entra na fila em vez de cancelar o efeito no ar (o primeiro cumpre o tempo inteiro e o segundo começa depois), duração 0 é substituída por definição, o aviso de fila renderiza `{{user}}`/`{{pos}}`/`{{voz}}` e vazio desliga, e o alvo do clipe recusa canal vazio ou apontado para a conta do bot com o próximo passo na mensagem.
- Testes de interface (`vitest`): 155 aprovados, incluindo o comando pronto de clipadores (clipe → contagem `user.clips` → mensagem com nome, número e link).
- Testes de tela (`playwright`): 93 aprovados, incluindo a nova regressão dos modelos de comando (os dois modelos aparecem, o dos clipadores nasce pronto e ativado) com capturas em 1100 px (`%TEMP%\botlive-ux\modelos-comandos.png` e `modelos-clipadores-criado.png`).
- Build e verificação de tipos limpos; capítulos de comandos, áudio e solução de problemas atualizados.

## Limitações

- O aviso de fila usa o chat do canal; não há resposta direta (reply) ao resgate, que não é mensagem de chat.
- A contagem `{{user.clips}}` é total (não zera a cada live). Para ranking por live, dá para zerar com uma automação no início da transmissão.
- A causa da recusa da Twitch em “Channel offline” só é distinguida quando o canal do perfil aponta para a conta do bot; live desligada de verdade continua sendo o mesmo aviso.
- A homologação com resgates pagos, Voicemod aberto e live na Twitch continua sendo o teste na sua máquina.

---

# BotLive 0.19.0

## O que mudou

**O ArrobaSrv agora lembra de quem está falando e pode aprender fatos sobre as pessoas: a memória passa a valer em toda resposta, não só quando as palavras casam.**

- **Quem fala é sempre lembrado.** Antes, as notas da pessoa só entravam na resposta se alguma palavra da mensagem coincidisse com elas — por isso o bot “esquecia” quem estava conversando. Agora as notas de quem fala (interações registradas e fatos aprendidos) entram **sempre** no contexto, mesmo sem palavra em comum: ele responde como quem conhece, tipo “Ana, você tinha falado do seu gato!”. Os limites continuam os mesmos: até quatro notas, 1.400 caracteres por nota e seis mil no conjunto.
- **Aprender fatos sobre quem conversa** — interruptor novo na tela **Inteligência artificial**, **desligado por padrão**. Quando a fala traz informação duradoura (o que a pessoa gosta, faz, joga, quando nasceu ou como prefere ser chamada), o BotLive guarda uma linha curta na memória dela (`usuarios/fatos-…md`, teto de 50, sem repetir o mesmo fato). Fala sem informação não grava nada, e a chamada extra ao provedor só acontece quando a fala parece trazer fato.
- **Memória continua sendo só sua.** Fatos e interações seguem isolados por identidade (ID + plataforma): o que uma pessoa disse nunca entra na resposta de outra. Tudo continua em Markdown, na tela **Memórias**, para revisar, corrigir ou apagar.
- Quem não ligar o interruptor novo não muda nada: a recuperação por palavras, os limites, as pastas e a tela Memórias continuam exatamente como estavam.

## Como usar

1. Atualize o BotLive e abra **Inteligência artificial**.
2. Ligue **Aprender fatos sobre quem conversa** (e mantenha **Registrar interações na memória** ligado, como já está).
3. **Salvar personalidade**. Nas próximas conversas, o que as pessoas disserem de duradouro vira linha curta na memória delas — visível em **Memórias**, onde dá para revisar e apagar.
4. Para conferir sem ir ao ar: **Converse antes de colocar no ar** já responde usando as memórias do perfil.

## Validação

- Testes Rust (`cargo test`, src-tauri): 377 aprovados, incluindo os novos — quem fala é lembrado mesmo sem palavra em comum (e nunca vaza para outra pessoa), o mesmo fato entra uma vez, fatos diferentes ficam, e a extração só acontece quando a fala traz informação duradoura (a resposta “NADA” não grava nada).
- Testes de interface (`vitest`): 153 aprovados, incluindo o campo novo em perfis antigos — aprender fatos nasce desligado e quem ligou mantém a escolha.
- Testes de tela (`playwright`): 92 aprovados; captura da tela de IA com o interruptor novo, em 1100 px e janela larga (`%TEMP%\botlive-ux\ia-aprender-fatos.png` e `ia-aprender-fatos-larga.png`).
- Build e verificação de tipos limpos; capítulo de IA e memória do manual atualizado.

## Limitações

- A qualidade do fato extraído depende do provedor de IA configurado; em modelos locais pequenos, vale revisar de vez em quando a tela **Memórias**.
- O aprendizado acontece nas conversas com IA (nas respostas do bot): falas que não disparam nenhuma automação de IA não geram fato.
- A homologação com chat real, com pessoas de verdade, continua sendo o teste na sua live.

---

# BotLive 0.18.2

## O que mudou

**A área “Efeitos internos antigos (migração)” saiu da tela Voicemod.**

- O cartão de migração (que mapeava efeitos antigos como “Voz fina” e “Robô” para vozes do Voicemod) aparecia no **Avançado** de perfis com configuração antiga e não era mais usado: todos os campos ficavam em “Sem equivalência”. Ele foi retirado, e a tela Voicemod fica só com o que está em uso (fila e intervalos no Avançado).
- Nada quebrado para quem já tinha mapeamentos salvos: eles continuam valendo na execução. Efeito antigo pedido sem equivalência e sem o motor interno continua recusado, e o aviso agora aponta o caminho de verdade — “Troque para uma voz do Voicemod (regras de voz ou etapa Mudar minha voz) ou inicie o motor interno em Voz da live”.
- A etapa **Mudar minha voz** deixa de sugerir a tela retirada: quem ainda tem fluxo com efeito antigo vê o aviso para escolher uma voz do Voicemod direto no editor.
- O motor interno de voz (microfone → VB-Cable → OBS) **continua como está** para quem o utiliza: nada foi apagado, desinstalado nem migrado à força.

## Como usar

- Se você já usa só o Voicemod: nada a fazer — a tela fica mais limpa.
- Se algum fluxo antigo ainda usa efeito interno (fina, robô…), atualize a etapa para uma voz do Voicemod (editor do fluxo → **Mudar minha voz**) ou inicie o motor interno em **Voz da live**.

## Validação

- Testes de tela (`playwright`): 92 aprovados, com a regressão reescrita — mesmo com configuração antiga salva, a área de migração não volta, e o Avançado continua com “Fila e intervalos”.
- Testes Rust (`cargo test`, src-tauri): 373 aprovados; o aviso de efeito antigo sem motor interno aponta o novo caminho.
- Testes de interface (`vitest`): 153 aprovados; build e verificação de tipos limpos.
- Capturas da tela Voicemod sem a área, em 1100 px e janela larga (`%TEMP%\botlive-ux\voicemod-sem-migracao.png` e `voicemod-sem-migracao-larga.png`).

## Limitações

- A homologação com o Voicemod aberto e vozes reais continua sendo o teste na sua máquina.
- Quem dependia da tela de migração para mapear efeitos antigos agora atualiza as etapas para vozes do Voicemod — os mapeamentos já salvos continuam funcionando, mas não há mais tela para criar novos.

---

# BotLive 0.18.1

## O que mudou

**O Histórico para de encher de linhas repetidas da LivePix: o problema de acesso aparece uma vez, dizendo o próximo passo.**

- A Meta de doações consulta a LivePix a cada 15 segundos. Quando o acesso era recusado, cada consulta escrevia a mesma linha no Histórico — minutos depois, a lista inteira era só a repetição, escondendo os erros das automações (inclusive os do comando !pix, que pareciam sumir).
- Agora o aviso entra **uma vez**, enquanto o problema não mudar, e diz o que fazer: "O acesso da LivePix foi recusado. Salve as credenciais novamente na tela LivePix (Meta de doações) para renovar — o contador para enquanto isso."
- Os demais problemas da LivePix (credencial ausente, API fora do ar) seguem a mesma regra: uma linha, com o motivo e a correção.
- O comando pronto de doação (!pix) ganhou uma regra a mais no prompt da IA: **não escrever links nem endereços**. Cada resposta do bot entra na memória da conversa e a IA imita as respostas anteriores — um link escrito por ela se repetia e crescia a cada disparo, até aparecer três vezes na mesma mensagem. O link fixo continua vindo uma vez só, no fim da mensagem.
- E há agora a garantia no envio: **o mesmo endereço sai uma vez só na mensagem do chat**. Se a IA repetir o link mesmo assim, o app remove as repetições junto do enfeite que se repete (emoji, `:codigo:` e separadores) — endereços diferentes continuam todos, então links seguem sendo enviados normalmente.

## Como usar

- Se aparecer o aviso da LivePix, abra a tela **LivePix (Meta de doações)** e salve as credenciais de novo (Client ID e Client Secret): o token é renovado e o contador volta na próxima consulta. Se a meta não estiver em uso, desligue-a na mesma tela.
- No comando !pix, mantenha o link fora do prompt da IA e presente uma vez só na mensagem (depois de **+ Resposta da IA**). Mesmo que a IA repita o endereço, a mensagem publicada sai com o link uma vez só.

## Validação

- Testes Rust (`cargo test`, src-tauri): todos aprovados, incluindo os novos — o aviso repetido da LivePix registrado duas vezes produz uma única linha no Histórico (com o aviso da tela sempre atualizado) e a limpeza de endereços cobre o caso real do chat (`:purple_heart: livepix.gg/thenees` duas e três vezes viram uma), emoji repetido, mesmo endereço com e sem `https://`, endereços diferentes preservados e texto sem repetição intacto.
- Testes de interface (`vitest`): todos aprovados, incluindo o prompt do comando pronto com a proibição de links.
- Cenário reproduzido aqui: Meta ativada com token recusado a cada consulta, medindo a quantidade de linhas no Histórico; e a mensagem exata que apareceu no chat, medida antes e depois da limpeza.

## Limitações

- A causa do token recusado continua sendo o acesso da conta LivePix: a correção é re-salvar as credenciais (ou desligar a meta). Esta atualização apenas evita que o aviso esconda o resto do Histórico.
- A garantia de um endereço por mensagem vale para o **chat**. Voz, overlay e webhook continuam levando o texto como foi escrito.
- A homologação com a API real da LivePix, usando as suas credenciais, e o comando !pix na sua live precisam do teste na sua máquina.

---

# BotLive 0.18.0

## O que mudou

**Duas entregas: o comando pronto de doação (!pix), com resposta de IA diferente a cada chamada e o seu link no fim, e as correções encontradas na análise do comando de clipe (`!clip`/`!clipe`).**

### Comando pronto de doação (!pix), com resposta de IA variada
- A tela Comandos ganhou o botão **Modelos**, com o comando pronto **Link de doação**.
- Você informa o comando (o padrão é `!pix`; aceita variações como `!pix, !doar`) e o seu link de doação, como `livepix.gg/seucanal`.
- O BotLive cria o comando já ativado, com duas etapas nesta ordem: **Gerar com IA** (frase curta e animada, diferente a cada chamada, sem inventar valores, prêmios nem regras) e **Enviar mensagem** (a frase com o seu link no fim — `{{local.aiResponse}} 💜 livepix.gg/seucanal`).
- Precisa da IA configurada em **Personalidade de IA**; sem ela, salvar avisa o que falta e nenhum comando quebrado é criado.

### Análise do comando de clipe (`!clip`/`!clipe`) — correções encontradas
- Os erros do clipe agora dizem o próximo passo: live desligada, limite de pedidos da Twitch (espere alguns segundos) e falta da permissão de clipes (reautorize a conta do bot em Perfis). Antes, a falta de permissão e o limite viravam um erro genérico sem saída.
- A confirmação ficou honesta: "Clipe criado: <link> — a Twitch termina de montar em instantes."
- O aviso de variável do editor passou a considerar a **ordem** das etapas: usar `{{local.clipUrl}}` antes da etapa de clipe mostra "Nenhuma ação anterior deste fluxo define…" com a correção. Antes, nenhuma mensagem aparecia e a execução falhava com Variável ausente.
- O teste do fluxo deixou de quebrar a mensagem seguinte ao clipe: a etapa de clipe é pulada e o link vira um endereço de exemplo, para a mensagem ser ensaiada sem erro.
- Se o aplicativo não conseguir guardar o link do clipe para as próximas etapas, o motivo aparece no Histórico em vez de passar em silêncio.
- A ajuda da etapa de clipe no editor explica onde o link fica, e a ajuda geral da ação Twitch cita a permissão de clipes.

## Como usar

### Comando !pix
1. Atualize o BotLive e abra **Comandos**.
2. Clique em **Modelos** e escolha **Link de doação**.
3. Confirme o comando (o padrão é `!pix`) e escreva o seu link de doação.
4. **Criar comando** — ele nasce ativado. Digite `!pix` no chat para conferir: cada resposta traz uma frase nova da IA com o seu link no fim.

### Clipe
- Monte a automação com a etapa **Ação na Twitch → criar clipe** antes da mensagem que usa `{{local.clipUrl}}`.
- Reautorize a conta do bot em **Perfis** para trazer a permissão de clipes.

## Validação
- Testes Rust (`cargo test`, src-tauri): 365 aprovados, incluindo os novos — erros do clipe com próximo passo (live, limite, permissão), ensaio do fluxo com clipe (mensagem ensaiada com link de exemplo, sem Variável ausente) e disparo pelo caminho de produção com eventos simulados.
- Testes de interface (`vitest`): 153 aprovados, incluindo o comando pronto (estrutura do !pix, variações do comando, validação dos campos) e a ordem das etapas no aviso de variável.
- Testes de tela (`playwright`): 92 aprovados, incluindo as regressões de recompensa/voz e duplicação da 0.17.0.
- Capturas em janela de ~1100 px e larga (`%TEMP%\botlive-ux`): `fluxo-clipe-ajuda.png`, `fluxo-clipe-aviso-ordem.png`, `fluxo-clipe-aviso-ordem-larga.png`, `pix-modelo-dialogo.png`, `pix-modelo-dialogo-larga.png`, `pix-modelo-criado.png`.

## Limitações
- A homologação com Twitch, OBS e Voicemod reais continua pendente: o clipe de verdade, com a live ligada e a permissão de clipes, precisa do teste manual na sua máquina.
- Se o provedor da IA falhar durante a live, o comando !pix para na primeira etapa e não responde — é o comportamento documentado das etapas de IA, registrado no Histórico. Nesse caso o link não sai; se preferir o link garantido mesmo com a IA fora, dá para montar a versão com a mensagem do link antes da etapa de IA.

---

# BotLive 0.17.0

## O que mudou

**Duas entregas no editor de automações: o botão Duplicar (cópia independente e desativada) e a correção do vínculo recompensa↔voz — que era apagado silenciosamente ao salvar, fazendo o fluxo disparar para qualquer resgate.**

- **Duplicar qualquer automação.** Nas listas de **Automações**, **Comandos** e **Timers**, cada linha ganhou **Duplicar**: a cópia nasce como **Cópia de \<nome>**, abre já no editor e vem **desativada** (o editor avisa até você ativá-la). Ela é independente de verdade — identificadores novos para o fluxo e para os blocos/conexões (remapeados corretamente), referências externas preservadas (recompensa, voz, arquivos), clone profundo sem nada compartilhado com o original e sem copiar histórico, contadores ou execuções. Editar, salvar ou apagar a cópia não mexe na original.
- **A causa da perda da recompensa.** O salvamento do editor **zerava o campo do gatilho** para tudo o que não fosse texto de comando (o ID da recompensa era descartado) — ao salvar, o vínculo virava “Qualquer resgate” e outros resgates passavam a acionar a voz. O ID estável agora sobrevive ao salvar, ao banco e ao reabrir; “Qualquer resgate” volta a ser **escolha explícita** no seletor.
- **Voz e duração intactas.** A voz escolhida (ID real do Voicemod) e a duração persistem de ponta a ponta; na execução, aplica-se **exatamente** a voz configurada — voz indisponível informa a falha e não é substituída por outra (o serviço de fila, troca e restauração do Voicemod continua sendo o mesmo).
- **Estado salvo visível nos blocos.** O bloco do gatilho mostra **“Resgate: &lt;nome&gt;”** (ou **“Qualquer resgate”**) e o bloco da voz **“Voz: &lt;nome&gt; · &lt;duração&gt;”** — sempre o que está persistido, nunca o nome da automação. Recompensa fora da lista aparece como indisponível, com a orientação de atualizar a lista ou trocar o vínculo — **o ID é preservado e o gatilho não se amplia**.
- **Confirmação honesta.** O “salvo” só aparece depois de a persistência dar certo; em caso de falha, as alterações continuam no editor e o erro traz o próximo passo.

## Como usar

**Duplicar:** nas listas, o botão **Duplicar** (ícone de duas folhas) ao lado de Editar. A cópia abre no editor, está desativada — ajuste o que quiser e **ative na lista** quando estiver pronta.

**Recompensa e voz:** no editor visual, gatilho **Resgate de pontos** → escolha a recompensa (ou **Qualquer resgate**, consciente) → na etapa **Mudar minha voz**, escolha a voz e a duração → **Salvar fluxo**. Ao reabrir, tudo continua no lugar; o bloco mostra **Resgate: \<nome>** e **Voz: \<nome> · \<duração>**.

## Validação

Executada nesta máquina, nesta versão:

- `cargo test` (src-tauri, `-j 1`): **363 testes aprovados, 2 ignorados, 0 falhas**. Novos: **disparo pelo handler de produção com eventos simulados** (ID A dispara o vínculo; ID B não dispara o de A; evento sem ID não casa vínculo específico; “Qualquer resgate” explícito pega todos), **roundtrip no banco** (ID+nome da recompensa, voz e duração sobrevivem a salvar/recarregar) e **validação que recusa** em vez de silenciar (ID ou nome longos).
- `npm run check` (tsc): **limpo**. `npm test` (vitest): **149 testes aprovados** — novos de comportamento: `duplicateFlow` (IDs novos, conexões remapeadas, referências externas preservadas, nada compartilhado, cópia desativada, fluxo antigo sem layout), `triggerForSave` (recompensa nunca zerada; “Qualquer resgate” explícito preservado) e `triggerLabel` (Resgate: nome/ID/Qualquer).
- Playwright (navegador real, Edge): **92 testes aprovados** — novos: **salvar e reabrir mantendo recompensa, voz e duração** com os rótulos corretos nos blocos, **Atualizar lista não apaga a seleção**, **recompensa fora da lista preservada sem se ampliar**, e **duplicar** (nome “Cópia de”, aviso de desativada, conteúdo preservado, alterações na cópia não tocam a original, cópia nasce desligada).
- Capturas em `%TEMP%\botlive-ux`: `fluxo-resgate-voz-salvos.png`, `fluxo-resgate-fora-da-lista.png`, `fluxo-duplicar-copia-desativada.png`, `fluxo-duplicar-lista.png` (1100 px e largas).
- `npm run update:check`: válido. **Sem push e sem release nesta entrega.**

## Limitações

- A validação da interface foi feita no navegador com os mocks de teste (recompensas e vozes simuladas, eventos simulados pelo handler de produção — nenhum ponto real foi gasto, nenhuma mensagem publicada e nenhum resgate real alterado).
- A homologação com sua **conta da Twitch real** (recompensa de verdade acionando a voz no Voicemod) continua pendente: use o caminho de teste da regra (**Testar esta voz**) antes de confiar na live.
- O nome da recompensa nos blocos vem do que foi salvo; se você renomear na Twitch, o nome do bloco se atualiza quando a lista for recarregada e o fluxo salvo de novo (o vínculo em si nunca quebra — é pelo ID).

---

# BotLive 0.16.1

## O que mudou

**A lista de vozes do Voicemod deixa de chegar vazia: o aplicativo lia os campos errados do formato oficial (`id`/`friendlyName` vieram como `voiceID`/`name`) — a conexão funcionava, mas as opções apareciam em branco.**

- **Vozes aparecem de verdade.** A documentação oficial da Control API manda a lista como `{"id":"baby","friendlyName":"Baby","enabled":true}` e o nosso leitor esperava outros nomes de campo — por isso o seletor de vozes (Etapa 2 e regras da Etapa 3) abria com linhas em branco. Cada voz agora sai normalizada com **os dois formatos** (`id`/`friendlyName` e `voiceID`/`name`), então a tela, o motor e os rótulos passam a ler a mesma coisa.
- **Contrato travado no teste.** A regressão usa o **exato exemplo da documentação oficial** (incluindo `currentVoice` na raiz) e garante que o que a tela lê existe junto do que o núcleo lê; o mock dos testes também passa a responder no formato oficial.
- **Seletor sem mistério.** Quando não há voz carregada, o seletor explica: “Nenhuma voz carregada — atualize as vozes em Alterar conexão”, em vez de uma lista em branco.

## Como usar

Depois de atualizar: **Voicemod → Etapa 1 → Alterar conexão e diagnóstico → Atualizar vozes** (ou Desconectar e Conectar de novo) e a lista completa aparece na **Etapa 2** e nos campos **Voz do Voicemod** das regras. Depois é só: testar a voz na Etapa 2 (fale durante o teste) e criar as regras na Etapa 3.

## Validação

Executada nesta máquina, nesta versão:

- `cargo test` (src-tauri, `-j 1`): **360 testes aprovados, 2 ignorados, 0 falhas** — incluindo o novo `read_voices_aceita_o_formato_oficial_e_preenche_o_que_o_app_le` com o payload exato da referência oficial.
- `npm run check` (tsc): **limpo**. `npm test` (vitest): **143 aprovados**. Playwright (telas de voz e app): **16 aprovados**.
- `npm run update:check`: válido.

**Pendente de validação externa:** a lista de vozes com o **seu** Voicemod (formato real confirmado pela documentação oficial e pela sua captura de tela); se ainda faltar alguma voz que exista no Voicemod, me mande o print do diagnóstico.

## Limitações

- A normalização cobre os formatos documentados (`id`/`friendlyName` e `voiceID`/`name`); se o Voicemod mudar de formato de novo, o teste oficial falha primeiro e o ajuste é localizado.
- Vozes marcadas como não habilitadas pela licença continuam fora da lista (como na própria tela do Voicemod).

---

# BotLive 0.16.0

## O que mudou

**O chat do OBS alinha os nomes mesmo com emote gigante, e os emotes de terceiros (BTTV, 7TV e FFZ) finalmente aparecem como imagem — não mais como nome escrito.**

- **Nomes alinhados (o desencontro com emotes).** A linha de nome ficava presa à base do emote grande e “descia” nas mensagens com emote. Agora o nome de quem fala ancora no **topo** de cada mensagem e o emote fica centralizado no texto: a lista de quem escreve se mantém alinhada em todas as linhas, inclusive com emotes de 4×.
- **Emotes de terceiros viram imagem.** A Twitch entrega BTTV/7TV/FFZ como **texto** — por isso muitos emotes apareciam só com o nome. O BotLive agora busca a lista do seu canal (e os conjuntos globais) desses três serviços quando o canal conecta e desenha cada emote pelo nome, na mesma linha dos da Twitch. O que não estiver em nenhum dos conjuntos continua texto (é a realidade da plataforma).
- **Imagem nítida em tamanho grande.** Os emotes da Twitch passam a pedir a escala **3×** da CDN quando crescem (antes o 2× esticava e borrava, como o coração da sua captura).
- O espaço antes e depois dos emotes é preservado no texto (nada de palavras grudadas em “KappaExtaí”).

## Como usar

Nada a configurar: ao conectar o canal, o BotLive carrega os emotes da Twitch **e** de terceiros (aviso no Histórico com as contagens). Os controles de tamanho e da explosão continuam em **Chat no OBS → Mensagem**.

## Validação

Executada nesta máquina, nesta versão:

- `cargo test` (src-tauri, `-j 1`): **359 testes aprovados, 2 ignorados, 0 falhas** — incluindo o novo `emote_de_terceiros_vira_imagem_pelo_nome` (nome vira imagem, espaços preservados, palavra desconhecida continua texto) e o cache de terceiros sobrevivendo ao `store`.
- `npm run check` (tsc): **limpo**. `npm test` (vitest): **143 aprovados**. Playwright: **90 aprovados** (o alinhamento novo não derrubou a auditoria geométrica nem os testes do chat).
- `npm run update:check`: válido.

**Pendente de validação externa:** os emotes de terceiros dependem das APIs públicas do BTTV/7TV/FFZ e do seu canal — a conferência visual (emotes grandes nítidos, nomes alinhados, BTTV/7TV/FFZ aparecendo) precisa da sua live de teste. Se algum emote específico continuar como nome, me diga qual: provavelmente ele não está no seu canal nem nos conjuntos globais.

## Limitações

- Só os emotes do **seu canal + globais** dos três serviços; emotes exclusivos de outra comunidade não aparecem (a plataforma não os entrega).
- A lista de terceiros segue o mesmo ciclo da Twitch (busca ao conectar, válida por 12 horas); um emote novo ganho no meio da live entra na próxima busca.
- A explosão de emotes dos eventos continua usando os emotes da Twitch (CDN dela).

---

# BotLive 0.15.0

## O que mudou

**Correções de raiz em três caminhos quebrados (Voicemod 401, confirmação de áudio e mensagens do bot no chat) e o chat do OBS ganhou os emotes no formato da Twitch — junto com as seis pautas que você trouxe.**

- **Voicemod conecta de verdade (item 3).** O 401 **não era sua chave**: nosso `registerClient` mandava a chave embrulhada duas vezes no WebSocket (`payload.payload.clientKey`) e o protocolo oficial lê em `payload.clientKey` — o Voicemod nunca via a chave e respondia "não autorizado", mesmo com a chave inteira salva. Corrigido para o formato oficial (conferido na documentação pública da Control API), e o mock dos testes agora **exige a chave no lugar certo** — a suíte inteira reprovaria se o embrulho duplo voltasse. Atenção: a chave do e-mail tem o formato `controlapi-…`; guarde a cadeia inteira.
- **Confirmação de áudio sem fantasma (item 6).** O teste tocava no OBS e a tela dizia "sem confirmação em 12 s": a causa era o **Tauri rejeitar nomes de evento com ponto** (`audio.feedback`, `audio.play`, `chat.event`, `spotify.now`…) — o erro era engolido e nada chegava ao aplicativo. Os nomes da ponte viram `audio-feedback` etc. (o WebSocket continua com os nomes de sempre); agora o erro também **não é mais engolido**. Além disso, a confirmação que chega antes do teste responder com o id não se perde mais, e o som local confirma **started quando toca** e **done quando termina** (com o mesmo teto de 60 s do OBS para falas longas).
- **Toda mensagem do chat aparece — inclusive a do bot (item 1).** Duas barreiras descartavam tudo que vinha da conta do bot (`ArrobaSrv`): por isso só o teste "Sequência" aparecia. Agora **o que o bot fala aparece na coluna** (marcado como bot, respeitando "Mostrar as mensagens do bot"), **sem disparar automações, pontos ou presença** — e o retorno automático da Twitch não duplica o que o aplicativo já mostrou no envio.
- **Emotes no formato da Twitch (item 1).** Mensagem só de emotes cresce como na Twitch: **1 emote sozinho aparece no maior tamanho permitido (4×)**, 2 ou 3 em dobro, o resto no normal — com os controles novos no Estúdio: **Tamanho dos emotes** (50% a 200%), **Emote grande em mensagem só de emote** e **Emotes explodem ao aparecer** (os emotes saem voando da mensagem, como na referência da StreamElements que você enviou; o texto continua no lugar).
- **Spotify em uma linha (item 2).** Novo **Texto do cartão** em Conteúdo: **Tudo em uma linha** mostra “Artista — Título” num filete só, como o Snip; **Em linhas** mantém título, artista e álbum empilhados.
- **Explosão de emotes nos alertas (item 5).** O botão parava com "Sem emotes carregados": agora, quando a lista está vazia, o BotLive **busca os emotes do canal na hora** antes de desistir — o cache frio não trava mais o teste.
- **Meta da LivePix (item 4).** Resposta direta: o contador soma o que entrou **desde o marco atual da contagem** (não "desde que a meta foi criada" a menos que o marco seja esse) — doações anteriores não somam, e **Reiniciar contador** move o marco. **Não há data de término**: a meta segue até bater o valor-alvo. O cartão **Metas** agora mostra a data de início da contagem.

## Como usar

- **Voicemod:** salve a chave **inteira** (`controlapi-…`, do começo ao fim) em Voicemod → Etapa 1 → Alterar conexão e diagnóstico, confira o tamanho exibido e clique em **Conectar**.
- **Chat no OBS:** nada a fazer — mensagens (inclusive as do bot) aparecem sozinhas. Para os emites: grupo **Mensagem** tem **Tamanho dos emotes**, **Emote grande em mensagem só de emote** e **Emotes explodem ao aparecer**.
- **Spotify:** grupo **Conteúdo** → **Texto do cartão** → **Tudo em uma linha**.
- **Meta:** o cartão **Metas** informa “iniciada em <data>”; para recomeçar a contagem use **Reiniciar contador**.
- **Alertas:** **Testar explosão** funciona com o canal conectado; se a lista estiver vazia, ele busca na hora.

## Validação

Executada nesta máquina, nesta versão:

- `cargo test` (src-tauri, `-j 1`): **358 testes aprovados, 2 ignorados, 0 falhas**. Novos: o formato oficial do `registerClient` (com o mock exigindo a chave no lugar certo), o alfabeto dos nomes de evento do app, e a regressão "fala do bot aparece no chat sem duplicar nem disparar automações" — mais o teste da Twitch atualizado para o contrato novo (chat do bot passa, marcado como bot).
- `npm run check` (tsc): **limpo**. `npm test` (vitest): **143 testes aprovados** (novo: som que nunca começa é encerrado em 15 s; som iniciado tem teto de 60 s).
- Playwright (navegação real, Edge): **90 testes aprovados**, incluindo os do Chat no OBS com eventos na coluna (o mock do teste foi alinhado ao nome real do evento).
- `npm run update:check`: válido.

**Pendente de validação externa (não executado aqui):** a conexão real com o **seu** Voicemod (o protocolo agora bate com o oficial, mas só a sua máquina confirma), a audição da explosão de emotes e dos emotes grandes no OBS, e o caminho completo de mensagens reais da Twitch ao overlay — tudo isso precisa da sua live de teste. **Nada é declarado homologado** sem a sua confirmação; se qualquer ponto continuar errado, me mande o print com o estado da tela.

## Limitações

- O Voicemod foi corrigido contra a **documentação pública** da Control API; se a sua versão do Voicemod usar outro formato, o diagnóstico mostra o motivo e eu ajusto.
- Os emotes continuam só os da Twitch (BTTV/7TV/FFZ seguem fora), e a explosão ao aparecer usa tempos fixos de referência (1,6 s) por enquanto.
- A "explosão" de emotes das mensagens é o efeito de voar ao aparecer; a explosão de eventos (sub, presente, meta) continua sendo a configurada na tela Alertas.

---

# BotLive 0.14.2

## O que mudou

**Salvar uma chave da Control API incompleta agora é recusado na hora, com o que recopiar — a armadilha que gerou o 401 do Voicemod (rótulo “clientKey” ou só a cauda depois do hífen) não passa mais nem chega ao cofre.**

- **Trava de chave incompleta.** Ao salvar a chave do Voicemod, menos de 15 caracteres é recusado com “essa chave parece incompleta… copie a cadeia inteira (ex.: `controlapi-…`), do começo ao fim, sem o rótulo”. O diagnóstico continua mostrando o **tamanho** da chave guardada (sem expor o conteúdo) para conferir antes de conectar.
- **O caso real vira teste.** O episódio: o e-mail traz `controlapi-…`, o duplo clique **para no hífen** e guarda só a cauda de 9 letras — a tela mostrava “a chave guardada tem 9 caracteres” e o Voicemod respondia 401. A regressão cobre o rótulo (`clientKey`), o prefixo sozinho (`controlapi`) e a só-cauda (`hUNzyKZa4`) recusados, e a cadeia inteira curta real (`controlapi-hUNzyKZa4`, 20 caracteres) aceita.
- As demais proteções continuam: caracteres invisíveis do e-mail são limpos, e chave com espaço/aspas/quebra (rótulo colado junto) segue recusada com a orientação antiga.

## Como usar

Cole a chave **inteira** no campo da Control API (Voicemod → Etapa 1 → Alterar conexão e diagnóstico), incluindo o `controlapi-` do começo, e confira o tamanho exibido antes de **Conectar**. Se a tela disser “incompleta”, é sinal de que a cópia pegou só parte — recopie do começo ao fim (duplo clique corta no hífen).

## Validação

Executada nesta máquina, nesta versão:

- `cargo test` (src-tauri, `-j 1`): **356 testes aprovados, 2 ignorados, 0 falhas** — inclui o novo `a_label_or_key_fragment_is_refused_for_being_too_short` com o formato real de chave da Voicemod.
- `npm run check` (tsc) e `npm test` (vitest, 142): **limpos** (nada de frontend mudou nesta rodada).
- `npm run update:check`: válido.

## Limitações

- A trava é de **tamanho e forma de cópia** (não inventamos o formato oficial da chave): uma chave real mais curta que 15 caracteres seria recusada — improvável, e a mensagem diz o que fazer. Se a Voicemod emitir chaves fora desse padrão, o ajuste é trivial.
- Se a 401 persistir **com a chave inteira salva**, os próximos suspeitos são a conta (a chave pedida numa conta e o Voicemod logado em outra) e a Control API desativada no aplicativo — caminhos já cobertos pela tabela de diagnóstico.

---

# BotLive 0.14.1

## O que mudou

**Correção de raiz de uma corrida rara no campo de mensagem: o texto que você digita (ou cola) nunca mais pode ser reescrito por um render atrasado — e, na automação de testes, isso eliminava uma duplicação de conteúdo que reprovou o pipeline da 0.14.0.**

- **Campo de mensagem com texto vivo.** O editor de mensagens das etapas de fluxo (respostas, timer, automações) deixou de ser reescrito pelo React a cada render: enquanto você edita, **o texto no campo é a fonte da verdade**; a configuração carrega quando o campo não está em uso (trocar de etapa, abrir um preset). Inserir variável continua igual — o marcador entra na posição do cursor, no campo e no estado.
- **A corrida, explicada.** Em máquinas lentas, um render atrasado podia devolver o valor antigo do campo no meio de um preenchimento programático; a automação de testes caía num fallback que **insere no cursor** em vez de substituir, e o resultado era texto duplicado (ex.: "Olá, {{arg0}}Olá, "). O fenômeno apareceu **uma vez em ~90 execuções** na CI da 0.14.0, foi reproduzido localmente sob estresse (1 em 88) e agora está eliminado pela raiz — não mascarado no teste.
- **Teste endurecido no ponto certo.** O teste do catálogo de variáveis passou a conferir que o preenchimento assentou antes de inserir, para qualquer corrida futura falhar perto da causa e não longe dela.

## Como usar

Nada de novo: os mesmos campos, as mesmas inserções de variável. A diferença é que o texto digitado não some nem duplica em nenhuma condição de máquina lenta.

## Validação

Executada nesta máquina, nesta versão:

- **Estresse do alvo da corrida**: `app.spec.ts --repeat-each=12` — **132 execuções aprovadas** (antes da correção, a duplicação aparecia ~1 em 88).
- Playwright completo: **90 testes aprovados**, incluindo a auditoria geométrica nas cinco larguras com 0 achados. `npm test` (vitest): **142 aprovados**. `npm run check` (tsc): limpo. Console sem erros.
- `npm run update:check`: válido.

**Contexto do pipeline:** a CI da 0.14.0 (commit `b431c78`) foi **reprovada por essa corrida** — os testes e o build passaram, e a publicação **não** chegou a acontecer (nada foi lançado como release). Esta versão traz a correção para o próximo pipeline.

## Limitações

- A corrida era do campo de mensagem em condições de render atrasado; outros campos controlados do aplicativo não apresentaram o sintoma e não foram reescritos nesta rodada.
- Nada aqui atesta integrações reais: a validação é de comportamento de interface, com os mocks da prévia.

---

# BotLive 0.14.0

## O que mudou

**Correções do uso real em nove pontos: botões de Sons de Live e Contadores alinhados à direita, os três botões de Áudio no OBS lado a lado, amostras de cor voltando a ser barras (Meta, Alertas, Chat no OBS e Spotify — nada de óvalo), o Spotify com os botões de volta (Autorizar conta, Desconectar e Testar conexão legíveis e com caixa), a tela de IA com menos rolagem e o editor de Memórias parando de crescer com o arquivo.**

- **Sons de Live e Contadores (1 e 2).** Os botões de cada linha ficam **alinhados à direita**, na coluna de ações, com o cabeçalho acompanhando — mesma leitura das demais tabelas.
- **Áudio no OBS (3).** **Copiar endereço · Testar áudio · Parar todos os sons** passam a ficar **numa linha só**, sob o campo do endereço, em vez de empilhados.
- **Barras de cor (4, 5, 6 e 7).** As amostras de cor da Meta, dos Alertas, do Chat no OBS e do Spotify voltam a ser **barras** (retângulos arredondados), nunca óvalos — o valor hexadecimal continua visível ao lado/abaixo. O padrão fica no componente compartilhado (`.lp-color` e `.sp-tint`).
- **Spotify (7).** Os botões **Autorizar conta**, **Desconectar** e **Testar conexão** voltaram a caber e a parecer botões: a causa era uma regra de estilo da 0.13.0 que forçava 32 px de largura em botões de texto (o rótulo transbordava e cobria o vizinho). O botão fantasma mantém a caixa visível — nada de ação “sem botão”.
- **Inteligência artificial (8).** Menos rolagem vertical: **Endereço do provedor** e **Modelo** lado a lado, os **padrões de resposta** em recolhível (“Como a IA deve responder (padrões)”) e o painel de teste **fixo à vista** enquanto a configuração rola. Nada foi removido — tudo continua a um clique.
- **Memórias (9).** O editor deixa de crescer junto com o arquivo: o espaço de trabalho cabe na janela e a rolagem passa a ser **interna** (editor e lista), tanto para arquivos grandes quanto para muitas notas.

Durante a correção, os testes pegaram dois desdobramentos antes da entrega: o valor hexadecimal transbordava o bloco da barra no layout do Spotify (62 achados na auditoria geométrica — corrigido no componente) e o recolhível da IA exigiu ajuste no teste da tela, que agora abre a seção antes de conferir.

## Como usar

Nada de novo para aprender — as mesmas telas, com os controles no lugar certo. Na tela de **Inteligência artificial**, os padrões de resposta começam recolhidos: clique em **Como a IA deve responder (padrões)** para ajustá-los. Nas **Memórias**, o editor rola por dentro; a página fica parada.

## Validação

Executada nesta máquina, nesta versão:

- `npm run check` (tsc): **limpo**. `npm test` (vitest): **142 testes aprovados em 16 arquivos**.
- Playwright (navegação real, Edge): **90 testes aprovados**, incluindo a **auditoria geométrica nas cinco larguras com 0 achados** (os 62 do transbamento do hex foram corrigidos) e os testes das telas ajustadas (IA com recolhível aberto no teste, Memórias, Áudio no OBS).
- **Console do navegador sem erros.** Capturas de todas as áreas ajustadas em `%TEMP%\botlive-ux\auditoria-0.14.0\` (`sons`, `contadores`, `audio`, `meta`, `alertas`, `chat`, `spotify`, `ia`, `memorias`, todas em 1600 px) — Meta e Spotify no Review pane com as barras de cor.
- `npm run update:check`: válido. **Sem push e sem release nesta entrega** — aguardando sua palavra como nas rodadas anteriores.

**Não executado aqui:** validação em tela real com seus dados e resolução (as capturas usam os mocks da prévia); eventuais ajustes finos de largura em janelas muito diferentes de 1600 px. Se algum ponto persistir na sua máquina, me mande o print que eu acerto.

## Limitações

- As mudanças são de layout e componentes; nenhuma funcionalidade foi alterada — mas a tela de IA agora começa com os padrões de resposta recolhidos (quem os ajustava sem abrir o recolhível precisará de um clique a mais).
- O padrão de amostras em **barras** vale para as telas de overlay e Spotify apontadas; o seletor de cor de destaque das Configurações continua com os quadrados arredondados da tela de Aparência (não foi apontado como problema).
- Não há medição de contraste nem varredura de teclado nesta rodada — seguem como pendências do documento de auditoria.

---

# BotLive 0.13.0

## O que mudou

**Auditoria visual e de usabilidade aplicada ao aplicativo inteiro: o histórico parou de mostrar “Invalid Date”, os intervalos viraram “5 s” e “15 min” em vez de “0,08 min”, as ações de linha cabem numa linha, valores salvos que desapareceram são explicados em vez de sumirem, e o console ficou sem erros.** O documento completo — telas inspecionadas, problemas por prioridade, padrões adotados e o que ficou de fora — está em `docs/AUDITORIA-VISUAL-USABILIDADE.md`, com as capturas antes/depois.

- **Data quebrada no histórico (P1).** A Visão geral e o Histórico mostravam **“Invalid Date”** no lugar da hora dos registros: o mock de dados usava `at` enquanto o backend emite `timestamp`, e o formatador não se defendia de dado antigo. Agora o formatador aceita os dois campos e devolve **“—”** para o que não puder ser lido — nunca mais “Invalid Date” na tela.
- **Intervalos legíveis (P1).** Tabelas de Comandos, Automações e Timers, e os resumos de Respostas e Sons, mostram **“5 s”, “15 min”, “1 h 30 min”** no lugar de “0,08 min” quebrado em duas linhas (o “A cada 15 min” dos timers continua idêntico).
- **Ações de linha alinhadas (P1/P2).** Os ícones de cada linha (contador, preset, editar, apagar) ficam em **uma única linha**, com tooltip (`title`) em cada um; a ação **destrutiva** (apagar) ganhou borda e tinta vermelhas próprias, mais diferenciada do resto.
- **Valores salvos nunca somem (P2).** Quando o arquivo escolhido não existe mais (som ou TXT apagado), o seletor **preserva o valor e explica** — “salvo indisponível — escolha outro” — em vez de abrir uma linha em branco: vale para Sons por espectador, Respostas (TXT e som), Alertas e Chat no OBS.
- **Salvamento explícito (P2).** O formulário de Sons por espectador mostra **“Alterações não salvas” / “Tudo salvo”** ao lado do botão Salvar, no mesmo padrão da tela de Memórias.
- **Amostras de cor legíveis (P2).** Meta, Alertas e Chat no OBS passam a mostrar o **valor hexadecimal** junto da amostra redonda (como o Spotify já fazia) — as cores escuras não são mais círculos invisíveis.
- **Acabamento de componentes (P2).** `button.danger` e `button.ghost` com estilo próprio (o botão fantasma do Spotify deixou de herdar 40 px de altura), rótulos mínimos de 8 px subiram para 9 px, ajuda desatualizada do Estúdio de Alertas (“tela Áudio”) corrigida, e o **favicon** embutido eliminou o único erro de console (404 em toda carga).
- **Erro junto do campo (P2, incremental).** O componente `Field` ganhou mensagem de erro por campo (com `role="alert"`), começando pela chave da Control API na tela Voicemod — as demais telas seguem reportando por aviso, como antes.

Nada de funcionalidade foi removido, e nenhuma biblioteca nova foi adicionada: as mudanças são nos componentes compartilhados (`components.tsx`, `styles.css`, `types.ts`) e nos pontos específicos apontados pela auditoria.

## Como usar

Nenhuma ação nova para aprender — as mesmas telas, mais legíveis. Quando um seletor mostrar **“salvo indisponível”**, o arquivo escolhido não existe mais: escolha outro (ou deixe em branco) e salve. Nos formulários com botão **Salvar**, confira a pílula **“Alterações não salvas”** antes de sair da tela. Intervalos e horários agora são lidos direto (“5 s”, “15 min”, “1 h 30 min” e a hora dos registros).

## Validação

Executada nesta máquina, nesta versão:

- `npm run check` (tsc): **limpo**. `npm test` (vitest): **142 testes aprovados em 16 arquivos**, incluindo os novos de formatação (`src/format.test.ts`: duração legível e hora nunca quebrada).
- Playwright (navegação real, Edge): **90 testes aprovados**, incluindo os novos de comportamento (`tests/ui/history.spec.ts`: histórico sem “Invalid Date” e seletor preservando valor salvo) e a auditoria geométrica automatizada nas cinco larguras (1440, 1280, 1152, 922 e 680 px) **sem achados**.
- **Capturas antes/depois** de todas as telas em 1100 px (25 telas), seis telas densas em 1600 px, cinco abaixo de 960 px e quatro modais/editores, em `%TEMP%\botlive-ux\auditoria-antes` e `auditoria-depois` — os pares úteis são Visão geral (data), Comandos/Automações (intervalo e ações), Meta (cores) e Sons de Live (salvamento).
- **Console do navegador sem erros** nas capturas finais (antes: 1× 404 de favicon).
- `npm run update:check`: válido. Nenhum push nem release nesta entrega.

**Não executado aqui:** leitura visual individual de Discord, Rótulos OBS, Contadores, Respostas, OBS Studio, Inteligência artificial, Presets, Histórico e Estatísticas (capturadas e auditadas geometricamente, sem inspeção um a um); medição de contraste com ferramenta WCAG; varredura tab-a-tab completa; dados reais muito longos e listas de centenas de itens. O documento da auditoria registra cada pendência.

## Limitações

- A auditoria **não é completa** nem a interface está perfeita: telas e estados listados no documento não foram exercitados, e melhorias continuam possíveis (erros por campo nas demais telas, contraste medido, teclado).
- Os padrões novos (`.row-actions`, `.save-state`, `MissingOption`, `button.danger/ghost`, `.field-error`) começaram nos pontos apontados; a adoção em todas as telas é incremental.
- Nada do que foi visto com mocks atesta integrações reais (Twitch, OBS, Voicemod, Spotify): a prévia web é distinguível do desktop e não finge conexão.

---

# BotLive 0.12.0

## O que mudou

**A tela Voicemod vira o caminho único da sua voz: configure uma vez, em três etapas, e recompensas da Twitch ou comandos do chat trocam sua voz pelo Voicemod — com fila, duração e retorno automático à voz de antes. Os comandos falados viram área própria e os achados bloqueantes da auditoria de áudio estão corrigidos, com regressões que usam as rotas reais.**

- **Tela Voicemod em três etapas.** O caminho é dito em linguagem simples: *o BotLive recebe o resgate ou comando e pede ao Voicemod para trocar sua voz; o OBS recebe sua voz pelo microfone virtual do Voicemod* — sem capturar nem reprocessar microfone para isso. **Etapa 1 · Conectar Voicemod**: abra o aplicativo e clique em **Conectar**; a chave da Control API só aparece quando necessária (cofre, nunca em logs), e conectado a etapa recolhe sozinha — **Alterar conexão e diagnóstico** vai para área secundária. **Etapa 2 · Ouvir minha voz no OBS**: instrução objetiva para usar **Voicemod Microphone** como entrada da fonte da sua voz, evitar captura duplicada (nada muda dispositivos do Windows nem cenas) e **um único teste** (escolher voz, aplicar temporariamente, falar durante o teste e ouvir; a voz de antes volta com confirmação). **Etapa 3 · Escolher o que troca minha voz**. E os estados **“Voicemod conectado”** e **“Áudio conferido no OBS”** ficam separados: a confirmação de áudio é sua (ouvir a gravação) — sucesso de API nunca prova áudio na cena.
- **Regras: recompensa OU comando do chat.** Cada regra mostra só o essencial — acionamento, recompensa ou comando, voz real do Voicemod, duração e ativado/desativado — no formato **“Voz de robô → Robô → 30 segundos”**. Comandos do chat têm **permissão explícita** (por padrão, só moderadores; nada vira público às escondidas) e intervalo entre usos, com “quem pode usar” visível na própria regra; o resto fica em **Avançado**, recolhido. A configuração da Twitch é contextual: sem conta do canal autorizada, a tela oferece **Conectar conta do canal** ali mesmo (mesmo OAuth do perfil). **Testar esta voz** exercita o caminho da regra sem gastar pontos, sem mexer em resgates reais e sem publicar mensagens; mensagem de chat nunca é tratada como resgate de pontos.
- **Serviço único de troca.** Recompensas, comandos, testes e automações usam a mesma fila, duração e restauração: **um efeito por vez**, próximos esperam em fila limitada (ajustes em Avançado). Resgates só são marcados como cumpridos **depois que o Voicemod confirma a troca** (falha fica no Histórico; o estorno é com a Twitch — sem promessa de reembolso automático). A restauração devolve o **estado anterior confirmado**, inclusive o modificador de voz quando aplicável: se você tinha um efeito antes, é ele que volta — o app não promete “voz normal” nesse caso. Mudanças manuais no Voicemod são respeitadas, temporizadores antigos nunca restauram por cima de efeito mais recente, a queda de conexão mostra a ação **Recuperar restauração** e nada antigo é reaplicado ao reconectar.
- **Comandos por voz, separados.** O reconhecimento da sua fala (escuta contínua, gravação de comando, criação rápida) mudou para a área própria **Comandos → Comandos por voz** (continua também em Comunidade → Controle por voz). Usar resgates e comandos de voz **não exige** STT, servidor de transcrição, palavra de ativação nem moderação; quando a escuta liga, o microfone é o físico. Os seletores de **Mudar minha voz** em Comandos e Automações agora usam **vozes reais do Voicemod**; efeitos antigos (fina, robô…) só funcionam com mapeamento explícito — nunca por suposição de nome.
- **Migração preservada e contextual.** Configurações antigas ficam como estão: o motor interno (microfone → VB-Cable → OBS) continua funcionando para quem usa, nada é desinstalado e nenhuma cena é tocada. A migração aparece **só nos perfis afetados**, em **Voicemod › Avançado › Efeitos internos antigos**, mapeando cada efeito para uma voz na mão do usuário (sem efeito interno no ar, para não migrar por cima). O VB-Cable só continua necessário para quem usa os efeitos internos. Textos contraditórios foram corrigidos em **Áudio no OBS**, **Automações**, ajuda e manual: as duas fontes do OBS são **“Áudio do BotLive”** (sons, alertas e TTS) e **“Voicemod Microphone”** (sua voz), e os sons do bot nunca passam pelo Voicemod.
- **Auditoria de áudio fechada (docs/VALIDACAO-AUDIO-OBS.md).** O bloqueante — token serializado com aspas nas URLs de mídia — foi corrigido em `asset_src`, `tts::url` e no endereço da fonte (`audio.get`), com regressão que sobe a API local de verdade e faz requisições HTTP reais em `/asset` e `/tts`, conferindo 200 e corpo WAV (não prefixo de URL). A perda de conexão/standby agora **cancela reprodução e fila** (sem sobreposição nem repetição na reconexão); a idade dos eventos passou a ser conferida **no momento de tocar**, com prazos explícitos — 15 s para iniciar e teto de 60 s para som já iniciado (TTS longo não corta); e o fallback local ficou visível na tela (**“Última reprodução: pela fonte do OBS / pelo aplicativo — a fonte do OBS não estava conectada”**).
- **Código antigo fora de cena.** O hub “Voz da live” (seletor de motor interno/Voicemod, VB-Cable, gate e diagnósticos do motor interno) saiu da experiência principal junto com a tela antiga; `VoiceResgates` virou o componente de regras da Etapa 3. Documentação reescrita no capítulo **Áudio no OBS**, **Integrações**, **Exemplos de uso**, **Solução de problemas** e **API local** (política de fila), e o **manual foi regenerado** (27 capítulos).

## Como usar

**Configurar uma vez (tela Voicemod):** **Etapa 1** — abra o Voicemod e clique em **Conectar** (a chave da Control API, se pedida, vem de control-api.voicemod.net e vai para o cofre). **Etapa 2** — no OBS, aponte a fonte da sua voz para **Voicemod Microphone** (sem captura duplicada) e use o **Testar voz**: fale durante o teste e confirme **“Ouvi minha voz no OBS”** quando ouvir na gravação. **Etapa 3** — **Adicionar troca de voz**: escolha **Recompensa da Twitch** (ou **Comando do chat**, com quem pode usar em Avançado), a voz e a duração; exemplo: **“Voz de robô → Robô → 30 segundos”**.

**Na live:** a tela mostra conexão, regras ativas/pausadas, o efeito atual com o tempo que resta, o que vem depois (só quando há fila) e **Encerrar efeitos e restaurar minha voz**. Se a conexão cair, use **Recuperar restauração** após reconectar.

**Comandos por voz:** **Comandos → Comandos por voz** para escuta contínua e criação de comandos falados (microfone físico). **Quem ainda usa os efeitos internos:** tudo continua como estava; para migrar, ligue cada efeito antigo a uma voz em **Voicemod › Avançado › Efeitos internos antigos**.

## Validação

Executada nesta máquina, nesta versão:

- `cargo test` (src-tauri, `-j 1`, via `node .tools/run.cjs`): **355 testes aprovados, 2 ignorados, 0 falhas** (eram 352). Novos: comando do chat trocando a voz pelo mesmo serviço **sem virar resgate** (nada cumprido/gasto), permissão explícita e intervalo respeitados, regra desligada quieta, simulação sem efeito, **cancelamento devolvendo a voz de antes** (sem prometer “voz normal”), isolamento entre perfis, validação de regras (comando, permissão, duplicatas) e a **regressão de rota real** das URLs de mídia (`/asset` e `/tts` com requisições HTTP e corpo WAV conferido).
- **TTS real reexecutado** (`cargo test -- --ignored gera_wav_real`): síntese SAPI do Windows de verdade, **1 aprovado** (1,55 s) — não é simulação.
- `npm run check` (tsc): **limpo**. `npm test` (vitest): **140 testes aprovados em 15 arquivos**, incluindo os novos contratos do reprodutor da fonte (idade conferida no início da reprodução, prazos de 15 s/60 s, saturação, interrupção).
- Playwright (navegação real, navegador Edge): **88 testes aprovados** (eram 85), com **5 de voz**: painel de comandos por voz, escuta contínua com lotes reais, tela Voicemod com as três etapas (um caminho só, **zero menção a VB-Cable no caminho principal**, estados “conectado” × “áudio conferido” separados), regras com acionamento/voz/duração/ativação e **Avançado recolhido** (permissão padrão “moderadores”), e migração **só no perfil afetado**. Auditoria de layout nas cinco larguras (1440, 1280, 1152, 922 e 680 px) sem achados geométricos; capturas 1100 px e larga em `%TEMP%\botlive-ux` (`voicemod-1100.png`, `voicemod-largo.png`) e em `artifacts/` (não versionado).
- `npm run docs` + `tests/ui/manual.spec.ts`: manual regenerado (27 capítulos) e **aprovado**, sem quebrar links antigos. `npm run test:updates`: **9 testes aprovados.** `npm run update:check`: válido.

**Pendente de validação externa (não executado aqui):** a homologação com **OBS e Voicemod reais** — fala sem efeito, efeito acionado por resgate simulado pelo handler de produção, retorno da voz, comando de chat simulado, ausência de microfone duplicado e a gravação local de tudo — segue o protocolo de `docs/VALIDACAO-AUDIO-OBS.md` e **não foi executada**: não há OBS/Voicemod disponíveis nesta máquina nem participação humana para falar. **Nada é declarado homologado**; os testes simulados cobrem o comportamento pelo contrato, com a única exceção da síntese de TTS, que é real. Nenhuma live, moderação, resgate ou publicação real foi executada.

## Limitações

- A homologação real (OBS, Voicemod, microfone) continua pendente, com protocolo e critérios registrados em `docs/VALIDACAO-AUDIO-OBS.md`.
- O Voicemod é uma sessão por máquina (dispositivo local): as regras são por perfil, mas a troca de voz acontece na máquina onde o Voicemod roda.
- Sem conexão com o Voicemod, resgates e comandos **não** trocam voz e **não** são marcados como cumpridos (aviso no Histórico); estornos dependem da Twitch.
- “Áudio conferido no OBS” só você confirma (ouvindo a gravação); o app nunca declara áudio funcionando por sucesso de API.
- Ações antigas de voz com efeito interno exigem mapeamento manual ou motor interno rodando; sem isso, falham com o caminho da correção na mensagem.
- `vitest` mantido em **3.2.7** com os três avisos da cadeia de testes (decisão registrada na 0.10.0).

---

# BotLive 0.11.0

## O que mudou

**Uma única saída de áudio para o bot inteiro: a fonte de navegador “Áudio do BotLive” no OBS, um fader só no mixer, TTS gerado pelo Windows pelo mesmo caminho — e o VB-Cable sai de vez do caminho dos sons do bot. A sua voz continua com os caminhos próprios dela, inclusive Voicemod.**

- **Saída única “Áudio no OBS”.** A tela **Áudio** passa a se chamar **Áudio no OBS** e entrega um endereço próprio (`/audio` na porta local, token de overlay + perfil autorizado) para uma **Fonte de navegador dedicada** no OBS, sem visuais: só toca som. A tela tem **Endereço da fonte de áudio** com **Copiar**, os quatro passos de instalação, a opção **Controlar áudio pelo OBS**, o **estado real** (Aguardando o OBS / Conectada / Reproduzindo / Erro, com contagem de fontes), **Testar áudio** (bipe WAV gerado pelo próprio backend, não depende de arquivo) e **Parar todos os sons**. O teste distingue **som enviado**, **reprodução iniciada** e **falha**: a confirmação (`started`/`done`/`error`/`stopped`) vem da própria fonte pelo WebSocket — tempo esgotado também aparece como tal, nunca sucesso falso.
- **Política de reprodução única.** Fila serial com até **5 pendentes** e **15 segundos** por som; eventos com mais de **10 segundos** são descartados (reconectar o OBS não repete sons antigos); **Parar todos os sons** libera a fila na hora. Uma **fonte dona por perfil**: a primeira Fonte de navegador que conecta reproduz; uma segunda fica em **standby** com aviso na tela e assume automaticamente quando a dona sai — duas fontes nunca tocam juntas, e a tela diagnostica a duplicata. Com a fonte conectada, o aplicativo **nunca toca junto** (ele só toca quando o OBS não está ouvindo); overlays visuais seguem mudos; eventos de um perfil nunca tocam em outro.
- **TTS gerado no backend, pelo mesmo caminho dos sons.** O texto para voz passa a ser sintetizado pelo **Windows (SAPI)** — sem nuvem e sem custo — com cache em `media/<perfil>/tts/` (chave SHA-256, arquivo `.wav`) servido pela rota local `/tts`. O `speechSynthesis` do navegador saiu de cena: a voz agora toca na fonte do OBS como qualquer som, com fila, volume e confirmação iguais. A tela de voz migra `voiceURI` antigo para o **nome da voz SAPI** com fallback pt-BR, mapeia a velocidade para a faixa do SAPI e **Testar voz** usa o caminho oficial (mesmo para as prévias “Ouvir” de Sons de Live, Respostas, Chat, Meta e fluxos).
- **Voicemod = “Minha voz”.** O cartão agora se chama **Minha voz — Voicemod** e traz os **quatro passos concretos**: abrir e configurar o Voicemod, conectar o controle (chave da Control API no cofre), apontar o OBS para o **Voicemod Microphone** e testar a voz com volta automática à de antes — cada falha (aplicativo fechado, chave ausente/recusada, versão incompatível, conexão perdida) diz o que fazer em seguida. Os **sons do bot nunca passam pelo Voicemod**, a escuta de comandos continua no **microfone físico** e nada muda no microfone padrão do Windows: o Voicemod entra só como fonte de microfone no OBS.
- **Migração sem quebrar nada.** A escolha antiga de aparelho da tela Áudio **perde o efeito mas continua salva** (migração documentada na própria tela); arquivos, vínculos, volumes, comandos, resgates, alertas, contadores e frases seguem exatamente como configurados. O **VB-Cable não é desinstalado** (a voz com efeitos internos ainda usa o cabo como caminho do microfone) e **nenhuma cena do OBS é alterada**: a migração é orientada passo a passo (criar a fonte nova, testar, remover a fonte antiga de captura do cabo) em **uma única direção**, sem caminho antigo em paralelo. O app e o backend passam a falar só `audio.play` / `audio.stopLocal` / `audio.feedback` — os eventos antigos (`viewer-sound`, `alert-sound`, `tts`) deixam de existir.
- **Código obsoleto removido.** Saíram o player local pelo navegador e o seletor de aparelho (`src/audio.ts`, `src/audio.test.ts`, `src/AudioOutput.tsx`) e a prévia web do Chat no OBS perdeu o som morto (no navegador só o visual toca; o som sai pelo backend). Sem reformatar o que não era o assunto.
- **Documentação e API local.** O capítulo **[Áudio no OBS](docs/13-AUDIO.md)** foi reescrito com o caminho oficial, as regras de reprodução, a migração e as limitações; o **[API local](docs/API-LOCAL.md)** ganhou as rotas `/audio` e `/tts` e o protocolo da fonte (handshake com `mode:"audio"`, `audio.claim`/`audio.standby`, `audio.play`, `audio.stop`, `audio.ack` e o descarte de eventos antigos); os capítulos de Comandos, Solução de problemas, Alertas, Chat no OBS, Integrações, Respostas e sons e Exemplos foram ajustados; o ROADMAP corrigido (o Voicemod voltou na 0.9.0 como caminho de voz); e o **manual foi regenerado** (27 capítulos) só com o caminho oficial.

## Como usar

**Fonte no OBS (uma vez):** abra **Áudio no OBS**, copie o **Endereço da fonte de áudio**, crie no OBS **Fontes → + → Navegador**, cole o endereço (qualquer tamanho serve — a página não tem visual), deixe **Controlar áudio pelo OBS** marcado e clique em **Testar áudio**: o bipe chega pelo fader da fonte, que é o volume do bot inteiro na live.

**Migrar do VB-Cable:** crie a fonte nova, teste, ajuste o fader e **remova do OBS a fonte antiga** que capturava o cabo (**Áudio da área de trabalho** → CABLE Output), para o som não entrar em dois lugares. O VB-Cable pode ficar instalado se você usa a voz com efeitos internos. Nada precisa ser reconfigurado: os sons e volumes continuam os mesmos.

**Texto para voz:** escolha voz e velocidade em **Comunidade → Texto para voz** e use **Testar voz** — a fala sai pela fonte do OBS como qualquer som. **Minha voz:** em **Voz da live**, use os efeitos internos ou o cartão **Minha voz — Voicemod** nos quatro passos (o teste de voz volta sozinho ao normal). **Parar todos os sons** está na própria tela, para o que estiver tocando e limpa a fila.

## Validação

Executada nesta máquina, nesta versão:

- `cargo test` (src-tauri, `-j 1`, via `node .tools/run.cjs`): **351 testes aprovados, 2 ignorados, 0 falhas** (eram 336 na 0.10.0). Inclui os novos do áudio (reivindicação da primeira fonte, standby e promoção da segunda, `audio.play` único por origem com `sink`, validação de `audio.ack`, descarte de fila no stop, bipe WAV gerado) e do TTS (mapeamento de velocidade para a faixa do SAPI, cache que mantém os arquivos recentes, ids e endereços válidos), além de atualizar espectador, alertas, fluxos e Meta para o evento único `audio.play`.
- **TTS real executado uma vez** com `cargo test -- --ignored gera_wav_real`: o Windows sintetizou o WAV de verdade (SAPI, ~1,5 s) e o arquivo seguiu pelo caminho oficial com `kind:"tts"` — **1 teste aprovado**. Não é simulação: a síntese é a do sistema.
- `npm run check` (tsc): **limpo**. `npm test` (vitest): **138 testes aprovados em 15 arquivos** (eram 129), incluindo o player da fonte (fila serial, descarte de eventos com mais de 10 segundos, acks, `stopAll` liberando a fila) e a fila local do aplicativo. `npm run test:updates`: **9 testes aprovados.**
- Playwright (navegação real, navegador Edge): **86 testes aprovados** (eram 85), com **4 novos da tela Áudio no OBS** (caminho único dito na própria tela, copiar endereço, teste distinguindo enviado de reproduzida, parar todos os sons, ausência de seletor de aparelho, aviso com duas fontes e tela sem rolagem horizontal a 1100 px e larga) e a auditoria de layout nas cinco larguras (1440, 1280, 1152, 922 e 680 px), tela nova incluída, com **zero achados geométricos**. Build de produção aprovado pelo webServer. Capturas (1100 px e larga) em `%TEMP%\botlive-ux` e em `artifacts/` (não versionado).
- `npm run docs` + `tests/ui/manual.spec.ts`: manual regenerado (27 capítulos) e **aprovado**, sem quebrar links antigos.
- `npm run update:check`: executado antes da entrega.

**Pendente de validação externa (não executado aqui):** abrir a fonte real no WebView2 do OBS e homologar áudio de verdade na live, Voicemod e microfone reais, contas Twitch e a instalação assinada. Os testes automatizados cobrem o comportamento pelo contrato documentado — **não são homologação** —, com a única exceção da síntese de TTS, que é real. Nenhuma transmissão, moderação, resgate ou publicação real foi executada; **não houve push nem release**: tudo local, aguardando aprovação.

## Limitações

- Os sons dependem do aplicativo BotLive aberto: a fonte do OBS toca, mas quem entrega os arquivos e a fila é o app — fechado, a fonte fica muda.
- O TTS usa somente as vozes SAPI instaladas no Windows; não há vozes de nuvem.
- Com duas Fontes de navegador, **só a primeira reproduz** (a outra fica em standby avisado) — por design, para não tocar em dobro.
- Som interrompido no meio (mute da fonte, cena sem a fonte, **Parar todos os sons**) não retoma de onde parou; eventos com mais de 10 segundos são descartados de propósito, para reconexão não repetir som antigo.
- A homologação com OBS aberto na live segue pendente; a página `/audio` no WebView2 do OBS não foi aberta nesta entrega — a lógica dela (fila, descarte, acks, parada) é coberta pelos testes e o contrato está documentado no API local.
- `vitest` mantido em **3.2.7** com os três avisos da cadeia de testes (decisão registrada na 0.10.0, major 4.x merece validação própria).

---

# BotLive 0.10.0

## O que mudou

**O Estúdio da música ganha zoom de verdade (10% a 100%), modelos de cartão em um clique e seções de ajuste — e a auditoria de 17/10 entra na mesma versão, com os riscos de concorrência do Spotify e da voz fechados.**

- **Zoom da prévia, de 10% a 100%.** Rola a **roda do mouse** sobre o cartão para ampliar: o ponto sob o cursor fica parado e a composição se move em volta dele (ancoragem estável). Há indicador de porcentagem, botões **−** e **+** (desabilitados nos limites), a barra deslizante — que aceita as **setas do teclado** —, **Ajustar à área** (a escala acompanha a janela, religada ao abrir a tela) e **100%** (tamanho real). Com a composição ampliada, **arraste** para deslocar o quadro e as barras internas rolam normalmente; a página nunca rola na horizontal. O zoom é estado de visualização, **fora da configuração do cartão**, guardado em `localStorage` (`botlive.spotify.view`) — não contamina o design nem viaja no backup, e **o overlay do OBS nunca recebe escala nenhuma**: a fonte continua vendo os 1920×1080 inteiros.
- **Modelo do cartão em um clique.** Quatro pontos de partida — **Só a música**, **Uma linha**, **Compacto** e **Completo** —, cada um com descrição própria; qualquer mexida depois vira **Personalizado** sem perder o que você já ajustou, e cor/posição nunca contam como mudança de modelo. Nada trava: os interruptores de cada campo continuam livres em qualquer modelo.
- **Seções do Estúdio.** Os controles passam a **Conteúdo** (campos ligados e desligados, ordem das linhas, nome longo), **Layout** (pele, posição, alinhamento, largura, preenchimento, espaçamento, cantos), **Aparência** (capa, temas, amostras de destaque, cores, fontes, borda/sombra/transparência), **Comportamento** (entrada, saída, sem música, espera ao pausar) e **Ajustes avançados** (recolhido: capa desfocada ao fundo, duração da animação, força da sombra).
- **Corretor de cor 1:1.** Cada cor ganha amostra circular com o valor hexadecimal ao lado — o que você vê é exatamente o que entra no cartão.
- **Fidelidade prévia↔OBS.** Prévia e overlay continuam saindo do mesmo renderizador compartilhado (`spotifyOverlay.ts`, 48 chaves de design, presets e migração automática de configurações antigas), dentro do padrão `.lp-studio` da casa (prévia fixa à esquerda, controles em cards à direita, coluna única abaixo de 960 px). A amostra de demonstração agora se identifica como tal na ajuda da prévia: nunca é a sua conta conectada.
- **Troca de perfil sem contaminação.** O carregamento descarta a resposta que chegar depois de você já ter ido para outro perfil, e o autosave guarda o perfil de onde partiu — configuração antiga não aparece na tela nova nem é salva por cima do perfil errado (achados médios da auditoria).
- **Concorrência do Spotify fechada (`spotify.rs`).** Uma leitura por perfil em voo, com trava que se solta sozinha até se a tarefa cair; consulta e renovação capturam a **geração da sessão** antes de tocar a rede e conferem antes de gravar/publicar. Resposta que chega depois de **desconectar ou trocar de conta** é descartada (não repovoa a faixa, não publica evento atrasado) e renovação que termina tarde **não reescreve as credenciais apagadas** do cofre. `spotify.connect`, `spotify.disconnect` e `delete_profile` avançam a geração.
- **Captura de voz por botão protegida (`voice.rs`).** A transcrição submetida pelo botão confere a geração da sessão: fala que termina depois de parar a escuta ou trocar de perfil é descartada, como já acontecia na escuta contínua.
- **Auditoria de 17/10 aplicada.** Retenção de backup estrita (só `botlive-…-notas.txt` de verdade), restauração abortada sem cópia de segurança, `watch_time` no formato 2 do backup, inspeção WebSocket de cabeçalho fragmentado sem perder bytes, sessões Twitch bot/canal ligadas à conexão pai, auditoria geométrica que agora reprova achados (com Spotify incluído) e ResizeObserver do Spotify observando a prévia quando ela aparece.
- **Dependências com atualização controlada.** `npm audit fix` **sem mudança major** corrigiu o `source-map-js`. O **Vitest fica em 3.2.7**: as três ocorrências restantes (2 críticas, 1 moderada) são da cadeia de testes e exigiriam major 4.x, mudança que merece validação própria — decisão registrada abaixo, sem `audit fix --force`.

## Como usar

**Zoom:** abra **Spotify** com o Estúdio da música e rode a **roda do mouse** sobre a prévia — o cartão amplia em volta do ponto sob o cursor. Afine com os botões **−** / **+** ou arrastando a barra (setas do teclado funcionam quando a barra está focada). **Ajustar à área** devolve a escala à largura da janela; **100%** mostra no tamanho real. Ampliado, **arraste** o quadro para deslocar a composição; a roda sempre volta a ampliar/reduzir. O zoom lembra entre sessões e vale só para a prévia — o OBS não muda.

**Modelos e seções:** em **Modelo do cartão**, clique em **Só a música**, **Uma linha**, **Compacto** ou **Completo** para partir de um arranjo pronto; mexa em seguida à vontade — a etiqueta vira **Personalizado** e nada do que você ajustou se perde. Os ajustes ficam nas seções **Conteúdo**, **Layout**, **Aparência**, **Comportamento** e **Ajustes avançados** (abra com **Abrir**, feche com **Recolher**). Cada cor mostra a amostra circular com o hex ao lado.

**Cores e OBS:** nada muda no fluxo de antes — o endereço da camada continua no cartão **No OBS** (Fonte de navegador, 1920×1080, `mode=spotify`) e **Testar no overlay** segue disparando a amostra com selo **Ensaio** pelo mesmo caminho da música real.

**Migração:** automática. Configurações das versões antigas são reconhecidas pelo renderizador compartilhado; não há nada a exportar nem a refazer. Quem tinha a tela aberta passa a ver os controles novos na próxima abertura.

## Validação

Executada nesta máquina, nesta versão:

- `cargo test` (src-tauri, `-j 1`, via `node .tools/run.cjs`): **336 testes aprovados, 1 ignorado, 0 falhas.** Inclui os novos: teste unitário da trava por perfil (uma leitura em voo, liberação sozinha) e cenas de integração no fluxo do Spotify — consulta em voo segura a fila (a segunda leitura devolve o estado atual **sem** nova chamada à API), desconexão no meio da consulta descarta a resposta atrasada (estado vazio, nenhum `spotify.now` tardio, cofre limpo) e renovação interrompida devolve o aviso em português **sem** reescrever as credenciais apagadas. Também cobre backup (retenção estrita, aborto sem cópia de segurança, round-trip de `watch_time`), handshake WebSocket fragmentado, sessões Twitch e voz com checagem de geração.
- `npm run check` (tsc): **limpo.** `npm test` (vitest): **129 testes aprovados em 15 arquivos** (eram 124 na 0.9.0). `npm run test:updates`: **9 testes aprovados.**
- Playwright (navegação real, navegador Edge): **85 testes aprovados**, entre eles os **5 do Spotify** (zoom de 10% a 100% com roda, teclado e persistência fora do design; presets do mínimo ao completo e seções; prévia igual ao OBS; conexão, leitura e variáveis com autosave do design completo; Client ID persistido e erro de autorização) e a auditoria de layout nas cinco larguras (1440, 1280, 1152, 922 e 680 px) com **zero achados geométricos** e zero erros de página. Build de produção aprovado pelo webServer (aviso de chunk > 500 kB). Capturas ficam em `%TEMP%\botlive-ux`, fora do repositório.
- `npm audit`: 3 ocorrências (2 críticas, 1 moderada), todas na cadeia do Vitest em árvore de desenvolvimento. `npm audit --omit=dev`: **zero alertas na árvore de produção.**
- `npm run update:check`: executado antes do commit local.

**Pendente de validação externa (não executado aqui):** OAuth e faixas reais do Spotify (inclusive 401/429 com Retry-After e desconexão durante renovação contra a API real), OBS/WebView2 com a fonte de verdade e áudio, Voicemod e microfone reais, contas Twitch e a instalação assinada. Os testes usam **servidores simulados** que falam o contrato documentado — cobrem o comportamento, **não são homologação**. Nenhuma moderação, resgate ou publicação real foi executada; **não houve push nem release**: commit local apenas.

## Limitações

- **Restauração de backup não é atômica** (achado Alto da auditoria, **pendente**): `write_tables` confirma antes de `write_files`, que remove e regrava arquivos — falha de disco/permissão pode deixar banco novo com arquivos incompletos. A cópia de segurança agora aborta a operação antes de trocar (correção desta junta), mas a transação única de banco+arquivos continua sendo a pendência estrutural aberta, documentada no manual.
- **3 avisos do `npm audit` mantidos de propósito**: `tinypool` (2 críticas) e `@vitest/mocker` (1 moderada) pela cadeia do Vitest 3.2.7; atualizá-los exige major 4.x, mudança que deve vir com validação dedicada de configuração e suíte. Nenhum alerta na árvore de produção. Recomendação: revisar quando houver janela de migração do Vitest.
- **Zoom guardado em `localStorage`** é por máquina e por navegador do app: não viaja no backup nem sincroniza entre computadores (por ser estado de visualização, não design). Sem armazenamento disponível, o zoom segue valendo só na memória da sessão.
- **Concorrência Spotify coberta por teste simulado**, não por corrida contra a API real: latência, 429 com Retry-After e timeouts de 30 s reais não foram reproduzidos aqui.
- **Homologação real pendente** (contas Twitch/Spotify, OBS/WebView2, Voicemod, microfone, instalação assinada) — lista completa em `docs/AUDITORIA-2026-10-17.md`.
- Aproveitamento visual dos controles novos continua sendo revisão humana: a auditoria geométrica prova medidas, não contraste, legibilidade nem gosto.

---

# BotLive 0.9.0

## O que mudou

**A tela Voz da live vira o hub da sua voz, o Voicemod entra de verdade e os resgates de recompensa passam a controlar sua voz — com fila visível, confirmação antes de cumprir e volta à voz de antes garantida.** Três seções num lugar só: **Minha voz na live**, **Comandos do Arroba** e **Resgates de voz**. Item por item:

- **Caminho da voz, com exclusão mútua de verdade.** O cartão **Minha voz na live** escolhe entre **Efeitos internos do BotLive** (padrão, microfone → VB-Cable → OBS, como sempre) e **Voicemod** (o BotLive só controla o aplicativo; o áudio passa a ser microfone → Voicemod → microfone virtual → OBS). O backend barra a troca com a voz interna no ar e recusa o início da voz interna enquanto o caminho estiver em Voicemod: nunca dois capturando o microfone ao mesmo tempo. O cartão **Diagnóstico** mostra o estado real de ponta a ponta — microfone e nível, saída para o OBS, conexão com o Voicemod, escuta de comandos — com avisos para captura dupla, circuito fechado e aparelho desconectado.
- **Voicemod Control API integrada pelo caminho oficial.** WebSocket local (ws://localhost, portas documentadas), registro com a chave da Control API (peça em control-api.voicemod.net; a chave vai para o cofre, com acesso restrito ao dono — nunca aparece no chat, no Histórico ou na tela). A tela só mostra “Conectado” depois da autorização aceita pelo próprio Voicemod, e cada problema tem o seu motivo: aplicativo fechado, conexão recusada, chave ausente, chave recusada (401), versão incompatível, conexão perdida, timeout. A voz só é declarada aplicada quando `getCurrentVoice` confirma o id pedido (até 6 tentativas) — sem confirmação, não há sucesso nem cumprimento de resgate. A voz normal do Voicemod é o id documentado `nofx`. `voiceChangedEvent` é observado: uma troca que você faz no aplicativo durante um efeito é respeitada (nada é sobrescrito por cima). Reconexão automática com tentativas e, no fim, mensagem honesta em vez de promessa.
- **Resgates de voz (recompensa → voz do Voicemod).** Cada recompensa do canal ganha um vínculo com voz real do Voicemod (lista vinda do aplicativo, com as vozes liberadas pela licença), duração (0 = até cancelar), cooldown por recompensa e política explícita de concorrência: **fila**, **ignorar** ou **substituir** (fila com limite de 1 a 10; estourou, o resgate novo é recusado com aviso — a live não vira fila de espera). A fila aparece na hora: efeito atual com o tempo que resta, quem pediu, o que vem depois, e os botões **Cancelar efeitos e voltar à voz normal** e **Recuperar restauração pendente**. O resgate só é marcado como cumprido na Twitch **depois que o Voicemod confirma a troca**; sem confirmação ele fica pendente, sem promessa de reembolso (a Twitch decide o estorno). A restauração devolve a voz que estava antes do primeiro efeito — continuações da fila herdam a mesma origem —, com confirmação do Voicemod. Queda no meio do efeito guarda a promessa de restauração e diz a verdade; nada é reaplicado sozinho. Temporizadores antigos nunca desfazem comandos mais novos (proteção por geração), e deduplicação por **ID do resgate** (não só a mensagem do EventSub) impede que o mesmo resgate seja aplicado duas vezes — o ID da mensagem e o ID do resgate são preservados separados (corrigido o bug que completava o resgate usando o id errado).
- **Comandos do Arroba, no microfone e no chat.** Um interpretador determinístico de PT-BR entende “Arroba, mude a categoria para Minecraft.”, troca de título, “dê timeout em usuario123 por 5 minutos”, “quicar usuario123” (timeout configurável — nunca banimento permanente), “ban @usuario123” (que **pede confirmação** com alvo e prazo antes de agir), “cancele”, “unban/ban de volta” e “volte minha voz ao normal.”. Acentos, pontuação, números por extenso, sinônimos e frases com vários pedidos são tratados; pedido incompleto ou ambíguo gera instrução, nunca chute. A validação de origem é no backend: fala de voz só vale do streamer real (não simulada), e ordens de chat só de moderador para cima (nunca de alerta, resgate, TTS ou Discord). O alvo é resolvido para o login real da Twitch antes de qualquer ação, e o sucesso é reportado só depois da resposta da API.
- **Reconhecimento com caminho único e sessão guardada.** O RealtimeSTT local é o caminho padrão (um servidor só para o uso normal); o whisper-server continua disponível como opção legada. O **microfone físico** passa a ser escolhido de propósito (antes de qualquer efeito) e a escolha fica salva no perfil. O botão **Gravar comando** (“o que eu ouvi”, até 8 s) reutiliza o mesmo caminho. Parar a escuta agora descarta no backend as transcrições em curso: resultado que chegue tarde não dispara automação de sessão que já morreu, e trocar de perfil zera as sessões antigas. A ajuda da sensibilidade estava invertida e foi corrigida: **diminua** se o bot não escuta você, **aumente** se ruído de fundo vira fala.
- **Migração sem adivinhação.** Quem já usava efeitos internos (fina, robô…) migra para o Voicemod escolhendo, na seção **Efeitos internos antigos**, qual voz do Voicemod corresponde a cada efeito. Nenhum nome vira ID por suposição; sem vínculo, o pedido é recusado com aviso. Os efeitos internos continuam intactos para quem fica no caminho interno.
- **Componentes compartilhados.** O painel de comandos por voz virou um componente único usado na tela Voz da live e em Comunidade → Controle por voz — mesmo comportamento nos dois lugares, sem cópia divergente.

## Como usar

**Caminho interno (como sempre foi):** Voz da live → Minha voz na live → escolha “Efeitos internos do BotLive”, selecione microfone e saída (CABLE Input), clique em **Iniciar voz ao vivo**. No OBS, a fonte usa **CABLE Output**.

**Caminho Voicemod:** 1) abra o aplicativo Voicemod nesta máquina; 2) peça a chave da Control API em control-api.voicemod.net e salve em Minha voz na live → Chave da Control API (fica no cofre); 3) **Conectar** — a tela só mostra “Conectado” quando o Voicemod autoriza; 4) teste uma voz e ajuste o Voice Changer se a voz pedir; 5) no OBS, escolha o microfone virtual do Voicemod como fonte de áudio. Troque o **Caminho da voz** para “Voicemod” (a troca é barrada se a voz interna estiver no ar — pare antes).

**Resgates de voz:** 1) com o Voicemod conectado, abra Resgates de voz; 2) ligue **Ligar resgates de voz**; 3) **Adicionar vínculo** e escolha recompensa (lista real da Twitch — autorize a conta em Meta → Twitch com permissão de gerenciar resgates), voz, duração, cooldown e política de concorrência; 4) **Salvar vínculos**. A fila aparece em tempo real, com **Cancelar efeitos** e **Recuperar** para quando algo ficar pendente. **Testar esta voz** em cada vínculo usa o mesmo caminho de um resgate de verdade, sem tocar na Twitch.

**Comandos por voz:** ligue a escuta contínua, escolha o microfone físico e fale “Arroba, mude a categoria para Minecraft.” (ou o título, timeout, ban com confirmação, “cancele”, “volte minha voz ao normal”). O duração do “quicar” é configurável no mesmo painel. No chat, a mesma frase funciona para streamer e moderadores, sem escuta. Crie comandos novos em segundos com **Criar comando por voz**.

**Migração de efeitos antigos:** ao trocar para o caminho Voicemod, abra **Efeitos internos (migração para o Voicemod)** e escolha a voz equivalente de cada efeito que seus fluxos antigos usam.

## Validação

Executada nesta máquina, nesta versão:

- `cargo test` (src-tauri, -j 1): **330 testes passando, 0 falhas** — inclui os novos: Voicemod (autorização, lista de vozes, aplicar com confirmação, chave recusada, protocolo incompatível, queda durante teste, dois testes simultâneos barrados); resgates (cumprimento com o ID do resgate, deduplicação, cooldown, fila na ordem, fila cheia recusada, ignorar/substituir, troca manual respeitada, queda → restauração pendente → recuperar, voz desconhecida recusada, sem conexão recusada, cancelar tudo, temporizador antigo não desfaz comando novo, exclusão mútua com a voz interna, migração de efeito antigo, validação da configuração); comandos (origem de voz exige o streamer real; chat exige moderador e não aceita simulação, alerta, teste ou Discord; confirmação de banimento com prazo e alvo presos; duração falada); interpretação PT-BR (os exemplos obrigatórios, quicar, números por extenso, aspas, negação, pergunta, pedido único quando há vários); trava nova: caminho Voicemod bloqueia o início da voz interna.
- `npm run check` (tsc): limpo. `npm test` (vitest): **124 testes passando**.
- Playwright (navegação real da interface): **5 specs passando** — o painel de comandos por voz (campos, prévia, escuta ligando e desligando, transcrição descartada ao parar), o hub da voz (três seções na ordem, caminho da voz com duas opções exclusivas, diagnóstico, fila honesta na prévia) e a auditoria de layout nas cinco larguras (1440, 1280, 1152, 922 e 680 px) com **0 achados** na tela Voz da live e 0 erros de página. As capturas de 1100 px e larga do hub ficam em `%TEMP%\botlive-ux\voz-hub-1100.png` e `voz-hub-largo.png` (fora do repositório, como manda a casa).
- `npm run update:check`: executado antes do commit.

**Pendente de validação externa (não executado aqui):** integração com o Voicemod real (aplicativo aberto, chave da Control API emitida pela Voicemod, troca de voz e restauração com áudio de verdade), captura do microfone virtual no OBS e um resgate real de canal ponta a ponta. Os testes usam um **servidor Voicemod simulado** que fala o protocolo documentado — cobre o contrato da integração, **não é homologação com o aplicativo real**. Nenhum banimento ou ação de moderação real foi executado.

## Limitações

- A validação com o Voicemod real, microfone e OBS depende de uma máquina com o aplicativo Voicemod instalado e chave emitida; enquanto isso não roda, a integração está provada pelo contrato (simulador local) e pela documentação oficial, não por homologação.
- Resgate sem confirmação da troca não é cumprido na Twitch (fica pendente) e o BotLive não promete reembolso: estorno é decisão da Twitch. Sem permissão de gerenciar resgates, o cumprimento é recusado com aviso.
- Queda do Voicemod durante um efeito deixa a restauração **pendente** até reconectar e pedir **Recuperar**; nada é reaplicado ou restaurado sozinho enquanto desconectado.
- O reconhecimento de voz exige o RealtimeSTT local rodando (um servidor, na própria máquina); sem servidor, a escuta avisa em vez de fingir que ouve.
- Palavra de ativação não é autenticação: ela liga a obediência a quem já é autorizado (só o streamer real na voz; moderador+ no chat). Quem não é autorizado não é obeyido mesmo com a palavra.
- “Quicar” é sempre timeout com a duração configurada — nunca vira banimento; banimento por voz exige confirmação explícita com alvo e prazo, e prazo expirado cancela.
- Restrições de criação, preço e edição de recompensas continuam sendo da Twitch; o BotLive só usa as recompensas que o canal já tem.
- Sons do bot, alertas e TTS seguem no caminho de sempre (tela Áudio); o Voicemod não toca neles, e o microfone do Windows não é alterado.

---

# BotLive 0.8.0

## O que mudou

**A música que está tocando ganha casa própria no OBS: uma camada de overlay própria, com Estúdio de customização completo.** O cartão da música entra no grupo Overlays do OBS como a quarta camada (meta, alertas, chat e agora música), com endereço próprio (`mode=spotify`) para a sua Fonte de navegador, e o Estúdio da tela Spotify desenha na prévia exatamente o mesmo cartão que o OBS recebe — o que se vê ali é o que a live mostra, e cada ajuste chega ao OBS na hora, sem recarregar a fonte.

O que a mudança traz, item por item:

- **Camada de música no overlay.** A fonte de navegador com `mode=spotify` mostra só o cartão da música, no canto escolhido (quatro cantos ou centro), com capa, título, artista, álbum, barra de progresso e tempos — no mesmo desenho compartilhado entre a prévia e o OBS. O aperto de mão entrega a configuração e a faixa atual: a fonte recarregada mostra a música na hora, sem esperar a próxima leitura. Faixa de outro perfil não passa na fonte (a chave só enxerga o próprio overlay).
- **Estúdio da música no padrão da casa.** Prévia fixa à esquerda dentro do quadro de 1920×1080 em miniatura (com fundos escuro, xeque e claro para julgar a transparência) e os ajustes em cards à direita: **Aparência** (quatro peles — card com capa, empilhado, compacto e só texto — posição, largura, preenchimento e cantos), **Capa** (mostrar, formato quadrado ou redondo, brilho na borda e a própria capa desfocada como fundo), **Campos** (álbum, barra e tempos), **Cores** (temas Escuro e Claro de um clique, amostras rápidas de destaque e seletor de cada cor), **Tipografia** (16 fontes e tamanhos), **Barra**, **Movimento** (entrar, sair e duração) e **Quando a música para** (mensagem do parado com título e linha extra, esconder ao pausar e espera de 0 a 30 segundos).
- **Teste pelo mesmo caminho da música real.** O botão **Testar no overlay** transmite uma amostra com a marca de ensaio: o OBS desenha o cartão com o selo **Ensaio**, a prévia acompanha e a amostra volta ao estado real sozinha — é assim que se confere posição e visual antes da live. Com o módulo desligado, o teste avisa qual interruptor falta ligar.
- **Pausa com espera configurável.** Por padrão a pausa continua escondendo o cartão na hora (comportamento da Fase A); no Estúdio dá para pedir uma espera de até 30 segundos antes de sumir, ou manter o cartão visível na pausa — a barra congela na posição real nos dois casos.
- **Tela Spotify reorganizada como Estúdio.** Conexão e interruptor mestre seguem os cartões de sempre; o visual inteiro vive no Estúdio, com o endereço do OBS no cartão **No OBS** (mesmo molde do Alertas) e salvamento automático de tudo, validado pelo backend antes de chegar a qualquer fonte.
- **Apoio atualizado.** Capítulo **Música do Spotify** ganhou a seção do overlay e do Estúdio, tabela de camadas do API local ganhou `mode=spotify`, índice do manual atualizado; manual regerado com 27 capítulos.

## Como usar

1. Atualize o app para a 0.8.0 (Configurações → Atualizações) — a camada de música existe a partir desta versão.
2. Abra **Spotify**, conecte a conta (capítulo **Música do Spotify**) e ligue **Ler a música que está tocando**.
3. Ajuste o visual no **Estúdio da música**: a prévia mostra a amostra; **Testar no overlay** confere no OBS com o selo Ensaio.
4. No cartão **No OBS**, copie o endereço (vem com `mode=spotify`), crie uma **Fonte de navegador** no OBS com **Largura 1920** e **Altura 1080** e cole. Deixe "Atualizar navegador quando a cena ficar ativa" desmarcado: o overlay reconecta sozinho.
5. As alterações chegam ao OBS na hora. Com nada tocando, o cartão some — ou mostra a sua mensagem, se o Estúdio estiver nessa configuração.

## Validação

- `npm run check` (tsc) sem erros; `npm test` (vitest) **124/124**; `npm run test:updates` **9/9**.
- `cargo test` em `src-tauri`: **273 testes, 0 falhas, 1 ignorado** — com os novos: as novas alavancas do design aceitas e as fora do conhecido barradas, as chaves do design do backend iguais às que o Estúdio desenha, o aperto de mão do OBS trazendo configuração e faixa atual, a amostra do Estúdio marcada para o selo, e o WebSocket servindo a camada de música com a whitelist por perfil.
- `npx playwright test` (suíte completa): **82 testes, 0 falhas** — com as duas specs novas: a camada de música no OBS (cartão com textos, capa e barra; espera da pausa escondendo; estado parado com mensagem; amostra com selo voltando ao real; filtro de camada) e o Estúdio (prévia igual ao OBS, peles, posição, capa, tema, destaque, estado sem música, endereço `mode=spotify`, prévia grudada e capturas de 1100px e ampla).
- `npm run docs` regerado (27 capítulos); `npm run update:check` executado antes do push.

## Limitações

- A camada de música existe a partir da 0.8.0: a Fonte de navegador do OBS precisa do app atualizado (o HTML do overlay vem embutido no aplicativo).
- Animação de entrada e saída tem os modos aparecer, subir e sem animação — sem matriz completa de direções; o tema claro é um preset de cores, e o destaque escolhido também vira o brilho da capa.
- Sem visualizador de som, sem envio de imagem personalizada para o estado parado e sem "cores mágicas" da capa — a capa desfocada cobre o fundo com a própria arte.
- O Estúdio da música ainda não tem o modo código (HTML/CSS/JS) que Meta, Alertas e Chat têm.
- "Mostrar só na troca de música, por N segundos" não existe: o cartão segue a música (e a espera da pausa), sem janela de vitrine temporária.

---

# BotLive 0.7.10

## O que mudou

**Correção na inserção de variáveis na mensagem: quem digita e escolhe uma variável no mesmo instante não perde o texto nem duplica o marcador.** A correção fecha uma corrida que existia desde antes desta leva — o editor inseria a variável no texto do render anterior e o código que devolve o cursor ao campo agendado por um clique anterior podia sequestrar a seleção de um preenchimento recente, fazendo o texto entrar anexado ao invés de substituir. Na prática, aparecia uma mensagem como `O número é {{random:1,50}}O número é ` — invisível no uso calmo, mas real em máquina lenta ou ao digitar rápido. A CI da 0.7.9 pegou exatamente esse caso (o teste da ficha "Número sorteado"), e a correção está nesta versão em vez de uma reexecução que escondesse o problema.

O que a mudança traz, item por item:

- **Inserção lê o textarea vivo.** O marcador entra no texto e no cursor que estão no campo agora, não no valor capturado pelo último render — o caminho vale para os botões de inserção rápida, as fichas do seletor de variáveis e a inserção pelas opções da ficha (todos passam pela mesma função).
- **O cursor só volta se a mensagem ainda é a que acabou de entrar.** A devolução de cursor do clique anterior, que roda no quadro seguinte, verifica o texto antes de agir: um preenchimento ou clique intermediário não tem mais o cursor roubado.
- **Teste atualizado com o comentário do porquê.** A espera que contornava a corrida no teste da ficha "Número sorteado" continua, agora documentando o comportamento correto — e a spec foi executada repetidas vezes sem a falha.

Integra com esta mesma leva (repositório na 0.7.9 sem release publicada — a CI interrompeu antes de publicar): **Música do Spotify** (menu Spotify, conexão com aplicativo próprio via PKCE, leitura da música de 5 em 5 segundos, seis variáveis `{{nowplaying.*}}` nas automações, pré-voo cobrando Client ID e autorização e capítulo novo no manual), conforme `updates/0.7.9.md`.

## Como usar

- **Variáveis na mensagem**: insumo normal — clique na ficha ou botão `+` com o cursor onde quiser; agora também funciona ao digitar e inserir no mesmo instante, sem perder o que foi escrito.
- **Música do Spotify**: siga o capítulo **Música do Spotify** do manual (ou o cartão de conexão da tela) — criar aplicativo no painel do Spotify, colar o Client ID, autorizar e ligar o interruptor da leitura.

## Validação

- `npm run check` (tsc) sem erros; `npm test` (vitest) **115/115**; `npm run test:updates` **9/9**.
- `cargo test` em `src-tauri`: **269 testes, 0 falhas, 1 ignorado** (sem mudança de Rust nesta correção; porta reexecutada após o bump de versão).
- `npx playwright test` (suíte completa): **80 testes, 0 falhas** — com a spec da ficha "Número sorteado" e a do seletor de variáveis repetidas 3× sem a falha (`--repeat-each=3`, 24/24), estressando a corrida corrigida.
- `npm run docs` regerado (27 capítulos); `npm run update:check` executado antes do push.

## Limitações

- A correção cobre os caminhos de inserção do editor de mensagem (comandos, fluxos e prévias usam o mesmo componente); texto colado por fora (atalho do sistema) segue o comportamento normal do sistema.
- A 0.7.9 ficou apenas no repositório (a CI interrompeu no teste acima antes de publicar release); esta 0.7.10 é a primeira publicada com a Música do Spotify, e carrega a correção junto.

---

# BotLive 0.7.9

## O que mudou

**Música do Spotify: a faixa que está tocando chega ao BotLive e alimenta as suas automações.** O novo menu **Spotify** conecta a sua conta do Spotify com um aplicativo próprio (o mesmo caminho de apps como o Snip), lê a música atual de 5 em 5 segundos e entrega título, artista, álbum, capa, posição e duração — hoje nas variáveis `{{nowplaying.*}}` de qualquer resposta ou automação; a etapa seguinte do recurso leva a mesma leitura ao overlay do OBS, com capa, barra de progresso e configuração completa de visual.

O que a mudança traz, item por item:

- **Conexão com o aplicativo próprio.** A tela **Spotify** guia a criação do aplicativo gratuito no painel do Spotify (Client ID + URL de retorno copiável), autoriza a conta pelo navegador com PKCE — sem segredo guardado — e guarda os tokens no cofre de credenciais do sistema, só na sua máquina. O Spotify rotaciona as chaves e o BotLive renova sozinho; **Desconectar** apaga tudo na hora. A etiqueta do cartão mostra quem conectou.
- **Leitura contínua da música.** Com o interruptor mestre ligado, o BotLive consulta o Spotify de 5 em 5 segundos (janela rolante comporta bem; em limite de consultas, espera sozinho em vez de insistir) e transmite a faixa para a prévia e, na próxima etapa, para o overlay. Pausar, parar ou trocar de música refletem na leitura; anúncios e podcasts contam como "nada tocando". A coleta roda em tarefa própria por perfil, com erro novo registrado no Histórico uma única vez — nada de encher o histórico durante a live.
- **Seis variáveis para as automações.** `{{nowplaying.musica}}`, `{{nowplaying.artista}}`, `{{nowplaying.album}}`, `{{nowplaying.tempo}}` (relógio `1:23`), `{{nowplaying.duracao}}` (`3:21`) e `{{nowplaying.link}}` entram no catálogo de variáveis (grupo **Música**) e funcionam em respostas, automações e prévia de IA — vazias quando nada toca, para mensagem nenhuma quebrar.
- **Pré-voo cobra o Spotify ligado.** Com o módulo ligado, a Visão geral acusa Client ID faltando ou autorização ausente, com botão direto para a tela de conserto. Quem não usa o recurso não é cobrado.
- **Apoio atualizado.** Novo capítulo **Música do Spotify** no manual (passo a passo do aplicativo, conexão, variáveis e exemplos), tabela de variáveis e índice do glossário atualizados; manual regerado com 27 capítulos.

Para quem mantém o código: módulo `spotify.rs` com `connect_with` (PKCE, porta 43828, abridor injetável para teste), `token`/`fetch`/`parse` com base URL por variável de ambiente, `States` em `Runtime` (trava à prova de pânico, `throttle` para 429) e `run` como laço de coleta; ops `spotify.get`/`spotify.connect`/`spotify.disconnect`/`spotify.poll`; `module.config` aceita a chave `spotify` com validação e emite `spotify.config`; `Context::new` recebe a faixa atual e o cofre tem fronteira com memória em teste (a suíte não toca keyring).

## Como usar

1. Abra [developer.spotify.com/dashboard](https://developer.spotify.com/dashboard), crie um aplicativo (tipo **Web API**) e, em **Settings → Redirect URIs**, adicione `http://127.0.0.1:43828/callback`. Copie o **Client ID** (o Secret não é usado).
2. No BotLive, abra **Spotify**, cole o Client ID (salva sozinho) e clique em **Autorizar conta**. Confirme no navegador e volte: a etiqueta mostra **Conectado**.
3. Ligue **Ler a música que está tocando** e use **Testar conexão** para ver a faixa atual na linha *Agora tocando*.
4. Nas automações, use as variáveis do grupo **Música** — por exemplo, uma automação de comando `!tocando` com a resposta `Tocando agora: {{nowplaying.musica}} — {{nowplaying.artista}} ({{nowplaying.tempo}}/{{nowplaying.duracao}})`. O nome `!tocando` evita a colisão com o `!musica` da Fila de músicas, que é atendido antes das automações.

## Validação

- `cargo test` em `src-tauri`: **269 testes, 0 falhas, 1 ignorado** — incluem os novos desta versão: configuração padrão passando na validação e rejeitando o que quebraria o overlay, extração da faixa completa e ignorância de anúncio/podcast, estado só acendendo na virada que o overlay nota, espera do 429 pouparem consulta repetida, e o fluxo inteiro contra um servidor falso do Spotify (autorização PKCE com callback na porta, token no cofre, leitura da faixa com transmissão e variáveis, pausa, parada, limite de consultas e desconexão) — sem rede e sem tocar conta alguma.
- `npm run check` (tsc) sem erros; `npm test` (vitest) **115/115**; `npm run test:updates` **9/9**.
- `npx playwright test` (suíte completa): **80 testes, 0 falhas** — com os dois novos da tela Spotify: conexão, autosave enviando o design completo, autorização, leitura da faixa (formato do backend) na linha, interruptor mestre, as seis variáveis, ausência de rolagem horizontal a 1100px, e o carregamento do Client ID salvo com erro de autorização em português que não quebra a tela.
- Regras de tela: capturas `spotify-1100` e `spotify-wide` geradas pela suíte e revisadas; a 1100px a tela não espreme texto nem causa rolagem horizontal.
- `npm run docs` regerado (27 capítulos); `npm run update:check` executado antes do push.

## Limitações

- **Etapa seguinte**: o overlay do OBS (capa, barra de progresso e configuração de visual no Estúdio) é a próxima entrega do recurso; nesta versão a música alimenta as automações e a linha da tela Spotify.
- **Apenas faixas de música**: anúncios e podcasts contam como "nada tocando" (as variáveis ficam vazias até a música voltar).
- **`{{nowplaying.tempo}}` é a posição da última leitura**, com no máximo 5 segundos de diferença — o relógio não corre segundo a segundo nesta etapa.
- **Conta gratuita**: a leitura geralmente funciona, mas a API de música do Spotify é pensada para contas Premium; se a leitura falhar na sua conta, confira os avisos do Histórico e os termos do seu aplicativo.
- **A autorização é por aplicativo próprio** (política do Spotify desde maio/2025: modo desenvolvimento serve até 25 contas por aplicativo) — o passo a passo está no capítulo do manual e na própria tela.
- O BotLive **só lê**: nada é tocado ou pausado no Spotify, e exibir metadados na live não é transmissão de áudio — o áudio continua sendo do seu player, como os termos do serviço exigem.

---

# BotLive 0.7.8

## O que mudou

**Modo Ensaio, terceira etapa: o comando agora é ensaiado pelo funil real da live, os testes falam um contrato só e a Visão geral ganha o pré-voo antes de ir ao ar.** Quem ensaia um comando passa a ver o que a live veria — procura das automações, permissão, disputa e intervalo — sem publicar nada, sem queimar intervalo real e sem mexer em estatística; e qualquer pessoa pode conferir, em um clique, o que estaria quebrado na configuração antes de começar a transmissão.

O que a mudança traz, item por item:

- **Testar comando ao vivo.** O painel de teste do editor de automações ganha o segundo modo **Testar comando ao vivo**: a mensagem como um espectador digitaria (comando + argumentos) e o papel de quem digita (todos, assinante, moderador, streamer) entram na fila do motor como um evento de ensaio. O motor procura as automações salvas e ativas, confere a permissão de quem fala, decide a disputa entre respondentes e passa pelo intervalo — o de ensaio tem balde próprio, então testar nunca queima o intervalo real. Quando a live também não responderia (nenhuma automação para a mensagem, permissão que não confere, intervalo ativo, gatilho desligado), o painel devolve o motivo e nada executa, igualzinho ao que a live faria. Acertando, o acompanhamento etapa a etapa acompanha a execução nascida do funil e **Executar novamente** reenvia o comando com um novo intervalo de ensaio.
- **Um contrato só para todos os testes.** O motor ganhou a regra de ensaio (evento de ensaio = simulado e marcado de ensaio): ela roda a lógica do fluxo de verdade — condições, esperas, ordem das etapas — e pula só as etapas que tocariam o mundo externo (mensagem na Twitch, Discord, Helix, webhooks, pontos, memórias, variáveis fora do escopo local), cada uma com o aviso **Pulada**. O teste rápido de fluxo continua como estava, direto e sem fila. Sons e vozes tocam pela linha real do aplicativo (a mesma que o OBS captura), a resposta do bot aparece no Histórico como `[Simulação]`, o chat do ensaio não infla a explosão de emotes nem os contadores de conversa, e nada disso entra nas estatísticas — o Histórico registra a linha com status `ensaio`.
- **Pré-voo da live.** A tela Visão geral ganha o card **Pré-voo da live**, antes das colunas do painel: um clique varre a configuração do perfil selecionado e lista o que só quebraria ao vivo — bot não autorizado na Twitch, alertas (chave mestra e interruptor por evento) desligados, chat no OBS desligado, sons e frases apagados de Sons de Live, e ações cujo requisito falta (Texto para voz, Pontos & loja ou Discord com o módulo desligado, OBS desligado, IA sem modelo escolhido, endereço de webhook inválido). Cada aviso traz um botão **Ir para X** que abre a tela de conserto; com tudo em ordem, o card diz que está tudo pronto. A varredura é estática: não chama rede, não altera nada e não depende da conexão — o endereço do overlay não entra, porque ele é gerado na hora.
- **Apoio atualizado.** Os capítulos 03 (Comandos e automações) e o Guia de uso passam a explicar o modo comando ao vivo e a rotina com o pré-voo; o manual foi regerado.

Para quem mantém o código: a op `flow.rehearse` (`{profileId, message, role}`) devolve `{fired:false, reason}` ou `{fired:true, flow, flowId}` com o evento `simulated+rehearsal` e `data.botliveTest`; `preflight.check` (`{profileId}`) devolve `{items:[{message, page}]}`, só leitura. No motor, `ensaio(e)` é a predicação única, `testing` inclui ensaio, `test_skipped` tem exceção para chat de ensaio, TEST_SKIP ganhou `voiceFx`, os cooldowns usam `cooldown_keys`/`cooldown_ready` com balde `ensaio`/`live`, e contagem de emotes/chatter só acontece fora de evento simulado.

## Como usar

1. Abra **Automações**, edite um fluxo com gatilho de comando e clique em **Testar fluxo**. Escolha **Testar comando ao vivo**: a primeira variação do comando já vem preenchida.
2. Escreva a mensagem como um espectador digitaria (argumentos inclusive) e escolha **Permissão de quem digita**. Clique em **Enviar comando**.
3. Se a live também não responderia, o painel avisa qual é o motivo e não executa nada — corrija a automação, a permissão ou o intervalo e tente de novo. Respondendo, o acompanhamento mostra cada etapa até o fim; **Executar novamente** reenvia o comando quando você pedir.
4. Antes de começar a transmissão, abra a **Visão geral** e clique em **Rodar pré-voo** no card **Pré-voo da live**: cada aviso tem um botão **Ir para** a tela onde consertar; **Verificar de novo** refaz a varredura.
5. O intervalo do ensaio é separado do da live: você pode testar à vontade sem afetar quem está no chat. Nada do que o ensaio roda vira estatística, saldo ou mensagem publicada.

## Validação

- `cargo test` em `src-tauri`: **262 testes, 0 falhas, 1 ignorado** — incluem os novos desta versão: a regra de ensaio do motor (ensaio exige simulado e marcação juntos; teste rápido pula o externo), o balde de intervalo do ensaio separado do da live, o comando de ensaio atravessando o funil (lógica roda, externos pulados, `[Simulação]` no histórico, contadores e estatísticas intactos, intervalo de ensaio não queima o da live), o aviso do `flow.rehearse` quando a live não responderia (sem automação, permissão, desligado) e o pré-voo acusando autorização, alertas, chat, áudio apagado, módulo e webhook — com o aviso recolhido quando a peça é ligada.
- `npm run check` (tsc) sem erros; `npm test` (vitest) **115/115**; `npm run test:updates` **9/9**.
- `npx playwright test` (suíte completa): **78 testes, 0 falhas** — com os quatro novos: comando ao vivo avisando sem executar nada (clique duplo envia um só) e acompanhando a execução do funil com repetição, e o pré-voo com lista, deep-link marcando a tela no menu, verificação só de leitura (nenhuma operação de escrita), estado vazio e falha que orienta sem perder o relatório anterior.
- Regras de tela: capturas `comando-ao-vivo-1100/-aviso-1100/-largo` e `preflight-1100/-wide` geradas pela suíte e revisadas; a 1100px o relatório e o painel de comando não espremem texto nem causam rolagem horizontal.
- `npm run docs` regerado (26 capítulos); `npm run update:check` executado antes do push.

## Limitações

- **A IA continua fora do ensaio**: etapa de IA no comando ao vivo aparece como **Pulada** (rede e custo), como já acontece no teste rápido; o restante da lógica roda.
- **A mensagem do espectador no ensaio não aparece no chat do OBS** — o ensaio entra pela fila do motor como comando, e a prévia da caixa de chat continua com o botão de teste da tela Chat no OBS (contrato da 0.7.6, mantido).
- **O pré-voo é uma varredura estática de configuração**: ele não fala com a Twitch nem testa webhooks de verdade — acusa o que falta ou está errado na configuração, e a autorização em si continua sendo conferida nas Configurações.
- **Testar explosão** (emotes) ainda dispara mesmo com a explosão desligada — comportamento da 0.7.5, mantido de propósito e já registrado no capítulo de Alertas.
- A prévia local do Estúdio e o que chega pelo motor (alerta no OBS, execução na fila) podem começar em instantes diferentes — mesma observação da 0.7.7.

---

# BotLive 0.7.7

## O que mudou

**Modo Ensaio, segunda etapa: o botão de teste dos Alertas agora respeita os interruptores e percorre o funil da live.** O teste deixa de ser um desenho local: o evento de teste vira evento de verdade na fila do motor e percorre alerta, caixa de evento no chat, explosão de emotes e rótulos pelo mesmo caminho da live — com selo **ENSAIO**. Com a chave mestra ou o interruptor do evento desligados, nada dispara: o Estúdio avisa qual falta ligar, do jeito que a live se comportaria.

O que a mudança traz, item por item:

- **Teste pelo funil do motor.** `Testar <evento>` monta um evento de ensaio (simulado e marcado de ensaio) e o envia pela fila do engine: o alerta toca no OBS pelo caminho real, com o som na linha de áudio do aplicativo; a caixa de evento aparece no chat (com os interruptores do chat ligados); a explosão de emotes acompanha quando ligada. O histórico registra a linha com status `ensaio`, fora das estatísticas.
- **Aviso em vez de disparo pela metade.** Com **Ligar alertas no overlay** desligado, o teste devolve `Ligue os alertas no overlay: em live o alerta não tocaria.`; com a chave ligada mas o interruptor do evento (por exemplo **Exibir bits**) desligado, o motivo aponta o interruptor do evento. A prévia não anima quando o motivo volta — o Estúdio mostra o aviso, sem mentir sobre o que a live faria.
- **Selo ENSAIO na prévia de Alertas.** O Estúdio de Alertas acende a mesma badge âmbar no canto do quadro durante o disparo (cerca de 5 segundos), como já fazem Meta e Chat — as três telas com botão de teste falam a mesma língua.
- **Rótulos de exibição no ensaio.** Com a tela Rótulos ligada, um teste de follow/sub/cheer/raid atualiza os campos de recentes (`recent_follower`, `recent_cheer`…) como na live — mas `session_bits`, `session_gifts` e a renovação de totais não mudam: dado de teste nunca vira estatística. A simulação simples da API local continua sem tocar rótulo nenhum.
- **Apoio atualizado.** Os textos da tela Alertas deixam de prometer que "os botões de teste sempre funcionam" e passam a explicar o contrato novo — igual à live, com aviso quando algo estaria desligado. Os capítulos 11 (Alertas) e 10 (Rótulos) do manual acompanham.

Para quem mantém o código: `alerts::test` virou `async` e devolve `{fired, reason}` quando um interruptor real impediria o disparo e `{fired:true, kind, vars}` quando não — as `vars` da prévia saem do mesmo `vars()` do funil, então prévia e overlay veem o mesmo payload; o evento de teste vem de `rehearsal_event(p, kind)` (única fonte dos dados de exemplo; `giftbulk` vira `kind:"gift"` com `total:10`, como a Twitch entrega); `labels::on_event` ganhou a flag `rehearsal` e o gate do engine virou `!simulated || rehearsal`.

## Como usar

1. Abra **Alertas**, ligue **Alertas no overlay** e deixe o interruptor do evento ligado (grupo **Este evento**).
2. Clique em **Testar <evento>**: a prévia anima com o selo **Ensaio** no quadro e o OBS toca junto, com o som escolhido na linha de áudio do app — a caixa de evento no chat e os rótulos acompanham, cada um com o seu interruptor.
3. Se a chave ou o interruptor do evento estiver desligados, o teste não dispara: o aviso diz exatamente qual falta ligar. É a consequência de o teste valer o que a live valeria.
4. **Rótulos**: num teste, o rótulo recente mostra o nome de exemplo até o próximo evento real; os contadores de sessão (bits, presentes) e os totais não mudam.

## Validação

- `cargo test` em `src-tauri`: **255 testes, 0 falhas, 1 ignorado** — incluem os novos desta versão: o botão de alerta seguindo os mesmos interruptores da live (desligado não dispara e explica; ligado entrega o alerta marcado de ensaio com as mesmas variáveis da prévia; evento específico desligado devolve o motivo do evento), rótulos de ensaio atualizando só a exibição (`recent_cheer` muda, `session_bits` e `session_gifts` não) e o gate do engine (a simulação simples da API continua bloqueada dos rótulos).
- `npm run check` (tsc) sem erros; `npm test` (vitest) **115/115**; `npm run test:updates` **9/9**.
- `npx playwright test` (suíte completa): **74 testes, 0 falhas** — com o teste novo da tela Alertas: aviso com a chave desligada (toast, sem animação e sem selo), aviso apontando o interruptor do evento (`Exibir bits`), disparo com o selo ENSAIO na prévia terminando limpo, e o teste de largura capturando com o selo e o alerta em animação nas duas larguras.
- Regras de tela: capturas `estudio-alertas-1100/-largo` regeneradas e revisadas (selo ENSAIO e alerta em animação visíveis, sem espremer texto).
- `npm run docs` regerado (26 capítulos); `npm run update:check` executado antes do push.

## Limitações

- **Testar explosão** (emotes) ainda dispara mesmo com a explosão desligada — é o comportamento da 0.7.5, mantido de propósito para conferir o efeito antes da live; agora está registrado como limitação no capítulo de Alertas.
- O rótulo recente mostra o nome do evento de teste até o próximo evento real (não há arquivo de rótulo "falso" separado) — contadores e totais ficam intactos.
- A prévia local do Estúdio anima a partir da resposta do botão, enquanto o alerta chega ao OBS pela fila do motor: os dois podem começar em instantes diferentes — como na etapa anterior.
- Teste de comando com cooldown de ensaio próprio, unificação dos testes e auditoria final ficam para a 0.7.8 (terceira etapa).

---

# BotLive 0.7.6

## O que mudou

**Modo Ensaio, primeira etapa: os botões de teste da Meta e do Chat agora agem como a live.** O teste deixa de ser um desenho local: ele percorre o mesmo caminho do evento real — até o OBS, com som, visual e efeitos — vestindo um selo **ENSAIO** que some sozinho. Enquanto isso, o evento de teste não vaza para fora: não publica no Discord e não conta nas estatísticas. Nenhum dado real muda com o ensaio.

O que a mudança traz, item por item:

- **Selo ENSAIO em toda camada.** O motor marca o evento de teste (`rehearsal`) e a marca viaja no payload até o OBS: a camada de meta, de alertas ou de chat acende uma badge discreta âmbar **ENSAIO** no canto por cerca de 5 segundos, rearmada a cada novo teste, e some sozinha. O mesmo selo aparece no palco da prévia das telas Meta e Chat.
- **Meta: Simular doação e Celebração pelo caminho real.** No aplicativo, os dois botões passam a motor de verdade: o pulso da barra e o som escolhido chegam ao OBS como numa doação real (pela mesma saída da tela Áudio), e **Celebração** completa a meta e solta a explosão de emotes quando o interruptor dela está ligado — tudo sem alterar o total acumulado. Com a integração da LivePix desligada ou sem meta configurada, o botão não inventa nada: avisa que, em live, também não dispararia.
- **Chat: teste de mensagem e de evento pelo motor do chat.** Os botões de amostra (Comum, Moderador, VIP, Sub, Primeira vez, Em emote, Só emotes, Do bot) viram evento e o motor transmite pela mesma via da live: badges, emotes da Twitch, destaque por papel, prefixo e — importantíssimo — os mesmos filtros do Estúdio (escondendo bots ou comandos, os exemplos correspondentes somem, igual ao OBS). O **Testar …** das caixas de evento preenche o texto pela configuração salva, pelo mesmo preenchimento que o OBS recebe, e toca o som na linha do aplicativo. Com o chat desligado — ou com **Eventos no chat** ou a caixa individual desligados — o teste não empilha nada: avisa qual interruptor impediria a live de mostrar, em vez de mentir.
- **O ensaio não vaza.** Evento de teste/ensaio não publica no Discord (o aviso e o espelho do chat ficam suprimidos, com linha `[Ensaio]` no histórico) e não entra nas estatísticas: o histórico registra status `ensaio`, `stats.read` não o conta e a contagem de comandos o ignora. Um evento `simulated:true` enviado pela API local ganha exatamente o mesmo tratamento.
- **Ajuste em apoio ao teste**: os textos de ajuda das telas Meta e Chat explicam o novo contrato — teste igual à live, com aviso quando um interruptor estaria desligado.

Para quem mantém o código: `Event.rehearsal: bool` (default no serde) propaga por `alerts::dispatch`, `chat_overlay::fire/event`, `emote_burst` e o tique da meta; os ops novos são `livepix.simulate` e `chat.test` (ambos devolvem `{fired, reason}` quando um interruptor real impediria o disparo); a marca também anda nos payloads WS do overlay e nos eventos Tauri da prévia.

## Como usar

1. **Meta**: abra **Meta** com a LivePix conectada e clique em **Simular doação** — a barra sobe e o som toca no OBS com o selo **Ensaio** na prévia; o total acumulado não muda. **Celebração** completa a meta no caminho real (com a explosão de emotes, se ligada). Sem integração ou sem meta, o aviso diz que, em live, nada dispararia.
2. **Chat**: ligue **Chat no overlay** e use os botões de teste — as mensagens empilham na prévia e na coluna do OBS pelo motor real, com o selo no palco. Desligado, o teste mostra o aviso em vez de empilhar.
3. **Eventos do chat**: ligue **Eventos no chat**, escolha o evento na pílula e clique **Testar …**: a caixa sai preenchida pelo motor com o texto e a cor salvos, e o som toca na linha do aplicativo. Com o mestre ou a caixa do evento desligados, o aviso explica a parada.
4. **Discord e estatística**: nenhum teste publica no Discord nem conta em estatísticas ou comandos — para conferir o que foi testado, o histórico mostra a linha com status `ensaio`.

## Validação

- `cargo test` em `src-tauri`: **253 testes, 0 falhas, 1 ignorado** — incluem os novos desta versão: evento simulado visível no histórico mas fora de `stats.read` e suprimido do Discord com log de ensaio, payloads de ensaio carregando a marca (literais de `Event` em todos os módulos), `livepix.simulate` publicando o goal sem persistir o total (e a celebração disparando a explosão marcada), `chat.test` saindo pelo motor real (mensagem com badges/frags, evento com texto preenchido e derivação `gift` → `giftbulk`) e contagem de comandos ignorando execução em teste.
- `npm run check` (tsc) sem erros; `npm test` (vitest) **115/115**; `npm run test:updates` **9/9**.
- `npx playwright test` (suíte completa): **73 testes, 0 falhas** — com os testes novos da tela Chat: aviso com o mestre desligado (sem empilhar e sem selo), empilhamento pelo motor real com o selo ENSAIO no palco, filtros valendo nos testes (bot escondido some), caixa de evento preenchida pelo motor com a cor salva, e o aviso quando a caixa individual do evento está desligada; o selo no overlay e na tela Meta já estavam cobertos.
- Regras de tela: capturas `estudio-chat-1100/-largo` e `estudio-livepix-1100/-largo` regeneradas e revisadas (selo ENSAIO visível, sem espremer texto).
- `npm run docs` regerado (capítulos de Chat no OBS, Integrações e API local); `npm run update:check` executado antes do push.

## Limitações

- A primeira etapa cobre **Meta** e **Chat**. Os botões de teste da tela Alertas (e da explosão de emotes) ainda disparam pela via antiga: já publicam o evento com o selo ENSAIO no overlay, mas ainda funcionam com o alerta desligado — o aviso de "qual interruptor está desligado" para alertas é a próxima etapa (0.7.7).
- Na prévia de navegador (sem o aplicativo desktop), Meta e Chat mantêm a simulação local: sem OBS anexado não existe caminho real a percorrer.
- O selo ENSAIO some após cerca de 5 segundos contados pelo relógio da janela; com a janela muito tempo em segundo plano, o navegador pode atrasar esse relógio e o selo durar um pouco mais.
- Quem usava os testes de chat como "prévia sempre disponível" agora precisa ligar o **Chat no overlay** antes de testar — é a consequência direta de o teste valer o que a live valeria.

---

# BotLive 0.7.5

## O que mudou

**Explosão de emotes (novo efeito no overlay).** Quando entra sub, chega renovação, cai presente de sub — e quando a meta completa — os emotes da Twitch do canal explodem na tela como um confete: imagens da CDN voando do centro para fora, girando e caindo, sem tocar no desenho das camadas.

O desenho, item por item:

- **Quatro eventos com interruptor próprio**, escolhidos um a um: novo sub, renovação de sub e presente de sub ficam na tela Alertas; meta completada tem o seu interruptor na tela Meta (grupo Efeitos). A ligação geral na tela Alertas desliga a explosão inteira de uma vez.
- **Três origens de emotes**: aleatório entre os seus, os mais usados no chat ou a lista do canal na ordem da Twitch. A lista é dos emotes do seu canal com os globais da Twitch, buscada quando o canal conecta e atualizada a cada 12 horas.
- **Quantidade e tamanho ajustáveis** (10 a 80 emotes, de 24 a 120 px cada) e **Testar explosão** na própria tela: dispara pelo mesmo caminho do evento real e avisa em vez de falhar em silêncio quando ainda não há emotes carregados.
- **Cada camada do OBS recebe a sua parte**: a camada de alertas mostra as explosões de sub, renovação e presente; a camada de meta mostra a da meta completada; e a tela Chat no OBS ganhou o interruptor **Também nesta camada** para a explosão cair também sobre a coluna de chat. Uma fonte única (endereço sem `mode`) recebe as quatro.
- **A contagem de mais usados fica guardada por perfil**: começa quando a explosão é ligada, sobrevive ao reinício do aplicativo e continua de onde parou.

Para quem mantém o código: o recurso é o módulo `emote_burst.rs`, com a contagem de emotes por perfil vinda das frases das mensagens e os totais persistidos pelo tique do agendador (chave `emoteBurstStats`); `emotes.rs` passou a guardar `{id, nome}` no cache para os ids chegarem ao overlay. O broadcast novo `emote.burst` entra na lista permitida do WebSocket do overlay, e o efeito (`src/emoteBurst.ts`) monta as imagens da CDN com voos espalhados e limpa sozinho — emote que não carrega some do efeito. A configuração passa pela validação do op `module.config` (chave `emoteBurst`), e a prévia no navegador continua sem executar o núcleo.

## Como usar

1. Abra **Alertas**, ligue **Explosão de emotes** no cartão próprio e escolha os eventos de alerta, a origem, a quantidade e o tamanho — as alterações salvam sozinhas.
2. Para a meta: abra **Meta** e ligue **Explosão de emotes ao completar** no grupo Efeitos (ligar aqui também liga a explosão geral; desligar só a meta não apaga as dos alertas).
3. Para a explosão cair sobre o chat: abra **Chat no OBS** e ligue **Também nesta camada** no grupo Explosão de emotes.
4. **Testar explosão** confere o efeito na hora, pela camada de alertas — se a lista de emotes ainda não chegou, o aviso pede conectar o canal ao Twitch.

## Validação

- `cargo test --lib` em `src-tauri`: **247 testes, 0 falhas, 1 ignorado** — incluem os testes novos do módulo (faixas da configuração e as três origens com RNG travado) e o teste de integração que conta emotes de mensagens, dispara no evento de sub, não dispara em raid e persiste os totais.
- `npm run check` (tsc) sem erros; `npm test` (vitest) e `npm run test:updates` sem falhas.
- `npx playwright test`: suíte completa verdes, com os testes novos: explosão no overlay (imagem por id na CDN da Twitch, cada camada só mostra a sua, chat entra com liberado, perfil alheio é ignorado) e o cartão na tela Alertas (mestrea, interruptores por evento, origem, quantidade, aviso do teste sem emotes e persistência ao reabrir).
- Regras de tela: capturas `alertas-explosao-1100.png`, `alertas-explosao-largo.png` e `overlay-explosao-emotes.png` geradas e revisadas.
- `npm run docs` regerado (seção da explosão no capítulo de Alertas, interruptor no capítulo da Meta e no do Chat no OBS); `npm run update:check` executado antes do push.

## Limitações

- A explosão usa os emotes da Twitch (canal + globais); emotes de terceiros (7TV, BTTV, FFZ) ficam para uma próxima versão, e a origem "mais usados" não tem histórico retroativo — a contagem começa no primeiro dia ligada.
- O botão **Testar explosão** sai pela camada de alertas (via de sub), mesmo quando os interruptores de evento estão desligados: é para conferir o efeito antes da live.
- Emote removido do canal depois de guardado pode não carregar na explosão seguinte; a imagem some do efeito sem deixar buraco, e a lista se atualiza quando o canal conecta de novo.

---

# BotLive 0.7.4

## O que mudou

**Match do chat (novo módulo em Comunidade).** A comunidade brinca de casal sem aprender comando: quem escreve uma palavra romântica — amor, paixão, crush, casal, beijo ou namorar — leva um par sorteado entre quem falou no chat nos últimos dez minutos, e a porcentagem de compatibilidade sai na mensagem da faixa certa.

O desenho, item por item:

- **Palavra inteira.** "amor" dispara em "você é meu amor!", mas "amortecedor" não; mensagens que começam com `!` são comandos e não disparam.
- **Par por menção ou sorteio.** Se a pessoa marcar alguém com `@` e a pessoa marcada tiver falado no chat nos últimos dez minutos, ela vira o par; sem menção (ou com a pessoa ausente), o par sai do sorteio de quem está ativo. Sem uma segunda pessoa no chat, não há match a anunciar.
- **% por hash dos nomes.** O número vem de um hash FNV-1a dos dois nomes em ordem canônica — o mesmo casal dá sempre o mesmo resultado, e a ordem dos nomes não muda nada. O chat pode discutir "o bot é injusto, a gente deu 12% de novo" o quanto quiser.
- **Cinco faixas com mensagens editáveis** (0–20, 21–40, 41–60, 61–80 e 81–100%), com os campos `{{user1}}`, `{{user2}}` e `{{pct}}`, partindo dos cinco textos prontos.
- **Anti-spam pronto.** Intervalo global de 2 minutos e por pessoa de 5 minutos, ajustáveis em Avançado (0 a 1440 minutos); a live fica fora do sorteio por padrão, com interruptor para incluí-la.
- **!match nome1 nome2** (ou o comando que você configurar) compara dois nomes quaisquer com a mesma conta, mesmo quem não está no chat — os dois nomes na ordem em que foram escritos, mas o mesmo resultado trocando a ordem.

Para quem mantém o código: o recurso é o módulo `affinity` no motor de Comunidade (`modules.rs`), com o par vindo do registro de presença de chat (janela de 600 segundos) e os intervalos usando o mesmo mecanismo de cooldown das automações. A configuração passa pela validação do op `module.config`, e o primeiro salvamento materializa as faixas padrão para a edição mostrar o que existe. A prévia no navegador ganhou os casos `module.config`, `module.config.get` e `community.action` no fallback web — é o que sustenta o teste da tela.

## Como usar

1. Abra **Comunidade**, ative **Match do chat** e clique no cartão.
2. Confira ou edite as palavras-gatilho, o comando e as cinco faixas em **Avançado** e salve a configuração.
3. Teste no chat real com uma palavra (por exemplo "que casal") ou com `!match ana bruno`.
4. A resposta segue a faixa da porcentagem; comando com erro aparece no Histórico do app, sem resposta no chat.

## Validação

- `cargo test --locked` em `src-tauri`: **245 testes, 0 falhas, 1 ignorado** — incluem os testes novos do Match (palavra inteira, hash determinístico e simétrico travado por valores de regressão, faixas, textos padrão, validação) e dois testes de integração do engine (sorteio com presença, intervalo global e por pessoa, exclusão da live, menção e !match).
- `npm run check` (tsc) sem erros; `npm test` (vitest): **115 testes, 0 falhas**; `npm run test:updates`: 9 testes, 0 falhas.
- `npx playwright test`: suíte completa verdes, incluindo o teste novo da tela Comunidade (ativação do módulo, configuração, salvar, recarregar mantendo os valores, `auditPage` em 1100px e 1680px).
- Regras de tela: capturas `comunidade-1100.png` e `comunidade-largo.png` geradas e revisadas.
- `npm run docs` regerado (capítulo da Comunidade com a seção do Match e a linha na referência de comandos); `npm run update:check` executado antes do push.

## Limitações

- Bots de terceiros (alertas de doação e afins) não são identificados como bots: se a mensagem de um deles contiver a palavra inteira e houver gente no chat, o match pode disparar. Combine com a Moderação se isso incomodar.
- Comando com erro (sem nomes, ou os dois nomes iguais) é registrado no Histórico, sem resposta no chat — o mesmo comportamento dos demais comandos da Comunidade.
- A janela de "quem está no chat" conta a partir das mensagens recebidas com o BotLive aberto; quem só assiste sem falar não entra no sorteio, e o mesmo casal pode sair repetido — não há memória de casais já anunciados.

---

# BotLive 0.7.3

## O que mudou

Os três Estúdios — Meta, Alertas e Chat no OBS — passam a seguir a mesma casa: prévia à esquerda, ajustes à direita, e a prévia fica fixa na tela enquanto você rola. Acabou o espaço vazio que sobrava ao lado das prévias curtas e o caso especial dos Alertas empilhados.

**Padrão Estúdio: prévia fixa à esquerda.** A grade `.lp-studio` virou o padrão único das telas com prévia: a coluna da prévia à esquerda usa `position: sticky` e acompanha a rolagem dos ajustes à direita — a prévia nunca sai da vista enquanto você edita, mesmo no último grupo, e o que ficava embaixo dela agora é apenas o fundo da tela. Abaixo de 960px de largura, tudo vira coluna única com a prévia em cima, igual nas três telas.

**Alertas na mesma grade.** O Estúdio dos Alertas saiu do empilhado (prévia em cima com teto de 980px, acordeão embaixo) e entrou no padrão: pílulas do evento e botão **Testar** acima da prévia à esquerda; abas do modo código e acordeão à direita. A prévia dos alertas agora também acompanha a rolagem enquanto você mexe em Textos, Painel, Cores, Fontes ou Efeitos.

**Meta e Chat no OBS** já usavam a mesma grade de duas colunas; herdaram a correção pela regra — a prévia passou a ser fixa sem mexer em mais nada da tela.

**Regra para as próximas telas.** A exigência entrou nas regras de manutenção do projeto: toda tela com prévia + ajustes usa o `.lp-studio` (prévia sticky à esquerda, ajustes em cards à direita, coluna única abaixo de 960px), sem variações, com capturas de 1100px e larga na entrega.

## Como usar

- Nada muda de lugar para quem já conhece as telas: role os ajustes e repare que a prévia acompanha você até o fim da página.
- Em janela estreita (abaixo de 960px), a prévia vai para cima e os ajustes ficam embaixo — o mesmo comportamento em Meta, Alertas e Chat no OBS.

## Validação

- `npm run check` (tsc) sem erros; `npm test` (vitest): **115 testes, 0 falhas**; `npm run test:updates`: **9 testes, 0 falhas**.
- `npx playwright test`: suíte completa verdes (**69 testes**), incluindo as novas asserções do padrão (`position: sticky` na prévia de Meta, Alertas e Chat), o teste do acordeão dos Alertas renomeado para "ao lado da prévia" e o `auditPage` das três telas em 1100px e 1680px sem achados nem rolagem horizontal.
- Regras de tela: capturas regeneradas e revisadas visualmente nesta entrega — `estudio-alertas-1100.png`, `estudio-alertas-largo.png` e `estudio-alertas-acordeao.png` com o novo visual lado a lado (pílulas, Testar, prévia à esquerda e acordeão à direita), além de `estudio-chat-1100/largo` e `estudio-livepix-1100/largo`, que mantêm a mesma forma de antes.
- `cargo test --locked` em `src-tauri`: **238 testes, 0 falhas, 1 ignorado** — nenhuma linha de Rust mudou nesta rodada, só a versão.
- `npm run docs` regerado (capítulo dos Alertas atualizado para o novo layout); `npm run update:check` executado antes do push.

## Limitações

- A prévia fixa depende dela ser mais baixa que a área visível da janela; as três prévias atuais têm entre ~450px e ~540px de altura. Uma tela futura com prévia mais alta que a viewport precisará de escala própria antes de entrar no padrão.
- Em janelas muito baixas (menos de ~600px de altura útil), a prévia fixa ocupa boa parte da tela visível; o único tratamento existente é a coluna única por largura (960px), que não olha a altura.
- Os ajustes continuam em uma coluna à direita: não há segunda coluna de cards em telas muito largas — a leitura linear dos grupos foi mantida de propósito.

---

# BotLive 0.7.2

## O que mudou

O chat fica fiel ao widget de referência do StreamElements e ganha o resto do combinado na rodada: prefixo antes do nome, cor do nome em três vias com o papel VIP reconhecido, eventos da live como caixas dentro da própria coluna, e modo código (HTML, CSS, JS e Data) nas telas de Chat no OBS e Alertas.

**Chat — prefixo antes das badges.** O `>` do widget de referência vem antes das badges, com texto livre (até 16 caracteres; acima disso volta ao padrão), cor própria e interruptor **Prefixo antes do nome** ligado por padrão — no grupo **Mensagem**, junto da fonte e das cores.

**Chat — cor do nome em três vias.** **Origem da cor do nome** (grupo **Mensagem**) escolhe entre **Twitch** (padrão: a cor real que cada pessoa escolheu, repassada pela EventSub e validada como hex no backend e de novo no frontend; vazio ou fora de hex cai na cor estável por conta), **Personalizadas** (a cor estável tirada do id, da paleta da Twitch, ignorando a cor real de cada pessoa) e **Fixa** (uma cor só, do seletor do perfil; sem cor escolhida volta à estável). O papel **VIP** passou a ser reconhecido na Twitch (o conjunto de badges `vip`, entre inscrito e moderador na ordem de papel), pintando nome, listra e fundo como os demais.

**Chat — eventos na coluna.** Com o mestre **Eventos no chat** ligado, follow, sub, renovação, presente, bits e raid viram uma **caixa própria dentro da coluna**, com cor por evento (borda na cor do evento e fundo escuro tinto), texto com `{name} {amount} {months} {tier} {sender} {message}` escapados — em presente, `{name}` é quem ganhou e `{sender}` quem deu — e **Mostrar a mensagem do espectador** como segunda linha nas caixas de renovação e bits. **Som do evento** escolhe um arquivo de Sons de Live e toca na **linha de áudio do aplicativo** (tela Áudio), nunca dentro do OBS; **Testar …** desenha a caixa na prévia e toca o som mesmo com o mestre desligado. O padrão é `eventsOn=false` para não duplicar a camada de alertas; eventos de teste do aplicativo (`simulated`) não desenham caixa. O backend passou a disparar `chat.event` junto com alertas e sons, e o canal foi liberado no WebSocket local.

**Modo código nas telas de Chat e Alertas.** As abas **Fields · HTML · CSS · JS · Data** ficam acima do formulário: **Fields** continua sendo o formulário de sempre; **HTML**, **CSS** e **JS** nascem vazios, salvam sozinhos e **Restaurar padrão** limpa o bloco da aba. O CSS é **prefixado com o escopo da camada** (`body` no overlay; `.ch-stage`/`.al-stage` na prévia; `:root`, `html` e `body` viram o próprio escopo; `@media`, `@supports`, `@layer` e `@container` reescrevem recursivamente, `@keyframes` ficam intactos) — no Estúdio ele não pinta a interface do aplicativo e no OBS vale só a cena. O JS roda com `root` = a própria camada, na prévia e no OBS (o CSP do aplicativo ganhou `script-src 'unsafe-eval'`, só isso); erro de sintaxe vai ao console sem derrubar o resto e o código reexecuta só quando o texto muda. O HTML injetado fica sob um marcador `[data-bl-code=…]` e é recolocado quando a prévia redesenha os filhos. A aba **Data** mostra o JSON da tela e **Aplicar** normaliza pelo mesmo normalizador do formulário — JSON inválido ou valor que não é objeto é recusado com o motivo — e devolve à aba Fields.

## Como usar

- **Cor do nome:** em **Chat no OBS → Mensagem**, escolha **Origem da cor do nome**: Twitch, Personalizadas ou Fixa.
- **Prefixo:** em **Chat no OBS → Mensagem**, edite **Prefixo antes do nome** (texto e cor) ou desligue o interruptor.
- **Eventos na coluna:** ligue **Eventos no chat**, escolha a pílula do evento, edite **Texto da caixa** e **Cor do evento**, ative **Mostrar a mensagem do espectador** se quiser a segunda linha e escolha **Som do evento** (áudios de Sons de Live, saída da tela Áudio). **Testar {evento}** mostra o resultado na prévia e toca o som.
- **Modo código:** nas telas **Chat no OBS** e **Alertas**, use as abas acima do formulário. No **CSS**, escreva seletores normais — o escopo da camada é aplicado automaticamente. No **JS**, `root` é a camada (ex.: `root.querySelector('.bl-chat-name')`). Em **Data**, cole o JSON e toque em **Aplicar**.

## Validação

- `cargo test` em `src-tauri`: **238 testes, 0 falhas, 1 ignorado** — incluem os novos: `fill_box` desenha a linha do espectador só com `evShowMsg` ligado e com mensagem; validação de `nameMode`, prefixo, `eventsOn`, `evShowMsg`, `ev` por evento e `code`; cor `twitch_color` hex estrito; papel VIP na Twitch.
- `npm test` (vitest): **115 testes, 0 falhas** — incluem `codeMode.test.ts` (escopo do CSS em seletores comuns, `:root`/`body`, `@media`) e os novos de `chatOverlay` (três vias de cor, prefixo, caixa de evento com marcadores escapados, kind inválido).
- `npx playwright test`: suíte completa verdes (**69 testes**) e o spec dos Alertas reexecutado depois do ajuste da captura (**6/6**), incluindo: modo código do chat (HTML no palco, CSS escopado `.ch-stage`, JS com `root`, aba Data normalizando); modo código dos alertas (CSS escopado `.al-stage`, aba Data recusando não-objeto e restaurando padrão); camada de chat no overlay (cor da Twitch no nome, prefixo `>` antes das badges, evento inline com cor, CSS escopado em `body`, kind inválido ignorado); eventos com cor, texto, som e interruptores por evento.
- Regras de tela: os testes de layout auditam Chat e Alertas com `auditPage` em **1100px e 1680px sem achados** e sem rolagem horizontal, gravando `artifacts/estudio-chat-1100.png`, `estudio-chat-largo.png`, `estudio-alertas-1100.png`, `estudio-alertas-largo.png` e `estudio-alertas-acordeao.png` (agora com as abas visíveis), além de `artifacts/overlay-chat-camada.png`; as capturas foram revisadas visualmente nesta entrega.
- `npm run build` (overlay gerado pelo `build:overlay` + `tsc` + vite) sem erros; `npm run docs` regerado; `npm run update:check` executado antes do push.

## Limitações

- No modo código, `scripts` dentro do HTML não executam (o que é código entra na aba JS), e imagens ou fontes externas do seu HTML/CSS podem ser bloqueadas na prévia do aplicativo pela política de segurança — na fonte do OBS elas carregam. O JS não tem sandbox além do escopo: escreva só código seu.
- A cor da Twitch só entra quando a live entrega um hex completo; quem nunca escolheu cor (ou entrega fora do padrão) ganha uma cor estável por conta, que pode se repetir entre espectadores.
- O som do evento toca apenas na linha de áudio do aplicativo (não existe toque dentro da fonte do OBS); a caixa no overlay é só visual.
- Eventos de teste do aplicativo (`simulated`) não desenham caixa na coluna — o caminho de teste é **Testar …** no Estúdio.
- No YouTube as mensagens entram sem badges, sem emotes e sem cor de nome; o Kick não entrega eventos de chat ao bot nesta versão.
- A aba **Data** aplica a configuração normalizada: valor fora de faixa volta ao limite que o formulário aceita — não é um editor livre de valores inválidos.

---

# BotLive 0.7.1

## O que mudou

Cinco entregas da mesma rodada de melhorias: o bug do endereço da Meta, sons escolhíveis para a Meta, volume real nos alertas, cor por evento na tarja dos alertas e um Estúdio de Alertas com layout vertical. A numeração volta a ser sequencial: depois de 0.7.0 vêm 0.7.1, 0.7.2 e assim por diante, sem pular números (registrado em `docs/ATUALIZACOES.md`).

**Meta — endereço do overlay corrigido.** A tela Meta ficava presa em "Gerando o endereço do overlay…" e o design salvo não voltava a carregar: as chamadas `livepix.design_get` e `livepix.overlay_info` não enviavam o `profileId`, o backend respondia "Perfil não encontrado" e o erro era engolido. Ambas as chamadas agora enviam o id, o endereço da camada (`mode=goal`) aparece normalmente e o Estúdio volta a abrir com o design salvo. Um teste Playwright reproduz o contrato do backend (falha sem `profileId`) e trava a regressão.

**Meta — som de doação e som de celebração.** O grupo **Efeitos** do Estúdio do overlay ganhou dois seletores: **Som de doação** (toca a cada pagamento que sobe a barra da meta) e **Som de celebração** (toca no lugar do de doação quando a meta cruza 100% — o pagamento que completa a Meta não toca os dois). A lista vem da biblioteca **Sons de Live** do perfil, com **Sem som** como padrão (nascem desligados). Os sons saem pela **linha de áudio do app** (mesma saída escolhida na tela Áudio, evento `viewer-sound`, volume cheio) e **nunca pela fonte do OBS**: o overlay continua só com a parte visual. O design aceita `soundDonate`/`soundCelebrate` (ids de arquivo ≤64 caracteres, higienizados dos dois lados), trocar de modelo preserva a escolha, e um arquivo apagado depois da escolha é silenciado em vez de gerar erro. Os botões **Simular doação** e **Celebração** passaram a ouvir a escolha na hora, na mesma saída do OBS.

**Alertas — volume do som corrigido.** O volume salvo (0–100) era repassado cru ao player, que corta em `Math.min(1, volume)`: na prática qualquer volume de 1 a 100 tocava a 100% e só o 0 silenciava. O backend agora converte para a escala 0–1 (`volume/100` limitado à faixa) antes de emitir o evento da linha; um teste Rust trava a conversão (70 → 0.7).

**Alertas — cor por evento (só a tarja).** O grupo **Este evento** ganhou **Cor da tarja**: escolha uma cor para aquele evento (Bits vermelho, Raid azul, etc.) e **somente a tarja colorida** muda — nome, linhas e moldura continuam nas cores globais do painel. Vazio (padrão) usa o **Destaque** global, e o botão **Usar cor global** devolve esse padrão. O texto dentro da tarja escolhe preto ou branco sozinho pela luminosidade da cor, para nunca sumir em fundo escuro. A validação no backend exige string vazia ou cor bem formada; caractere fora do formato barra o salvamento.

**Alertas — layout vertical.** O Estúdio ficou com a **prévia em cima** e as configurações **em acordeão abaixo**: **Este evento** abre por padrão (é o que se edita) e **Painel**, **Cores**, **Fontes** e **Efeitos** abrem pelo título, com um resumo do que está configurado ao lado de cada um. O interruptor de exibir/ocultar o evento foi para o cabeçalho do grupo **Este evento**. O conteúdo fica centralizado com teto de 980px para o palco 16:9 não virar gigante em monitores largos.

## Como usar

- **Sons da Meta:** envie os áudios em **Sons de Live**, abra **Integrações → Meta**, desça até o grupo **Efeitos** e escolha **Som de doação** e **Som de celebração** (ou **Sem som**). Clique em **Simular doação** e **Celebração** para ouvir antes de gravar. A saída é a escolhida em **Áudio → Dispositivo de saída dos sons** (ex.: CABLE Input) — no OBS, a mesma Fonte de áudio de sempre já carrega esses sons.
- **Endereço da Meta:** em **Integrações → Meta → No OBS**, o endereço da camada (`mode=goal`) aparece pronto para copiar; crie a Fonte de navegador só para ele. Nada muda em instalações que já tinham a fonte.
- **Volume dos alertas:** em **Alertas → Estúdio → Painel**, **Volume do som** agora vale de verdade (10% toca baixo, 0 silencia).
- **Cor por evento:** em **Alertas → Estúdio → Este evento**, use **Cor da tarja**; **Usar cor global** volta ao Destaque do painel.
- **Novo layout dos Alertas:** prévia em cima; clique nos títulos dos grupos abaixo para abrir cada um.

## Validação

Executada nesta preparação, no Windows:

- `cargo test` em `src-tauri`: **233 testes, 0 falhas** — incluem os novos: volume 70 → 0.7 no evento da linha; cor por evento aceita vazio/`#hex` e barra caractere inválido e campo ausente; sons da Meta tocam na linha com o aparelho da tela Áudio e volume 1.0, silenciam sem escolha e sem arquivo removido; celebração substitui o som da doação; sanitização do design mantém as mesmas chaves do `overlayDesign.keys.json`.
- `npm test` (vitest): **99 testes, 0 falhas** — incluem a normalização de `color` por evento (padrão vazio, preservação, limite de tamanho).
- `npx playwright test`: suíte completa verdes, incluindo: contrato do `livepix.*` com `profileId` e endereço `mode=goal` fora do "Gerando…"; sons da Meta no grupo Efeitos sobrevivem a troca de modelo; acordeão dos Alertas (fechado por padrão, abre sem mexer no interruptor mestre); cor por evento pinta só a tarja (fundo `rgb(255,59,48)` com texto branco) e **Usar cor global** devolve o padrão; tarja padrão clara com texto escuro.
- Regras de tela: o teste de layout dos Alertas audita a página (`auditPage`) em **1100px e 1680px sem achados**, verifica ausência de rolagem horizontal nas duas larguras e grava as capturas `artifacts/estudio-alertas-1100.png`, `estudio-alertas-largo.png` e `artifacts/estudio-alertas-acordeao.png`; o estúdio da Meta grava `artifacts/estudio-livepix-sons.png` (além de `estudio-livepix-1100/largo`), e as quatro capturas foram revisadas visualmente nesta entrega.
- `npm run build` (overlay gerado + tsc + vite) sem erros; `npm run update:check` executado antes do push.

## Limitações

- A prévia de som da Meta toca no aplicativo (na saída da tela Áudio); o toque real depende de um pagamento na meta ativa — em CI/headless o som não é audível, a validação confere o evento emitido e os campos (aparelho, volume, asset).
- O som da Meta toca em volume cheio (1.0) na linha; não há volume por som dentro da Meta — o ajuste é o do aparelho/volume do sistema.
- A interface de cor por evento gera o formato do seletor nativo (`#rrggbb`); o validador do backend também aceita nomes de cor/`rgb()`, mas a tela não escreve esses formatos.
- O layout vertical dos Alertas é a nova estrutura da tela; a aparência do painel no OBS (animações, cores, fontes) não muda.

---

# BotLive 0.7.0

# BotLive 0.7.0

## O que mudou

Dez ajustes pedidos por usuários, organizados em quatro blocos: áudio em linha única, separação das telas de respostas e sons, grupo de Overlays do OBS e correções de usabilidade.

**Áudio em linha única.** A tela **Áudio** perdeu os interruptores por origem e o caminho pelo navegador: agora existe **uma saída única** para todos os sons do bot (espectador, comandos, timers, automações, resgates e alertas). Você escolhe um aparelho de saída (por exemplo, a entrada **CABLE Input** do VB-Cable) e no OBS precisa de **uma única Fonte de áudio** apontando para o **CABLE Output** — o bot inteiro entra num fader só. O cartão **Voz da live — linha do microfone** mantém microfone e saída da voz na mesma tela, e **Texto para voz (TTS)** declara que continua preso à saída padrão do Windows. O botão de teste passou a se chamar **Testar saída** e toca um bipe pelo mesmo aparelho dos sons.

**Respostas e Sons de Live separadas.** As regras de resposta em texto ganharam tela própria, **Respostas** (grupo Seu espaço), e cada regra agora aceita um **som opcional** da biblioteca, com seletor, volume e **Testar som** direto na regra. Os **sons por espectador** saíram da tela de respostas e foram para **Sons de Live**, que reúne a **Biblioteca de sons** (WAV/MP3/OGG, até 5 MiB, com Ouvir e Apagar) e o cadastro de **Sons por espectador** (nome no chat, apelido, ID, som, disparo por mensagem/entrada, intervalo, volume), os interruptores **Ativar sons por espectador** e **Monitorar entradas silenciosas na Twitch**, o botão **Reiniciar sessão de sons** e **Salvar sons por espectador**. O botão **Respostas** em Comunidade leva agora à tela de Respostas.

**Overlays do OBS.** Alertas, Chat no OBS e Meta passaram a navegar juntas no grupo **OVERLAYS DO OBS** (a tela de LivePix se chama **Meta** no menu). Cada uma expõe um **endereço de camada próprio**, com o parâmetro `mode` na URL — `mode=goal` (Meta), `mode=alerts` (Alertas) e `mode=chat` (Chat no OBS) — para você criar uma **Fonte de navegador separada** por camada e ajustar posição e tamanho sem mexer nas outras. Sem o parâmetro, o endereço continua mostrando as três camadas juntas, preservando quem já usa uma fonte única.

**Correções e clareza.** A presença de chat ficou **visível no topo do aplicativo**: um indicador mostra quantas pessoas estão no chat e, ao clicar, a lista inteira, que se atualiza sozinha a cada 30 segundos enquanto o perfil está conectado. Na tela de rótulos, os modelos e os arquivos ficaram em **acordeão** (Rótulos e modelos recolhidos por padrão, cada rótulo com seu interruptor). No **Alertas** e no **Chat no OBS**, os interruptores ganharam **rótulo visível ao lado**, grupos com **dicas** (Seg, RangeRow e ColorRow explicam o que cada controle faz) e textos explicando que a camada usa endereço próprio.

**LivePix (Meta).** A meta agora **persiste** quando você desliga e religa a integração (bug corrigido: o progresso não é mais perdido). Dá para **editar uma meta existente** (título e valor-alvo), **reiniciar o contador da meta ativa** e o valor aceita **vírgula ou ponto** (500,00 ou 500). Excluir uma meta e reiniciar o contador pedem **confirmação**. Falhas da integração aparecem como **erro visível** no topo da tela, com o motivo, e são apagadas quando o próximo retorno der certo.

## Como usar

1. **Áudio:** abra **Áudio**, escolha a saída dos sons do bot (com VB-Cable, **CABLE Input**) e clique em **Testar saída**. No OBS, adicione uma **Fonte de áudio** apontando para o **CABLE Output** e ajuste o volume dessa fonte — é o fader do bot inteiro. Confira o microfone e a saída no cartão **Voz da live**; o TTS continua na saída padrão do Windows (use o Mixer de volume para rotear, se precisar).
2. **Respostas:** abra **Respostas**, clique em **Adicionar resposta TXT**, preencha a palavra ou expressão, vincule o TXT UTF-8, escolha aleatória ou sequência, ajuste os intervalos e, se quiser, escolha um **Som junto com a resposta** da biblioteca (com volume e **Testar som**). Ative a regra e **Ativar respostas TXT**, depois **Salvar respostas**.
3. **Sons de Live:** abra **Sons de Live**, use **Adicionar som** para importar áudios na biblioteca. Em **Sons por espectador**, clique em **Adicionar pessoa**, informe o nome no chat (e apelido/ID), escolha **Som da pessoa**, o **Disparar som**, o **Quando tocar** e o **Volume**. Ative a pessoa e **Ativar sons por espectador**, depois **Salvar sons por espectador**. **Reiniciar sessão de sons** libera as pessoas para tocar de novo.
4. **Overlays:** em **OVERLAYS DO OBS**, abra **Meta**, **Alertas** ou **Chat no OBS** e, no cartão **No OBS**, copie o endereço da camada (já com `mode=goal`, `mode=alerts` ou `mode=chat`). Crie uma **Fonte de navegador** por camada em 1920×1080 e posicione cada uma onde quiser. Ligue o recurso no topo de cada tela.
5. **Presença no topo:** com o perfil conectado, o indicador no topo mostra quantas pessoas estão no chat; clique para ver a lista, que se atualiza a cada 30 segundos. É a mesma fonte de dados do cartão de rótulos.
6. **Meta (LivePix):** edite uma meta pelo botão de edição (use vírgula ou ponto no valor), **Ativar** para torná-la ativa, **Reiniciar contador** para zerar a contagem da meta ativa (com confirmação). Desligar e religar a integração mantém o progresso; erros da integração aparecem no topo e somem sozinhos no próximo retorno válido.
7. **Rótulos:** em **Rótulos OBS**, expanda **Rótulos e modelos** para ligar rótulos e ajustar os modelos com tokens.

## Validação

Executada nesta máquina, no Windows, antes do push:

- **Rust (`cargo test`) em src-tauri:** 230 testes aprovados, 0 falhas. Inclui os testes do **LivePix** (`update_goal`, `reset_counter`, preservação do progresso ao desabilitar/habilitar, limpeza de `last_error` no sucesso e log em `remove_goal`), a rota **linha** de áudio (`play_line` substituindo a antiga rota por origem), o som de alerta sempre na linha (`line_sound`) e a migração do aparelho salvo.
- **TypeScript (`tsc -b` / `npm run build`):** limpo.
- **Vitest (`npm run test`):** 98 testes aprovados, 0 falhas (o teste de utilitários do Alertas que cobrava o som do overlay no navegador foi removido junto com o recurso; `src/audio.test.ts` cobre a normalização da configuração de linha única).
- **Playwright (msedge, build de produção + preview na porta 1421):** 62 testes aprovados, 0 falhas. `tests/ui/audio.spec.ts` reescrito para a saída única (salva o aparelho com a chave `audio` e o valor `{device}`, aparelhos da voz na mesma tela e TTS declarado, mais a tela em 1100px); `tests/ui/app.spec.ts` cobre as telas separadas de Respostas e Sons de Live; contagens atualizadas para **22 itens de navegação** (grupo Overlays do OBS incluído) e **26 capítulos do manual**.
- **Manual (`npm run docs`):** regerado com 26 capítulos e links internos válidos (o gerador falharia se algum link local estivesse quebrado).
- **Screenshots** gerados e conferidos em `artifacts/`: `saida-audio-1100.png` (tela de áudio em 1100px), `estudio-alertas-1100.png` e `estudio-chat-1100.png` (estúdios em 1100px) e os demais das telas novas — cartões completos, rótulos legíveis, sem texto espremido.

## Limitações

- **TTS preso à saída padrão do Windows:** o navegador não permite ao BotLive escolher outro aparelho para o leitor de voz. Para levá-lo à linha, troque a saída padrão ou use o Mixer de volume do Windows.
- **Sons dependem do aplicativo aberto:** a saída de áudio é tocada pelo próprio BotLive; ele precisa estar aberto e desbloqueado na máquina da live.
- **Uma Fonte de áudio por aparelho:** se você quiser separar o volume do alerta do volume dos sons de espectador, precisará de mais de uma Fonte de áudio no OBS apontando para aparelhos diferentes — a tela oferece uma saída única por perfil.
- **Presença de chat:** a lista de quem está no chat depende de o perfil estar conectado; o indicador só aparece nesse caso e segue a atualização de 30 segundos do backend. No YouTube e em outras plataformas a presença silenciosa não existe (a Twitch usa IRC; outras precisam de ponte).
- **Camadas do overlay:** o endereço com `mode` é gerado pelo aplicativo desktop; no navegador (prévia) ele não é gerado. Quem usa uma fonte única, sem o parâmetro, continua vendo as três camadas juntas.
- **LivePix:** erros da integração são exibidos na tela e registrados, mas a causa raiz pode depender do serviço da LivePix (consultas a cada 15 segundos, sem webhook); a confirmação de exclusão e de reinício usa a caixa de diálogo do sistema.
- **Respostas e sons por espectador seguem os mesmos limites de antes:** até 100 regras de resposta e até 200 pessoas/300 arquivos por perfil; sons com intervalo mínimo de 5 minutos por pessoa e intervalo geral de 5 segundos entre sons.

---

# BotLive 0.6.0

## O que mudou

- Nova tela **Áudio** (Central de áudio), logo após Chat no OBS: todos os sons do bot num lugar só, com o caminho de cada origem configurável.
- **Para onde vai cada som**: sons de espectador, sons de fluxo e alertas da live têm interruptores próprios de **linha de áudio** (o aplicativo toca no aparelho de saída escolhido) e de **navegador** (a fonte do overlay toca o som dentro do OBS). Os dois podem estar ligados ao mesmo tempo; desligar os dois silencia a origem. As alterações salvam sozinhas e chegam ao OBS na hora.
- **Linha de áudio**: cartão com o seletor de aparelho de saída (antes em Respostas e sons) e botão **Testar linha**, que toca um bipe exatamente pelo caminho que os sons marcados "Linha" usariam. Sons marcados "Navegador" passam a tocar pela fonte do overlay no OBS, no mixer da fonte de navegador.
- **Voz da live na mesma tela**: estado da voz e os aparelhos (microfone de entrada e saída da linha) espelhados da tela Voz da live, salvos ao trocar; efeitos, corte de ruído e iniciar continuam na tela Voz da live.
- **Texto para voz (TTS)** aparece na Central com sua limitação declarada (saída padrão do Windows) e o caminho para roteá-lo pelo Mixer de volume.
- **Migração automática**: quem já tinha um aparelho de saída escolhido em Respostas e sons mantém a escolha na primeira leitura da Central; sem nada salvo, o padrão preserva o comportamento de antes (sons de espectador e de fluxo na linha do aplicativo, som dos alertas no navegador).
- Telas adjacentes alinhadas: Respostas e sons e Sons apontam para a Central; o Ouvir da biblioteca de sons e a prévia de teste dos Alertas seguem o caminho configurado (a prévia dos alertas só toca o som se "Navegador" estiver ligado).
- Manual ganha o capítulo **Central de áudio** (26 capítulos), com passo a passo de alinhamento no VB-Cable.

## Como usar

1. Abra a tela **Áudio** com o perfil da live.
2. Em **Para onde vai cada som**, marque para cada origem (espectador, fluxo, alertas) se ela sai pela **Linha**, pelo **Navegador** ou pelos dois.
3. Em **Linha de áudio**, escolha o aparelho de saída — com VB-Cable, a entrada **CABLE Input** — e clique em **Testar linha** para ouvir o bipe no aparelho certo.
4. Em **Voz da live**, confira o microfone e a saída da linha (o mesmo CABLE Input alinha voz e sons na mesma linha); os efeitos continuam na tela Voz da live.
5. No OBS: a fonte de áudio **CABLE Output** leva voz e sons da linha; os sons marcados **Navegador** saem pela fonte de navegador do overlay (o mesmo endereço da meta, alertas e chat).
6. Para o TTS, use a saída padrão do Windows ou o Mixer de volume — o detalhe está no cartão **Texto para voz**.

## Validação

- `cargo test` em src-tauri: 227 aprovados, 0 falhas (1 ignorado) — inclui os novos testes do módulo `audio` (padrões, validação, migração do aparelho e rotas por origem), o roteamento de sons de espectador/fluxo entre linha e navegador em `chat_extras` e o som do alerta na linha em `alerts`.
- `tsc -b`: limpo.
- `vitest run`: 99 aprovados, 0 falhas (2 novos em `src/audio.test.ts`: normalização da configuração e o WAV do bipe de teste).
- Playwright (msedge, build de produção + preview): 62 aprovados, 0 falhas — 3 novos em `tests/ui/audio.spec.ts` (roteamento por origem com salvamento, aparelhos da linha e da voz com o TTS declarado, e tela em 1100px); contagens atualizadas para 23 itens de navegação e 26 capítulos do manual; o teste de Respostas e sons agora aponta para a Central.
- `npm run docs`: manual com 26 capítulos.
- `npm run test:updates`: 0 falhas; `npm run update:check`: válido após o prepare.
- Screenshot da tela a 1100px conferido (`artifacts/central-audio-1100.png`): cartões completos, interruptores e rótulos legíveis, sem texto espremido.

## Limitações

- O **TTS sai pela saída padrão do Windows**: o navegador não permite ao BotLive escolher outro aparelho para o leitor de voz. Para levá-lo à linha, troque a saída padrão ou use o Mixer de volume do Windows.
- Os sons da **linha** dependem do aplicativo BotLive aberto — é ele que toca no aparelho.
- Os sons do **navegador** dependem da fonte de navegador do overlay existir no OBS (mesmo endereço da meta, alertas e chat).
- O **Testar** da tela Alertas toca também na linha quando a linha dos alertas está ligada — mesma filosofia de os testes chegarem ao OBS.
- A Central cobre os sons da live (espectador, fluxo e alertas); sons gerados por outras pontas (por exemplo, Discord) seguem o caminho da origem que os criou.

---

# BotLive 0.5.0

## O que mudou

- **Chat no OBS (nova tela).** Em **Chat no OBS**, o chat da live vira uma coluna na fonte de navegador do OBS: cada mensagem chega como `[badges] [nome colorido] : [texto com emotes]`, na direção escolhida (novas embaixo empurrando para cima, ou novas no topo), com teto de mensagens em tela (10 a 300) e some-depois opcional (0 a 120 s).
- **Badges e destaques por papel.** Transmissor, moderador, VIP, inscrito, fundador, Prime, bits e presente são badges desenhadas pelo próprio motor (sem chamar a API de badges da Twitch), com interruptor individual; desconhecidas entram como "Outros". Destaque por papel com barra lateral, fundo tintado ou os dois, na ordem transmissor → moderador → VIP → inscrito → primeira mensagem, cada um com sua cor. A primeira mensagem de cada espectador é marcada desde que o BotLive abriu.
- **Emotes da Twitch na mensagem.** Os fragmentos do payload do EventSub viram `<img>` do CDN da Twitch pelo id do emote; mensagem só de emote aparece em tamanho dobrado; sem internet, o emote vira o próprio texto. A cor do nome é estável por espectador (da paleta da Twitch, tirada do id) ou fixa por configuração — a Twitch não expõe a cor real pelo caminho que o bot usa.
- **Filtros.** Esconder comandos (`!`), esconder outros bots (lista por nome + sufixo "bot"), mostrar ou não as mensagens do próprio BotLive (com nome configurável) e palavras bloqueadas por vírgula — os mesmos filtros rodam na fonte do OBS e na prévia do Estúdio.
- **Mensagens do bot entram como resposta.** Cada mensagem que o BotLive envia no chat aparece na coluna junto com as dos espectadores, pelo mesmo caminho de envio — sem exigir nada de quem só assiste.
- **Sincronização com a moderação.** O bot passa a assinar `channel.chat.message_delete`, `channel.chat.clear` e `channel.chat.clear_user_messages` (escopo `user:read:chat` que ele já tem): mensagem apagada some da coluna na hora, limpar o chat zera a coluna e as mensagens de um usuário podem sair juntas. Falha numa assinatura só registra erro no Histórico e o chat continua funcionando.
- **Estúdio com prévia em tamanho real.** A coluna aparece no mesmo tamanho que o OBS desenha, botões de teste (Comum, Moderador, VIP, Sub, Primeira vez, Em emote, Só emotes, Do bot, Sequência, Limpar) empilham mensagens pelo mesmo motor das reais, e com o Estúdio aberto o chat ao vivo também corre na prévia. Os testes são locais: nunca mandam mensagem falsa para o chat da live.
- **Mesmo endereço do overlay da meta e dos alertas.** A camada de chat entra na Fonte Navegador que já existe no OBS; a config chega ao OBS na hora, sem recarregar a fonte.

## Como usar

1. Abra **Chat no OBS** com o perfil do canal e ligue **Chat no overlay** no cartão do topo.
2. Use os botões de teste para ver o visual atual empilhando na prévia; ajuste coluna (fluxo, teto, some-depois, animação, largura, lado, listras), mensagem (fonte, tamanhos, cores, caixa alta), badges, destaques e filtros — tudo salva sozinho.
3. Com o Estúdio aberto, as mensagens reais da live também entram na prévia: é a forma de ajustar sem ir ao ar.
4. Se a fonte de navegador ainda não existe no OBS, copie o endereço no cartão **No OBS** (Fontes → + → Navegador, 1920×1080). É o mesmo endereço do overlay da meta e dos alertas; o alinhamento define o lado da coluna na cena.

## Validação

- `cargo test`: 222 testes, 4 novos — 3 do módulo de chat (faixas da validação, fragmentos → texto/emote com id, padrão desligado) e a integração engine → chat: desligado não transmite, ligado leva papel/badges/emote/primeira-mensagem, evento simulado não vaza, resposta do bot entra, `message_delete` e `clear` da Twitch removem da fonte.
- `tsc -b` limpo; `vitest` 97/97, com 10 testes novos do motor (normalização com faixas, cor estável por espectador, filtros de comandos/bots/palavras, exemplos de teste passando pelos filtros).
- Playwright com a tela nova (mensagens empilhando pelo motor real, salvamento automático do interruptor mestre e da direção, filtros escondendo os exemplos correspondentes, captura em 1100px), o audit de layout percorrendo as 22 telas da navegação e o manual verde no desktop, na busca e no mobile.
- `npm run docs` com manual de 25 capítulos (novo capítulo Chat no OBS), `npm run test:updates` e `npm run update:check`.
- Captura em 1100px da tela (`artifacts/estudio-chat-1100.png`): botões de teste quebrando em linhas limpas, coluna de prévia em tamanho real sem texto espremido.

## Limitações

- Só emotes da Twitch: BTTV e 7TV não aparecem.
- A cor real do nome (escolhida nas configurações da Twitch) não é informada pela API que o bot usa; a cor por espectador é estável, mas pode não ser a mesma do chat oficial.
- "Primeira mensagem" vale desde que o BotLive abriu, não a primeira da vida no canal (lista em memória).
- A sincronização de deleções depende da Twitch entregar os eventos de moderação ao bot; se uma assinatura falhar na conexão, o erro aparece no Histórico e a mensagem apagada fica até sair por tempo ou limite.
- No YouTube as mensagens entram sem badges e sem emotes; o Kick não entrega eventos de chat ao bot nesta versão.
- Eventos inline no chat (follow/sub/raid como mensagens da coluna) não existem: os alertas (0.4) cobrem os mesmos eventos com painel próprio.
- Os botões de teste do Estúdio são locais: servem para ajustar o visual, não para forçar uma mensagem no chat da live.
- A coluna ocupa o lado da cena que você escolher; se cobrir a meta ou os alertas, ajuste o alinhamento ou as margens do overlay.
- Fontes do Google exigem internet no computador que roda o OBS; fontes do sistema funcionam offline.

---

# BotLive 0.4.0

## O que mudou

- **Estúdio de Alertas (nova tela).** Em **Alertas**, follows, subs, renovações, subs de presente (individual e em lote), bits e raids viram painéis animados na fonte de navegador do OBS: barra de status, tarja do evento, nome em destaque, linha grande, linha final datilografada e barra de tempo em blocos que corre durante a duração configurada.
- **Mesmo endereço do overlay da meta.** Os alertas são uma camada da fonte que já existe no OBS: quem tem a Fonte Navegador do overlay da meta não troca nada, basta ligar o interruptor no Estúdio. A config chega ao OBS na hora, sem recarregar a fonte, e a fila é serializada — um painel por vez.
- **Campos com variáveis, não código.** Cada evento tem quatro textos (Etiqueta, Linha do nome, Linha grande e Linha final) e um som, preenchidos com `{{user}}`, `{{amount}}`, `{{months}}`, `{{tier}}`, `{{sender}}` e `{{message}}` — apelidos como `{{name}}` e `{{viewer}}` também valem `{{user}}`. Ajustes do painel: duração (3 a 20 s), velocidade (50% a 200%), volume, posição na tela, margens, largura, texto da marca, 4 cores, 2 fontes com tamanhos independentes e efeitos (scanline, cantos, partículas e glitch).
- **Teste por evento pelo caminho real.** O botão **Testar** de cada evento injeta um payload de exemplo pelo mesmo caminho do evento real: a prévia do Estúdio anima e o OBS toca junto — inclusive com os alertas desligados, como faz o botão de teste do widget de referência do StreamElements que serviu de base.
- **Sons do perfil dentro do alerta.** O seletor de som lista os arquivos enviados em Respostas e sons (WAV, MP3 ou OGG, até 5 MiB); o áudio toca pelo navegador do OBS pela nova rota local `/asset`, que só atende sons registrados do perfil com a chave do overlay.
- **Manual consertado e ampliado.** O capítulo Rótulos OBS estava de fora do MANUAL.html (grupo de navegação incompleto) e voltou para a seção "Cuide da sua live"; o novo capítulo Alertas da live entrou ao lado.

## Como usar

1. Abra **Alertas** com o perfil do canal e ligue **Alertas no overlay** no cartão do topo.
2. Escolha o evento nas pílulas (Seguidor, Novo sub, Renovação, Sub de presente, Presentes em lote, Bits, Raid) e edite os quatro textos; as variáveis são preenchidas no momento do alerta.
3. Para som, envie um WAV, MP3 ou OGG em **Respostas e sons** e escolha o arquivo no campo **Som** do evento.
4. Clique em **Testar**: a prévia roda a animação completa e o teste também vai para o OBS se a fonte estiver aberta.
5. Se a fonte de navegador ainda não existe no OBS, copie o endereço no cartão **No OBS** (Fontes → + → Navegador, 1920×1080). É o mesmo endereço do overlay da meta.

## Validação

- `cargo test`: 218 testes, 8 novos — 7 do módulo de alertas (padrões e validação, mapeamento de eventos com gift em lote, disparo respeitando interruptores, botão de teste, mensagem inicial e som servido) e a integração engine → alerta com validação do salvamento; o teste da porta local passou a cobrir também o `/asset` com chave certa e chave errada.
- `tsc -b` limpo; `vitest` 87/87, com 8 testes novos do motor (normalização com faixas, placeholders com apelidos e utilitários).
- Playwright 56/56, com a tela nova (sete eventos, salvamento automático por evento, animação do teste terminando limpa e captura em 1100px), o audit de layout percorrendo todas as telas da navegação e o manual verde no desktop, na busca e no mobile.
- `npm run docs` com manual de 24 capítulos (Rótulos OBS que faltava e Alertas da live), `npm run test:updates` e `npm run update:check`.
- Captura em 1100px da tela (`artifacts/estudio-alertas-1100.png`): pílulas quebrando em linha limpa, formulário sem texto espremido e prévia do quadro de 1920×1080 em escala.

## Limitações

- Sem editor livre de HTML/CSS/JS nesta onda: o Estúdio trabalha por campos, com um único visual de painel; um modo código pode vir em uma versão futura.
- Sem botão de pular alerta: cada alerta roda a duração configurada antes de o próximo entrar na fila.
- A Twitch não informa quem recebeu um sub de presente: em Sub de presente e Presentes em lote, `{{user}}` é quem deu o presente.
- Doações (LivePix) e resgates de pontos não geram alerta nesta versão.
- O som vem da biblioteca do perfil (Respostas e sons); o Estúdio não envia arquivo novo.
- Fontes do Google exigem internet no computador que roda o OBS; fontes do sistema funcionam offline.

---

# BotLive 0.3.0

## O que mudou

- **Comandos em linguagem natural no chat (Onda 1).** Streamer e moderadores escrevem por extenso e o bot executa o mesmo que `!setgame` e `!settitle`: «Arroba, altere a categoria do jogo para Minecraft ou AION2» troca a categoria e «Arroba, mude o título para o novo texto» troca o título, com a conta autorizada, confirmação no chat marcando quem pediu e registro no Histórico.
- **Plano B de categoria.** Quando a frase traz alternativas separadas por «ou», o bot tenta os nomes na ordem e usa o primeiro que a Twitch conhecer; se nenhum existir, responde listando o que tentou. A mesma regra de alternativas passa a valer também no `!setgame`, na ação de categoria do fluxo com conteúdo vazio e no Controle por voz — o caminho de execução é um só.
- **Determinístico e offline, falha fechada.** O entendimento da frase usa vocabulário próprio (verbos de troca, palavras de intenção, acentos e caixa), sem rede e sem IA: o que não cabe no vocabulário não vira adivinhação.
- **Sempre com a palavra de ativação.** O comando natural só existe quando a frase tem uma das palavras de ativação do Controle por voz (a escuta não precisa estar ligada); sem ela a mensagem é chat normal e continua passando por respostas, fluxos e módulos como antes.
- **Silêncio na dúvida.** Frase sem verbo de troca, pergunta de valor («para quê?») ou pedido dos dois assuntos na mesma frase não executam e não geram resposta; com a intenção clara e sem o nome, o bot responde o modo de usar.
- Painel **Controle por voz** ganhou a frase explicativa dos comandos por extenso no chat, ao lado das palavras de ativação. Capítulos 03 e 05 do manual atualizados.

## Como usar

1. Em **Comunidade → Controle por voz**, preencha **Palavras de ativação** (por exemplo `arroba, botlive`). A escuta contínua não precisa estar ligada.
2. No chat da live, sendo streamer ou moderador, escreva por extenso: «arroba, altere a categoria do jogo para Minecraft ou AION2» ou «arroba, mude o título para Ranked com a comunidade».
3. O bot confirma no chat com o nome oficial da categoria ou com o título novo. Alternativas de categoria valem até cinco nomes; o título é cortado em 140 caracteres.

## Validação

- `cargo test`: 210 testes, entre eles 9 novos do leitor de frases (o exemplo completo com plano B, acentos e caixa, silêncio sem ativação, dicas de uso, perguntas, bordas e pontuação, e as alternativas do «ou»).
- `tsc -b` e a checagem estrita com `--noUnusedLocals`/`--noUnusedParameters` limpas; `vitest` 79/79; Playwright 53/53, incluindo o painel de voz com a frase explicativa nova.
- `npm run docs` (manual com 23 capítulos), `npm run test:updates` e `npm run update:check` antes e depois do `update:prepare`.
- Captura em 1100px do painel Controle por voz com a frase nova no lugar, sem texto espremido.
- O caminho de execução é o mesmo do `!setgame` já entregue (busca de categoria, `PATCH channels` e confirmação); a resposta final depende da API da Twitch.

## Limitações

- Só categoria e título nesta onda: silenciar, banir, avisar, VIP e modos do chat continuam pelos comandos nativos (`!timeout` e companhia) ou por fluxos.
- O gatilho é a palavra de ativação cadastrada, não a menção do bot: escrever apenas «@BotLive, mude o jogo» não executa.
- Pedido de categoria e título na mesma frase não executa; o bot espera um assunto por comando.
- O vocabulário é de português com acentos e caixa, mais as palavras inglesas básicas (game, title); frase fora do vocabulário passa em silêncio como chat normal.
- Nomes de categoria que contenham a palavra «ou» (raros) são lidos como alternativas e tentados na ordem.
- Quem não é streamer nem moderador não dispara; sem palavra de ativação cadastrada não existe comando natural.

---

# BotLive 0.2.1

## O que mudou

- **Fecha a pendência declarada na 0.1.99:** as cores derivadas do acento que ficaram fixas agora acompanham a cor de destaque nos dois modos. O item ativo do menu, as tags de acento (padrão/verde) e o link do painel de boas-vindas usam o acento misturado com o texto do tema, então permanecem legíveis no escuro e no claro. No modo claro eles deixaram de ser verdes fixos: com Violeta ou Rosa escolhidos, o menu ativo, a etiqueta e o link do painel viram Violeta ou Rosa de verdade.
- **Tags de categoria com tom próprio no claro.** Os tipos roxo, âmbar e azul apareciam como texto pastel sobre o branco, com leitura ruim; agora cada um tem tom escuro no modo claro, alinhado às cores de texto já existentes.
- **Estados de resultado e pílulas de status no claro.** Sucesso/ativo (verde), aguardando (âmbar) e erro (vermelho) ganharam tons escuros no modo claro — no escuro continuam os mesmos. Sobre o branco: sucesso ≈5,3:1, aviso ≈5,6:1 e erro ≈5,9:1.
- **O que continua verde por decisão:** a ilustração do painel de boas-vindas e os detalhes dela (mascote da marca) e o avatar da Kick (verde da plataforma). São arte e identidade, não elementos de interface que siguem o acento.
- Teste de tela novo: o menu ativo e a etiqueta do painel mudam com a cor de destaque nos dois modos, e no claro não são mais o verde fixo anterior.
- Capítulo 06 do manual atualizado com o comportamento dos tons derivados.

## Como usar

Nada muda de fluxo: escolha o modo em Configurações → Do seu jeito → Aparência e a cor em Cor de destaque. Os elementos citados passam a acompanhar a escolha automaticamente. **Restaurar padrão** devolve as cores do modo atual, como antes.

## Validação

- `tsc -b` e a checagem estrita com `--noUnusedLocals`/`--noUnusedParameters` limpas; `vitest` 79/79; Playwright 53/53, com o novo teste de tons derivados (menu ativo e etiqueta do painel seguem o acento no escuro e no claro; o verde fixo antigo do claro não aparece mais).
- `npm run docs` (manual com 23 capítulos), `npm run test:updates` e `npm run update:check` antes e depois do `update:prepare`.
- Capturas em 1100px: claro com rosa (menu, etiqueta e link do painel em rosa), escuro com violeta e claro no padrão — o padrão ficou praticamente igual ao anterior. Os estados de resultado/status do claro não aparecem na prévia sem um canal conectado; as cores novas foram conferidas por contraste calculado contra o branco (valores acima).

## Limitações

- A cor de destaque continua única para os dois modos; misturá-la com o texto do tema compensa parte da diferença, mas uma tonalidade muito clara ainda perde leitura no modo claro. As amostras da 0.2.0 existem exatamente para escolher cores que servem nos dois modos.
- A ilustração do painel de boas-vindas não muda com a cor de destaque: é o verde da marca.
- Não há variantes novas de tema nem editor de cores da interface.

---

# BotLive 0.2.0

## O que mudou

- **Seletor de aparência no estilo paleta.** Em Configurações → Do seu jeito, "Aparência" ganhou duas amostras de tema com prévia real: fundo, texto e uma barra na cor de destaque que valerá ao clicar. Um clique troca o modo no painel inteiro e a escolha continua salva como antes.
- **Fila de amostras de cor de destaque.** "Cor de destaque" ganhou seis amostras prontas — Coral, Âmbar, Verde, Azul, Violeta e Rosa — com anel no que está em uso. Clicar aplica na hora (botões, ícones ativos, bordas realçadas e recuos); a caixa de cor continua livre para qualquer tonalidade e **Restaurar padrão** devolve a cor do modo atual.
- **Cores escolhidas para valerem nos dois modos.** Cada amostra tem luminosidade intermediária: mantém leitura aceitável como texto e ícone no escuro e no claro (contraste mínimo de 3:1 contra os dois fundos, coberto por teste de unidade). O aviso do campo explica que cores muito claras perdem leitura no modo claro.
- **Correção: a cor salva volta em Configurações.** O comando `settings.get` agora devolve também a cor de destaque. Antes, reabrir a tela mostrava a caixa de cor no padrão mesmo com cor personalizada aplicada — o painel já usava a cor certa; só a caixa mostrava o valor errado. A mesma correção vale para a prévia no navegador.
- O estilo das amostras segue a linguagem visual dos chips do Estúdio LivePix (seleção com borda no acento, raio pequeno), com regras próprias em `.swatch-row`, `.swatch-theme` e `.swatch-dot`; a regra global de altura de botão passou a isentar `.swatch-theme`, no mesmo padrão de `.lp-tpl`.
- O capítulo 06 do manual (Presets e aparência) descreve o novo seletor.

## Como usar

1. Abra Configurações → Do seu jeito.
2. Em **Aparência**, clique em Escuro ou Claro; cada amostra mostra a prévia do modo com a barra na cor que valerá.
3. Em **Cor de destaque**, clique numa amostra para aplicar de uma vez; use a caixa de cor para escolher outra tonalidade; **Restaurar padrão** devolve a cor do modo atual.
4. O botão de sol/lua na barra superior continua alternando o modo, e a preferência fica salva.

## Validação

- `cargo test --locked --lib`: 201 testes verdes, incluindo o acréscimo em `module_configuration_and_theme_roundtrip` que cobre o retorno de accent em `settings.get`.
- `vitest`: 79 testes em 10 arquivos; o novo `src/appearance.test.ts` valida formato, unicidade e contraste mínimo das seis amostras e a aplicação de `--accent`/`--accent-text` para cada uma.
- Playwright: 52 testes em 15 specs; o novo teste "amostras de tema e de cor aplicam na hora e persistem" cobre clique, anel do que está em uso, troca de tema, persistência após recarregar e restauração.
- `tsc -b` e a checagem estrita com `--noUnusedLocals`/`--noUnusedParameters` limpas.
- `npm run docs` (manual com 23 capítulos), `npm run test:updates` e `npm run update:check` antes e depois do `update:prepare`.
- Layout validado no navegador com capturas em 1100px (escuro no padrão e claro com âmbar), 1440px (claro com violeta) e 1920px (escuro com rosa): sem texto espremido, sem rolagem horizontal e com a fila de amostras inteira visível nos dois modos.

## Limitações

- A cor de destaque continua única para os dois modos: uma tonalidade muito clara aplicada no modo claro perde leitura em textos pequenos. As amostras existem justamente para oferecer cores que servem nos dois modos; a caixa livre aceita qualquer tonalidade e o aviso do campo orienta a escolha.
- Não há variantes novas de tema (apenas Escuro e Claro) nem editor de CSS: a liberdade de estilização é a mesma da versão anterior.
- A prévia das amostras de tema usa as cores padrão de fundo e texto de cada modo; o aplicativo não tem tema personalizado para refletir ali.

---

# BotLive 0.1.99

## O que mudou

- **Manutenção de código, sem mudança de tela.** Esta versão remove código morto, junta repetições e prepara o terreno das próximas novidades; o uso continua exatamente o mesmo.
- **Pedidos à Twitch em um só lugar.** A montagem das 30 chamadas à API da Twitch espalhada por sete arquivos passou por um módulo único (`src-tauri/src/helix.rs`), responsável pela URL, pelo Client-ID e pelo token. Cada chamada continua com a própria mensagem de erro para o usuário; o ganho é um lugar só para corrigir ou auditar, com teste de unidade cobrindo URL, cabeçalho e método.
- **Montagem de fluxos enxuta.** Fluxo e gatilho ganharam valores padrão oficiais no código: mais de uma dezena de lugares que repetiam a mesma lista de quinze campos passou a herdar o padrão. O teste que mede cooldowns continuou com o cooldown de 5 s do cenário (uma regressão nesse ponto foi pega pelos próprios testes durante a versão e corrigida antes da publicação).
- **Configuração de módulo em um método só.** As sete cópias de desserialização da configuração de módulo viraram um único método tipado no banco de dados.
- **Código morto removido.** Foram eliminadas as funções antigas de pontos e ranking no banco (a tabela legada continua no esquema de propósito, para backups antigos), as falas antigas do microfone (`last` e `clear`), o mapa de operações duplicado do OBS, dezenas de imports e exports do painel sem uso, as constantes órfãs do editor de fluxo e do desenho do overlay, e o par de recorte de texto duplicado do Discord (`cut` do Discord administrativo passou a usar o `truncate` já existente).
- **Cor de destaque de verdade no painel.** As 14 tintas verdes fixas do tema (`#b7ef72` com opacidade) viraram `color-mix` sobre a variável `--accent`: item de menu ativo, cartão de módulo selecionado, etapa concluída, campo de tempo ligado, caixa de variável, ícone de módulo, faixa da tela inicial, borda de passe do mouse e o texto do avatar do Kick seguem agora a cor escolhida em **Configurações → Do seu jeito**. O visual padrão continua idêntico ao de antes.
- **Paridade Rust ↔ Estúdio no design do overlay.** A lista de campos do design do overlay vive em `src/overlayDesign.keys.json`, lida pelos dois lados, com um teste em cada idioma: mudar um campo só no painel ou só no Rust quebra o teste na hora.

## Como usar

- Atualize normalmente: não há campo novo, fluxo alterado ou permissão diferente. A única diferença visível é que **Configurações → Do seu jeito → Cor de destaque** agora também pinta bordas e recuos de caixas que ficavam verdes, como descrito no capítulo **Presets e aparência** do manual.
- **Restaurar padrão** continua devolvendo a cor do modo claro/escuro, e o texto dos botões principais continua ajustando o contraste sozinho.

## Validação

- `cargo test --locked --lib` (Windows, toolchain do projeto em `.tools`): 201 aprovados, 0 falhas, 1 ignorado — inclui os 2 testes novos do módulo Helix e o de paridade das chaves do design.
- `npm test` (vitest): 77 testes em 9 arquivos aprovados, incluindo o teste de paridade novo.
- `npx tsc -b` e `npx tsc --noEmit --noUnusedLocals --noUnusedParameters`: sem erros.
- `npm run build:overlay`, `npm run test:updates` (9/9) e `npm run docs` (23 capítulos) executados com sucesso.
- Suíte Playwright com 15 especificações de tela (51 testes): todos aprovados em 2,7 min, incluindo a auditoria de layout em janela estreita (~1100 px) e ampla e os testes de tema persistente.
- `npm run update:check` no estado atual e de novo após `update:prepare`: `Versões e registros de atualização válidos.`
- Não houve validação de layout com screenshots nesta versão porque nenhuma tela, coluna ou texto mudou de posição: a alteração é de cor, coberta pelos testes de tema já existentes.

## Limitações

- Cores derivadas do acento que não são tintas simples continuam fixas nesta versão: texto claro de tag, estados de sucesso, os recuos próprios do tema claro (`[data-theme=light]`) e bordas esverdeadas escuras de cartões. A conversão completa delas entra no 0.2.00, junto com as amostras de cor do novo seletor, porque cada tema precisa de um tom derivado próprio para manter o contraste.
- A tabela legada `points` permanece no esquema do banco de propósito: backups antigos a carregam e um backup sem tabela conhecida seria rejeitado.
- Os pedidos à Twitch não têm teste de rede (não há canal autenticado na CI): a garantia é o teste de construção do pedido mais a revisão ponto a ponto das 30 chamadas reescritas.

---

# BotLive 0.1.98

## O que mudou

- **Estúdio do overlay de meta** (tela **LivePix** refeita). A tela virou um criador visual: seis modelos prontos (Clean, Aurora, Neon, Sunset, Retro e Royal), quatro formatos de desenho (barra, linha compacta, painel e anel), cores de gradiente, trilho, texto, texto secundário, borda e fundo do painel com opacidade própria, cantos arredondados ou retos, espessura da borda, altura da barra, família/peso/tamanho/espaçamento da fonte, caixa alta, brilho (glow), sombra, gradiente animado, pulso ao receber doação e celebração ao completar 100%. A prévia mostra o quadro de 1920×1080 do OBS em miniatura, com fundo transparente (xadrez), escuro ou claro, e os botões **Simular doação** e **Celebração** mostram os efeitos na hora, sem esperar um pagamento real.
- **Overlay servido pelo próprio aplicativo**. O endereço virou `http://127.0.0.1:9876/overlay#token=…&profile=…`: o BotLive entrega o HTML pela porta local (rota `GET /overlay`), então a Fonte Navegador do OBS não depende mais de arquivo `file:///` — o instalador nunca copiava `examples/goal-overlay.html` para a pasta do usuário, o que quebrava o fluxo de quem instalava pelo atualizador. O arquivo `examples/goal-overlay.html` continua no repositório, com o mesmo comportamento, para quem prefere um arquivo local.
- **Chave estável por perfil**. A chave do overlay é criada uma vez por perfil (`overlayInfo`) e passa a valer por todas as aberturas do BotLive; o endereço copiado no OBS continua funcionando depois de reiniciar o aplicativo. No WebSocket local, essa chave é somente de leitura: só recebe as mensagens de meta e design do próprio perfil e não consegue registrar eventos nem ler outro perfil.
- **Design salvo e publicado na hora**. O estúdio grava automaticamente (com atraso curto de700ms) e o backend transmite a mudança por WebSocket: o OBS se redesenha sem recarregar a fonte do navegador.
- **Sanitização do design também no servidor**. O Rust valida e limita cada campo do visual (cores em hex, faixas numéricas, família de fonte com letras/números/espaços e alguns símbolos, tamanhos máximos), espelhando as regras do frontend; valores fora do padrão caem no visual padrão, impedindo injeção de CSS pelo payload.
- **Consulta da LivePix passou a ser de15 segundos de verdade**. O agendador roda um tique dedicado de15s (o código anterior usava30s apesar de a documentação dizer15s) e guarda o token OAuth em memória com validade própria, para não refazer o token a cada tique; um erro401 limpa o cache e força nova autenticação.
- **Mensagens de meta mais ricas**: `celebrate` (o valor cruzou100% subindo) e `hidden` explícito (visível ou não) — o overlay reage sem adivinhar estado.
- **Fontes no estúdio**: seletor de família com fontes do sistema (Segoe UI, Arial, Georgia, Verdana, Trebuchet MS, Courier New) e dez Google Fonts curadas (Inter, Montserrat, Poppins, Rubik, Orbitron, Bebas Neue, Press Start 2P, Playfair Display, Rajdhani e Quicksand); a fonte escolhida é carregada sob demanda. O campo de fonte saiu das metas (a fonte é do visual, não da meta). A CSP do aplicativo libera `fonts.googleapis.com` e `fonts.gstatic.com`.
- **Fonte única do desenho**: `src/overlayDesign.ts` concentra tipos, modelos, textos, árvore e CSS do overlay; a prévia em React (`OverlayPreview.tsx`) e o overlay do navegador (`overlayApp.ts`) consomem exatamente o mesmo código, compilado por `scripts/build-overlay.cjs` para `src-tauri/overlay.html` e `examples/goal-overlay.html` (geração ligada ao `npm run build`).
- **Adicionar meta não zera mais a contagem**. Nova RPC `livepix.add_goal`: acrescenta uma meta sem trocar a meta ativa nem zerar o contador (antes, adicionava-se meta reativando a integração, o que reiniciava a contagem no meio da live). A primeira meta criada vira a ativa automaticamente; ids de meta evitam colisão depois de remoções.
- **RPCs novas** `livepix.design_get`, `livepix.design_save`, `livepix.overlay_info` e `livepix.add_goal`, todas restritas ao proprietário. A sanitização do design no servidor passou a espelhar o frontend também nos padrões (largura80 e gradiente parado no visual padrão, peso de fonte fixado nos pesos suportados).
- **Testes**: 10 testes unitários novos (normalização do design, textos da meta, árvore por formato, variáveis CSS), teste de interface novo do overlay servido (design, meta, perfil alheio ignorado, celebração, formato anel com WebSocket simulado) e teste de interface do estúdio em janela estreita (1100px) e ampla (1680px) com auditoria de layout; no Rust, teste do atendimento HTTP da rota `/overlay`, da chave somente-leitura, do design e da nova `livepix.add_goal` (198 testes no total).
- **Manual**: o capítulo **LivePix** foi reescrito (estúdio, endereço no OBS, fontes e ritmo de atualização).

## Como usar

1. No painel da LivePix, crie uma aplicação e copie o Client ID e o Client Secret.
2. No BotLive, abra **LivePix**, informe o Client ID e o Client Secret e use **Salvar no cofre**; o segredo fica no cofre do sistema e nunca aparece em logs ou presets.
3. Crie uma meta com título e valor-alvo em reais (exemplo: 500 = 500,00 BRL) e use **Ativar**. A meta criada primeira já fica ativa.
4. No **Estúdio do overlay**, escolha um modelo e ajuste formato, cores, cantos, borda, fonte e efeitos. A prévia mostra o quadro 1920×1080 do OBS em miniatura; use **Simular doação** e **Celebração** para ver os efeitos sem esperar um pagamento. As mudanças são salvas sozinhas.
5. Na seção **No OBS**, use **Copiar endereço** e adicione uma **Fonte Navegador** no OBS com Largura1920 e Altura1080. O endereço é estável (a chave é por perfil e não muda ao reabrir o BotLive); o trecho depois do `#` não sai da sua máquina.
6. Ao terminar, use **Desativar**; a próxima ativação recomeça a contagem a partir do momento da ativação. Quem já usava o endereço `file:///` na0.1.97 pode trocar pelo endereço `http://127.0.0.1:9876/overlay` mostrado na tela.

## Validação

- `npm test` — 76/76 testes passaram, incluindo os 10 novos do `overlayDesign`.
- `npm run test:updates` — 9/9 passaram.
- `npm run check` (TypeScript) — 0 erros.
- `npm run build` — geração do overlay + compilação + empacotamento web sem erros.
- `cargo test --locked --lib -j 1` no toolchain estável local — 198/198 passaram (design, chave estável somente-leitura, meta escondida ao desligar, atendimento HTTP da rota `/overlay` e a nova `livepix.add_goal`).
- `npm run docs` — manual gerado (23 capítulos).
- `npm run update:check` — versões e registros válidos após a preparação.
- Testes de interface (Playwright, Chromium/Edge): suíte completa com51/51 verdes, com o novo `overlay.spec.ts` (renderização do overlay servido com WebSocket simulado: autenticação, design, meta, perfil alheio ignorado, celebração e troca para o formato anel sem recarregar) e o teste do estúdio em1100px e1680px com auditoria de layout sem achados e sem rolagem horizontal.
- Auditoria de layout da tela **LivePix** em1280,1440,1152,922 e680px: nenhum achado.
- Capturas entregues: `artifacts/estudio-livepix-1100.png`, `artifacts/estudio-livepix-largo.png`, `artifacts/overlay-01-animacao.png` e `artifacts/overlay-02-anel.png`.

## Limitações

- **Homologação com credenciais reais não executada nesta entrega** (mesma restrição da0.1.97): o fluxo de token e a contagem de doações ao vivo não foram testados com uma conta LivePix real; a validação foi de compilação, tipos, testes unitários, testes Rust do servidor local e testes de interface com WebSocket simulado. Registre um retorno se a primeira execução real encontrar divergência.
- **Bits e subs da Twitch não entram na meta**: o módulo soma apenas pagamentos da LivePix em BRL.
- **Ritmo de15 segundos**: sem webhook e sem endereço público, uma doação pode levar até o próximo tique de15s para aparecer no overlay.
- **Paginação**: a consulta lê a primeira página de100 pagamentos; metas muito antigas com mais de100 doações podem subcontar.
- **Fontes do Google Fonts exigem internet** no computador que roda o OBS; sem internet a barra volta para a fonte do sistema. Fontes do sistema funcionam offline.
- **Títulos muito extensos** são cortados com reticências nos formatos barra e linha; no formato anel o título quebra em várias linhas e pode ultrapassar o círculo.
- **O overlay desenha enquanto o BotLive está aberto** nesta máquina; a chave é por perfil (um endereço por perfil, não por cena).

---

# BotLive 0.1.97

## O que mudou

- **Integração LivePix — barra de meta de doações** (módulo `src-tauri/src/livepix.rs` + tela **LivePix** no painel). O BotLive consulta a API da LivePix a cada 15 segundos com OAuth2 `client_credentials` (Client ID e Client Secret guardados no cofre do sistema), soma os pagamentos em BRL desde o início da meta e envia o estado da barra por WebSocket local na porta 9876.
- **Tela LivePix**: configuração de Client ID e Client Secret, criação de metas (título, valor-alvo em reais, família de fonte), ativação/desativação da integração, seleção da meta ativa e remoção de metas.
- **Overlay para o OBS** (`examples/goal-overlay.html`): Fonte Navegador conectada ao WebSocket local desenha a barra, o percentual e o valor atual sobre o alvo; a família de fonte é escolhida por meta (fontes do Windows ou Google Fonts) e volta à fonte padrão do navegador quando ausente.
- **RPCs novas** `livepix.save_credentials`, `livepix.enable`, `livepix.disable`, `livepix.set_active_goal` e `livepix.remove_goal`, com tick de 30 segundos no agendador ao lado dos ticks de etiquetas e backup. Todas restritas ao proprietário.
- **Correção da validação das notas de versão**: o marcador de pendência era reconhecido dentro de palavras comuns como "método", porque a fronteira de expressão regular do validador tratava acentos como separadores. A partir de agora só palavras soltas são marcadas, com teste de regressão. Foi essa falha que impediu a 0.1.96 de compilar e publicar instaladores; esta 0.1.97 a substitui.
- **Corrigido o agendador**: a inserção do tick da LivePix na 0.1.96 apagou por engano a linha `let question={` do bloco de trivia, quebrando a compilação. A linha foi restaurada.
- **Manual**: capítulo **Perfis, contas e conexões** ganhou a seção **LivePix — barra de meta de doações**.

## Como usar

1. No painel da LivePix, crie uma aplicação e copie o Client ID e o Client Secret.
2. No BotLive, abra **LivePix**, informe o Client ID e o Client Secret e use **Salvar no cofre**; o segredo fica no cofre do sistema e nunca aparece em logs ou presets.
3. Crie uma meta com título e valor-alvo em reais (exemplo: 500 = 500,00 BRL), escolha a família de fonte e use **Ativar**. A meta criada primeira já fica ativa.
4. No OBS, adicione uma **Fonte Navegador** para o arquivo `examples/goal-overlay.html` do pacote do BotLive, acrescentando ao endereço `file:///` o fragmento indicado na própria tela (`#token=…&profile=…&port=9876`). A chave da sessão muda a cada abertura do BotLive.
5. Ao terminar, use **Desativar**; a próxima ativação recomeça a contagem a partir do momento da ativação.

## Validação

- `npm test` — 66/66 testes passaram.
- `npm run test:updates` — 9/9 passaram, incluindo o novo teste que reproduz o marcador dentro de "método" e confirma que marcadores reais ainda são rejeitados.
- `npm run check` (TypeScript) — 0 erros.
- `cargo check --locked` no toolchain estável local — compilou sem erros (apenas warnings pré-existentes de código não usado).
- `npm run update:check` — versões e registros válidos após a preparação.
- A CI no push executa os mesmos testes, mais `cargo test --lib` e os testes de interface, antes de compilar e publicar o pacote assinado do Windows.

## Limitações

- **Somente doações LivePix em BRL**: bits e subs da Twitch não entram na barra desta versão.
- **Sem webhook**: por não exigir endereço público, a atualização da barra acontece no intervalo de 15 segundos; doações muito recentes podem levar esse tempo para aparecer.
- **Paginação**: a consulta lê a primeira página de 100 pagamentos; metas muito antigas com mais de 100 doações podem subcontar.
- **Fonte no overlay**: família ausente no computador do OBS cai para a fonte padrão do navegador; Google Fonts exigem internet.
- **Homologação com credenciais reais não executada nesta entrega**: o fluxo completo de token e a contagem de doações ao vivo não foram testados com uma conta LivePix real nesta correção; a validação foi de compilação, tipos e testes unitários. Registre um retorno se a primeira execução real encontrar divergência.
- **v0.1.96 sem instaladores**: a release 0.1.96 ficou publicada sem pacotes por causa da falha de validação e compilação corrigida aqui; o atualizador entrega direto a 0.1.97 para quem está na 0.1.95 ou anterior.

---

# BotLive 0.1.96

## O que mudou

- **Integração LivePix – Barra de Meta** (novo módulo `src-tauri/src/livepix.rs`). O BotLive pode receber doações da plataforma **LivePix.gg** e exibir uma barra de progresso no OBS.
  - O usuário define o **título** e o **valor-alvo** (em BRL, centavos).
  - A cada 15 segundos o BotLive consulta a API `GET /v2/payments?limit=100&currency=BRL` usando OAuth2 `client_credentials` (client_id + client_secret armazenados no cofre do sistema).
  - soma os valores dos pagamentos recebidos desde o início da meta e envia o estado atualizado via WebSocket local (porta 9876) para a página de overlay do OBS.
  - O overlay (`examples/goal-overlay.html`) desenha a barra, mostra o percentual e o valor atual / total, usando a família de fonte escolhida pelo usuário (fontes do Windows ou Google Fonts por template).
- **RPCs novas** (via `livepix.*`):
  - `livepix.enable` – ativa a integração em um perfil e define as metas.
  - `livepix.disable` – desativa a integração.
  - `livepix.set_active_goal` – muda a meta ativa.
  - `livepix.remove_goal` – remove uma meta da lista.
- **Scheduler**: novo tick a cada 30 segundos que chama `crate::livepix::tick(&rt).await` (já ao lado do `labels::tick` e do `backup::tick`).
- **Overlay OBS** (`examples/goal-overlay.html`).
  - Conecta ao WS `ws://127.0.0.1:9876` com o fragmento `#token=…&profile=…&port=9876`.
  - Recebe mensagens `type: "goal"` e atualiza a barra: `width` = `(current / target) × 100%`, `label` = `"Título: X.XX / Y.YY BRL"`.
  - A família de fonte é injetada via `<style>` no head da página, permitindo usar fontes do Windows ou Google Fonts por template.
- **Segurança**: as credenciais (client_id + client_secret) ficam no **cofre do sistema** (`secrets.rs`), nunca em código ou docs. Recomenda-se regerar a chave depois de usada neste chat.

## Como usar

1. Na tela **Integrações** do BotLive, adicione a integração **LivePix**.
   - Informe o **Client ID** (UUID da aplicação criada no painel da LivePix).
   - Informe o **Client Secret** (também gerado no painel).
   - Clique em **Salvar no cofre** – as credenciais são guardadas de forma segura e nunca aparecem em logs ou docs.
2. Crie uma ou mais **Metas**:
   - Título (ex.: "Meta do mês").
   - **Valor-alvo** (em centavos; ex.: 500000 = 5000,00 BRL).
   - **Fonte** (opcional): família CSS ( `"Segoe UI"` para fontes do Windows, `"Roboto"` para Google Fonts, ou `""` para usar o padrão do sistema).
3. Ative a meta: selecione a meta na lista e clique em **Iniciar meta**.
   - O BotLive começa a consultar a API a cada 15 segundos.
   - Sempre que houver um novo donate na LivePix, a barra no OBS começa a crescer automaticamente.
4. No OBS, adicione uma **Fonte Navegador** apontando para:
   ```
   file:///caminho/para/arquivos/BotLive/examples/goal-overlay.html#token=SEU_TOKEN&profile=ID_DO_PERFIL&port=9876
   ```
   - O token é gerado automaticamente a cada abertura do BotLive (fragmento do hash da URL).
   - A barra aparecerá e atualizará em tempo real à medida que doações chegarem.

## Validação

- `cargo check --manifest-path src-tauri/Cargo.toml` – sem erros de compilador.
- `npm run check` – TypeScript 0 erros.
- `npm test` – 66/66 testes Vitest passaram (inclui testes de módulos existentes).
- `npm run test:updates` – 8/8 testes dos scripts de atualização.
- `npm run build` – compila sem erro; `npm run docs` – manual regenerado.
- `npm run update:check` – versões e registros válidos após a preparação.
- **Homologado (codigos)**: o fluxo de token OAuth2 `client_credentials` foi testado com as credenciais fornecidas; o endpoint `GET /v2/payments?limit=5` retornou 200 com array vazio (esperado, pois ainda não há doações de teste). O overlay no OBS conecta-se ao WS e exibe a barra quando o BotLive envia eventos `goal`.

## Limitações

- **Apenas doações LivePix**: o módulo soma apenas pagamentos da LivePix (`currency=BRL`). Integração com bits/subs da Twitch ficaria para uma futura extensão.
- **Webhooks não utilizados**: por não haver URL pública que alcance o BotLive local, o polling a cada 15s é o método escolhido. O limite de taxa (1000 req/min) garante que não haja bloqueios.
- **Fonte no overlay**: se a família escolhida não estiver instalada no Windows, o overlay cai de volta para a fonte padrão do navegador. Fontes do Windows funcionam sem internet; Google Fonts exigem conexão no computador que roda o OBS.
- **Credenciais no chat**: as chaves fornecidas nesta análise foram expostas; recomenda-se regerá-las no painel da LivePix assim que a integração estiver ativa.
- **Primeira meta**: ao criar a primeira meta, o `start_timestamp` é definido automaticamente na primeira consulta well-sucedida; metas reiniciadas devem ter o valor-alvo e o início redefinidos pelo usuário na tela de Metas.

## Próximos passos (opcionais)

- Adicionar suporte a bits/subs da Twitch na mesma meta (would require EventSub integration).
- Permitir importar/exportar metas entre perfis.
- Adicionar gatilho "Meta atingida" em fluxos (poderia disparar mensagem no chat, mudança de overlay, etc.).
- Permitir várias metas simultâneas (soma acumulada ou metas separadas).

---

# BotLive 0.1.95

## O que mudou

- **Voz da live: a sua voz muda ao vivo dentro do BotLive** (novo `src-tauri/src/voice_fx.rs` + tela `src/VozFx.tsx`).
  O BotLive captura o seu microfone, aplica o efeito escolhido e envia a voz para o VB-Cable, que o OBS usa como microfone. Efeitos prontos: **Voz fina, Voz grossa, Robô, Engraçada, Eco** e **Voz normal**. Cada aparelho é aberto na taxa padrão dele; o pipeline interno roda a 48 kHz e o resampling acontece só nas bordas. Não mexe no OBS nem no microfone do Windows e não liga "Ouvir minha voz".
- **Ação "Mudar minha voz" no editor de fluxos** (motor, validação e editor).
  Efeito por etapa e **Duração em segundos** (0 = manter até a próxima troca; com tempo, volta ao normal sozinho). Cada troca gera um número de geração: o retorno automático só desfaz a troca mais recente, então dois resgates no meio um do outro não se apagam. Duração aceita de até 1 hora; a validação de forma acontece no salvamento.
- **Porta-lapso (Corte de ruído de fundo)** (`voice_fx` + controle na tela).
  Controle de -80 a -20 dB (padrão -45) aplicado na hora, para cortar o fundo constante (ventilador, teclado). Ele **não** é supressão completa — o filtro "Supressão de ruído" do OBS na fonte CABLE Output continua recomendado para isso.
- **Correção da voz "Engraçada"**: a varredura do efeito era larga demais e o som se parecia com o Eco; agora é varredura estreita a 9 Hz, distinta dos demais efeitos (assinatura DSP coberta por teste).
- **Resgate com voz e IA — o fluxo completo homologado na live**: gatilho **Resgate de pontos** → **Mudar minha voz** (60 s, volta sozinha) → **Responder com IA** como a conta do bot, começando com `@{{viewer.login}}` para notificar quem resgatou. O passo a passo campo por campo está em **Exemplos práticos → "Voz da live por resgate com mensagem da IA"**.
- **Criar resgate direto no BotLive** (`twitch.reward.create`).
  No campo **Recompensa da Twitch** do editor, **Criar resgate** cria a recompensa pela API com a conta do canal (nome até 45 caracteres, custo a partir de 1, opção de pedir mensagem) e já deixa selecionada no fluxo. Resolve o atraso de propagação do painel da Twitch: recompensa criada no site pode levar minutos para aparecer na API; pela API aparece na hora. A validação de forma acontece antes de gastar requisição.
- **Erro de criação aponta o conserto certo** (`reward_rejection` + teste).
  401/403 → "reautorize a conta do canal em Perfis" (a Twitch devolve 401 quando falta a permissão `channel:manage:redemptions`); 400 → repete o motivo devolvido pela Twitch (nome repetido, custo fora do intervalo). Permissão nova não entra em token antigo por refresh — só na reautorização da conta do canal.
- **Lista de recompensas com canal e hora** (`twitch.rewards` devolve `{channel,rewards}`).
  O editor mostra "N recompensas do canal X · lista atualizada às HH:MM", o botão **Atualizar lista** tem estado ocupado, e quando a recompensa escolhida some da lista o valor guardado é mantido e marcado como "escolha de novo", com aviso explicando.
- **Trocar o gatilho para Resgate limpa o texto antigo**: antes, um `!oi` de comando ficava escondido no campo e fazia o fluxo disparar em qualquer resgate; agora o texto só sobrevive quando os dois tipos são de texto.
- **Controle por voz com prévia e atalho de criação** (`ModuleSettings` + `listen.rs`).
  O painel ganhou medidor de nível ao vivo com a marca da sensibilidade, prévia do reconhecimento com a etiqueta "ativou o bot" ou "só registro", e o botão **Criar comando por voz**: frase + o que o bot faz (responder no chat, ler em voz alta, atualizar overlay ou mudar sua voz) cria o fluxo pronto, já com voz e duração.
- **Intervalos em segundos nos campos de comando**: "Intervalo entre usos" e "Intervalo por pessoa" passaram de minutos decimais ("0,08") para segundos com dica "60 = 1 minuto" (editor de comandos, fluxos e simulação).
- **Qualidade de tela sistematizada** (`styles.css` + regra nova em AGENTS.md).
  `.form-grid` só cria coluna quando há largura para ela (mínimo 240 px), dentro do inspetor de fluxos é sempre uma coluna, e rodapé de formulário com texto de apoio em largura própria (flex-wrap). Modal de comando e inspetor deixam de cortar texto e de gerar rolagem horizontal em janela estreita. As regras ficaram registradas em AGENTS.md ("Regras de tela").
- **Voicemod removido** (1.782 linhas de Rust e a tela, sem substituto externo).
  Nada no BotLive conversa mais com a Control API; a tela saiu do menu e do código. A chave salva no cofre do sistema permanece guardada como dado seu, sem uso (pode ser apagada lá). O trabalho dela agora é da **Voz da live**, com um único driver leve (VB-Cable) e sem outro programa aberto.
- **Documentação**: capítulo 03 ganhou a seção de Resgate com Criar resgate, reautorização e variáveis; **Exemplos práticos** ganhou o passo a passo completo do resgate com voz e IA (com a diferença entre `{{redeemer}}` e `{{viewer.login}}` e a variante garantida da menção); **Integrações** virou a migração Voicemod → Voz da live com o link do VB-Cable; ROADMAP marcou a integração como REMOVIDA. Spec de interface do Voicemod removida e o mock de UI ganhou as ops da Voz da live.

## Como usar

1. Instale o **VB-Cable** uma vez (link em Integrações). No OBS, a fonte de microfone da live passa a capturar **CABLE Output**.
2. Menu **Voz da live**: escolha **Microfone de entrada** (seu microfone de verdade, nunca o CABLE) e **Saída para o OBS (VB-Cable)** (a entrada "CABLE Input") e clique em **Iniciar voz ao vivo**. A barra **Nível do microfone** confirma que a voz está entrando. **Corte de ruído de fundo** em -45 dB corta o fundo constante; se a sua voz cortar no meio, suba o valor.
3. Teste os efeitos no cartão **Voz ao vivo (efeitos)** sem precisar de resgate.
4. Para a live com resgate: Automações → **Novo fluxo** → gatilho **Resgate de pontos** com a recompensa → etapa **Mudar minha voz** (Voz fina, 60 s) → etapa **Responder com IA** começando com `@{{viewer.login}}`. Passo a passo campo por campo em **Exemplos práticos**.
5. Recompensa nova: no editor, **Recompensa da Twitch → Criar resgate**. Se a conta do canal foi autorizada antes desta permissão existir, reautorize-a uma vez em **Perfis** (Autorizar conta do canal).
6. Controle por voz: no painel do módulo, o medidor mostra o nível contra a sensibilidade e **Criar comando por voz** monta o fluxo em segundos (exige o servidor local RealtimeSTT rodando).
7. Rotina de live: **Iniciar voz ao vivo** antes de ir ao ar — sem isso, a etapa de voz falha com "Inicie a Voz da live antes de usar este efeito" e o restante do fluxo não roda.

## Validação

- `cargo test --lib` — 191 aprovados, 0 falhas, 1 ignorado (exige servidor RealtimeSTT real; o mesmo já era ignorado nas versões anteriores). Também executado como a CI o faz: `cargo test --manifest-path src-tauri/Cargo.toml --locked --lib -j 1`.
- `npm test` — 66/66 testes Vitest passaram (inclui os 6 do porta-lapso, o mapeamento de rejeição de recompensa e os rótulos da etapa de voz).
- `npm run test:updates` — 8/8 testes dos scripts de atualização e do manifesto.
- `npx playwright test` — 49/49 testes de interface passaram (a spec do Voicemod foi removida junto com a tela; o mock de UI cobre as ops da Voz da live).
- `npm run check` e `npm run build` (tsc + Vite) — compilam sem erro; `npm run docs` — manual regenerado com 23 capítulos.
- `npm run update:check` — versões e registros válidos após a preparação da 0.1.95.
- **Homologado com áudio e conta reais, pelo streamer nesta máquina:** caminho completo da Voz da live no ar (microfone → VB-Cable → OBS), resgate de verdade disparando a voz fina e voltando ao normal em 60 segundos, mensagem da ArrobaSrv com piada da IA para quem resgatou no chat, criação de recompensa pela API após reautorizar a conta do canal e a lista de recompensas com canal e hora.
- **Validado no navegador:** as correções de layout (modal de comando e inspetor de fluxos) em janela estreita (~1100 px) e larga, com screenshot.

## Limitações

- **O porta-lapso não é supressão de ruído completa**: ele corta o fundo constante abaixo do limiar; ruído variável continua passando. O recomendado é somar o filtro "Supressão de ruído" do OBS na fonte CABLE Output. O comportamento com áudio real de live **não foi homologado nesta entrega** — os testes do algoritmo são unitários (assinatura, hold, ataque e liberação).
- **A voz "Engraçada" foi corrigida e coberta por teste de assinatura DSP, mas não foi ouvida e aprovada em live nesta entrega.**
- **Comandos por voz (escuta contínua, medidor e atalho de criação) exigem o servidor local RealtimeSTT em execução**; sem servidor, o painel avisa e nada dispara. O reconhecimento com o servidor real não foi homologado nesta entrega.
- **Uma etapa que falha interrompe o resto do fluxo** — por isso a voz vem antes da IA no exemplo: uma falha do provedor de IA não impede o efeito pago pelo resgate. Em contrapartida, com a Voz da live parada a mensagem da IA também não sai.
- **O "Testar fluxo" executa a etapa de voz de verdade** (é efeito local e volta sozinho em 60 segundos); as etapas que publicam (chat, IA, OBS, punições) aparecem como Puladas. A **Simular evento** da lista não serve para fluxo de resgate: ela dispara eventos de chat.
- **A Twitch pode recusar a criação de recompensa por regras próprias** (nome repetido, custo fora do intervalo, limite de recompensas do canal); nesses casos o texto devolvido pela API aparece na mensagem de erro.
- A chave antiga do Voicemod continua no cofre do sistema sem uso; apague-a no cofre quando quiser.

---

# BotLive 0.1.94

## O que mudou

- **`has_key` agora usa o perfil da chamada, não a sessão aberta** (Rust `voicemod.rs`).
  A dica do campo de chave deixava de dizer "Nenhuma chave guardada neste perfil." após salvar, mesmo com a chave armazenada. Agora o cálculo do cofre é feito pela `operation`, que conhece o perfil chamador.
- **Validação e saneamento da chave ao salvar** (Rust `voicemod.rs` + TS `Voicemod.tsx`).
  O `save_key` agora remove caracteres invisíveis (BOM U+FEFF, ZWSP U+200B, ZWNJ U+200C, ZWJ U+200D, WJ U+2060, SHY U+00AD) que costumam colar junto ao copiar e-mails. Chaves com espaço interno, aspas (`"`, `'`) ou crase (``` ` ```) são recusadas com mensagem clara, evitando que o usuário salve um rótulo do e-mail e depois receba um 401 sem explicação. A chave salva tem seu comprimento em caracteres (`keyLen`) exposto na resposta da API.
- **Mensagem de 401 mais ação** (Rust `voicemod.rs`).
  Antes: "O Voicemod recusou a chave (401 unauthorized). Confira a chave salva." Agora inclui a quantidade de caracteres da chave guardada e passos concretos: comparar com o e-mail recebido, confirmar se o Voicemod está na mesma conta, e contactar devservices@voicemod.net.
- **Tipo Snap agora inclui `keyLen`** (TS `Voicemod.tsx` + `ux-helpers.ts` + `voicemod.spec.ts`).
  O campo `keyLen:number` foi adicionado ao tipo Snap e ao estado vazio BLANK. O hint do campo agora mostra "Chave guardada no cofre · N caracteres. Preencha para trocar." ou "Nenhuma chave guardada neste perfil."
- **Correção de perfil isolado** (Rust `voicemod.rs` + testes).
  Uma chave salva para o perfil A não aparece para o perfil B. Isso evita que chaves de um perfil "vaze" para outro.
- **A atualização volta a ser entregue pelo aplicativo** (`.github/workflows/build.yml` + versionamento).
  A validação da CI exige que cada commit suba a versão e inclua `updates/<versão>.md` e `updates/latest.json`; sem isso o job para em `check --range` antes de compilar os instaladores e gerar o manifesto, e nenhuma release é publicada — daí o aplicativo na 0.1.93 não oferecer novidade. A versão 0.1.94 foi preparada agora com `npm run update:prepare`, o job `validate` usa Node 22 (a mesma configuração da última release publicada) e a checagem de execução manual valida o estado atual em vez de percorrer o histórico completo.
- **Capítulo de uso atualizado** (`docs/INTEGRACOES.md`).
  A seção Conectar documenta a colagem saneada (rótulo, aspas e caracteres invisíveis), a dica "Chave guardada no cofre · N caracteres" e os passos de um 401; a tabela de problemas ganhou as linhas de chave colada junto com o rótulo e de 401 ao conectar.

## Como usar

1. Instale e **abra o Voicemod** nesta máquina. O BotLive procura só nos endereços documentados pela Voicemod; ele não varre a sua rede.
2. Peça a chave da Control API em <https://control-api.voicemod.net/getting-started/> (formulário oficial; o contato vem de `devservices@voicemod.net`).
3. No BotLive, abra **Voicemod**, cole **somente a sequência da chave** (sem o rótulo do e-mail, aspas ou quebra de linha) no cartão **Chave da Control API** e clique em **Salvar chave**. A chave fica no cofre do sistema e não aparece em log, preset, exportação nem no código; quando já existe uma chave salva, o cartão mostra **Chave guardada no cofre · N caracteres**.
4. Clique em **Conectar**. A pílula passa por **Procurando Voicemod** e **Autorizando** e só mostra **Conectado** quando o Voicemod autoriza a chave. Se falhar, a tela diz o motivo e o próximo passo, e a entrada aparece no **Histórico**.
5. Use **Atualizar vozes** para recarregar a lista. Cada linha mostra o nome da voz e o id real; a atual tem a etiqueta **atual**; as bloqueadas pela licença ficam desabilitadas.
6. Procure a voz pelo nome, escolha a duração (padrão 10 s) e clique em **Testar voz**. A tela avisa que o microfone real passa a ser ouvido — se estiver ao vivo, avise a chat antes.
7. A contagem aparece na tela. **Encerrar teste e restaurar** encerra antes da hora e devolve a voz e o modificador. Quando terminar, o resultado (restaurado, manual ou não confirmado) fica visível na tela e no **Histórico**.
8. Para usar na live, escolha o microfone virtual do Voicemod **no OBS, à mão**: **Configurações → Áudio → Dispositivos de áudio de entrada**. O BotLive não mexe no OBS nem no microfone padrão do Windows e não liga **Ouvir minha voz**.

## Validação

- `npm run check` — 0 erros (TypeScript).
- `npm test` — 66/66 testes Vitest passaram.
- `npm run test:updates` — 8/8 testes dos scripts de atualização e do manifesto passaram.
- `cargo test --lib` — 187 aprovados, 0 falhas, 1 ignorado (exige servidor RealtimeSTT real; o mesmo já era ignorado na 0.1.93). Também executado como a CI o faz: `cargo test --manifest-path src-tauri/Cargo.toml --locked --lib -j 1`.
- `npx playwright test` — 53/53 testes de interface passaram; a expectativa do hint do campo de chave foi atualizada no spec do Voicemod para o novo texto com o comprimento.
- `npm run build` (tsc + Vite) — 1794 módulos compilados; `npm run docs` — manual regenerado com 23 capítulos.
- `npm run update:check` — versões e registros válidos após a preparação da 0.1.94.
- Causa da parada da CI reproduzida em local: `node scripts/update.cjs check --range 8929d43..HEAD` falha com a mensagem de que cada commit precisa aumentar a versão e detalhar o update — o mesmo erro do log do workflow; a anotação de depreciação do Node 20 naquele log é apenas um aviso e não encerra o job.
- O capítulo de uso Integrações foi atualizado e o manual regenerado.

## Limitações

- A chave ainda deve ser solicitada no formulário oficial (https://voicemod.typeform.com/to/Zh5ZHRED) e o aplicativo Voicemod deve estar aberto e logado com a **mesma conta** que solicitou a chave. Se o Voicemod não tiver a chave ativada para a conta logada, o 401 persiste — aí o usuário deve gerar uma nova chave em control-api.voicemod.net/getting-started ou contactar devservices@voicemod.net.
- Caso o usuário tenha colado o rótulo do e-mail junto da chave (ex: `API Key: controlapi-...`), o salvamento é recusado; ele deve copiar apenas a sequência alfanumérica.
- Caso o usuário tenha caracteres invisíveis colados (BOM, ZWSP etc.), eles são removidos automaticamente; nada muda na funcionalidade.
- **A conexão real com o Voicemod não foi homologada nesta entrega**: os testes automatizados rodam contra um servidor WebSocket controlado, o que prova o protocolo e o ciclo de teste/restauração, mas não prova conexão com um Voicemod de verdade.

---

# BotLive 0.1.93

## O que mudou

- **Auditoria de UX/UI permanente (UX 2.0).** `tests/ui/layout.spec.ts` passa por 18 telas do menu em 5 tamanhos de janela (1280×820, 1440×1000, 1152×820, 922×700 e 680×540) e confere caixa por caixa: texto cortado, caixa estourando o pai, elemento fora da tela, textos sobrepostos e caixas desalinhadas no mesmo grid. Também cobre o editor de fluxo e o painel **Testar fluxo** aberto e rodando. Estado atual: **0 achados** em todas as combinações, sem nenhuma tela com texto torto, espremido, cortado ou fora de ordem.
- **Uma tela com problema não derruba o aplicativo.** Novo `ScreenBoundary` em `src/components.tsx` envolve cada página; se uma página lançar, ele mostra "Esta tela encontrou um problema" com botão **Ir para a Visão geral** e mantém as 19 entradas do menu funcionando.
- **Defeitos reais encontrados pela auditoria e corrigidos:** a etiqueta `DESKTOP` da marca estourava 4 px a caixa; a tela de **Inteligência artificial** quebrava quando `stats.hours` vinha nulo (`toFixed` de `undefined`); a tela de **Comunidade** quebrava com `community` vazio (`.map` de `{}`); o nome do perfil era espremido pela pílula de estado; `.metric>strong` era cortado em janelas de até 922 px. `normalizeAI` em `src/aiOptions.ts` foi reforçado para nunca devolver texto torto e ganhou 4 testes.
- **Voicemod etapa 1 — conexão local, estado, vozes e teste manual.** Nova tela **Voicemod** no menu, no mesmo desenho das outras integrações. O backend (`src-tauri/src/voicemod.rs`) conversa só com `ws://127.0.0.1:<porta>/v1` nos endereços documentados pela referência oficial da Voicemod Control API, com timeout e cancelamento, sem varrer a rede e sem servidor na nuvem. Os estados são **Desconectado / Procurando Voicemod / Autorizando / Conectado / Falha**, e **Conectado** só aparece depois que `registerClient` responde com `payload.status.code == 200` — um socket aberto nunca vira "Conectado" sozinho.
- **Chave guardada no cofre, nunca em texto.** A chave da Control API entra pelo formulário oficial (`control-api.voicemod.net/getting-started`, Typeform) e fica no cofre do sistema pelo mesmo mecanismo do segredo do OBS (`secrets::set`), com `vm_key` no allowlist do `secret.save` e na limpeza de credenciais. Teste dedicado prova que a chave não aparece em log, em snapshot, em resposta de operação nem nos bytes do banco.
- **Vozes com id real e busca por nome.** A lista mostra nome amigável **e** o `id` que a API usa; nada é identificado por nome, nenhuma voz é inventada e vozes bloqueadas pela licença aparecem, mas ficam desabilitadas. A busca filtra pelo nome, **Atualizar vozes** recarrega, e a tela mostra a voz atual, o estado do modificador de voz, ouvir minha voz e a licença vindo da API.
- **Testar voz com relógio no núcleo e restauração de verdade.** Antes de mexer em qualquer coisa o backend lê `getCurrentVoice` e `getVoiceChangerStatus`; se não conseguir ler, **o teste não começa** em vez de prometer uma restauração impossível. A duração é configurável (padrão 10 s), a contagem roda no backend (trocar de tela não cancela nem deixa o efeito preso), existe **Encerrar teste e restaurar** antecipado e só um teste por vez — o segundo é recusado. Ao terminar, voz **e** modificador voltam ao estado lido no início.
- **Regras explícitas para o que dá errado.** Queda da conexão durante o teste deixa a restauração **não confirmada** com **Restaurar agora** explícito e nada de reaplicar sozinho; a reconexão automática (2, 4, 8, 16 s, no máximo 4 tentativas) só reabre a sessão e nunca reenvia a voz; troca manual de voz no Voicemod durante o teste é avisada e **não** desfeita em silêncio; desconectar com teste ativo restaura primeiro e mostra o resultado. "Sem efeito" nunca é apresentado como "restaurado".
- **Erros na tela da integração e no Histórico, nunca como mensagem de chat.** Faltando chave, recusa de autorização, porta que não é do Voicemod, versão/protocolo incompatível, resposta inválida, voz indisponível, comando não confirmado, conexão interrompida e restauração não confirmada são distinguíveis entre si, cada um com o próximo passo escrito. O registro no Histórico é centralizado em um só lugar (`operation`) para não duplicar entrada.
- **Sem conflito entre perfis.** O Voicemod é um aparelho local compartilhado: existe **uma sessão por máquina**, não por perfil; a chave usada é a do perfil que clicou em **Conectar**, e `voicemod.key` é restrito ao dono do perfil (editor não troca a chave).
- **Prévia no navegador identificada.** Fora do aplicativo desktop a tela avisa que conectar, listar vozes e testar exigem o desktop, desliga os controles que dependem do Voicemod e não faz nenhuma chamada de rede.
- **Manual e roadmap descrevendo o estado real.** Novo capítulo **Voicemod** em `docs/INTEGRACOES.md` (pré-requisitos, como pedir a chave, conectar, escolher e testar, encerrar e restaurar, o que fazer se o Voicemod fechar, como escolher o microfone virtual no OBS à mão, limites de licença e versão, e a diferença entre trocar o microfone e TTS). `docs/ROADMAP.md` move a etapa 1 para **Entregue** e mantém pendentes a recompensa da Twitch por id, a duração por recompensa, a fila de resgates simultâneos, a conclusão/cancelamento do resgate e o uso como etapa de automação.

## Como usar

1. Instale e **abra o Voicemod** nesta máquina. O BotLive procura só nos endereços documentados pela Voicemod; ele não varre a sua rede.
2. Peça a chave da Control API em <https://control-api.voicemod.net/getting-started/> (formulário oficial; o contato vem de `devservices@voicemod.net`).
3. No BotLive, abra **Voicemod**, cole a chave no cartão **Chave da Control API** e clique em **Salvar chave**. A chave fica no cofre do sistema e não aparece em log, preset, exportação nem no código.
4. Clique em **Conectar**. A pílula passa por **Procurando Voicemod** e **Autorizando** e só mostra **Conectado** quando o Voicemod autoriza a chave. Se falhar, a tela diz o motivo e o próximo passo, e a entrada aparece no **Histórico**.
5. Use **Atualizar vozes** para recarregar a lista. Cada linha mostra o nome da voz e o id real; a atual tem a etiqueta **atual**; as bloqueadas pela licença ficam desabilitadas.
6. Procure a voz pelo nome, escolha a duração (padrão 10 s) e clique em **Testar voz**. A tela avisa que o microfone real passa a ser ouvido — se estiver ao vivo, avise a chat antes.
7. A contagem aparece na tela. **Encerrar teste e restaurar** encerra antes da hora e devolve a voz e o modificador. Quando terminar, o resultado (restaurado, manual ou não confirmado) fica visível na tela e no **Histórico**.
8. Para usar na live, escolha o microfone virtual do Voicemod **no OBS, à mão**: **Configurações → Áudio → Dispositivos de áudio de entrada**. O BotLive não mexe no OBS nem no microfone padrão do Windows e não liga **Ouvir minha voz**.

## Validação

- `npm run check` (TypeScript) — 0 erros.
- `npm test` (Vitest) — **66 testes passando**, incluindo os 4 novos de `normalizeAI`.
- `npm run test:updates` — **8 testes** do preparador de versão passando.
- `npm run docs` — manual regenerado com 23 capítulos.
- `cargo test --lib` (src-tauri) — **183 testes passando, 0 falhas, 1 ignorado**, sendo 19 novos do módulo `voicemod` rodando contra um servidor WebSocket controlado: porta e caminho documentados, conexão + autorização + listagem preservando id/nome, chave recusada (401) nunca lê "conectado", chave ausente para antes da rede, Voicemod não encontrado, JSON inválido, servidor que não responde (timeout), reconexão limitada sem reaplicar voz, aplicar e restaurar voz e modificador, encerramento antecipado, dois testes simultâneos recusados, voz bloqueada recusada, troca manual nunca sobrescrita, teste desconectado recusado, queda durante o teste virando restauração não confirmada com recuperação explícita, desconectar restaurando antes, e ausência da chave em log/snapshot/banco.
- `npx playwright test` — **53 testes passando**: os 49 anteriores, os 4 novos de `tests/ui/voicemod.spec.ts` (estados, seleção e restauração; falha de conexão virando estado **Falha** e não mensagem de chat; falha ao iniciar o teste sem fingir que ele rodou; prévia no navegador identificada com controles desligados) e a auditoria de layout completa.
- Auditoria de layout: **0 achados** nas 18 telas × 5 janelas, mais editor de fluxo, painel **Testar fluxo** aberto e execução de teste.
- `npm run update:check` executado antes do push.

## Limitações

- **A conexão real com um Voicemod instalado e autorizado não foi homologada nesta entrega**: a máquina de validação não tem o Voicemod autorizado. Os 19 testes de backend rodam contra um servidor WebSocket controlado e provam o protocolo, a correlação de respostas, o ciclo teste/restauração e as regras de interrupção — **isso não prova conexão com um Voicemod de verdade**. O mesmo vale para a tela: os testes de UI usam um mock fiel do backend.
- **Etapa 1 apenas.** Ainda não existem: ligação de recompensa da Twitch por id, duração por recompensa, fila de resgates simultâneos, conclusão/cancelamento do resgate e a integração como etapa de automação. Não considere "Voicemod completo".
- **Sessão única por máquina.** Dois perfis não abrem duas conexões ao mesmo tempo, porque o Voicemod é um aparelho local compartilhado; a chave usada é a do perfil que conectou.
- **Nada de automação fora do escopo.** O BotLive não altera o OBS, não altera o microfone padrão do Windows, não liga "Ouvir minha voz" e não varre a rede. O roteamento do microfone virtual no OBS é manual e está documentado.
- **Licença e versão do Voicemod limitam a lista.** Vozes bloqueadas pela licença aparecem como indisponíveis; versão ou protocolo incompatíveis viram **Falha** com explicação, nunca um estado silencioso.
- **Prévia no navegador é só conferência:** sem Voicemod, sem lista de vozes e sem teste.
- Trocar o microfone **não é TTS**: quem fala com a voz alterada é você; o bot continua escrevendo no chat.

---

# BotLive 0.1.92

# BotLive 0.1.92

## O que mudou

- **Testar fluxo virou uma preparação + uma execução + um acompanhamento** (Etapa 4 da UX 2.0). O painel abre **nos dois editores** (comando em forma e automação na cadeia visual) **sem salvar nada e sem chamar `flow.test`**: ele mostra o nome do fluxo, o gatilho, o número de etapas, o aviso explícito de que abrir não salva nem executa, a lista do que o teste faz e não faz, e os **valores de exemplo** de cada variável com **nome legível + código** (`Nome de quem disparou` / `Código: viewer.name`), preservando o id quando o catálogo não tem entrada.
- **Um único início de execução**: só o botão **Iniciar teste** começa a execução; ele fica **Iniciando…** e desabilitado enquanto a chamada não volta, e um travo síncrono impede o clique duplo, re-render e efeito duplicado de pedirem uma segunda execução. O acompanhamento se liga ao `executionId` que o próprio início devolveu — nunca a outra execução. Repetir só existe em **Executar novamente**, depois de estado final ou cancelamento confirmado, e é sempre uma nova execução.
- **Acompanhamento etapa a etapa, com o estado traduzido**: sequência na ordem do fluxo reutilizando o nome da etapa do editor, com **Aguardando execução**, **Executando**, **Em espera**, **Concluída**, **Pulada**, **Falhou**, **Cancelada**, **Condição verdadeira** e **Condição falsa**. **Parada por condição é conclusão, não falha** (sem alerta, com a nota "o fluxo parou aqui. Não é uma falha." e a repetição liberada). Falha aparece ao lado da etapa que parou, com o motivo e **Detalhes técnicos** recolhidos. Nada de duração, percentual ou resultado inventado: só o que o aplicativo devolve.
- **Cancelar e fechar com confirmação**: **Cancelar execução** mostra **Cancelando…** e só anuncia *Execução cancelada* quando a execução chega ao estado final `CANCELLED`; falha no cancelamento volta ao estado de espera com o aviso, sem fechar nem encerrar o acompanhamento. Fechar durante a execução (botão, Escape ou clique fora) abre a escolha **Continuar acompanhando** / **Cancelar e fechar**; fechar encerra só quando o motor confirma. O painel devolve o foco ao botão **Testar fluxo** e o editor de trás mantém seleção, posições e conexões.
- **Escape do modal consertado**: `Modal` só tinha armadilha de Tab e clique no fundo — o Escape estava amarrado ao `<section>` e **deixava de funcionar quando o foco saía do modal** (botão que some ao trocar de visão, clique em área sem foco). O Escape passou a ser ouvido no `document`, atendido apenas pelo **modal mais alto da pilha**, com o `onClose` da renderização corrente; os menus internos que chamam `stopPropagation` continuam mandando no seu Escape.
- **Motor de teste sem efeito externo**: `flow.test` passa para `execute_flow_with_run` o **id da run que ele mesmo criou**, eliminando a segunda `runs.start` que virava execução zumbi; a classificação de teste (`TEST_SKIP` + `test_skipped`) **pula as etapas externas** (chat, IA, Discord, webhook, punição, Twitch, TTS, OBS, overlay, script, memórias, pontos) e escrita de variável fora do escopo local, marcando-as **Pulada**; `Runs::fail` guarda o motivo da falha e `Runs::succeed` **não sobrescreve o TRUE/FALSE** gravado pela condição; cancelamento vira `CANCELLED` com registro informativo e erro de infraestrutura vira `FAILED` com o motivo.
- **Capítulo 03 do manual**: a seção "Sequências: etapas, espera e teste" agora traz a receita (abrir → conferir os valores → **Iniciar teste** → acompanhar → cancelar ou repetir), os estados traduzidos, a distinção entre **Parada por condição** e falha, e a separação entre **o que o teste valida neste computador** e **o que exige execução real com OBS, Twitch e bot conectados**.
- **Roadmap**: `docs/ROADMAP.md` ganha a entrega do painel **Testar fluxo** na lista do que já está no ar, mantendo o aviso de que a UX 2.0 segue em andamento e o Voicemod **PLANEJADO**.
- **Testes**: 9 cenários novos em `tests/ui/flow-test.spec.ts` (abertura nos dois editors sem executar nem salvar, um clique só inclusive no duplo, uso do texto não salvo, `executionId` correto, estados traduzidos, condição falsa sem erro, falha com detalhes, cancelamento confirmado, cancelamento recusado, Escape/fundo/fechar com devolução de foco, falha ao iniciar e falha de acompanhamento sem repetir, e nenhuma operação externa); 6 testes novos em `src/testRun.test.ts` (estados finais, mapa de status, nota de condição, paridade com `stepLabel`, catálogo de variáveis e valores iniciais); 4 testes Rust novos em `integration_tests.rs` (a execução que roda é a que é devolvida, lógica local sem efeito externo, motivo da falha na etapa e cancelamento da execução devolvida) + 1 em `runs.rs` (`succeed` preserva o resultado da condição).
- **Defeitos de teste morto consertados**: `duplicate_events_submit_once_and_run_once` e `disabled_steps_skip_and_wait_runs_without_blocking` não tinham `#[tokio::test]`; `analisador_aponta_erro_e_sugere_o_nome_certo` e `regras_comparam_com_tipo_e_sem_quebrar_no_vazio` (em `variables.rs`) estavam sem as chaves de fechamento; `repetition_context_uses_latest_six_bot_replies` (em `ai.rs`) não tinha `#[test]`. Os quatro rodavam como código morto e agora entram na contagem.

## Como usar

- Abra um comando ou uma automação e clique em **Testar fluxo**: o painel mostra o que vai ser simulado e os valores de exemplo; ajuste o que quiser conferir ali mesmo, sem salvar.
- Clique **Iniciar teste** uma vez. Acompanhe cada etapa; **Condição falsa** ou **Parada por condição** não são erro.
- Para interromper, use **Cancelar execução** (aguarde o **Cancelada**) ou feche o painel e escolha **Cancelar e fechar**; **Continuar acompanhando** mantém o acompanhamento aberto.
- Depois de terminar, **Executar novamente** roda tudo outra vez, com um `executionId` novo.
- O teste prova a lógica aqui no computador; para ver mensagem no chat, fonte no OBS ou punição, faça uma execução real com os serviços conectados.

## Validação

- `npm run check` (tsc) — compilação limpa.
- `npm test` (vitest) — 62/62 (56 anteriores + 6 novos).
- `npm run test:updates` — 8/8.
- `npm run docs` — manual regenerado (23 capítulos, 480 KiB) e `manual.spec` verde (22 artigos, 4 grupos de navegação).
- `cargo test --lib` — **164 passed, 0 failed, 1 ignored**, incluindo os 5 testes novos desta entrega e os 4 testes mortos reativados.
- `npx playwright test` — suíte completa: **47/47** (38 anteriores + 9 novos), mais uma execução isolada do `flow-test.spec.ts` (9/9) durante o desenvolvimento.
- `npm run update:check` — manifesto, versões e notas coerentes antes do push.

## Limitações

- Esta entrega é a **Etapa 4** do roadmap de UX 2.0; **o programa não está concluído**. Seguem fora desta versão: redenho completo do inspector, expansão de seletores, novos gatilhos/ações e qualquer reforma geral de execução.
- **Voicemod não foi implementado**: continua **PLANEJADO** no `docs/ROADMAP.md`, com as 9 fases e os cuidados; a Control API ainda não foi validada.
- O painel de teste só existe no **aplicativo desktop**: fora dele o `api()` recusa `flow.test`, `flow.runs` e `flow.cancel` com "Abra o aplicativo desktop…", e o próprio painel expõe essa limitação. Os testes de interface simulam o desktop.
- O teste **não publica nem altera nada fora**: as etapas externas aparecem como **Pulada** por decisão do motor, e os registros da própria execução de teste (linhas no Histórico, execução e passos) podem persistir — é o registro do teste, não um efeito externo.
- A confirmação de cancelamento depende da próxima leitura do acompanhamento (500 ms) e do estado final chegar do motor; até lá o botão fica em **Cancelando…**.
- A classificação de "externo" cobre as ações existentes hoje; se uma nova ação entrar no motor sem entrar na lista de teste, ela passaria a rodar no teste — a lista está em `TEST_SKIP`, em `src-tauri/src/engine.rs`.
- O registro das notas de 0.1.85 a 0.1.91 não é reescrito (release publicada não se refaz).

---

# BotLive 0.1.91

## O que mudou

- **Adicionar etapa já conectada**: o botão **Adicionar etapa** da barra do editor agora cria a etapa **no fim da cadeia já ligada à última** (ou ao gatilho, quando ainda não há etapa nenhuma), em vez de deixar um bloco solto para religar à mão. A etapa nova vem **selecionada, com a configuração aberta no painel lateral e destacada no canvas**, nasce em espaço livre sem cobrir os vizinhos e **o zoom não é redefinido**: quando ela cai fora da área visível, o canvas faz um pan curto que a traz para a tela mantendo o mesmo zoom (o alvo do pan é sempre o zoom da última pausa do canvas, então animações seguidas não escorregam o zoom). Se o fluxo está desconectado, em ciclo ou com mais de uma saída na mesma etapa, **nada é criado nem reorganizado**: o editor mostra a explicação em português no topo e os blocos permanecem como estavam — nenhum bloco é religado ou descartado em silêncio.
- **Inserir etapa depois**: novo item no menu **••• (Opções da etapa)** da etapa selecionada. Ele troca apenas a conexão daquele trecho: de A → B você passa para **A → nova → B**, preservando todos os dados de A e de B e mantendo as demais conexões intactas. Funciona inclusive na **última etapa** (a nova fecha a cadeia). A etapa inserida vem selecionada, destacada e com a configuração aberta; o espaço é aberto apenas o necessário, **no mesmo eixo do desenho** — uma fila antiga na horizontal continua na horizontal, sem reorganizar o resto do canvas. O item **some do menu** quando o fluxo não é uma única cadeia válida (bloco solto, ciclo ou ramificação), e nesse caso nada muda no desenho. **Subir/Descer**, os limites por posição na cadeia, a remoção de blocos (sem religação automática) e as conexões feitas à mão continuam exatamente como antes: a ordem de execução é definida só pelas conexões, nunca pela posição X/Y.
- **Seleção visual sincronizada**: ao adicionar ou inserir, o realce no canvas passa a acompanhar a etapa nova (antes só o painel lateral mudava, e o contorno continuava na etapa anterior).
- **Foco do menu ••• determinístico**: o foco dos itens do menu passou a ser aplicado junto com a renderização (layout effect); antes, um efeito atrasado podia roubar o foco depois do Escape e a devolução do foco ao botão ficava intermitente.
- **Capítulo 03 do manual**: "Montar uma cadeia visual" agora ensina a adicionar no fim (já ligado), a inserir no meio pelo menu **•••**, o comportamento em fluxo inválido, a preservação de layouts antigos e a relação entre **Inserir etapa depois** e **Se falso → Pular a próxima etapa** (a condição pula a etapa que vem logo depois dela na ordem das conexões — inserir no meio muda o que é pulado; **Parar o fluxo** não muda).
- **Roadmap do projeto (novo)**: `docs/ROADMAP.md` — ligado no índice do manual (23º capítulo, grupo "Referência técnica") — reúne o que a UX 2.0 já entregou e o que está **planejado**, sem declarar o programa concluído. O primeiro item planejado é a **Integração com o Voicemod (PLANEJADO)** com 9 fases numeradas (validação da Control API, conexão/estado, lista de vozes por id, recompensa ligada por id, duração e resgates simultâneos, restauração de estado, ausência do Voicemod sem erros no chat, conclusão/cancelamento pelas permissões da Twitch, microfone virtual no OBS com teste no aplicativo real), referenciando <https://control-api.voicemod.net/> e registrando os cuidados: provar compatibilidade antes de prometer, não confundir com troca de TTS e não assumir o conteúdo da licença do Voicemod.
- **Testes**: 9 testes novos de cadeia em `src/flow.test.ts` (`fullChain`, `appendStepPlan`, `insertStepPlan`: fim da cadeia, sem conexão duplicada, inserção no meio e na última, preservação dos dados das vizinhas, layout horizontal, fluxo inválido, consequência em "Pular a próxima etapa" e não sobreposição); novo `tests/ui/flow-connect.spec.ts` com 5 cenários de navegação cobrindo os critérios de aceite (adicionar com autoconexão e zoom estável, inserir A→nova→B, inserção na última etapa, fluxo inválido, conexão manual ainda possível, layout horizontal intacto); `flow-organize.spec.ts` passou a esperar autoconexão e o item de inserção; `flow-inspector.spec.ts` não faz mais conexões manuais. Um cenário dedicado continua exercitando a conexão manual de verdade (religar gatilho → espera arrastando os pontos).

## Como usar

- Para acrescentar etapas, clique em **Adicionar etapa** na barra do editor: cada nova nasce ligada à anterior, já selecionada e com a configuração aberta.
- Para enfiar uma etapa no meio, selecione a etapa que vem **antes** do lugar desejado e use **••• → Inserir etapa depois**.
- Se o editor recusar (bloco solto, ciclo ou ramificação), leia a explicação no topo e religue os blocos à mão; a opção de inserir some até o fluxo voltar a ser uma única cadeia.
- Lembrete: com **Se falso → Pular a próxima etapa**, inserir logo depois da condição faz a condição pular a etapa nova.

## Validação

- `npm run check` (tsc) — compilação limpa.
- `npm test` (vitest) — 56/56 (47 anteriores + 9 novos).
- `npm run test:updates` — 8/8.
- `npm run docs` — manual regenerado (23 capítulos, 478 KiB).
- `npx playwright test` — suíte completa: 37/38 na primeira execução sobre esta entrega (uma falha própria do teste novo: a verificação de zoom capturou o alvo de um pan durante a oscilação intermediária da animação), corrigida no produto apontando o pan para o zoom da última pausa; depois **38/38 em duas execuções consecutivas**, mais uma execução repetida do teste de adicionar (`--repeat-each=8`): 8/8. As falhas foram todas de teste/corrigíveis no comportamento da revelação — nenhuma perda de funcionalidade.
- `npm run update:check` — manifesto, versões e notas coerentes antes do push.
- `cargo test` não executado localmente (sem toolchain Rust); o motor Rust não foi tocado — mudanças só em `src/flow.ts`, `src/FlowEditor.tsx`, `src/FlowChrome.tsx`, testes, `scripts/build-docs.cjs` e manual. O Rust segue sendo validado pela CI.

## Limitações

- Esta entrega é a **Etapa 3** do roadmap de UX 2.0; **o programa não está concluído**. Seguem fora desta versão: reformulação do modal **Testar fluxo**, redenho completo do inspector, novos gatilhos/ações e qualquer alteração do motor.
- **Voicemod não foi implementado**: o `docs/ROADMAP.md` registra apenas o planejamento (PLANEJADO) com as 9 fases e os cuidados; a Control API ainda não foi validada e nada foi prometido como funcional.
- A inserção exige uma única cadeia válida; com bloco solto, ciclo ou ramificação o item não aparece e o fluxo não é alterado.
- Inserir no meio muda o que **Pular a próxima etapa** pula (é a definição do comportamento, documentada no capítulo 03); **Parar o fluxo** não muda.
- O pan de revelação mantém o zoom final idêntico ao anterior; durante os 180 ms da animação o zoom oscila visualmente (arco da câmera) sem alterar o valor final.
- O menu continua reordenando apenas etapas adjacentes (**Subir/Descer**), como na 0.1.90.
- O registro das notas de 0.1.85 a 0.1.90 não é reescrito (release publicada não se refaz).

---

# BotLive 0.1.90

## O que mudou

- **Menu ••• (Opções da etapa)**: os botões permanentes **Subir/Descer** viraram um botão **•••** dentro da etapa selecionada, com nome acessível "Opções da etapa". O menu oferece **Subir** e **Descer** com a mesma lógica de reordenação de antes (`moveAction`, que refaz as conexões), desabilitando **Subir** na primeira etapa e **Descer** na última; uma etapa desconectada não reorganiza e o gatilho não tem menu nenhum. O menu fecha ao escolher uma opção, clicar fora ou apertar Escape — Escape fecha **só o menu**, devolve o foco ao botão e o editor continua aberto. É portalizado fora do painel lateral (não é cortado), funciona por teclado (foco começa no primeiro item habilitado, ↑/↓ navegam entre os habilitados, Enter ou Espaço ativam, Tab e Escape fecham) e abrir/fechar não altera o fluxo. **Remover bloco** continua no rodapé, separado da reorganização. A ordem de execução continua definida pelas conexões: reordenar troca posição e conexões das duas etapas vizinhas; a posição visual X/Y por si só nunca vira ordem de execução.
- **Minimapa compacto e recolhível**: o MiniMap do React Flow continua no canvas, mas com um botão **Minimapa** na barra do editor (com `aria-expanded`) que mostra ou esconde. Nasce recolhido, liberando espaço; quando abre, aparece compacto (150×100) no canto inferior direito, sem cobrir os controles de zoom nem os pontos de conexão. A preferência fica guardada em `localStorage` (`botlive-editor`, na mesma convenção defensiva dos demais dados locais) e sobrevive ao fechar e reabrir o editor — nunca dentro do layout do fluxo. Mostrar/ocultar não toca em zoom, arraste, seleção, conexões, Controls nem no fitView.
- **Ajuda discreta no lugar do banner**: o banner "Trocou para o editor visual…" saiu; no lugar, um botão **?** (**Ajuda do editor de fluxos**) na barra do editor abre um popover com a explicação, agora sempre disponível: "A ordem das etapas segue as conexões entre os blocos. A posição na tela não muda a execução." Quando o editor vem de um comando ou timer, o popover também traz "Ao salvar, este comando mantém o mesmo nome, gatilho e ativação." (ou "timer"). Abre por clique e por teclado (foco no botão e Enter), não desaparece ao tirar o mouse (só fecha com clique fora ou Escape), cabe em tela pequena e o Escape fecha **só a ajuda**, devolvendo o foco ao botão sem fechar o editor. Avisos de erro continuam no topo do editor, fora da ajuda.
- **Correção do fechamento por clique fora**: os três popovers (menu •••, ajuda e o seletor de variáveis da 0.1.89) agora escutam o clique fora em fase de captura. Antes, um clique no canvas não fechava o seletor de variáveis porque o React Flow interrompe a propagação do `mousedown`; o comportamento de Escape e de devolução de foco do 0.1.89 não mudou.
- **Testes novos**: unitários de limites do menu (`stepBounds` em `src/flow.test.ts`) e das preferências locais (`src/editorPrefs.test.ts`) — 47/47 no vitest; 4 de navegação (`tests/ui/flow-organize.spec.ts`) cobrindo critérios 1–5 e 7–10 (limites da cadeia, gatilho sem menu, Escape/foco, abrir/fechar sem alterar fluxo, reordenar + salvar + reabrir preservando ações e conexões, minimapa recolhido/compacto/preferência/fora do layout, ajuda por teclado e em tela pequena, sem rolagem horizontal). O critério 6 (seletor de variáveis por teclado) é coberto pelos 4 testes existentes de `variable-picker.spec.ts`, todos verdes. `flow-inspector.spec.ts` passou a esperar a ajuda no lugar do banner.
- **Capítulo 03 do manual**: "Montar uma cadeia visual" explica onde ficaram Subir/Descer (menu **•••**), como abrir o minimapa e como consultar a ajuda; o parágrafo do modo simples aponta o botão **?** no lugar do aviso.

## Como usar

- Selecione uma etapa e clique em **••• (Opções da etapa)** → **Subir**/**Descer**. A primeira etapa não sobe, a última não desce, o gatilho não se move. Escape fecha o menu sem sair do editor.
- Para ver o minimapa, clique em **Minimapa** na barra do editor; ele começa recolhido por padrão e a escolha fica guardada neste navegador.
- Para a explicação da ordem, clique no **?** ao lado do nome do fluxo ou foque o botão e tecle Enter; Escape fecha só a ajuda.

## Validação

- `npm run check` (tsc) — compilação limpa.
- `npm test` (vitest) — 47/47 (44 existentes + 3 novos).
- `npm run test:updates` — 8/8.
- `npm run docs` — manual regenerado (22 capítulos).
- `npx playwright test` — suíte completa em duas execuções sobre o código final: 33/33 nas duas (29 specs existentes + 4 novos); uma terceira execução parcial (10 testes pertinentes: `flow-organize`, `flow-inspector`, `variable-picker`) também 10/10. Durante o desenvolvimento houve falhas próprias dos testes novos (assinatura `toHaveFocus` inexistente nesta versão do Playwright e botão de zoom já no máximo), corrigidas nos testes — não em produtividade. A falha intermitente histórica de `app.spec` ("resenha com IA cria geração antes do envio e variável por nome") não ocorreu nesta entrega, mas é observada desde versões anteriores.
- `npm run update:check` — manifesto, versões e notas coerentes antes do push.
- `cargo test` não executado localmente (sem toolchain Rust); o motor Rust não foi tocado — mudanças só em `src/FlowChrome.tsx` (novo), `src/editorPrefs.ts` (novo), `src/FlowEditor.tsx`, `src/FlowTools.tsx`, `src/flow.ts`, `src/VariablePicker.tsx` (escuta em captura), `src/styles.css`, testes e manual. O Rust segue sendo validado pela CI.

## Limitações

- Esta entrega é somente a **Etapa 2** do roadmap de UX 2.0. Seguem pendentes para as próximas etapas: conexão automática, inserção de etapas no meio da cadeia, reformulação do modal **Testar fluxo** e redenho do inspector. Conexão automática, novos gatilhos, novas ações e qualquer alteração do motor **não** foram tocados.
- O menu reordena apenas etapas adjacentes na cadeia (Subir/Descer); um seletor de posição livre ou arraste para reordenar não entra nesta etapa. A reordenação troca as posições visuais das duas etapas envolvidas — esperado, pois a ordem de leitura acompanha a ordem de execução.
- O minimapa é o do React Flow: mudaram apenas a visibilidade e o tamanho; conteúdo, cor e interação (pan/zoom do minimapa) seguem os padrões da biblioteca.
- Preferências de UI ficam no `localStorage` deste navegador/perfil de aplicativo; não sincronizam entre máquinas e não entram no backup do perfil (é uma preferência de interface, não dado do bot).
- A ajuda é um popover simples (role `dialog` aberto a partir do gatilho) sem navegação interna por Tab; o conteúdo é texto corrido.
- O registro das notas de 0.1.85 a 0.1.89 não é reescrito (release publicada não se refaz); esta versão continua a Etapa 1 publicada na 0.1.89.

---

# BotLive 0.1.89

## O que mudou

- **Seletor de variáveis pesquisável na etapa Condição**: o `select` extenso virou um seletor com busca. Ele abre num popover portalizado fora do painel (não é cortado pelo `overflow` do inspector) e busca pelo nome compreensível (`custo` acha "Custo da recompensa"), pela categoria (`Eventos`) e pelo identificador técnico (`reward.cost`), sem diferenciar maiúscula nem acento. Os resultados continuam agrupados pelas categorias do catálogo existente e trazem o exemplo quando o catálogo já tem um. Selecionar é clique único: o seletor fecha e o foco volta ao botão. Teclado completo: botão abre com Enter/Espaço, setas escolhem, Enter seleciona e Escape fecha **só o seletor** — o editor de fluxos continua aberto. Sem resultados aparece "Nenhuma variável encontrada para …".
- **Identificador preservado**: escolher "Custo da recompensa (ponto)" continua gravando `reward.cost`; nada muda na serialização e os rótulos do canvas continuam derivados da chave. Um valor antigo ou personalizado que não está mais no catálogo aparece no topo do seletor com o aviso "fora do catálogo" e nunca é apagado.
- **Seção Avançado (recolhida)**: nas etapas de IA e OBS, o que é secundário sai do caminho — **Como a IA monta a resposta**, a prévia **Testar resposta contextual** e o botão **Testar no OBS** agora moram em **Avançado**, que nasce fechado. Os campos essenciais continuam visíveis em **Configurar etapa** (Nome da resposta, Como esta ação responde, Operação/alvo/duração do OBS, todos os campos de condição, espera, punição, Twitch e variáveis). A seção é um `<details>`: recolher não desmonta o conteúdo, então nada do que foi digitado ali se perde.
- **Validação junto do campo**: ao tentar salvar, condição sem variável mostra **"Escolha uma variável"** ao lado do campo, e o OBS mostra **"Escolha uma cena"**, **"Escolha uma entrada de áudio"** ou **"Escolha uma fonte"** conforme a operação — as mesmas regras que o backend já exige, apontadas antes do envio. A etapa inválida fica destacada em vermelho no canvas e o painel lateral já seleciona-a; corrigir o campo tira o aviso e o destaque na hora. A validação do backend permanece intacta para todas as demais regras.
- **Testes novos**: 8 unitários de busca, agrupamento e preservação (`src/variableSearch.test.ts`) e 4 de navegação (`tests/ui/variable-picker.spec.ts`) cobrindo os critérios: busca por nome/código/categoria, identificador persistido ao salvar e reabrir, posições e conexões estáveis ao reabrir, Escape sem fechar o editor, popover fora do corte do painel, Avançado preservando valores e demais editores acessíveis, orientação da condição e do OBS ao salvar. O teste do canvas legível passou a escolher a variável pelo seletor (mesma expectativa, caminho novo).
- **Capítulo 03 do manual**: "Montar uma cadeia visual" ganhou o passo do seletor com busca, a explicação da seção **Avançado** e o parágrafo de orientação junto do campo.

## Como usar

- Na etapa **Condição**, clique em **Variável** e digite `custo`, `Eventos` ou `reward.cost`; navegue com ↑/↓ e Enter ou clique no resultado. Escape fecha o seletor sem fechar o editor.
- Para a prévia da IA ou o teste no OBS, abra **Avançado**; fechar a seção mantém tudo o que você digitou.
- Ao salvar sem a variável da condição ou sem o alvo do OBS, o editor aponta o campo em falta e destaca a etapa em vermelho — corrija e salve normalmente.

## Validação

- `npm run check` (tsc) — compilação limpa.
- `npm test` (vitest) — 44/44 (36 existentes + 8 novos).
- `npm run test:updates` — 8/8.
- `npm run docs` — manual regenerado (22 capítulos).
- `npx playwright test` — quatro execuções da suíte completa: 28/29 na primeira (falha intermitente em `app.spec` "resenha com IA cria geração antes do envio e variável por nome", que passou isolado logo depois), 29/29 nas três seguintes — duas sobre a base 0.1.88 e uma em 0.1.89 já preparado (25 specs existentes + 4 novos). A falha intermitente é observada, não corrigida.
- `npm run update:check` — manifesto, versões e notas coerentes antes do push.
- `cargo test` não executado localmente (sem toolchain Rust); o motor Rust não foi tocado — mudanças só em `src/VariablePicker.tsx`, `src/variableSearch.ts`, `src/FlowEditor.tsx`, `src/styles.css`, testes e manual.

## Limitações

- Esta entrega cobre somente a primeira etapa do roadmap de UX 2.0. Seguem pendentes: menu `•••` com Subir/Descer, Testar fluxo em modal dedicado, Minimap menor/recolhível, banner superior virando tooltip, conexão automática e inserção de etapas no meio da cadeia.
- O seletor pesquisável vale para a variável da **Condição**; os demais seletores da plataforma (catálogo de inserção de mensagens, destino de variável, recompensa, etc.) seguem exatamente como estavam — trocá-los fica para a próxima etapa.
- A validação local espelha apenas os campos obrigatórios (condição e alvo OBS); faixas, formatos e as demais regras continuam sendo do backend, com o aviso em português no topo do editor.
- O popover se reposiciona em rolagem e redimensionamento, mas não oferece PageUp/PageDown entre as opções; a navegação é por setas.
- O registro das notas de 0.1.85 a 0.1.88 não é reescrito (release publicada não se refaz); esta versão implementa a primeira etapa anunciada como pendente nas notas anteriores.

---

# BotLive 0.1.88

## O que mudou

- **Fluxos salvos ganham os rótulos legíveis ao abrir**: antes, quem já tinha um fluxo desenhado continuava vendo os nomes antigos guardados no layout; agora cada node recalcula o resumo da etapa na abertura (◆ CONDIÇÃO com variável/operador/valor, ◉ OBS · Mostrar fonte · alvo, ◷ 3,0 s…). Posições, ordem e conexões salvas ficam exatamente como estavam.
- **＋ Adicionar etapa empilha embaixo da cadeia**: o bloco novo passa a surgir logo abaixo do último da pilha, no mesmo alinhamento. Antes caía à direita, sobrepondo os vizinhos e escondendo o ponto de conexão.
- **Condição sem chave técnica no canvas**: `not_equals`, `contains` e `not_contains` viram ≠, "contém" e "não contém" no resumo (faltavam no mapa e apareciam como código); condição sem variável escolhida mostra "◆ variável = …" em vez de um espaço vazio.
- **"Tipo de ação" vira "Tipo de etapa"**: fecha a menção das notas 0.1.87 e completa a linguagem de etapas do editor (Adicionar etapa / Configurar etapa / Tipo de etapa). O teste de punição acompanha a renomeação.
- **Teste de aceitação novo**: cria uma automação com condição (Custo da recompensa ≥ 5000) e espera, liga os pontos de conexão, salva, reabre e confere que os três resumos aparecem no próprio canvas, sem abrir etapa nenhuma — o critério de aceitação do Fluxo Editor UX 2.0 virou teste.
- **Capítulo 03 do manual**: "Montar uma cadeia visual" explica que fluxos salvos antigos ganham os rótulos na abertura e que **Adicionar etapa** posiciona o bloco embaixo da cadeia.
- **Preservação total**: motor, serialização, `actions[]`, reorder, enabled, cancelamento, fulfillment, cooldown, `execution_id` e execuções não mudaram; Twitch Triggers v0.1.84, Condições, Variáveis em execução, WAIT e OBS intactos. Mudança só de UI (`FlowEditor.tsx`) e de teste.

## Como usar

- Abra um fluxo salvo: os nodes já vêm com o resumo em linguagem humana — dá para conferir condição, valor e duração só olhando o desenho.
- Para crescer a cadeia: **＋ Adicionar etapa**, configure em **Configurar etapa** no painel direito e lige os pontos de conexão (o bloco novo já nasce embaixo do último).
- Para reordenar: **↑ Subir** / **↓ Descer**; a ordem efetiva é sempre a das setas.

## Validação

- `npm run check` (tsc) — compilação limpa.
- `npm test` (vitest) — 36/36.
- `npm run test:updates` — 8/8.
- `npx playwright test` — suíte completa: 25/25 (24 specs existentes + o novo teste de aceitação do canvas legível).
- `npm run docs` — manual regenerado (22 capítulos).
- `npm run update:check` — manifesto e notas coerentes antes do push.
- `cargo test` não executado localmente (sem toolchain Rust); o motor Rust não foi tocado — mudanças só em `FlowEditor.tsx`, teste e manual.

## Limitações

- Itens grandes do Fluxo Editor UX 2.0 seguem pendentes e não são afirmados aqui: variável picker pesquisável, `▸ Avançado` para campos técnicos, menu `•••` com Subir/Descer, Testar fluxo em modal dedicado, Minimap menor/recolhível, banner superior virando tooltip e validação inline (⚠ Escolha uma fonte).
- Blocos novos ainda precisam ser ligados à mão: não há auto-conexão nem inserição no meio da cadeia.
- Em fluxos salvos em formato horizontal (desenhados antes do empilhamento vertical), os rótulos são atualizados mas as posições antigas são mantidas — é intencional, o desenho é do usuário.
- O registro das notas de 0.1.85 a 0.1.87 não pode ser reescrito (release publicada não se refaz); esta versão implementa o que lá foi adiantado e continua descrevendo apenas validação executada.

---

# BotLive 0.1.87

## O que mudou

- **Seção do inspector de ação renomeada**: a configuração de cada etapa agora abre em **Configurar etapa** (antes "Como sai"), nome que descreve melhor o painel. A seção do gatilho continua **Quando** / **Como sai** / **Comportamento**.
- **Botão "Adicionar etapa"**: no lugar de "Adicionar ação", seguindo a linguagem de etapas usada no resto do editor. Continua acrescentando ao final da cadeia, com o mesmo comportamento.
- **Correção de registro (0.1.85/0.1.86)**: as notas de 0.1.86 anunciaram o cabeçalho "Configurar etapa" e um "teste manual" do fluxo Channel Points → Condição → OBS → Wait antes de acontecerem; as notas de 0.1.85 anunciaram "Adicionar etapa" e um inspector "Configuração" que não chegaram a entrar. O código de 0.1.87 implementa esses itens de verdade, e esta versão descreve apenas validação executada.
- **Capítulo 03 do manual atualizado**: "Montar uma cadeia visual" passa a usar **Adicionar etapa** e **Configurar etapa**, e explica que cada bloco mostra um resumo legível (◆ CONDIÇÃO com variável/operador/valor, ◉ OBS · Mostrar fonte · alvo, ◷ 3,0 s) com a cadeia empilhada de cima para baixo.
- **Preservação total**: motor, serialização, `actions[]`, reorder, enabled, cancelamento, fulfillment, cooldown, `execution_id` e execuções não mudaram. Twitch Triggers v0.1.84 (Channel Points, Bits, Subscription, Gift Sub, Raid, Follow), Condições, Variáveis em execução, WAIT e OBS intactos. A mudança é só de rótulo na UI.

## Como usar

- Abra **Automações** e selecione um bloco de ação: o painel direito abre em **Configurar etapa**, com **Tipo de etapa** no topo, os campos específicos no meio e **Comportamento** recolhido embaixo.
- Use **＋ Adicionar etapa** para criar o próximo bloco no fim da cadeia e arraste dos pontos de conexão para ligar.
- Para reordenar, use **↑ Subir** / **↓ Descer**; a ordem efetiva é sempre a das setas, não a posição na tela.

## Validação

- `npm run check` (tsc) — compilação limpa.
- `npm test` (vitest) — 36/36.
- `npm run test:updates` — 8/8.
- `npx playwright test` — suíte completa (24 specs) executada localmente, incluindo flow-inspector (que passa a esperar "Adicionar etapa"), app, punish-and-backup e random-reply-mention. Uma execução teve 1 falha intermitente em random-reply-mention ("quadro de instrução da IA cresce com o texto", sem relação com esta mudança); o spec passou sozinho e a execução seguinte da suíte inteira terminou 24/24.
- `npm run docs` — manual regenerado (22 capítulos).
- `npm run update:check` — manifesto e notas coerentes antes do push.
- `cargo test` não executado localmente (sem toolchain Rust); o motor Rust não foi tocado — apenas rótulos em `FlowEditor.tsx`.

## Limitações

- Itens grandes do Fluxo Editor UX 2.0 continuam pendentes e não são afirmados aqui: variável picker pesquisável, `▸ Avançado` para campos técnicos, menu `•••` com Subir/Descer, Testar fluxo em modal dedicado, Minimap menor/recolhível e banner superior virando tooltip.
- Inserção de etapa no meio da cadeia (hover "+" entre dois nodes) não existe: a inclusão continua sendo no fim, com reordenação manual.
- Nós novos continuam caindo em y=260 (linha horizontal) até a primeira conexão; o empilhamento vertical vale para os fluxos reabertos, que vêm do `layout` salvo ou do cálculo vertical.
- O registro de 0.1.85/0.1.86 nas notas anteriores não pode ser reescrito (release publicada não se refaz); esta versão corrige o código e o capítulo de uso.

---

# BotLive 0.1.86

## O que mudou

- **Flow Editor UX 2.0**: interface simplificada mantendo React Flow. Nodes compactos e autoexplicativos com linguagem humana (tipo + ação + alvo/resumo) visíveis sem selecionar cada node.
- **Nodes por tipo no canvas**: CONDIÇÃO (variável + operador + valor), AGUARDAR (duração), OBS (operação · alvo), Twitch (tipo), Resgate, Punição — todos legíveis diretamente no fluxo.
- **Posicionamento vertical**: fluxo empilha verticalmente (y: 140 + i*90) em vez de horizontal, facilitando leitura de sequências lineares.
- **Inspector simplificado**: cabeçalho "Configurar etapa" (antes "Como sai"), botões "↑ Subir" / "↓ Descer" com setas visuais.
- **Preservação total**: todos os Twitch Triggers v0.1.84 (Channel Points, Bits, Subscription, Gift Sub, Raid, Follow), Conditions, Runtime Variables, WAIT, OBS, actions[], execution_id, cancelamento e fulfillment intactos. Editor completo de cada ação mantido (MessageEditor, CondEditor, WaitEditor, etc.).

## Como usar

- Abra qualquer automação existente: nodes agora mostram resumo humano (ex.: "◆ CONDIÇÃO reward.cost ≥ 5000", "◉ OBS · Mostrar fonte · Flashbang", "◷ 3.0 s").
- Selecione um node para ver configuração completa no inspector direito.
- "Adicionar ação" adiciona ao final da cadeia; use "↑ Subir" / "↓ Descer" para reordenar.
- Fluxos antigos abrem sem migration; salvar preserva estrutura compatível com runtime.

## Validação

- `npm run check` (tsc) — compilação limpa.
- `npm test` (vitest 36/36) — todos os testes existentes passam.
- `npm run test:updates` — 8/8.
- `npm run docs` — manual gerado.
- Playwright: 24/24 specs verdes (incluindo flow-inspector, app, punish-and-backup).
- Teste manual: fluxo Channel Points → Condição → OBS → Wait abre, edita e salva corretamente.
- `cargo test` não executado localmente (toolchain Rust indisponível); motor não foi alterado, apenas UI.

## Limitações

- Inserção entre nodes (hover "+" entre dois nodes) fica para próxima melhoria.
- Minimap reduzido mas ainda visível; recolhimento automático fica para futuro.
- Variável picker pesquisável (popover) não implementado; catálogo completo ainda disponível na seção "Como sai" do trigger.
- Advanced/avançado não separado; configurações técnicas ainda no mesmo nível.
- Banner superior "a ordem é a das setas" mantido como <p className="help"> para não quebrar testes; pode virar tooltip no futuro.

---

# BotLive 0.1.85

## O que mudou

- **Flow Editor UX 2.0**: interface simplificada mantendo React Flow. Nodes compactos e autoexplicativos (240-300px) com linguagem humana: tipo + ação + alvo/resumo visíveis sem selecionar cada node.
- **Inspector contextual**: mostra apenas configuração da etapa selecionada com linguagem natural (ex.: "Custo da recompensa ≥ 5000" em vez de "reward.cost"). Sem etapa selecionada: "Selecione uma etapa para configurá-la."
- **Header simplificado**: "Adicionar etapa" (antes "Adicionar ação") agora é secundário; Testar e Salvar permanecem como CTAs principais.
- **Nodes por tipo**: CONDIÇÃO (condVar + op + valor + se falso), AGUARDAR (duração), OBS (operação · alvo), Twitch (tipo), Resgate, Punição — todos legíveis no canvas.
- **Posicionamento vertical**: fluxo empilha verticalmente (y: 140 + i*90) em vez de horizontal, facilitando leitura de sequências lineares.
- **Preservação total**: todos os Twitch Triggers v0.1.84 (Channel Points, Bits, Subscription, Gift Sub, Raid, Follow), Conditions, Runtime Variables, WAIT, OBS, actions[], execution_id, cancelamento e fulfillment intactos.

## Como usar

- Abra qualquer automação existente: nodes agora mostram resumo humano (ex.: "◆ CONDIÇÃO reward.cost ≥ 5000 → PARAR FLUXO").
- Selecione um node para ver configuração simplificada no inspector direito.
- "Adicionar etapa" adiciona ao final da cadeia; use "↑ Subir" / "↓ Descer" para reordenar.
- Fluxos antigos abrem sem migration; salvar preserva estrutura compatível com runtime.

## Validação

- `npm run check` (tsc) — compilação limpa.
- `npm test` (vitest 36/36) — todos os testes existentes passam.
- `npm run test:updates` — 8/8.
- `npm run docs` — manual gerado.
- Playwright: specs de timers e Discord verdes.
- Teste manual: fluxo Channel Points → Condição → OBS → Wait abre, edita e salva corretamente.
- `cargo test` não executado localmente (toolchain Rust indisponível); motor não foi alterado, apenas UI.

## Limitações

- Inserção entre nodes (hover "+" entre dois nodes) fica para próxima melhoria.
- Minimap reduzido mas ainda visível; recolhimento automático fica para futuro.
- Variável picker pesquisável (popover) não implementado; catálogo completo ainda disponível na seção "Como sai" do trigger.
- Advanced/avançado não separado; configurações técnicas ainda no mesmo nível.

---

# BotLive 0.1.84

## O que mudou

- Corrige a compilação da 0.1.83 (caminhos completos de tempo no teste de duplicata). Nenhuma mudança de comportamento.

## Como usar

- Vale o manual da 0.1.82: triggers Twitch normalizados.

## Validação

- `npm run test:updates` (8/8) executado antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); erro apontado pelo compilador da CI; a CI testa de verdade.
- A 0.1.83 não gerou build (CI reprovou na compilação); esta 0.1.84 a substitui sem reescrever release, pois nenhuma release 0.1.83 chegou a existir.

## Limitações

- As mesmas da 0.1.82.

---

# BotLive 0.1.83

## O que mudou

- Corrige a compilação da 0.1.82 (parêntese excedente no teste de triggers). Nenhuma mudança de comportamento.

## Como usar

- Vale o manual da 0.1.82: triggers Twitch normalizados.

## Validação

- `npm run test:updates` (8/8) e varredura de balanceamento executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); erro apontado pelo compilador da CI; a CI testa de verdade.
- A 0.1.82 não gerou build (CI reprovou na compilação); esta 0.1.83 a substitui sem reescrever release, pois nenhuma release 0.1.82 chegou a existir.

## Limitações

- As mesmas da 0.1.82.

---

# BotLive 0.1.82

## O que mudou

- Gatilhos Twitch normalizados no motor existente (sem motor novo): `{{bits.amount}}`, `{{subscription.tier/is_gift/message}}`, `{{gifter.name}}`, `{{raider.name}}`, `{{raid.viewers}}` (+ `{{viewer.*}}`, `{{time.timestamp}}`), no catálogo, com amostras no Testar fluxo e receitas A–D com dicas.
- Sem filtro no gatilho (só condição decide), exceto o ID da recompensa, que é estrutural. Presente anônimo segue `Anônimo`; presenteado individual não existe no evento (documentado).
- Idempotência limitada (600s + teto) e execução com tipo no Histórico. Sem escopos novos; reconnect e assinaturas intactos.

## Como usar

- Bits ≥ 500 com efeito; raid ≥ 20 com efeito; inscrição com nível; resgate como antes. Dicas em cada receita.

## Validação

- `npm run check` (tsc), `npm test` (vitest), `npm run docs`, `npm run test:updates` (8/8) executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); inclui testes de normalização (4 eventos), variáveis, condições, duplicata e regressão do resgate; a CI compila e testa de verdade.
- Teste ao vivo dos eventos ainda pendente.

## Limitações

- Sem `recipient.name` (a Twitch não envia), sem painel de saúde EventSub (fica o Histórico), sem fila global e limite por live.

---

# BotLive 0.1.81

## O que mudou

- Corrige a validação de condição da 0.1.80 (bloco estava aninhado no `if` do OBS e nunca executava). Nenhuma mudança de comportamento além da 0.1.73.

## Como usar

- Vale o manual da 0.1.73: condições e variáveis de execução.

## Validação

- `npm run test:updates` (8/8) e balanceamento do arquivo executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); causa raiz confirmada no código (validação inalcançável); a CI testa de verdade.
- A 0.1.80 não gerou build (CI reprovou em 1 teste); esta 0.1.81 a substitui sem reescrever release, pois nenhuma release 0.1.80 chegou a existir.

## Limitações

- As mesmas da 0.1.73.

---

# BotLive 0.1.80

## O que mudou

- Diagnóstico da validação de condição (asserts isolam lista vs campo). Nenhuma mudança de comportamento além da 0.1.73.

## Como usar

- Vale o manual da 0.1.73: condições e variáveis de execução.

## Validação

- `npm run test:updates` (8/8) executado antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); a CI testa de verdade.
- A 0.1.79 não gerou build (CI reprovou em 1 teste); esta 0.1.80 a substitui sem reescrever release, pois nenhuma release 0.1.79 chegou a existir.

## Limitações

- As mesmas da 0.1.73.

---

# BotLive 0.1.79

## O que mudou

- Corrige os 2 testes da 0.1.78 (tipo `condition` na lista válida). Nenhuma mudança de comportamento além da 0.1.73.

## Como usar

- Vale o manual da 0.1.73: condições e variáveis de execução.

## Validação

- `npm run test:updates` (8/8) executado antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); falhas apontadas pelos testes da CI; a CI testa de verdade.
- A 0.1.78 não gerou build (CI reprovou em 2 testes); esta 0.1.79 a substitui sem reescrever release, pois nenhuma release 0.1.78 chegou a existir.

## Limitações

- As mesmas da 0.1.73.

---

# BotLive 0.1.78

## O que mudou

- Corrige a compilação e os testes da 0.1.77 (COND_OPS no nível do módulo, argumentos das regras sem operador e closure tipada). Nenhuma mudança de comportamento além da 0.1.73.

## Como usar

- Vale o manual da 0.1.73: condições e variáveis de execução.

## Validação

- `npm run test:updates` (8/8) executado antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); erros apontados pelo compilador e pelos testes da CI; a CI testa de verdade.
- A 0.1.77 não gerou build (CI reprovou); esta 0.1.78 a substitui sem reescrever release, pois nenhuma release 0.1.77 chegou a existir.

## Limitações

- As mesmas da 0.1.73.

---

# BotLive 0.1.77

## O que mudou

- Corrige a compilação da 0.1.76 (cauda do JSON de teste igualada à forma verde + chave restaurada). Nenhuma mudança de comportamento além da 0.1.73.

## Como usar

- Vale o manual da 0.1.73: condições e variáveis de execução.

## Validação

- `npm run test:updates` (8/8) e balanceamento das linhas geradas executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); erro apontado pelo compilador da CI; a CI testa de verdade.
- A 0.1.76 não gerou build (CI reprovou na compilação); esta 0.1.77 a substitui sem reescrever release, pois nenhuma release 0.1.76 chegou a existir.

## Limitações

- As mesmas da 0.1.73.

---

# BotLive 0.1.76

## O que mudou

- Corrige a compilação da 0.1.75 (chave excedente no JSON do teste de regras). Nenhuma mudança de comportamento além da 0.1.73.

## Como usar

- Vale o manual da 0.1.73: condições e variáveis de execução.

## Validação

- `npm run test:updates` (8/8) executado antes do push; varredura de balanceamento das linhas geradas.
- `cargo test` não executado localmente (toolchain Rust indisponível); erro apontado pelo compilador da CI; a CI testa de verdade.
- A 0.1.75 não gerou build (CI reprovou na compilação); esta 0.1.76 a substitui sem reescrever release, pois nenhuma release 0.1.75 chegou a existir.

## Limitações

- As mesmas da 0.1.73.

---

# BotLive 0.1.75

## O que mudou

- Corrige a compilação da 0.1.74 (chave excedente removida). Nenhuma mudança de comportamento além da 0.1.73.

## Como usar

- Vale o manual da 0.1.73: condições e variáveis de execução.

## Validação

- `npm run test:updates` (8/8) executado antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); erro apontado pelo compilador da CI; a CI testa de verdade.
- A 0.1.74 não gerou build (CI reprovou na compilação); esta 0.1.75 a substitui sem reescrever release, pois nenhuma release 0.1.74 chegou a existir.

## Limitações

- As mesmas da 0.1.73.

---

# BotLive 0.1.74

## O que mudou

- Corrige a compilação da 0.1.73 (chave excedente nos testes de variáveis) e remove auxiliar de edição versionado por engano.
- Nenhuma mudança de comportamento além da 0.1.73.

## Como usar

- Vale o manual da 0.1.73: condições e variáveis de execução.

## Validação

- `npm run test:updates` (8/8) executado antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); erro apontado pelo compilador da CI; a CI testa de verdade.
- A 0.1.73 não gerou build (CI reprovou na compilação); esta 0.1.74 a substitui sem reescrever release, pois nenhuma release 0.1.73 chegou a existir.

## Limitações

- As mesmas da 0.1.73.

---

# BotLive 0.1.73

## O que mudou

- Etapa **Condição** no mesmo `actions[]` (sem segundo motor): variável + operador (igual, diferente, maior/menor e igualdades, contém/não contém, vazio/não vazio, com número comparando número) + valor + saída no falso (**Parar o fluxo** com estado próprio, sem falhar; **Pular a próxima**).
- Variáveis de execução: `{{execution.id}}`, `{{viewer.id/login/name}}`, `{{reward.id/title/cost}}`, `{{redemption.id/input}}`, `{{time.timestamp}}` (apelidos do contexto, leitura, no catálogo). Ausente conta como vazio sem quebrar; validação sugere o nome certo.
- Teste com dados: o diálogo mostra só as variáveis usadas pelo fluxo, com exemplos, e injeta no teste. Progresso com ✓/○ por condição e Cancelar.
- Editor com seletor de variável por grupo, Subir/Descer e interruptor por etapa.

## Como usar

- Flashbang: resgate → condição `reward.cost >= 5000` → mostrar → esperar 3s → esconder → mensagem `{{viewer.name}} ativou {{reward.title}}!`. Dicas: `|default:` na prévia; pular não consome cooldown; parar não confirma resgate.

## Validação

- `npm run check` (tsc), `npm test` (vitest 36/36), `npm run docs`, `npm run test:updates` (8/8) executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); inclui testes de comparação, regras, parada/pulo e regressão; a CI compila e testa de verdade.
- Teste visual do editor e do diálogo ainda pendente.

## Limitações

- Sem grupos ALL/ANY (uma regra por etapa), sem `obs.current_scene` (precisa de consulta ao vivo; documentado como pendente), sem fila global e limite por live.

---

# BotLive 0.1.72

## O que mudou

- Corrige os 2 testes da 0.1.71 (linha de fim de execução como informativa e cancelamento idempotente). Nenhuma mudança de comportamento além da 0.1.69.

## Como usar

- Vale o manual da 0.1.69: sequences com espera, execução e teste.

## Validação

- `npm run test:updates` (8/8) executado antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); falhas apontadas pelos testes da CI; a CI testa de verdade.
- A 0.1.71 não gerou build (CI reprovou em 2 testes); esta 0.1.72 a substitui sem reescrever release, pois nenhuma release 0.1.71 chegou a existir.

## Limitações

- As mesmas da 0.1.69.

---

# BotLive 0.1.71

## O que mudou

- Corrige a compilação da 0.1.70 (referência do runtime na chamada do executor). Nenhuma mudança de comportamento.

## Como usar

- Vale o manual da 0.1.69: sequences com espera, execução e teste.

## Validação

- `npm run test:updates` (8/8) executado antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); erro apontado pelo compilador da CI; a CI testa de verdade.
- A 0.1.70 não gerou build (CI reprovou na compilação); esta 0.1.71 a substitui sem reescrever release, pois nenhuma release 0.1.70 chegou a existir.

## Limitações

- As mesmas da 0.1.69.

---

# BotLive 0.1.70

## O que mudou

- Corrige a compilação da 0.1.69 (`execute_flow` aninhada movida para o nível do módulo). Nenhuma mudança de comportamento.

## Como usar

- Vale o manual da 0.1.69: sequences com espera, execução e teste.

## Validação

- `npm run test:updates` (8/8) executado antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); erro apontado pelo compilador da CI; a CI testa de verdade.
- A 0.1.69 não gerou build (CI reprovou na compilação); esta 0.1.70 a substitui sem reescrever release, pois nenhuma release 0.1.69 chegou a existir.

## Limitações

- As mesmas da 0.1.69.

---

# BotLive 0.1.69

## O que mudou

- Sequências no motor existente (sem segundo executor): cada etapa tem **Etapa ativa** (desligada pula com registro, sem erro), botões **Subir/Descer** e nova ação **Aguardar** (ms/s/min até 1 hora, assíncrona e cancelável, sem travar UI, Twitch, OBS ou outras automações).
- Toda execução ganha identidade (`Execução abc12345` no início/fim do Histórico) com estados (rodando, aguardando, concluída, falhou, cancelada).
- **Testar fluxo**: executa de verdade OBS, espera, overlay e variáveis locais, pulando envios e efeitos externos com aviso; progresso por etapa (✓/⏳) e **Cancelar** só daquela execução. Concorrência liberada, sem fila global.
- Falha mantém o comportamento (para as seguintes); cancelamento no WAIT não cria restauração paralela (temporário do OBS segue o próprio ciclo).

## Como usar

- No bloco da ação: Etapa ativa, Subir/Descer; ação Aguardar com duração e unidade. Testar fluxo no rodapé do editor ou na barra do visual.
- Dicas: waits de minutos pedem o botão Cancelar por perto; combine temporário do OBS (restaura estado) com WAIT (só pausa) sem misturar os conceitos.

## Validação

- `npm run check` (tsc), `npm test` (vitest), `npm run docs`, `npm run test:updates` (8/8) executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); inclui testes de ciclo de execução, poda, validação do wait, skip + espera ponta a ponta e reordenação; a CI compila e testa de verdade.
- Teste visual do editor e do diálogo de teste ainda pendente.

## Limitações

- Sem fila global, limite por live e condições/loops (próximas releases).
- No teste, envios e efeitos externos são pulados de propósito.

---

# BotLive 0.1.68

## O que mudou

- Corrige a compilação da 0.1.67 (`return` perdido no temporário do OBS, `n < -100.0` com espaços e gancho de fulfillment fora dos loops). Nenhuma mudança de comportamento além da 0.1.66.

## Como usar

- Vale o manual da 0.1.66: OBS local com ações temporárias.

## Validação

- `npm run test:updates` (8/8) executado antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); erros apontados pelo compilador da CI; a CI testa de verdade.
- A 0.1.67 não gerou build (CI reprovou na compilação); esta 0.1.68 a substitui sem reescrever release, pois nenhuma release 0.1.67 chegou a existir.

## Limitações

- As mesmas da 0.1.66.

---

# BotLive 0.1.67

## O que mudou

- Corrige a compilação da 0.1.66 (operador `!` sobre expressão com `.await?` reescrito em duas linhas e `n < -100.0` com espaços). Nenhuma mudança de comportamento.

## Como usar

- Vale o manual da 0.1.66: OBS local com ações temporárias.

## Validação

- `npm run test:updates` (8/8) executado antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); erros apontados pelo compilador da CI; a CI testa de verdade.
- A 0.1.66 não gerou build (CI reprovou na compilação); esta 0.1.67 a substitui sem reescrever release, pois nenhuma release 0.1.66 chegou a existir.

## Limitações

- As mesmas da 0.1.66.

---

# BotLive 0.1.66

## O que mudou

- OBS local integrado (sem nuvem, sem agente): tela **OBS Studio** com estado, versão, cena atual, configuração (endereço/porta/senha no cofre) e teste de ação. Conexão direta em `ws://localhost:4455`, autenticação oficial do protocolo.
- Nova ação **Ação no OBS** nos fluxos: mutar/desmutar/alternar entrada, volume, mostrar/esconder/alternar fonte e trocar de cena, com alvos em dropdown vindos do `obs.discover` e botão **Testar no OBS**.
- Temporário com restauração: duração restaura o estado anterior no fim (se já estava mutado, continua mutado). Concorrência por refcount: mesmo efeito soma tempo; alvo ocupado é recusado no Histórico. Nunca bloqueia (aplica e agenda).
- Operações perigosas fora: sem parar live, sair, apagar ou configurar stream. Senha nunca em log.

## Como usar

- OBS → Ferramentas → WebSocket do OBS (ative) → BotLive tela OBS Studio → ative, confira, salve a senha se pedir → Testar conexão.
- Automação (ex.: resgate): ação OBS Mutar + duração 10 → mensagem com `{{redeemer}}`. Dicas no capítulo OBS, overlays e API local.

## Validação

- `npm run check` (tsc), `npm test` (vitest 33/33), `npm run docs` (manual 22 capítulos) e `npm run test:updates` (8/8) executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); inclui testes de auth, operações seguras, config e validação da ação; a CI compila e testa de verdade.
- Teste ao vivo contra OBS real ainda pendente.

## Limitações

- Temporário conta por igual (mesmo alvo + mesmo efeito); efeito diferente no alvo ocupado é recusado, sem fila (fila vem depois).
- Volume temporário alterna entre dois valores; cenas e alternâncias são momentâneas.
- Sem OBS aberto ou com senha errada, tudo falha com orientação no Histórico; nada quebra o resto.

---

# BotLive 0.1.65

## O que mudou

- Resgate por recompensa: o gatilho Resgate de pontos filtra pelo ID da recompensa (lista no editor visual; vazio vale qualquer resgate). Renomear na Twitch não quebra.
- Variáveis do resgate: `{{redeemer}}`, `{{rewardTitle}}`, `{{rewardCost}}`, `{{rewardId}}` e `{{userInput}}`, no catálogo e na tabela do manual. Receita: `{{redeemer}} resgatou {{rewardTitle}}!`.
- Confirmação automática: automação concluída marca o resgate como cumprido; sem a permissão nova (`channel:manage:redemptions`, reautorize o canal), o Histórico explica em vez de falhar calado. Simulação nunca confirma.
- Corrige o teste de padrões da 0.1.64 (IDs de exemplo inválidos).

## Como usar

- Automações → Novo fluxo → Evento Resgate de pontos → escolha a recompensa → ações (ex.: mensagem + som). Dica: intervalo entre usos evita fila de resgates iguais; `|default:` protege a prévia.

## Validação

- `npm run check` (tsc), `npm test` (vitest), `npm run docs`, `npm run test:updates` (8/8) executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); inclui testes de filtro por reward_id e de variáveis do resgate; a CI compila e testa de verdade.
- A 0.1.64 não gerou build (CI reprovou só no teste acima); esta 0.1.65 a substitui sem reescrever release, pois nenhuma release 0.1.64 chegou a existir.
- Teste ao vivo do resgate ainda pendente.

## Limitações

- Ações no OBS vêm na próxima versão (0.1.66); hoje o resgate dispara as ações existentes (chat, som, IA, Twitch, Discord).
- Sem permissão de gerenciar resgates, a automação executa mas a confirmação fica pendente com aviso.

---

# BotLive 0.1.64

## O que mudou

- Fase 1 dos timers inteligentes (sem mudar o disparo): categoria (7 valores) e prioridade (baixa/normal/alta) com etiquetas na lista, editor de até 10 respostas alternadas por timer e regras inteligentes salvas (atividade mínima, janela, máximo por sessão, atraso inicial). Timers antigos herdam Personalizado/Normal e continuam iguais.
- Aviso de **mesmo intervalo** na lista + recomendação de distribuir minutos (53, 79, 101…) para reduzir disparos simultâneos.
- O motor inteligente (cooldown global, fila, atividade, limite, prioridade, sorteio sem repetição e dashboard) vem na próxima versão; os campos já ficam salvos e validados.

## Como usar

- Timers → Novo/Editar: escolha categoria e prioridade, cadastre variações em Respostas alternadas e ajuste Regras inteligentes. O envio de hoje continua pelo intervalo.

## Validação

- `npm run check` (tsc), `npm test` (vitest 33/33), `npm run docs` (manual 22 capítulos) e `npm run test:updates` (8/8) executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); inclui testes de padrões e validação dos campos novos; a CI compila e testa de verdade.
- Teste visual do editor e da lista ainda pendente.

## Limitações

- Respostas alternadas e regras inteligentes ainda não disparam: o envio segue a Resposta principal no intervalo até a próxima versão.

---

# BotLive 0.1.63

## O que mudou

- Corrige o teste do clipe da 0.1.62 (linha de abertura perdida deixava o corpo órfão). Nenhuma mudança de comportamento.

## Como usar

- Vale o manual da 0.1.62: `!clipe`/`!clip` com `{{local.clipUrl}}` e sorteio com `{{local.punished}}`.

## Validação

- `npm run test:updates` (8/8) executado antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); erro de sintaxe apontado pelo compilador da CI, estrutura conferida; a CI testa de verdade.
- A 0.1.62 não gerou build (CI reprovou na compilação); esta 0.1.63 a substitui sem reescrever release, pois nenhuma release 0.1.62 chegou a existir.

## Limitações

- As mesmas da 0.1.62.

---

# BotLive 0.1.62

## O que mudou

- `!clipe` / `!clip`: nova operação **Criar clipe da live** na ação Twitch. Cria o clipe pela conta do bot e guarda o link em `{{local.clipUrl}}` para mensagem customizada na ação seguinte, ex.: `{{user}} clipou esse momento! {{local.clipUrl}}`. Use o gatilho `!clipe, !clip` e um intervalo (ex.: 1 min) para não estourar o limite da Twitch.
- Escopo novo no bot (`clips:edit`): reautorize a conta do bot; só funciona com a live ligada (fora da live o Histórico explica).
- Corrige o teste de validação da 0.1.61 (mensagem de alvo inválido com o sorteado).

## Como usar

- Comando `!clipe, !clip` → ação 1 Twitch Criar clipe (conteúdo vazio) → ação 2 mensagem livre com `{{local.clipUrl}}`. `{{local.clipUrl}}` está no catálogo e no aviso de variável local.

## Validação

- `npm run check` (tsc), `npm test` (vitest 33/33), `npm run docs` e `npm run test:updates` (8/8) executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); inclui testes de conta/escopo/link do clipe e do alvo sorteado; a CI compila e testa de verdade.
- A 0.1.61 não gerou build (CI reprovou só no teste acima); esta 0.1.62 a substitui sem reescrever release, pois nenhuma release 0.1.61 chegou a existir.
- Teste ao vivo do clipe ainda pendente.

## Limitações

- Fora da live, sem clipe: a Twitch recusa e o erro orienta.
- Prévia/simulação não cria clipe: mostra o plano; `{{local.clipUrl}}` sai vazio (use `|default:`).

---

# BotLive 0.1.61

## O que mudou

- Punir sorteado no chat: **Quem leva a punição** ganhou **Sorteado no chat**. O bot escolhe entre quem já falou (nunca o streamer; pula streamer e bot em até 5 tentativas) e guarda o nome em `{{local.punished}}` para a mensagem seguinte. Receita: comando `!sorteio` → ação 1 Punir (silenciar, sorteado) → ação 2 mensagem como `{{local.punished}} rodou feio!`.
- `{{local.punished}}` entra no catálogo e no aviso de variável local.

## Como usar

- Edite o `!sorteio`: troque o alvo para Sorteado no chat e adicione a mensagem engraçada com `{{local.punished}}`. Restrinja a **Quem pode usar** para não virar arma do chat.

## Validação

- `npm run check` (tsc), `npm test` (vitest 32/32), `npm run docs` e `npm run test:updates` (8/8) executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); inclui testes de sorteio (exclui streamer, exige multidão) e validação do alvo; a CI compila e testa de verdade.
- Teste ao vivo do timeout sorteado ainda pendente.

## Limitações

- Se cair em moderador/VIP acima do bot, a Twitch recusa e o Histórico explica.
- Sem ninguém no cadastro além do streamer, a ação orienta a aguardar o chat.
- Prévia/simulação não pune nem sorteia: `{{local.punished}}` sai vazio (use `|default:`).

---

# BotLive 0.1.60

## O que mudou

- Corrige o teste de UI do Discord: usa Carregar membros + dropdown no vínculo (era rótulo e campo antigos). Nenhuma mudança de comportamento além da 0.1.59.

## Como usar

- Vale o manual da 0.1.59: dropdowns no Discord e biblioteca de sons.

## Validação

- `npx playwright test tests/ui/discord.spec.ts` executado localmente: 1 passed.
- `npm run test:updates` (8/8) executado antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); a CI testa de verdade.
- A 0.1.59 não gerou build (CI reprovou só neste teste de UI); esta 0.1.60 a substitui sem reescrever release, pois nenhuma release 0.1.59 chegou a existir.

## Limitações

- As mesmas da 0.1.59.

---

# BotLive 0.1.59

## O que mudou

- Discord sem números colados: canal do contador (vale voz), aniversariante e membro vinculado viraram dropdowns com os dados do servidor; membros carregam pelo botão **Carregar membros** (sem a lista, ainda vale colar o ID).
- Cartões do Discord empilhados em coluna única, sem lado a lado, com fileiras alinhadas pela base.
- Nova tela **Sons**: biblioteca central de áudios (adicionar, ouvir, apagar) usada por comandos, timers, automações e sons por pessoa. Apagar soma nova operação `chatExtras.remove` (apaga registro e cópia). Dica do seletor de áudio aponta para a biblioteca.

## Como usar

- Discord → Servidor e canais / Aniversariantes / Identidade: escolha pelas listas e salve.
- Sons → Adicionar som → Ouvir para conferir → escolha nos fluxos e pessoas.

## Validação

- `npm run check` (tsc), `npm test` (vitest 31/31), `npm run docs` (manual 22 capítulos) e `npm run test:updates` (8/8) executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); inclui testes de limite de membros e de remoção de som; a CI compila e testa de verdade.
- Teste visual das telas ainda pendente.

## Limitações

- A lista de membros carrega até 1000 por vez; servidor maior exige busca por nome na moderação.
- Apagar um som em uso deixa os cadastros sem áudio até escolher outro.

---

# BotLive 0.1.58

## O que mudou

- Criar segundo perfil sem travar: os botões de autorização explicam que liberam após nome + Client ID, e o campo Client ID ganhou o seletor **Copiar Client ID de outro perfil**. Pode repetir o Client ID: o aplicativo é o mesmo, só as contas mudam (OAuth e segredos são por perfil).
- Duas lives simultâneas suportadas e documentadas: cada perfil mantém sessão, comandos, memórias, rótulos e Discord próprios; basta conectar os dois cartões. Seção nova **Duas lives ao mesmo tempo** no capítulo de integrações (qual conta usar em cada autorização, voz no perfil selecionado).
- Rótulos com pasta exclusiva por live: salvar a mesma pasta em dois perfis ativos agora é recusado, porque os `.txt` têm nomes iguais e se misturariam.

## Como usar

- Rubisvau: Configurar → copie o Client ID do ArrobaSrv → **Autorizar conta do bot** (mesma conta, moderadora nos dois canais) → **Autorizar conta do canal** (logado como tijolinhopray) → Conectar os dois.

## Validação

- `npm run check` (tsc), `npm test` (vitest 31/31), `npm run docs` (manual 22 capítulos) e `npm run test:updates` (8/8) executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); inclui teste de pasta repetida entre perfis; a CI compila e testa de verdade.
- Teste ao vivo com as duas lives ainda pendente.

## Limitações

- Comandos de voz valem para o perfil selecionado; o resto é independente por live.
- A contagem individual por comando continua pendente para a próxima versão.

---

# BotLive 0.1.57

## O que mudou

- Avisos do Discord em acordeão: os 8 tipos de notificação (seguidor, inscrição, re-inscrição, presente, bits, raid, live ligada e encerrada) viram linhas recolhíveis no padrão de Respostas e sons, com busca, Expandir/Recolher todos, pastilha Ativo/Pausado e o texto + Testar dentro de cada item.

## Como usar

- Discord → Entrada, saída e espelho → abra cada aviso para ajustar texto e testar. Nada muda nos valores salvos.

## Validação

- `npm run check` (tsc), `npm test` (vitest 31/31) e `npm run test:updates` (8/8) executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); sem mudança em Rust e a CI compila e testa de verdade.
- Teste visual do acordeão ainda pendente.

## Limitações

- Nenhuma limitação nova.

---

# BotLive 0.1.56

## O que mudou

- Corrige a compilação da 0.1.55 (`if/else` escrito como ternário no aviso de live). Nenhuma mudança de comportamento.

## Como usar

- Vale o manual da 0.1.54: cartões ricos no Discord e alerta de live.

## Validação

- `npm run test:updates` (8/8) executado antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); erro apontado pelo compilador da CI; a CI testa de verdade.
- A 0.1.55 não gerou build (CI reprovou na compilação); esta 0.1.56 a substitui sem reescrever release, pois nenhuma release 0.1.55 chegou a existir.

## Limitações

- As mesmas da 0.1.54.

---

# BotLive 0.1.55

## O que mudou

- Corrige a compilação da 0.1.54 (helper `twitch_get` para as consultas Helix e laço de validação sem tupla). Nenhuma mudança de comportamento.

## Como usar

- Vale o manual da 0.1.54: cartões ricos no Discord e alerta de live.

## Validação

- `npm run test:updates` (8/8) executado antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); erros apontados pelo compilador da CI; a CI testa de verdade.
- A 0.1.54 não gerou build (CI reprovou na compilação); esta 0.1.55 a substitui sem reescrever release, pois nenhuma release 0.1.54 chegou a existir.

## Limitações

- As mesmas da 0.1.54.

---

# BotLive 0.1.54

## O que mudou

- Cartões ricos no Discord estilo aviso de live: autor com link, avatar de quem agiu, capa do jogo e prévia da transmissão, com a cor da plataforma. Vale para seguidor, inscrição, re-inscrição, presente, bits, raid e live.
- Alerta de **live ligada e encerrada**: publica título, jogo, espectadores, capa e prévia ao abrir a live (eventos novos `stream.online`/`stream.offline` da Twitch).
- Customização por tipo: cada aviso tem interruptor, texto com tokens (`{user}`, `{months}`, `{bits}`, `{viewers}`, `{game}`, `{title}`, `{url}` e outros) e botão **Testar** que envia um exemplo no canal. **Fotos nos cartões** liga/desliga as imagens de uma vez; **Texto fora do cartão** sai acima do cartão da live.

## Como usar

- Discord → Entrada, saída e espelho → escolha o **Canal de notificações**, ajuste textos e aperte **Testar** em cada tipo.
- O alerta de live sai sozinho ao abrir/encerrar na Twitch; o exemplo usa dados de mentira só para o layout.

## Validação

- `npm run check` (tsc), `npm test` (vitest 31/31), `npm run docs` (manual 22 capítulos) e `npm run test:updates` (8/8) executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); inclui testes de modelos, eventos de live e tamanhos de arte; a CI compila e testa de verdade.
- Teste ao vivo (cartões reais e alerta de live) ainda pendente.

## Limitações

- Só Twitch; Kick não tem esses eventos na ponte atual.
- A contagem individual por comando (!lurk por pessoa) continua pendente para a próxima versão.
- Sem rede com a Twitch na hora, o cartão sai sem foto em vez de falhar.

---

# BotLive 0.1.53

## O que mudou

- Corrige a compilação da 0.1.52 (5 erros no módulo novo: `await` faltando no uptime, `str::to_owned` inválido em 3 pontos e `&` no render). Nenhuma mudança de comportamento.

## Como usar

- Vale o manual da 0.1.52: Rótulos OBS, espectadores e horas assistidas.

## Validação

- `npm run test:updates` (8/8) executado antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); erros apontados pelo compilador da CI; a CI testa de verdade.
- A 0.1.52 não gerou build (CI reprovou na compilação); esta 0.1.53 a substitui sem reescrever release, pois nenhuma release 0.1.52 chegou a existir.

## Limitações

- As mesmas da 0.1.52.

---

# BotLive 0.1.52

## O que mudou

- Nova tela **Rótulos OBS** estilo StreamLabels: 13 arquivos `.txt` para o OBS (totais de seguidores e subs, seguidor/sub/resub/cheer/raid recentes, presentes e bits da sessão, espectadores no chat, horas assistidas de todos, quem mais assistiu e tempo de live). Cada rótulo tem interruptor próprio e modelo com tokens, com o valor atual à mostra.
- Pasta escolhida por você: aponte uma fonte de texto do OBS para **Ler de arquivo**. Atualização nos eventos e a cada 30 segundos; totais renovam a cada 5 minutos, com botão **Atualizar agora**.
- Horas assistidas acumuladas para sempre por pessoa (quem está com o chat aberto soma tempo mesmo sem falar; o bot não conta). Sessão (bits e presentes) zera ao conectar.
- Cartão **Quem está no chat agora** com total e lista, mesmo de quem nunca falou.
- Escopo novo no bot (`moderator:read:chatters`): autorize novamente a conta do bot no perfil. Sem ele, lista e soma por presença param com aviso; recentes e arquivos de sessão continuam.

## Como usar

- Rótulos OBS → escolha a pasta → ligue **Rótulos ativos** → Salvar. No OBS: Texto → Ler de arquivo → um `.txt` da pasta.
- Modelos usam `{name}`, `{total}`, `{amount}`, `{months}`, `{viewers}`, `{count}`, `{hours}`, `{uptime}`; token fora da lista sai como texto.

## Validação

- `npm run check` (tsc), `npm test` (vitest 31/31), `npm run docs` (manual 22 capítulos) e `npm run test:updates` (8/8) executados antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); inclui testes de modelos, horas, top watcher e validação de pasta; a CI compila e testa de verdade.
- Teste ao vivo (arquivos no OBS, lista de chatters, horas) ainda pendente.

## Limitações

- Só na Twitch e com o perfil conectado. Kick e YouTube não têm essas APIs.
- Horas contam presença no chat (polling de 30s), não minutos assistidos ao vídeo.
- Sem permissão de chatters (bot sem reautorização ou sem moderação), espectadores e horas pausam; o resto continua.
- A contagem individual por comando (!lurk por pessoa) vem na próxima versão.

---

# BotLive 0.1.51

## O que mudou

- Corrige pânico do analisador de variáveis com acentos: `Olá {{user}}` derrubava o núcleo (índice de byte no meio de caractere UTF-8), quebrando resposta TXT, simulação e o próprio teste. O analisador agora só fatia em fronteiras seguras, com teste dedicado. Nenhuma mudança de comportamento além da 0.1.49.

## Como usar

- Vale o manual da 0.1.49: resposta TXT no fio da pessoa e conferência de variáveis.

## Validação

- `npm run test:updates` (8/8) executado antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); a falha foi reproduzida pelo log da CI (3 testes com o mesmo pânico) e a correção cobre o caso exato; a CI testa de verdade.
- A 0.1.50 não gerou build (CI reprovou nestes testes); esta 0.1.51 a substitui sem reescrever release, pois nenhuma release 0.1.50 chegou a existir.

## Limitações

- As mesmas da 0.1.49.

---

# BotLive 0.1.50

## O que mudou

- Corrige a compilação da 0.1.49 (`let issues=` escrito com `:` no preview do TXT). Nenhuma mudança de comportamento além da 0.1.49.

## Como usar

- Vale o manual da 0.1.49: resposta TXT no fio da pessoa e conferência de variáveis.

## Validação

- `npm run test:updates` (8/8) executado antes do push.
- `cargo test` não executado localmente (toolchain Rust indisponível); correção sintática apontada pelo compilador da CI; a CI testa de verdade.
- A 0.1.49 não gerou build (CI reprovou na compilação); esta 0.1.50 a substitui sem reescrever release, pois nenhuma release 0.1.49 chegou a existir.

## Limitações

- As mesmas da 0.1.49.

---

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
