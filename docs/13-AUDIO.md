# Saída de áudio

A tela **Áudio** reúne todos os sons do bot numa saída só. Em vez de cada origem escolher seu caminho, você aponta **um único aparelho de saída** — por exemplo a entrada **CABLE Input** do VB-Cable — e todos os sons do bot saem por ele. No OBS, isso vira **uma única Fonte de áudio** para o bot inteiro: você controla voz e efeitos num fader só, sem montar um mix de várias fontes.

Os sons cobertos são: **espectador, comandos, timers, automações, resgates, alertas e os sons da Meta** (doação e celebração, escolhidos no Estúdio do overlay em [Integrações](INTEGRACOES.md)).

## Saída dos sons do bot

No cartão **Saída dos sons do bot**, escolha o aparelho em **Dispositivo de saída dos sons** e clique em **Testar saída**: o bipe toca exatamente pelo aparelho que qualquer som do bot usaria. As alterações salvam sozinhas.

Com o **VB-Cable** instalado, escolha a entrada **CABLE Input**. No OBS, adicione **Fontes → + → Áudio da área de trabalho** apontando para o **CABLE Output**: todos os sons do bot passam a entrar na live por aquela fonte, e o volume dela é o fader do bot inteiro.

Se o aparelho escolhido for desconectado, os sons são recusados com um aviso em vez de sumir no silêncio; escolha outra saída e a tela salva sozinha. **Atualizar dispositivos** recarrega a lista quando você ligar ou desligar um cabo.

## Voz da live

A tela **Voz da live** é o hub da sua voz: **Minha voz na live**, **Comandos do Arroba** e **Resgates de voz** num lugar só.

### Minha voz na live

A voz com efeitos tem caminho próprio — microfone → processamento → linha —, separada dos sons e do TTS. No cartão **Minha voz na live** você escolhe o **Caminho da voz**:

- **Efeitos internos do BotLive** (padrão, como sempre foi): o BotLive captura o microfone e envia a voz para o VB-Cable — o que o microfone captura, o OBS recebe. Nada muda até você clicar em **Iniciar voz ao vivo**; ao parar, o microfone volta ao normal.
- **Voicemod**: o BotLive não captura nem retransmite áudio — ele só pede ao aplicativo Voicemod (nesta máquina) para trocar a voz. O caminho do áudio vira microfone → Voicemod → microfone virtual do Voicemod → OBS. Para usar, peça a chave da **Control API** em control-api.voicemod.net, salve-a (ela fica no cofre, nunca no chat nem no Histórico) e clique em **Conectar**: a tela só mostra “Conectado” depois da autorização aceita pelo próprio Voicemod, e cada falha tem o seu motivo (aplicativo fechado, chave ausente, chave recusada, versão incompatível, conexão perdida). **Testar uma voz** troca no Voicemod por alguns segundos e devolve a voz de antes, com confirmação.

Só um caminho fica de pé por vez: o BotLive barra a troca com a voz interna no ar e recusa o início da voz interna enquanto o caminho estiver em Voicemod — nunca dois capturando o microfone ao mesmo tempo. O cartão **Diagnóstico** mostra o estado real de ponta a ponta: microfone e nível, saída para o OBS, conexão com o Voicemod e se a escuta de comandos está ativa, com avisos para captura dupla, aparelho desconectado e circuito fechado.

A voz só sai pelo aparelho escolhido na tela Voz da live (caminho interno); a saída aqui embaixo é o caminho dos sons do bot.

### Comandos do Arroba

A mesma tela concentra a escuta contínua (RealtimeSTT local, microfone físico escolhido de propósito — antes de qualquer efeito), a prévia do reconhecimento, o botão **Gravar comando** (“o que eu ouvi”, até 8 segundos) e a criação rápida de comandos por voz. O duração do “quicar” (timeout falado) também fica aqui. O áudio vai só para o RealtimeSTT na própria máquina. No chat, streamer e moderadores escrevem por extenso — “arroba, mude a categoria para Minecraft” — e o bot obedece com as mesmas palavras, sem a escuta ligada.

### Resgates de voz

No caminho Voicemod, cada recompensa do canal pode trocar sua voz por um tempo combinado: escolha a recompensa (lista real da Twitch), a voz (lista real do Voicemod), duração, cooldown e o que acontece se outro efeito já estiver no ar (fila, ignorar ou substituir). A fila aparece na hora — efeito atual com o tempo que resta e o que vem depois — com os botões **Cancelar efeitos e voltar à voz normal** e **Recuperar restauração pendente**. A seção **Efeitos internos antigos** liga efeitos antigos (fina, robô…) a vozes do Voicemod quando você migrar de caminho — o mapeamento é sempre escolhido por você, nada é adivinhado.

## Texto para voz (TTS)

O leitor de voz dos fluxos sai pela **saída padrão do Windows**: o navegador não permite ao BotLive escolher outro aparelho para essa voz, então o TTS é o único som com caminho fixo. Para colocá-lo na linha, troque a saída padrão do Windows ou use o Mixer de volume para rotear o áudio do BotLive. Voz e velocidade ficam em Comunidade → Texto para voz. O cartão **Texto para voz** marca essa saída como **Fixa** para deixar a limitação explícita.

## Passo a passo: alinhar tudo numa linha só

1. Instale o **VB-Cable** (um driver leve, uma vez).
2. Em **Áudio → Saída dos sons do bot** escolha **CABLE Input** e clique em **Testar saída**.
3. Em **Áudio → Voz da live** escolha seu microfone na entrada e **CABLE Input** na saída da linha.
4. No OBS, adicione a fonte de áudio **CABLE Output** (áudio da área de trabalho) — voz e sons do bot passam a entrar na live pela mesma fonte.
5. Ajuste o volume dessa única fonte no mixer do OBS: ela controla o bot inteiro.

## Limitações

- O TTS sai pela saída padrão do Windows (limitação do navegador); use o Mixer de volume para rotear.
- Os sons do bot dependem do aplicativo BotLive aberto — é ele que toca no aparelho.
- A voz da live depende de você clicar em Iniciar na tela Voz da live; ela tem aparelho próprio e não é afetada pela saída dos sons.
- Sons gerados fora do BotLive (por exemplo, efeitos do próprio OBS ou áudio de outra ponta, como o Discord) não passam por esta saída; cada fonte continua com seu áudio.
