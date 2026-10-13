# Alertas da live no overlay

A tela **Alertas** transforma follows, subs, presentes, bits e raids em painéis animados dentro da sua fonte de navegador do OBS. O painel é uma "janela de sistema" com barra de status, tarja do evento, nome em destaque, linha grande, linha final datilografada e uma barra de tempo que corre durante o alerta.

Os alertas ficam numa **camada de overlay própria**, com um endereço próprio que carrega o parâmetro `mode=alerts`. Isso significa que você cria uma **Fonte de navegador só para os alertas** e ajusta posição e tamanho sem mexer na meta nem no chat. O endereço é estável — não muda ao reabrir o BotLive e não precisa de arquivo local. (Sem o parâmetro, o endereço do overlay mostra as três camadas juntas, para quem já usa uma fonte única.)

## Ligar os alertas

1. Abra **Alertas** com o perfil do canal.
2. Ative **Alertas no overlay** no cartão do topo.
3. Escolha o evento nas pílulas (Seguidor, Novo sub, Renovação, Sub de presente, Presentes em lote, Bits, Raid) e edite os textos.
4. Clique em **Testar** para ver a animação na prévia — e no OBS, se a fonte estiver aberta.
5. Se ainda não tem a fonte no OBS, copie o endereço no cartão **No OBS** e siga os passos (Fontes → + → Navegador, 1920×1080).

Enquanto os alertas estão desligados, nada toca na live; os botões de teste continuam funcionando, e o teste é a única exceção que o OBS recebe mesmo com tudo desligado.

## Cada evento tem quatro textos e um som

| Evento | Quando toca | Variáveis úteis |
|---|---|---|
| Seguidor | Novo follow | `{{user}}` |
| Novo sub | Primeira inscrição | `{{user}}`, `{{tier}}` |
| Renovação | Renovação de sub | `{{user}}`, `{{months}}`, `{{message}}` |
| Sub de presente | Um sub presenteado (total 1) | `{{user}}` |
| Presentes em lote | Vários subs de uma vez (total > 1) | `{{user}}`, `{{amount}}` |
| Bits | Cheer com bits | `{{user}}`, `{{amount}}`, `{{message}}` |
| Raid | Raid recebido | `{{user}}`, `{{amount}}` |

Os quatro textos de cada evento são **Etiqueta** (a tarja colorida), **Linha do nome**, **Linha grande** e **Linha final**. Em qualquer um deles você pode usar `{{user}}`, `{{amount}}`, `{{months}}`, `{{tier}}`, `{{sender}}` e `{{message}}`; apelidos como `{{name}}`, `{{username}}` e `{{viewer}}` também valem `{{user}}`. Quando a informação não existe (uma mensagem vazia, por exemplo), a variável vira espaço em branco e a linha continua limpa. Cada evento tem interruptor próprio: desligue só o que não quiser sem mexer nos demais.

## Ajustes do painel

O grupo **Painel** controla duração (3 a 20 s), velocidade das animações (50% a 200%), volume do som, posição na tela (topo/meio/base e esquerda/centro/direita), margens, largura do painel e o texto da marca. **Cores** tem quatro seletores (destaque, texto, fundo e linhas) e **Fontes** escolhe a fonte pequena (barra e tarja) e a grande (nomes e linhas), com tamanhos independentes. **Efeitos** liga e desliga scanline, cantos da moldura, partículas e glitch no impacto.

As alterações salvam sozinhas e chegam ao OBS na hora, sem recarregar a fonte.

## Sons

O seletor **Som** de cada evento lista os arquivos enviados na tela [Sons de Live](RESPOSTAS-E-SONS.md) (WAV, MP3 ou OGG, até 5 MiB). O áudio toca na **saída dos sons do bot** — o aparelho escolhido na tela [Áudio](13-AUDIO.md) — junto com o painel, usando o volume ajustado no grupo Painel. Sem som selecionado, o alerta é silencioso. Capture essa saída no OBS (por exemplo, a fonte de áudio apontando para o CABLE Output) para ouvir os alertas na live.

## Limitações desta primeira versão

- Não existe editor de HTML/CSS/JS livre: o Estúdio trabalha por campos, com um único visual de painel. Um modo código pode vir em uma versão futura.
- Não há botão de pular alerta: cada alerta roda a duração configurada antes do próximo entrar na fila.
- A Twitch não informa quem recebeu um sub de presente; em **Sub de presente** e **Presentes em lote**, `{{user}}` é quem deu o presente.
- Doações (LivePix) e resgates de pontos não geram alerta nesta versão.
- As fontes do Google exigem internet no computador que roda o OBS; fontes do sistema funcionam offline.
