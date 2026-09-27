# Histórico de atualizações

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
