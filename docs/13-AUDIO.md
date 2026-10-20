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

- **Um som por vez** (fila serial): novos sons entram na fila (até **5 pendentes**); um som tem **15 segundos para começar** a tocar e teto de **60 segundos** quando já iniciado (falas longas de texto para voz cabem sem serem cortadas no meio).
- **Parar todos os sons** limpa a fila na hora e para o que está tocando — na fonte e no aplicativo.
- **Eventos antigos são descartados**: um som com mais de 10 segundos (por exemplo, enquanto o OBS reinicia) não toca atrasado; reconectar não repete sons antigos.
- **O aplicativo nunca toca junto com a fonte**: quando a fonte está conectada, os sons vão só para ela; o app só toca quando o OBS não está ouvindo.
- **Overlays visuais são mudos**: Alertas, Chat no OBS e Meta desenham na camada deles; o som sempre sai pela fonte de áudio.
- **Trocar de perfil isola os sons**: cada perfil tem sua própria fila e sua própria fonte — eventos de um perfil nunca tocam em outro.

## Texto para voz (TTS)

O TTS é gerado no próprio computador pelo **Windows (SAPI)** — sem nuvem e sem custo — e sai pelo **mesmo caminho** de qualquer som: chega ao OBS pela fonte Áudio do BotLive, com as mesmas regras de fila e volume. Voz e velocidade ficam em **Comunidade → Texto para voz**; **Testar voz** ouve pelo mesmo caminho da live. As falas geradas ficam em cache na pasta de dados (grupo `media`) e são reusadas quando o texto não muda; o cache é limitado e limpa os arquivos mais antigos sozinho.

## Minha voz: a tela Voicemod

Sua voz é outro assunto — ela **nunca** passa pela saída dos sons do bot nem pelo Voicemod junto com eles. A tela **Voicemod** configura tudo em três etapas, uma vez:

> O BotLive recebe o resgate ou comando e pede ao Voicemod para trocar sua voz. O OBS recebe sua voz pelo microfone virtual do Voicemod.

**Etapa 1 · Conectar Voicemod.** Abra o aplicativo Voicemod nesta máquina e clique em **Conectar**. Quando precisar, cole a chave da **Control API** (peça em control-api.voicemod.net, menu Control API): ela vai direto para o cofre, nunca para o chat, os logs ou o Histórico. Conectado, a etapa recolhe sozinha; **Alterar conexão e diagnóstico** fica em área secundária.

**Etapa 2 · Ouvir minha voz no OBS.** Na cena, edite a fonte de áudio da sua voz e escolha **Voicemod Microphone** como entrada — e evite capturar o mesmo microfone em duas fontes, senão a voz entra em dobro. Nada aqui muda seus dispositivos do Windows nem mexe nas suas cenas. Há **um único teste**: escolha a voz, aplique temporariamente e ouça — **fale durante o teste**, é a sua voz que o OBS precisa ouvir; ao terminar, a voz de antes volta sozinha, confirmada pelo próprio Voicemod. Dois estados ficam separados: **Voicemod conectado** (a API respondeu e autorizou) e **Áudio conferido no OBS** (só você confirma, ouvindo a gravação — uma chamada de API com sucesso nunca prova áudio na cena).

**Etapa 3 · Escolher o que troca minha voz.** Cada regra liga um acionamento a uma voz por um tempo — exemplo: **“Voz de robô → Robô → 30 segundos”**. O acionamento é **recompensa da Twitch** (lista real do canal; o botão de conectar a conta do canal fica no próprio contexto) ou **comando do chat** (uma palavra, tipo `!robô`, com **permissão explícita** — por padrão, só moderadores — e intervalo entre usos). A regra mostra só o essencial: acionamento, recompensa ou comando, voz real do Voicemod, duração e ativado/desativado. O resto (permissão, intervalo, política de fila) fica em **Avançado**, recolhido. **Testar esta voz** exercita o caminho da regra sem gastar pontos, sem mexer em resgates reais e sem publicar mensagens.

**Uma troca por vez, na sequência.** Resgates e automações (etapa **Mudar minha voz**) entram na mesma fila: o efeito no ar cumpre o seu tempo inteiro e o próximo começa em seguida — ninguém é cancelado no meio. A exceção é a duração **0** (“até a próxima troca”): ela é, pela própria definição, substituída pelo que vier depois. Quando um pedido entra na fila, o bot pode avisar quem resgatou no chat com uma mensagem sua — **Mensagem quando o resgate entra na fila**, em **Avançado → Fila e intervalos** — usando `{{user}}`, `{{pos}}` e `{{voz}}`; o campo vazio desliga o aviso. O tamanho da fila continua limitando quantos esperam: passou disso, a troca nova é recusada com aviso (o estorno dos pontos é na própria Twitch).

### Durante a live

A tela mostra primeiro o que importa: **estado da conexão**, **regras ativas ou pausadas**, **o efeito atual com o tempo que resta**, **o que vem depois (só quando há fila)** e o botão visível **Encerrar efeitos e restaurar minha voz**. A política padrão é simples: **um efeito por vez**, com os próximos esperando numa fila limitada; os ajustes de fila e intervalo ficam em **Avançado**.

A restauração devolve **o estado anterior confirmado**, incluindo o modificador de voz quando aplicável. Se você tinha um efeito escolhido antes, é esse efeito que volta — o BotLive não promete “voz normal” quando está devolvendo uma voz com efeito. Mudanças manuais feitas no Voicemod durante um efeito são respeitadas (nada é sobrescrito) e temporizadores antigos nunca restauram por cima de um efeito mais recente. Se a conexão cair, a tela diz com todas as letras e oferece **Recuperar restauração**; nada de efeito antigo é reaplicado ao reconectar.

### Comandos por voz (área própria)

O reconhecimento da sua fala mora em **Comandos → Comandos por voz** (e no módulo Controle por voz da tela Comunidade): escuta contínua, gravação de comando e criação rápida. Quando ligado, ele captura o **microfone físico** — antes de qualquer efeito. **Nada disso é necessário** para as trocas de voz do Voicemod: sem STT, sem servidor de transcrição, sem palavra de ativação e sem moderação para usar resgates e comandos de voz.

### Efeitos internos antigos (quem ainda usa)

O motor interno (microfone → VB-Cable → OBS) continua funcionando como estava para quem o utiliza: nada é apagado, desinstalado ou migrado sem você pedir. Quem já está no caminho do Voicemod não tem etapa de migração: as regras de voz e a etapa **Mudar minha voz** usam vozes do Voicemod diretamente, e mapeamentos salvos em versões anteriores continuam valendo. Efeito antigo pedido sem o motor interno e sem equivalência é recusado com aviso dizendo o próximo passo.

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
