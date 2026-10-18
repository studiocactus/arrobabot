# Roadmap

O que já está no BotLive e o que está planejado. Este documento é a referência central de direção; o histórico de entregas continua em [Atualizações e commits](ATUALIZACOES.md).

## Entregue

- **0.7 — Áudio em linha única e Overlays do OBS (0.7.0)** — dez ajustes de feedback dos usuários. A tela **Áudio** passa a ter uma **saída única** para todos os sons do bot (espectador, comandos, timers, automações, resgates e alertas): um aparelho só, uma Fonte de áudio no OBS para o bot inteiro, sem interruptores por origem e sem caminho pelo navegador; a voz da live e o TTS seguem com seus caminhos próprios. **Respostas TXT** viram tela própria, cada regra com um **som opcional** da biblioteca; os **sons por espectador** saem de Respostas e vão para **Sons de Live** (biblioteca + pessoas + reiniciar sessão). **Alertas, Chat no OBS e Meta (LivePix)** ficam no grupo **Overlays do OBS**, cada um com **endereço de camada próprio** via parâmetro `mode` na URL, para Fontes de navegador separadas. A **presença de chat** visível no topo mostra quantas pessoas estão no chat e a lista, a cada 30s. **LivePix**: meta persiste ao desligar/religar, dá para **editar a meta**, **reiniciar contador**, aceita valor com **vírgula**, pede **confirmação ao excluir** e mostra erros de integração visíveis. **Rótulos OBS** e as telas de Alertas/Chat ganharam **rótulos em acordeão, toggles rotulados e dicas** de uso. Ver [Saída de áudio](13-AUDIO.md), [Respostas TXT e sons](RESPOSTAS-E-SONS.md) e [Camadas do overlay](API-LOCAL.md).

- **0.6 — Central de áudio (0.6.0)** — a tela **Áudio** reúne todos os sons do bot num lugar só. A versão 0.6 permitia escolher, por origem, entre **linha de áudio** e **navegador**; a partir da 0.7.0 todos os sons saem por uma **saída única** (ver a entrada 0.7 acima). Os aparelhos da Voz da live (microfone e saída) ficam na mesma tela e o TTS aparece com sua limitação declarada (saída padrão do Windows). Ver [Saída de áudio](13-AUDIO.md).

- **0.5 — Chat no overlay (0.5.0)** — a tela **Chat no OBS** transforma o chat da live em uma coluna na mesma fonte de navegador do overlay: badges desenhadas por papel (transmissor, moderador, VIP, inscrito, fundador, Prime, bits, presente), emotes da Twitch em tamanho dobrado quando a mensagem é só de emote, destaque por papel com barra ou fundo, primeira mensagem marcada, listras, quatro animações de entrada, teto de mensagens e some-depois. Filtros de comandos, bots e palavras bloqueadas; mensagens do próprio bot entram como resposta; mensagens apagadas pela moderação somem do overlay na hora (eventos `message_delete`/`clear` da Twitch). Prévia em tamanho real no Estúdio, com os mesmos motores do OBS e testes que seguem os filtros.

- **0.4 — Estúdio de Alertas (0.4.0)** — follows, subs, presentes, bits e raids viram painéis animados ("janela de sistema") na mesma fonte do overlay da meta, com configuração por evento (etiqueta, nome, linha grande, linha final, som), fontes, cores, efeitos e testes que entram pelo mesmo caminho do evento real. Ver [Alertas da live](11-ALERTAS.md).

- **Comandos naturais — onda 1** — streamer e moderadores escrevem por extenso (com a palavra de ativação do Controle por voz) para mudar título e categoria; o parser é determinístico e falha fechado. Silenciar, banir, VIP e moderadores ficam para a onda 2.

- **UX 2.0 (em andamento)** — as primeiras etapas já estão na versão atual:
  - Resumo legível de cada etapa direto no bloco do canvas, com a cadeia empilhada de cima para baixo.
  - **Adicionar etapa** cria a etapa no fim da cadeia já ligada à última, selecionada e com a configuração aberta, em espaço livre e sem redefinir o zoom.
  - **Inserir etapa depois** (menu **•••**) encaixa uma etapa no meio da sequência (A → nova → B), preservando os dados das vizinhas e o eixo do desenho.
  - Menu **Opções da etapa** com **Subir/Descer** por conexões, limites por posição na cadeia e devolução de foco ao fechar.
  - Comportamento de fluxos inválidos (desconectado, cíclico ou com ramificação): o editor explica e não reorganiza nada em silêncio.
  - Minimapa compacto e ajuda do editor em popover, com preferência preservada.
  - Painel **Testar fluxo** reformado: abre nos dois editores sem salvar nem executar, mostra o que o teste faz e os valores de exemplo com o nome legível de cada variável, começa uma única execução por clique (o duplo não duplica), acompanha cada etapa com o estado traduzido — parada por condição não é tratada como falha — e cancela ou fecha durante a execução só depois da sua escolha, sem efeito do lado de fora.

> A etapa de UX 2.0 segue em andamento: este documento não declara o programa concluído. Próximas entregas serão listadas aqui conforme forem aprovadas.

- **Voicemod — etapa 1: conexão local e teste manual (0.1.93)** — a tela **Voicemod** usava a [Control API oficial](https://control-api.voicemod.net/) por WebSocket local, sem nuvem e sem varrer a rede, com chave no cofre do sistema e teste manual com restauração. **REMOVIDO em 0.1.95**: a integração saiu do produto (tela, conexão e operações); a voz da live passou a ser feita pelo **Voz da live**, dentro do próprio BotLive, sem depender do Voicemod.

## Planejado

### Integração com o Voicemod — removida em 0.1.95, reintroduzida em 0.9.0

A integração foi retirada do produto em 0.1.95 (a etapa 1 de 0.1.93 saiu do código e nada falava mais com a Control API) e **voltou na 0.9.0** como o caminho opcional **Minha voz** da tela **Voz da live**: o BotLive fala com a Control API local do Voicemod para trocar a sua voz (lista real de vozes, resgates de recompensa com fila e restauração garantida, teste com volta à voz de antes), sem reprocessar nem retransmitir áudio — os sons do bot nunca passam pelo Voicemod. O caminho interno continua existindo: a tela **Voz da live** aplica os efeitos no microfone dentro do próprio BotLive e envia o resultado para o VB-Cable, que o OBS usa como microfone. A troca de voz por resgate da Twitch e a ação **Voz da live** passaram a existir nessa tela. Ver [Áudio no OBS](13-AUDIO.md).
