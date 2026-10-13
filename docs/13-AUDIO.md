# Central de áudio

A tela **Áudio** reúne todos os sons do bot num lugar só e decide, para cada origem, por onde ele sai. Em vez de cada tela escolher seu caminho sozinha, você marca na Central o que vai para a **linha de áudio** — o aplicativo toca no aparelho de saída que você aponta para um cabo virtual e captura no OBS — e o que vai para o **navegador** — a fonte do overlay toca o som dentro do próprio OBS.

Os dois caminhos podem estar ligados ao mesmo tempo; desligar os dois silencia a origem.

## Linha de áudio e navegador

| Caminho | Quem toca | Onde aparece no OBS |
|---|---|---|
| Linha de áudio | O aplicativo BotLive, no aparelho de saída escolhido na Central | Na fonte de áudio que capturar esse aparelho (ex.: CABLE Output do VB-Cable) |
| Navegador | A fonte de navegador do overlay | No mixer do OBS como áudio da fonte de navegador |

A linha de áudio não altera a saída padrão do Windows e não depende do OBS: é o BotLive tocando direto no aparelho escolhido. Por isso funciona com qualquer cena, inclusive sem fonte de navegador.

## Para onde vai cada som

| Origem | Interruptores | De onde vem |
|---|---|---|
| Sons de espectador | Linha e Navegador | Sons por espectador, na tela Respostas e sons |
| Sons de fluxo | Linha e Navegador | O áudio ligado a comandos, timers e automações (Tocar áudio ao disparar) |
| Alertas da live | Linha e Navegador | O som de seguidor, sub, bits e raid configurado na tela Alertas |

Cada origem tem interruptores próprios: dá para mandar os sons das pessoas só para o navegador e o som dos alertas para os dois, por exemplo. As alterações salvam sozinhas e valem na hora.

## A linha de áudio

No cartão **Linha de áudio** escolha o aparelho de saída e use **Testar linha**: o bipe toca exatamente pelo caminho que qualquer som marcado "Linha" usaria. Com o **VB-Cable** instalado, escolha a entrada **CABLE Input** — no OBS, a fonte de áudio **CABLE Output** entrega tanto a voz da live quanto os sons da linha.

Se o aparelho escolhido for desconectado, novos sons são recusados com um aviso em vez de sumir no silêncio; escolha outra saída e a Central salva. Para alinhar tudo, escolha o mesmo aparelho (por exemplo, CABLE Input) na linha da Central e na saída da Voz da live.

## Voz da live

A voz com efeitos continua tendo caminho próprio (microfone → processamento → linha), separada dos sons e do TTS. A Central mostra o estado da voz e os dois aparelhos que a tela **Voz da live** usa — microfone de entrada e saída da linha — para você configurar tudo junto. Efeitos, corte de ruído e o botão de iniciar continuam na tela Voz da live; nada muda até você clicar em Iniciar lá.

## Texto para voz (TTS)

O leitor de voz dos fluxos sai pela **saída padrão do Windows**: o navegador não permite ao BotLive escolher outro aparelho para essa voz, então o TTS é o único som com o caminho fixo. Para colocá-la na linha, troque a saída padrão do Windows ou use o Mixer de volume para rotear o áudio do BotLive. Voz e velocidade ficam em Comunidade → Texto para voz.

## Passo a passo: alinhar tudo numa linha só

1. Instale o **VB-Cable** (um driver leve, uma vez).
2. Em **Áudio → Linha de áudio** escolha **CABLE Input** e clique em **Testar linha**.
3. Em **Áudio → Voz da live** escolha seu microfone na entrada e **CABLE Input** na saída.
4. No OBS, adicione a fonte de áudio **CABLE Output** (áudio desktop) — voz e sons da linha passam a entrar na live.
5. Marque em **Para onde vai cada som** o caminho de cada origem; os sons marcados **Navegador** saem pela fonte de navegador do overlay (o mesmo endereço da meta, alertas e chat).

## Limitações

- O TTS sai pela saída padrão do Windows (limitação do navegador); use o Mixer de volume para rotear.
- Os sons da linha dependem do aplicativo BotLive aberto — é ele que toca.
- Os sons marcados **Navegador** dependem da fonte de navegador do overlay existir no OBS (mesmo endereço do overlay da meta e dos alertas).
- O botão **Testar** da tela Alertas toca também na linha quando a linha dos alertas está ligada, igual ao teste chega ao OBS.
- Sons de chat no Discord ou de outras plataformas seguem o caminho da origem que os gerou; a Central cobre os sons do bot na live.
