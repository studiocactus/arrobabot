# Saída de áudio

A tela **Áudio** reúne todos os sons do bot numa saída só. Em vez de cada origem escolher seu caminho, você aponta **um único aparelho de saída** — por exemplo a entrada **CABLE Input** do VB-Cable — e todos os sons do bot saem por ele. No OBS, isso vira **uma única Fonte de áudio** para o bot inteiro: você controla voz e efeitos num fader só, sem montar um mix de várias fontes.

Os sons cobertos são: **espectador, comandos, timers, automações, resgates, alertas e os sons da Meta** (doação e celebração, escolhidos no Estúdio do overlay em [Integrações](INTEGRACOES.md)).

## Saída dos sons do bot

No cartão **Saída dos sons do bot**, escolha o aparelho em **Dispositivo de saída dos sons** e clique em **Testar saída**: o bipe toca exatamente pelo aparelho que qualquer som do bot usaria. As alterações salvam sozinhas.

Com o **VB-Cable** instalado, escolha a entrada **CABLE Input**. No OBS, adicione **Fontes → + → Áudio da área de trabalho** apontando para o **CABLE Output**: todos os sons do bot passam a entrar na live por aquela fonte, e o volume dela é o fader do bot inteiro.

Se o aparelho escolhido for desconectado, os sons são recusados com um aviso em vez de sumir no silêncio; escolha outra saída e a tela salva sozinha. **Atualizar dispositivos** recarrega a lista quando você ligar ou desligar um cabo.

## Voz da live

A voz com efeitos tem caminho próprio — microfone → processamento → linha —, separada dos sons e do TTS. O cartão **Voz da live** mostra o estado da voz e os dois aparelhos que a tela **Voz da live** usa (microfone de entrada e saída da linha), para você alinhar tudo num lugar só. Escolha o microfone de verdade na entrada e o **CABLE Input** na saída da linha. Efeitos, corte de ruído e o botão de iniciar continuam na tela **Voz da live**: nada muda até você clicar em Iniciar lá.

A voz só sai pelo aparelho escolhido na tela **Voz da live**; a saída aqui embaixo é o caminho dos sons do bot.

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
