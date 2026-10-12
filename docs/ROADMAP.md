# Roadmap

O que já está no BotLive e o que está planejado. Este documento é a referência central de direção; o histórico de entregas continua em [Atualizações e commits](ATUALIZACOES.md).

## Entregue

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

### Integração com o Voicemod — REMOVIDA

A integração com o Voicemod foi retirada do produto: a etapa 1 entregue em 0.1.93 saiu do código em 0.1.95, e nada conversa mais com a Control API. O objetivo de trocar a voz da live foi alcançado por outro caminho, sem depender de outro programa aberto: a tela **Voz da live** aplica os efeitos no microfone dentro do próprio BotLive e envia o resultado para o VB-Cable, que o OBS usa como microfone. A troca de voz por resgate da Twitch e a etapa de automação (ação **Voz da live**) passaram a existir nessa tela.
