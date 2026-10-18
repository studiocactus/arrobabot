# Música do Spotify

A tela **Spotify** lê a música que está tocando na sua conta e entrega o título, o artista, o álbum, a capa e a posição para o BotLive: no **overlay do OBS** (um cartão com capa, barra de progresso e visual que você ajusta no Estúdio) e nas **automações** (variáveis `{{nowplaying.*}}`). É o caminho que ferramentas como o Snip fazem com arquivos de texto, mas dentro do BotLive: sem programa extra, sem arquivo solto e com a leitura acontecendo enquanto o app está aberto.

O BotLive **só lê**: nada é tocado, pausado ou enviado ao Spotify. Apenas os metadados da música aparecem — o áudio continua sendo do seu player, como sempre.

## Criar o seu aplicativo no Spotify (2 minutos)

O Spotify entrega a música apenas para aplicativos autorizados, e desde 2025 cada aplicativo em modo de desenvolvimento serve até 25 contas — por isso cada streamer cria o **seu próprio aplicativo gratuito**, igualzinho apps como o Snip pedem:

1. Abra [developer.spotify.com/dashboard](https://developer.spotify.com/dashboard) e entre com a sua conta do Spotify.
2. Clique em **Create app**. Preencha nome e descrição à vontade (ex.: "BotLive da minha live") e, em **Which API/SDKs are you planning to use**, marque **Web API**. Aceite os termos e crie.
3. Dentro do app, abra **Settings**. Em **Redirect URIs**, adicione exatamente o endereço que a tela do BotLive mostra:
   `http://127.0.0.1:43828/callback`
   Salve (**Save settings**).
4. Em **Settings → Client credentials**, copie o **Client ID**. O Client Secret não é usado: a autorização do BotLive é do tipo PKCE, que não guarda segredo.

## Conectar a conta no BotLive

1. Abra **Spotify** com o perfil do canal.
2. Cole o **Client ID** no campo de conexão (a tela salva sozinha).
3. Clique em **Autorizar conta**. O navegador abre a página do Spotify; entre (se ainda não estiver) e confirme **Conceder acesso**. Volte ao BotLive — a etiqueta do cartão passa a mostrar **Conectado** e o nome da conta.
4. Ligue **Ler a música que está tocando**. O BotLive consulta o Spotify de 5 em 5 segundos enquanto estiver aberto.
5. Clique em **Testar conexão** para ver a faixa atual aparecer na linha *Agora tocando* — sem esperar a próxima leitura.

Para trocar de conta: **Desconectar** e autorize de novo. Apagar o perfil também apaga a autorização do cofre do sistema.

## A música no overlay do OBS

A música fica numa **camada de overlay própria**, com endereço que carrega o parâmetro `mode=spotify`. Crie uma **Fonte de navegador só para a música** (Largura 1920, Altura 1080, sem atualizar ao ativar a cena) e o cartão aparece no canto que você escolher — sem mexer na meta, nos alertas nem no chat. O endereço sai no cartão **No OBS** da tela, com um clique para copiar.

O **Estúdio da música** (à direita da tela) desenha na prévia **exatamente o mesmo cartão** que o OBS recebe, dentro do quadro de 1920×1080 em miniatura — o que você vê ali é o que a live mostra, e as alterações chegam ao OBS na hora, sem recarregar a fonte.

**Zoom da prévia (só da prévia, de 10% a 100%):** role a **roda do mouse** sobre o cartão (o ponto sob o cursor fica parado, a composição se move em volta dele), use os botões **−** e **+**, arraste a barra do indicador de porcentagem (com a barra em foco, as **setas do teclado** também mexem), ou **Ajustar à área** para a escala acompanhar a janela e **100%** para ver no tamanho real. Com a composição ampliada, **arraste** para deslocar o quadro; as barras internas rolam normalmente sem rolar a página. O zoom é guardado na sua máquina (fora da configuração do cartão) e **nunca muda o que o OBS recebe** — a fonte continua recebendo os 1920×1080 inteiros.

**Modelo do cartão** — um clique de partida, sem travar a edição: **Só a música** (título), **Uma linha** (música e artista), **Compacto** (capa pequena + as duas linhas) e **Completo** (capa, álbum, tempos e barra). Qualquer mexida depois vira **Personalizado** sem perder o que você já ajustou; cor e posição nunca contam como mudança de modelo, e os interruptores de cada campo continuam livres em qualquer modelo.

Os ajustes ficam em cinco seções: **Conteúdo** (quais campos aparecem: título, artista, linha do álbum, capa, barra, tempos, duração e ícone do Spotify, mais a ordem das linhas e o título em duas linhas), **Layout** (pele do cartão, posição na tela, alinhamento do texto, largura, preenchimento, espaçamento e cantos), **Aparência** (formato e tamanho da capa, temas, amostras de destaque, corretor de cor com amostra circular e valor hexadecimal, fontes, tamanhos, borda, sombra e transparência), **Comportamento** (como entra, como sai, o que mostrar quando não há música e a espera antes de sumir ao pausar) e **Ajustes avançados** (capa desfocada ao fundo, duração da animação e força da sombra — abre e recolhe quando você quiser).

O botão **Testar no overlay** dispara uma amostra pelo mesmo caminho da música real: o OBS desenha o cartão com o selo **Ensaio** e a amostra volta ao estado real sozinha — é assim que você confere a posição e o visual antes da live. Quando nada está tocando, o que aparece é uma **amostra de demonstração** (nunca a sua conta conectada). Com a música desligada ou nada tocando, o cartão some (ou mostra a mensagem, se você escolher).

## Usar a música nas automações

Seis variáveis alimentam qualquer resposta, automação ou prévia de IA. Vazias quando nada toca — uma mensagem nunca quebra por o streamer estar sem música:

| Variável | Conteúdo |
|---|---|
| `{{nowplaying.musica}}` | Título da música |
| `{{nowplaying.artista}}` | Artista (com colaboradores, se houver) |
| `{{nowplaying.album}}` | Álbum |
| `{{nowplaying.tempo}}` | Posição atual, no relógio `1:23` |
| `{{nowplaying.duracao}}` | Duração da música, no relógio `3:21` |
| `{{nowplaying.link}}` | Link da música no Spotify |

Exemplo de comando de chat — crie uma automação com gatilho de comando e resposta:

```
Tocando agora: {{nowplaying.musica}} — {{nowplaying.artista}} ({{nowplaying.tempo}}/{{nowplaying.duracao}}) {{nowplaying.link}}
```

> **Por que `!tocando` e não `!musica`?** O `!musica` já é do pedido de músicas do módulo Fila de músicas e é atendido antes das automações. Use outro nome no gatilho — `!tocando`, `!song`, o que preferir.

## O que a leitura acompanha

- Apenas **faixas de música**. Anúncios e podcasts contam como "nada tocando" e a variável fica vazia até a música voltar.
- A posição na música (`{{nowplaying.tempo}}`) é a da última leitura, com no máximo 5 segundos de diferença.
- Pausar no player zera as variáveis na próxima leitura — o BotLive não distingue "pausado" de "parado".

## Se algo não funcionar

- **"A autorização do Spotify expirou"**: o Spotify troca as chaves periodicamente; autorize a conta de novo (o app faz isso sozinho quando consegue).
- **"O Spotify pediu uma pausa nas consultas"**: o Spotify limitou as consultas por alguns segundos; o BotLive espera sozinho e continua.
- **O pré-voo avisa**: com o módulo ligado, a Visão geral acusa Client ID faltando ou autorização ausente, com botão direto para a tela.
- **Conta gratuita**: a leitura geralmente funciona, mas a API de música é pensada para contas Premium; se a leitura falhar na sua conta, confira os avisos do Histórico.

## Privacidade e regras

- Client ID e tokens ficam **na sua máquina** (cofre de credenciais do sistema); nada é enviado aos servidores do BotLive — não existe servidor do BotLive no meio.
- Apenas o escopo de leitura da música é pedido (`user-read-currently-playing`, mais os de estado e perfil para mostrar quem conectou).
- Exibir o **título, artista e álbum** da música na live é uso de metadados, permitido; **transmitir o áudio** do Spotify na live viola os termos do serviço — o áudio é sempre do seu player.
