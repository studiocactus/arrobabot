# Chat no overlay

A tela **Chat no OBS** transforma o chat da live em uma coluna dentro da sua fonte de navegador: mensagens com badges de moderador/VIP/sub, emotes da Twitch, destaque por papel, listras de fundo e sincronização com a moderação — quando alguém apaga uma mensagem no chat, ela some do overlay na hora.

A coluna de chat fica numa **camada de overlay própria**, com um endereço próprio que carrega o parâmetro `mode=chat`. Crie uma **Fonte de navegador só para o chat** e posicione a coluna onde quiser, sem mexer na meta nem nos alertas. (Sem o parâmetro, o endereço do overlay mostra as três camadas juntas, para quem já usa uma fonte única.)

## Ligar o chat

1. Abra **Chat no OBS** com o perfil do canal.
2. Ative **Chat no overlay** no cartão do topo.
3. Use os botões de teste (Comum, Moderador, VIP, Sub, Primeira vez, Em emote, Só emotes, Do bot) para ver o visual atual empilhando na prévia.
4. Com o Estúdio aberto, as mensagens reais da live também entram na prévia — é a forma de ajustar sem ir ao ar.
5. Se ainda não tem a fonte no OBS, copie o endereço no cartão **No OBS** e siga os passos (Fontes → + → Navegador, 1920×1080).

Enquanto o chat está desligado, nada aparece na fonte; os botões de teste continuam funcionando — e eles seguem os mesmos filtros de cima, então esconder bots ou comandos também esconde os exemplos correspondentes.

## O que cada mensagem mostra

A estrutura é `[badges] [nome colorido] : [texto com emotes]`, na ordem que você escolher (**Novas embaixo** empurra a coluna para cima; **Novas no topo** entra pelo topo). Cada badge é desenhada pelo próprio motor — transmissor, moderador, VIP, inscrito, fundador, Prime, bits e presente — com interruptor individual no grupo **Badges**; badges desconhecidas entram como **Outros**.

Os emotes vêm direto do CDN da Twitch pelo id que a mensagem traz; sem internet, o emote vira o próprio texto. Uma mensagem só de emotes aparece em tamanho dobrado. A Twitch não expõe a cor do nome pelo caminho que o bot usa, então cada espectador ganha uma **cor estável por id** (da paleta da Twitch) — ou use **Cor fixa no nome** para travar uma cor só.

**Destaques por papel** pinta barra lateral, fundo tintado ou os dois, na ordem transmissor → moderador → VIP → inscrito → primeira mensagem, cada um com sua cor. “Primeira vez” é desde que o BotLive abriu, não a primeira da vida no canal.

## Filtros

| Filtro | O que faz |
|---|---|
| Mostrar as mensagens do bot | As respostas do próprio BotLive entram na coluna com o nome configurado |
| Esconder outros bots | Some nomes da **Lista de bots** e nomes terminados em “bot” |
| Esconder comandos (!) | Mensagens que começam com `!` não aparecem |
| Palavras bloqueadas | Mensagem com alguma das palavras (separadas por vírgula) não aparece |

## Ajustes da coluna

O grupo **Coluna** controla fluxo, máximo de mensagens em tela (10 a 300), some depois de (0 a 120 s; 0 = nunca), animação de entrada (fade, deslize, pisca ou datilografia), largura (240 a 960 px), lado na tela e listras alternadas. **Mensagem** escolhe a fonte, o tamanho do texto e do nome, a cor do texto e o nome em caixa alta. As alterações salvam sozinhas e chegam ao OBS na hora, sem recarregar a fonte.

## Limitações desta primeira versão

- Só emotes da Twitch: BTTV e 7TV não aparecem.
- A cor real do nome (escolhida nas configurações da Twitch) não é informada pela API que o bot usa; a cor por espectador é estável, mas pode não ser a mesma do chat oficial.
- “Primeira mensagem” vale desde que o BotLive abriu, não a primeira da vida no canal.
- A sincronização de deleções depende da Twitch entregar os eventos de moderação ao bot; se uma assinatura falhar na conexão, o erro aparece no Histórico e as mensagens apagadas ficam até sair por tempo ou limite.
- No YouTube as mensagens entram sem badges e sem emotes; o Kick não entrega eventos de chat ao bot nesta versão.
- Eventos inline no chat (follow/sub/raid como mensagens da coluna) não existem: use os [Alertas da live](11-ALERTAS.md), que cobrem os mesmos eventos com painel próprio.
- A coluna ocupa o lado da cena que você escolher; se cobrir a meta ou os alertas, ajuste o alinhamento aqui ou as margens do overlay.
