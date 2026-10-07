# Roadmap

O que já está no BotLive e o que está planejado. Este documento é a referência central de direção; o histórico de entregas continua em [Atualizações e commits](ATUALIZACOES.md).

## Entregue

- **UX 2.0 (em andamento)** — as primeiras etapas já estão na versão atual:
  - Resumo legível de cada etapa direto no bloco do canvas, com a cadeia empilhada de cima para baixo.
  - **Adicionar etapa** cria a etapa no fim da cadeia já ligada à última, selecionada e com a configuração aberta, em espaço livre e sem redefinir o zoom.
  - **Inserir etapa depois** (menu **•••**) encaixa uma etapa no meio da sequência (A → nova → B), preservando os dados das vizinhas e o eixo do desenho.
  - Menu **Opções da etapa** com **Subir/Descer** por conexões, limites por posição na cadeia e devolução de foco ao fechar.
  - Comportamento de fluxos inválidos (desconectado, cíclico ou com ramificação): o editor explica e não reorganiza nada em silêncio.
  - Minimapa compacto e ajuda do editor em popover, com preferência preservada.
  - Painel **Testar fluxo** reformado: abre nos dois editores sem salvar nem executar, mostra o que o teste faz e os valores de exemplo com o nome legível de cada variável, começa uma única execução por clique (o duplo não duplica), acompanha cada etapa com o estado traduzido — parada por condição não é tratada como falha — e cancela ou fecha durante a execução só depois da sua escolha, sem efeito do lado de fora.

> A etapa de UX 2.0 segue em andamento: este documento não declara o programa concluído. Próximas entregas serão listadas aqui conforme forem aprovadas.

- **Voicemod — etapa 1: conexão local e teste manual (0.1.93)** — a tela **Voicemod** usa a [Control API oficial](https://control-api.voicemod.net/) por WebSocket local, sem nuvem e sem varrer a rede. A chave fica no cofre do sistema; **Conectado** só aparece após o `registerClient` respondido com 200. A tela lista as vozes com o **id** real de cada uma, mostra a voz atual, o modificador de voz, ouvir minha voz e a licença, busca por nome e permite **Testar voz** por um tempo configurável (padrão 10 s) com contagem no núcleo, encerramento antecipado e devolução do estado anterior. Queda da conexão durante o teste deixa a restauração **não confirmada** com **Restaurar agora** explícito; reconexão automática nunca reenvia a voz; troca manual nunca é desfeita em silêncio. Erros vão para a tela e para o **Histórico**, nunca para o chat. A conexão real **não foi homologada** nesta entrega (sem Voicemod instalado/autorizado na máquina de validação): os testes automatizados rodam contra um servidor WebSocket controlado.

## Planejado

### Integração com o Voicemod (etapa 1 ENTREGUE · etapas seguintes PENDENTES)

O objetivo final continua sendo trocar a voz do bot pelo microfone virtual do Voicemod durante falas em TTS, usando a [Voicemod Control API](https://control-api.voicemod.net/). A etapa 1 (conexão, estado, lista de vozes e teste manual com restauração) já está na versão atual e está descrita em **Entregue** acima e no capítulo [Perfis e conexões](INTEGRACOES.md).

**Validado nesta etapa:** a referência oficial, o endereço `ws://127.0.0.1:<porta>/v1`, o formato de `registerClient` + `clientKey`, a correlação de respostas por `id`/`action` e os eventos espontâneos usados (`voiceChangedEvent`, `toggleVoiceChanger`). **Não validado:** a conexão com um Voicemod real instalado e autorizado.

O que continua pendente:

1. **Recompensa ligada por id** — o painel de recompensa escolhe a voz pelo id obtido na lista; nada de escrever id à mão.
2. **Duração por recompensa** — definir quanto tempo a voz vale, recompensa por recompensa.
3. **Fila de resgates simultâneos** — decidir o que acontece quando duas recompensas são resgatadas ao mesmo tempo (uma espera, a última vence, etc.).
4. **Conclusão e cancelamento do resgate** — encerrar a troca de voz quando o tempo acabar ou o usuário cancelar o resgate na Twitch, respeitando as permissões de reembolso da conta do canal.
5. **A integração como etapa de automação** — usar a troca de voz dentro do editor de fluxos, com o mesmo aviso de efeito real no microfone.
6. **Microfone virtual no OBS sem passo manual** — hoje o roteamento é feito à mão no OBS (descrito na documentação); automatizar segue pendente.

Cuidados importantes desta integração:

- **Provar antes de prometer** — a etapa 1 está entregue como código e testes contra servidor controlado; não declare Voicemod concluído nem homologado enquanto a conexão real não for verificada com o aplicativo aberto e autorizado.
- **Não confundir com troca de TTS** — isto é sobre o microfone virtual usado como fonte de áudio, não sobre trocar o motor de voz do bot.
- **Licença do Voicemod** — o que está disponível pode depender da licença do usuário; vozes bloqueadas pela licença aparecem como indisponíveis em vez de falhar em silêncio.
