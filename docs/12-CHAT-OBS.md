# Chat no overlay

A tela **Chat no OBS** transforma o chat da live em uma coluna dentro da sua fonte de navegador: prefixo antes do nome, mensagens com badges de moderador/VIP/sub, cor de cada espectador, emotes da Twitch, destaque por papel, caixas de evento (follow, sub, bits, raid) na própria coluna, listras de fundo e sincronização com a moderação — quando alguém apaga uma mensagem no chat, ela some do overlay na hora.

A coluna de chat fica numa **camada de overlay própria**, com um endereço próprio que carrega o parâmetro `mode=chat`. Crie uma **Fonte de navegador só para o chat** e posicione a coluna onde quiser, sem mexer na meta nem nos alertas. (Sem o parâmetro, o endereço do overlay mostra as três camadas juntas, para quem já usa uma fonte única.)

## Ligar o chat

1. Abra **Chat no OBS** com o perfil do canal.
2. Ative **Chat no overlay** no cartão do topo.
3. Use os botões de teste (Comum, Moderador, VIP, Sub, Primeira vez, Em emote, Só emotes, Do bot) para ver o visual atual empilhando na prévia. Com o chat ligado, cada teste percorre o mesmo caminho da live e o palco acende o selo **Ensaio**; com ele desligado, o teste avisa que nada apareceria.
4. Com o Estúdio aberto, as mensagens reais da live também entram na prévia — é a forma de ajustar sem ir ao ar.
5. Se ainda não tem a fonte no OBS, copie o endereço no cartão **No OBS** e siga os passos (Fontes → + → Navegador, 1920×1080).

Enquanto o chat está desligado, nada aparece na fonte — nem nos testes: clicar num botão de teste mostra o aviso do que a live faria com o interruptor desligado. Ligue o chat para os testes correrem pelo mesmo caminho da live, com o selo **Ensaio** no palco. Os testes seguem os mesmos filtros de cima: esconder bots ou comandos também esconde os exemplos correspondentes.

## O que cada mensagem mostra

A estrutura é `[prefixo] [badges] [nome colorido] : [texto com emotes]`, na ordem que você escolher (**Novas embaixo** empurra a coluna para cima; **Novas no topo** entra pelo topo). Cada badge é desenhada pelo próprio motor — transmissor, moderador, VIP, inscrito, fundador, Prime, bits e presente — com interruptor individual no grupo **Badges**; badges desconhecidas entram como **Outros**.

O **prefixo** é o `>` de saudação do widget de referência: vem antes das badges, com texto livre (até 16 caracteres) e cor própria — desligue no grupo **Mensagem** se não quiser.

Os emotes vêm direto do CDN da Twitch pelo id que a mensagem traz — e os de **terceiros (BTTV, 7TV e FFZ)** também aparecem: o BotLive busca a lista do seu canal quando ele conecta e desenha cada emote pelo nome. Sem internet, o emote vira o próprio texto. O tamanho segue o padrão da Twitch, ajustável: mensagem só de emotes cresce — **um emote sozinho aparece no maior tamanho permitido (4×)**, dois ou três em dobro, o resto no normal — e **Tamanho dos emotes** amplia ou reduz tudo em porcentagem. O interruptor **Emotes explodem ao aparecer** solta os emotes voando da mensagem (como na referência da StreamElements), sem mexer no texto. Os nomes de quem fala ficam alinhados no topo de cada mensagem, mesmo quando o emote é grande. As mensagens da **conta do bot** também aparecem na coluna — o que o bot fala na live é visível para você — e não disparam automações, pontos nem contagem de presença.

A **origem da cor do nome** é uma escolha de três vias no grupo **Mensagem**:

| Opção | Comportamento |
|---|---|
| **Twitch** (padrão) | A cor real que cada pessoa escolheu na Twitch, do jeito que a live entrega; quem nunca escolheu ganha uma cor estável por conta |
| **Personalizadas** | Todo mundo na cor estável própria (tirada do id, da paleta da Twitch), ignorando a cor real |
| **Fixa** | Uma cor só para todo o chat, escolhida no seletor de cor |

**Destaques por papel** pinta barra lateral, fundo tintado ou os dois, na ordem transmissor → moderador → VIP → inscrito → primeira mensagem, cada um com sua cor. “Primeira vez” é desde que o BotLive abriu, não a primeira da vida no canal.

## Eventos na coluna

Com o mestre **Eventos no chat** ligado, follow, sub, renovação, presente, bits e raid viram uma **caixa própria dentro da coluna** — a mesma estrutura das mensagens, com cor e som por evento. Funciona junto ou separado da camada de alertas: dá para ter só as caixas aqui, ou as duas.

- Escolha o evento nas pílulas (Follow, Sub, Renovação, Sub presente, Presentes em lote, Bits, Raid) e edite **Texto da caixa** e **Cor do evento**; a cor pinta a borda e tinge o fundo da caixa, que tem base escura para o texto branco ler em qualquer cena.
- O texto aceita `{name}`, `{amount}`, `{months}`, `{tier}`, `{sender}` e `{message}`; em presente, `{name}` é quem ganhou e `{sender}` quem deu.
- **Mostrar a mensagem do espectador** entra como segunda linha nas caixas de renovação e bits com texto escrito.
- **Som do evento** escolhe um arquivo da tela [Sons de Live](RESPOSTAS-E-SONS.md) e toca pela **saída única do BotLive** (tela [Áudio no OBS](13-AUDIO.md) — a fonte de áudio dedicada), nunca pela camada visual dentro do OBS; o botão **Testar …** preenche o texto pelo motor e desenha a caixa na prévia pelo mesmo caminho da live (com o selo **Ensaio**), tocando o som escolhido.
- Cada evento tem interruptor próprio (**Exibir caixa de …**) além do mestre.

Por padrão os eventos estão **desligados**, para não duplicar a camada de alertas. Com o mestre desligado, nada muda na coluna — e o botão **Testar …** avisa que, em live, a caixa também não apareceria; o mesmo vale quando a caixa individual do evento está desligada.

## Filtros

| Filtro | O que faz |
|---|---|
| Mostrar as mensagens do bot | As respostas do próprio BotLive entram na coluna com o nome configurado |
| Esconder outros bots | Some nomes da **Lista de bots** e nomes terminados em “bot” |
| Esconder comandos (!) | Mensagens que começam com `!` não aparecem |
| Palavras bloqueadas | Mensagem com alguma das palavras (separadas por vírgula) não aparece |

## Ajustes da coluna

O grupo **Coluna** controla fluxo, máximo de mensagens em tela (10 a 300), some depois de (0 a 120 s; 0 = nunca), animação de entrada (fade, deslize, pisca ou datilografia), largura (240 a 960 px), lado na tela e listras alternadas. O grupo **Explosão de emotes** tem um único interruptor, **Também nesta camada**: ligado, a explosão de sub, presente ou meta completada configurada na tela [Alertas](11-ALERTAS.md) cai também sobre a coluna de chat; desligado, a explosão toca só nas camadas de alertas e meta. **Mensagem** escolhe a fonte, o tamanho do texto, do nome e dos emotes (com o modo **Emote grande em mensagem só de emote** e a explosão dos emotes ao aparecer), a cor do texto, o nome em caixa alta, a origem da cor do nome e o prefixo. As alterações salvam sozinhas e chegam ao OBS na hora, sem recarregar a fonte.

## Modo código

Acima do formulário, as abas **Fields · HTML · CSS · JS · Data** abrem o código da própria camada:

| Aba | O que faz |
|---|---|
| **Fields** | O formulário desta tela, como sempre foi |
| **HTML** | Markup extra no topo da coluna — moldura, fundo, ornamento. `scripts` aqui não executam; o que é código entra na aba JS |
| **CSS** | Estilos próprios, valendo **só dentro da coluna** — no Estúdio eles não pintam a interface do aplicativo; no OBS valem a cena |
| **JS** | Código que roda na prévia e no OBS com `root` = a própria coluna; erro de sintaxe vai ao console sem derrubar o resto |
| **Data** | A configuração desta tela em JSON; **Aplicar** normaliza como o formulário faria e devolve à aba Fields (objeto fora do formato é recusado com o motivo) |

**Restaurar padrão** limpa o bloco da aba (HTML, CSS e JS começam vazios) e o que você escrever continua salvando sozinhas. O CSS é prefixado com o escopo da camada — por exemplo `.bl-chat-name{color:#ff0}` pinta o nome no OBS e na prévia, e `body`/`:root` pintam a própria janela da coluna.

## Limitações desta primeira versão

- Emotes de terceiros (BTTV, 7TV e FFZ) aparecem desde que o emote exista no seu canal ou nos conjuntos globais deles — a lista é buscada quando o canal conecta; um emote de outra comunidade que não esteja nesses conjuntos continua como texto.
- A cor da Twitch só entra quando a live entrega um hex completo; quem nunca escolheu cor (ou entrega fora do padrão) ganha uma cor estável por conta — e ela pode se repetir entre espectadores.
- “Primeira mensagem” vale desde que o BotLive abriu, não a primeira da vida no canal.
- A sincronização de deleções depende da Twitch entregar os eventos de moderação ao bot; se uma assinatura falhar na conexão, o erro aparece no Histórico e as mensagens apagadas ficam até sair por tempo ou limite.
- No YouTube as mensagens entram sem badges, sem emotes e sem cor de nome; o Kick não entrega eventos de chat ao bot nesta versão.
- No modo código, `scripts` dentro do HTML não executam (o que é código entra na aba JS), e imagens ou fontes externas do seu CSS podem ser bloqueadas na prévia do aplicativo pela política de segurança — na fonte do OBS elas carregam. O JS roda sem sandbox além do escopo: escreva só código seu.
- A coluna ocupa o lado da cena que você escolher; se cobrir a meta ou os alertas, ajuste o alinhamento aqui ou as margens do overlay.
