# Música do Spotify

A tela **Spotify** lê a música que está tocando na sua conta e entrega o título, o artista, o álbum, a capa e a posição para o BotLive — hoje nas **automações** (variáveis `{{nowplaying.*}}`) e, na sequência do recurso, no **overlay do OBS**. É o caminho que ferramentas como o Snip fazem com arquivos de texto, mas dentro do BotLive: sem programa extra, sem arquivo solto e com a leitura acontecendo enquanto o app está aberto.

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
