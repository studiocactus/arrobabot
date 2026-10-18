# Áudio no OBS

A tela **Áudio no OBS** reúne todos os sons do bot numa saída só: uma **Fonte de navegador** local e dedicada no OBS, chamada **Áudio do BotLive**. Não há aparelho do Windows nem VB-Cable no caminho dos sons — o OBS recebe o áudio por uma página servida pelo próprio BotLive (nesta máquina, porta local, perfil autorizado), e o **fader dessa fonte no mixer vira o volume do bot inteiro**, preservando os volumes individuais de cada som.

Os sons cobertos são: **espectador, comandos, timers, automações, resgates, alertas, os sons da Meta** (doação e celebração, escolhidos no Estúdio do overlay em [Integrações](INTEGRACOES.md)) e o **texto para voz (TTS)**.

## Como usar (uma vez)

1. Abra a tela **Áudio no OBS** e copie o **Endereço da fonte de áudio**.
2. No OBS: **Fontes → + → Navegador**, cole o endereço e nomeie **Áudio do BotLive**. A página não tem visual nenhum — só toca som —, então qualquer tamanho serve.
3. Na fonte, mantenha marcada **Controlar áudio pelo OBS**: é o que permite tocar sem interação e sincroniza a parada com o mute.
4. Volte ao BotLive e clique em **Testar áudio**: o bipe é gerado pelo próprio aplicativo (não depende de arquivo) e chega à fonte pelo mesmo caminho de qualquer som.

O endereço é estável e não muda ao reabrir o BotLive. Se você criar uma **segunda** Fonte de navegador com o mesmo endereço, ela fica em espera e a tela avisa: a primeira fonte que conectou reproduz; quando ela sai, a de espera assume automaticamente.

## Estado e diagnóstico

A tela mostra o estado real da saída, sem fingir sucesso:

- **Aguardando o OBS**: nenhuma fonte conectada — confira o endereço colado, o aplicativo aberto e a opção **Controlar áudio pelo OBS**.
- **Conectada**: a fonte está ouvindo (a tela mostra quantas; com duas, avisa que **só a primeira reproduz**).
- **Reproduzindo**: a fonte confirmou que começou a tocar.
- **Erro**: a fonte recusou, o formato não é suportado ou a resposta não chegou.

**Testar áudio** e **Parar todos os sons** são os controles da tela. O teste distingue **som enviado** de **reprodução iniciada** e de **falha**: a confirmação vem da própria fonte do OBS (ou do aplicativo, quando ele toca), e tempo esgotado também aparece como tal — nunca sucesso falso.

## Regras de reprodução

- **Um som por vez** (fila serial): novos sons entram na fila (até **5 pendentes**) e cada som tem **15 segundos** para terminar antes de ser descartado.
- **Parar todos os sons** limpa a fila na hora e para o que está tocando — na fonte e no aplicativo.
- **Eventos antigos são descartados**: um som com mais de 10 segundos (por exemplo, enquanto o OBS reinicia) não toca atrasado; reconectar não repete sons antigos.
- **O aplicativo nunca toca junto com a fonte**: quando a fonte está conectada, os sons vão só para ela; o app só toca quando o OBS não está ouvindo.
- **Overlays visuais são mudos**: Alertas, Chat no OBS e Meta desenham na camada deles; o som sempre sai pela fonte de áudio.
- **Trocar de perfil isola os sons**: cada perfil tem sua própria fila e sua própria fonte — eventos de um perfil nunca tocam em outro.

## Texto para voz (TTS)

O TTS é gerado no próprio computador pelo **Windows (SAPI)** — sem nuvem e sem custo — e sai pelo **mesmo caminho** de qualquer som: chega ao OBS pela fonte Áudio do BotLive, com as mesmas regras de fila e volume. Voz e velocidade ficam em **Comunidade → Texto para voz**; **Testar voz** ouve pelo mesmo caminho da live. As falas geradas ficam em cache na pasta de dados (grupo `media`) e são reusadas quando o texto não muda; o cache é limitado e limpa os arquivos mais antigos sozinho.

## Voz da live (caminho separado)

A tela **Voz da live** é o hub da sua voz: **Minha voz na live**, **Comandos do Arroba** e **Resgates de voz** num lugar só. Sua voz **nunca** passa pela saída dos sons do bot nem é reprocessada por ela.

### Minha voz na live

A voz com efeitos tem caminho próprio — microfone → processamento → linha —, separada dos sons e do TTS. No cartão **Minha voz na live** você escolhe o **Caminho da voz**:

- **Efeitos internos do BotLive** (padrão, como sempre foi): o BotLive captura o microfone e envia a voz para o VB-Cable — o que o microfone captura, o OBS recebe. Nada muda até você clicar em **Iniciar voz ao vivo**; ao parar, o microfone volta ao normal.
- **Voicemod** (cartão **Minha voz**): o BotLive não captura nem retransmite áudio — ele só pede ao aplicativo Voicemod (nesta máquina) para trocar a voz. O caminho do áudio vira microfone → Voicemod → microfone virtual do Voicemod → OBS. Os sons do bot **não passam pelo Voicemod** — eles seguem pela fonte Áudio do BotLive. Para usar, em quatro passos:
  1. **Abra e configure o Voicemod** nesta máquina, com sua voz e efeitos por lá.
  2. **Conecte o controle**: peça a chave da **Control API** em control-api.voicemod.net, cole abaixo e clique em **Conectar**. A chave fica no cofre, nunca no chat nem no Histórico, e a tela só mostra “Conectado” depois da autorização aceita pelo próprio Voicemod.
  3. **Aponte o OBS** para o microfone virtual: na cena, use **Voicemod Microphone** como microfone. O microfone padrão do Windows **não muda**.
  4. **Teste a voz** na tela: o efeito vale só durante o teste e a voz de antes volta sozinha ao terminar.

  Cada falha tem o seu motivo (aplicativo fechado, chave ausente, chave recusada, versão incompatível, conexão perdida) com o que fazer em seguida — por exemplo, conferir se o Voicemod está aberto e logado e se a licença libera a Control API.

Só um caminho fica de pé por vez: o BotLive barra a troca com a voz interna no ar e recusa o início da voz interna enquanto o caminho estiver em Voicemod — nunca dois capturando o microfone ao mesmo tempo. O cartão **Diagnóstico** mostra o estado real de ponta a ponta: microfone e nível, saída para o OBS, conexão com o Voicemod e se a escuta de comandos está ativa, com avisos para captura dupla, aparelho desconectado e circuito fechado. Nos dois caminhos, a escuta de comandos continua no seu **microfone físico** (antes de qualquer efeito).

### Comandos do Arroba

A mesma tela concentra a escuta contínua (RealtimeSTT local, microfone físico escolhido de propósito — antes de qualquer efeito), a prévia do reconhecimento, o botão **Gravar comando** (“o que eu ouvi”, até 8 segundos) e a criação rápida de comandos por voz. O duração do “quicar” (timeout falado) também fica aqui. O áudio vai só para o RealtimeSTT na própria máquina. No chat, streamer e moderadores escrevem por extenso — “arroba, mude a categoria para Minecraft” — e o bot obedece com as mesmas palavras, sem a escuta ligada.

### Resgates de voz

No caminho Voicemod, cada recompensa do canal pode trocar sua voz por um tempo combinado: escolha a recompensa (lista real da Twitch), a voz (lista real do Voicemod), duração, cooldown e o que acontece se outro efeito já estiver no ar (fila, ignorar ou substituir). A fila aparece na hora — efeito atual com o tempo que resta e o que vem depois — com os botões **Cancelar efeitos e voltar à voz normal** e **Recuperar restauração pendente**. A seção **Efeitos internos antigos** liga efeitos antigos (fina, robô…) a vozes do Voicemod quando você migrar de caminho — o mapeamento é sempre escolhido por você, nada é adivinhado.

## Migração (de versões com VB-Cable)

Antes, cada som saía por um aparelho de saída do Windows (muitas vezes o VB-Cable) e o OBS capturava esse aparelho com uma fonte **Áudio da área de trabalho**. Agora existe uma saída só, a fonte de navegador. Para migrar:

1. Crie a fonte **Áudio do BotLive** como acima e clique em **Testar áudio**.
2. Confira o som no fader da fonte nova e ajuste o volume dela no mixer do OBS.
3. **Remova do OBS a fonte antiga** que capturava o cabo (**Áudio da área de trabalho** apontando para **CABLE Output**): sem isso os sons entram em dois lugares.
4. Pronto: a escolha de aparelho antiga da tela Áudio **perdeu o efeito**, mas nada foi apagado — arquivos, vínculos, volumes, comandos, resgates, alertas e as configurações continuam exatamente como você deixou (os dados antigos ficam preservados, sem uso).

O **VB-Cable pode continuar instalado** se você usa a voz com efeitos internos; para os sons do bot ele não é mais necessário. Nada foi desinstalado e nenhuma cena do OBS foi alterada por aqui — a migração é toda feita por você, no seu ritmo.

## Limitações

- Os sons dependem do aplicativo BotLive aberto: é a fonte do OBS que toca, mas quem entrega os arquivos e a fila é o app. Com ele fechado, a fonte fica muda.
- O TTS usa as vozes do Windows (SAPI instaladas no sistema); não há vozes de nuvem.
- Um som interrompido no meio (mute da fonte, troca de cena com a fonte desligada, **Parar todos os sons**) não retoma de onde parou; ele é encerrado e a fila segue.
- Sons de outras origens (áudio do Discord, efeitos do próprio OBS, música tocada em outro programa) não passam por esta saída; cada fonte continua com seu áudio.
- A homologação com OBS, Voicemod e microfone reais não foi executada nesta entrega (ver notas da versão): a validação automatizada cobre o caminho completo — incluindo a geração **real** de TTS pelo Windows nesta máquina —, mas não substitui um teste com OBS aberto na sua live.
