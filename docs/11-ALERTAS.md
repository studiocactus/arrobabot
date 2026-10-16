# Alertas da live no overlay

A tela **Alertas** transforma follows, subs, presentes, bits e raids em painéis animados dentro da sua fonte de navegador do OBS. O painel é uma "janela de sistema" com barra de status, tarja do evento, nome em destaque, linha grande, linha final datilografada e uma barra de tempo que corre durante o alerta.

Os alertas ficam numa **camada de overlay própria**, com um endereço próprio que carrega o parâmetro `mode=alerts`. Isso significa que você cria uma **Fonte de navegador só para os alertas** e ajusta posição e tamanho sem mexer na meta nem no chat. O endereço é estável — não muda ao reabrir o BotLive e não precisa de arquivo local. (Sem o parâmetro, o endereço do overlay mostra as três camadas juntas, para quem já usa uma fonte única.)

## Ligar os alertas

1. Abra **Alertas** com o perfil do canal.
2. Ative **Alertas no overlay** no cartão do topo.
3. Escolha o evento nas pílulas (Seguidor, Novo sub, Renovação, Sub de presente, Presentes em lote, Bits, Raid) e edite os textos.
4. Clique em **Testar** para ver a animação na prévia — e no OBS, se a fonte estiver aberta.
5. Se ainda não tem a fonte no OBS, copie o endereço no cartão **No OBS** e siga os passos (Fontes → + → Navegador, 1920×1080).

Enquanto os alertas estão desligados, nada toca na live — nem nos testes: clicar em **Testar** mostra o aviso do que a live faria com o interruptor desligado. Com a chave e o interruptor do evento ligados, o teste corre pelo mesmo caminho da live: o OBS toca o alerta junto com a prévia, a caixa de evento no chat, a explosão de emotes e os rótulos de exibição acompanham (cada um com o seu interruptor) e o quadro acende o selo **Ensaio**. O teste não vira estatística, mensagem da Twitch nem aviso no Discord.

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

Os quatro textos de cada evento são **Etiqueta** (a tarja colorida), **Linha do nome**, **Linha grande** e **Linha final**. Em qualquer um deles você pode usar `{{user}}`, `{{amount}}`, `{{months}}`, `{{tier}}`, `{{sender}}` e `{{message}}`; apelidos como `{{name}}`, `{{username}}` e `{{viewer}}` também valem `{{user}}`. Quando a informação não existe (uma mensagem vazia, por exemplo), a variável vira espaço em branco e a linha continua limpa. O interruptor de cada evento fica no cabeçalho do grupo **Este evento**: desligue só o que não quiser sem mexer nos demais.

O seletor **Cor da tarja** pinta **somente a tarja colorida** daquele evento — o resto do painel continua nas cores globais do grupo **Cores**. Sem cor escolhida, a tarja usa o Destaque do painel, e o botão **Usar cor global** devolve esse padrão. O texto dentro da tarja escolhe preto ou branco sozinho conforme a clareza da cor, para não sumir em fundo escuro.

## Ajustes do painel

O Estúdio segue o padrão das telas de prévia do BotLive: a **prévia fica à esquerda e acompanha a rolagem** (fixa na tela enquanto você ajusta), e as configurações, **em acordeão à direita**, ocupam só a altura do grupo aberto. **Este evento** já nasce aberto; clique nos títulos **Painel**, **Cores**, **Fontes** e **Efeitos** para abrir cada um (um resumo do que está configurado aparece ao lado do título). Em janela estreita, a prévia vai para cima e as configurações ficam embaixo.

O grupo **Painel** controla duração (3 a 20 s), velocidade das animações (50% a 200%), volume do som, posição na tela (topo/meio/base e esquerda/centro/direita), margens, largura do painel e o texto da marca. **Cores** tem quatro seletores (destaque, texto, fundo e linhas) e **Fontes** escolhe a fonte pequena (barra e tarja) e a grande (nomes e linhas), com tamanhos independentes. **Efeitos** liga e desliga scanline, cantos da moldura, partículas e glitch no impacto.

As alterações salvam sozinhas e chegam ao OBS na hora, sem recarregar a fonte.

## Modo código

Acima do acordeão, as abas **Fields · HTML · CSS · JS · Data** abrem o código da própria camada:

- **Fields** é o formulário desta tela (o acordeão de sempre).
- **HTML** é markup extra no topo do painel — moldura, fundo, ornamento; `scripts` não executam aqui.
- **CSS** são estilos próprios valendo **só dentro da camada**: no Estúdio eles não pintam a interface do aplicativo e no OBS valem a cena.
- **JS** roda na prévia e no OBS com `root` = a própria camada; erro de sintaxe vai ao console sem derrubar o resto.
- **Data** mostra a configuração desta tela em JSON; **Aplicar** normaliza como o formulário faria e devolve à aba Fields, e objeto fora do formato é recusado com o motivo.

**Restaurar padrão** limpa o bloco da aba — HTML, CSS e JS começam vazios, e o visual continua sendo o do Estúdio até você escrever código. O que você escrever salva sozinho, como no formulário.

## Sons

O seletor **Som** de cada evento lista os arquivos enviados na tela [Sons de Live](RESPOSTAS-E-SONS.md) (WAV, MP3 ou OGG, até 5 MiB). O áudio toca na **saída dos sons do bot** — o aparelho escolhido na tela [Áudio](13-AUDIO.md) — junto com o painel, usando o volume ajustado no grupo Painel. Sem som selecionado, o alerta é silencioso. Capture essa saída no OBS (por exemplo, a fonte de áudio apontando para o CABLE Output) para ouvir os alertas na live.

## Explosão de emotes

O cartão **Explosão de emotes** solta os seus emotes da Twitch no overlay, como um confete, quando acontece um dos quatro eventos escolhidos: **novo sub**, **renovação de sub**, **presente de sub** e **meta completada** (o interruptor da meta fica na tela Meta). Cada evento tem o seu, e a ligação geral desliga tudo de uma vez.

| Campo | O que faz |
|---|---|
| Ligar explosão de emotes | Interruptor mestre: sem ele, nada explode |
| Novo sub / Renovação de sub / Presente de sub | Escolha, evento a evento, se a explosão acontece |
| Emotes usados | Aleatório entre os seus · Os mais usados no chat · Todos, na ordem da Twitch |
| Quantidade | De 10 a 80 emotes por explosão |
| Tamanho | De 24 a 120 px o lado de cada emote |
| Testar explosão | Dispara na hora, pelo mesmo caminho do evento real |

A lista vem dos **emotes do seu canal com os globais da Twitch**, buscada quando o canal conecta e atualizada a cada 12 horas. A origem **Os mais usados no chat** conta cada emote que aparece nas mensagens desde a primeira vez que a explosão é ligada; os totais ficam guardados e continuam de onde pararam no dia seguinte. **Testar explosão** avisa em vez de falhar em silêncio quando ainda não há emotes carregados — conecte o canal ao Twitch e tente de novo.

A explosão respeita a **camada** da fonte do OBS: a camada de alertas recebe as de sub, renovação e presente; a camada de meta recebe a da meta completada; e a tela [Chat no OBS](12-CHAT-OBS.md) tem a opção **Também nesta camada** para a explosão cair também sobre a coluna de chat. Os ajustes de quantidade e tamanho valem para as três camadas.

## Limitações desta primeira versão

- No modo código, `scripts` dentro do HTML não executam (o que é código entra na aba JS), e imagens ou fontes externas do seu CSS podem ser bloqueadas na prévia do aplicativo pela política de segurança — na fonte do OBS elas carregam. O JS roda sem sandbox além do escopo: escreva só código seu.
- Não há botão de pular alerta: cada alerta roda a duração configurada antes do próximo entrar na fila.
- A Twitch não informa quem recebeu um sub de presente; em **Sub de presente** e **Presentes em lote**, `{{user}}` é quem deu o presente.
- Doações (LivePix) e resgates de pontos não geram alerta nesta versão.
- A explosão de emotes usa os emotes da Twitch (canal + globais); emotes de terceiros (7TV, BTTV, FFZ) ficam para uma próxima versão, e a contagem de mais usados começa quando a explosão é ligada, sem histórico retroativo.
- **Testar explosão** dispara mesmo com a explosão desligada — é para conferir o efeito antes da live; o interruptor mestre e os por evento valem para os eventos de verdade.
- As fontes do Google exigem internet no computador que roda o OBS; fontes do sistema funcionam offline.
