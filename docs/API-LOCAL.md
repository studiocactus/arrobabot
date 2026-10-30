# OBS, overlays e API local

## Visão geral

O BotLive expõe um WebSocket em 127.0.0.1, somente neste computador. Ele serve para receber eventos de ferramentas externas e emitir atividade para overlays. No outro sentido, o BotLive também fala com o OBS Studio desta máquina pelo WebSocket do OBS (ação **Ação no OBS** e tela **OBS Studio**).

## Controlar o OBS pelo BotLive

Para configurar pelo resultado, use as [receitas e controles visuais do OBS](16-OBS-AUTOMACOES.md): texto pelo chat, imagem por resgate, filtros, seleção de cena/grupo e edição de conteúdo. As listas e operações são compartilhadas entre o teste OBS Studio e o editor de automações.

Na tela **OBS Studio**: ative a integração, confira endereço (padrão `127.0.0.1`) e porta (padrão `4455`) e salve a senha se o OBS pedir (fica no cofre, nunca em log). **Testar conexão** mostra versão do OBS, versão do WebSocket e cena atual, além das listas de cenas, entradas e fontes.

No OBS, ative em Ferramentas → WebSocket do OBS. A porta 4455 nunca é exposta à internet: tudo acontece nesta máquina.

Nas automações, a ação **Ação no OBS** oferece mutar/desmutar/alternar entrada, volume, mostrar/esconder/alternar fonte e trocar de cena, com alvos em dropdown vindos do próprio OBS. Duração em segundos torna o efeito temporário e restaura o estado anterior no fim (se já estava mutado, continua mutado). Alvo ocupado por outro efeito é recusado no Histórico; o botão **Testar no OBS** executa na hora.

Dicas de uso: use nomes estáveis nas fontes; temporário curto (5–15 s) para zoeiras de resgate; intervalo no gatilho para não empilhar efeitos; confira o Histórico (tipo `obs`) a cada teste.

### Mídia e transformação de fontes

No mesmo seletor da ação OBS, escolha Reproduzir, Pausar, Parar, Reiniciar, Próxima mídia ou Mídia anterior. Selecione uma entrada que aceite controles de mídia. Próxima/anterior precisam de uma lista compatível, como uma fonte VLC. Uma fonte que não aceita o comando devolve erro; o BotLive não confirma sucesso sem a confirmação do OBS.

Girar fonte define um ângulo absoluto de -360 a 360 graus. Espelhar horizontalmente e verticalmente invertem o sinal da escala no eixo escolhido. Posição, recorte e os demais campos não são sobrescritos. O alvo é um item direto da cena de programa atual; para uma fonte dentro de um grupo, escolha o grupo no seletor de destino. Não há duração temporária para essas operações.

Essas operações usam a mesma fila e cancelamento das ações anteriores. Transformar um item ocupado por um efeito temporário é recusado até a liberação. Nenhuma delas inicia ou encerra a live. O botão Testar no OBS produz um efeito real, enquanto o ensaio de fluxo não altera o OBS.

| Identificador da operação | Alvo | Valor numérico |
|---|---|---|
| media_play / media_pause / media_stop | entrada de mídia | ignorado |
| media_restart / media_next / media_previous | entrada de mídia compatível | ignorado |
| rotate | fonte direta da cena atual | ângulo absoluto em graus |
| flip_horizontal / flip_vertical | fonte direta da cena atual | ignorado |

A chamada interna obs.execute recebe profileId, op, target, num e secs. Para as operações desta tabela, secs deve ser zero. Não é uma chamada disponível com a chave pública de overlay. Referência de protocolo: [OBS WebSocket 5.x](https://github.com/obsproject/obs-websocket/blob/master/docs/generated/protocol.md).

Abra **Configurações → API local & overlays** para consultar porta e chave. A porta padrão é 9876. Alterar a porta exige reiniciar o app. A chave muda a cada abertura.

A chave autoriza acesso a eventos de todos os perfis; ela não é uma chave limitada a um overlay. O filtro profile do exemplo apenas seleciona o que mostrar na tela.

## Preparar o overlay de texto

1. Abra o BotLive desktop.
2. Copie a chave da sessão nas Configurações.
3. Descubra o UUID do perfil: em Memórias, clique em Abrir pasta; o nome da pasta do perfil é esse identificador.
4. Localize examples/overlay.html no projeto.
5. No OBS, crie uma fonte de navegador e use a URL do arquivo local com os parâmetros abaixo.
6. Defina tamanho compatível com a cena, por exemplo 800 × 240.
7. Crie uma ação Atualizar overlay e dispare um evento real.
8. Confira o texto na cena.

Modelo de endereço, substituindo caminho, token e UUID:

```text
file:///C:/BotLive/examples/overlay.html#token=TOKEN_DA_SESSAO&profile=UUID_DO_PERFIL&port=9876
```

Use barras / na URL. Para nomes de pasta com espaços, use %20. No OBS, utilize o campo de URL da fonte de navegador para conservar o fragmento iniciado por #, em vez de selecionar apenas um arquivo sem os parâmetros.

## Camadas do overlay: parâmetro mode

O overlay servido pelo BotLive (`http://127.0.0.1:9876/overlay#token=…&profile=…`) pode mostrar as camadas numa fonte só ou cada camada na sua própria Fonte de navegador. O parâmetro `mode`, acrescentado ao final do endereço, escolhe qual camada aquela fonte mostra:

| mode | Camada | Tela que gera o endereço |
|---|---|---|
| `mode=goal` | Meta (LivePix) | Meta |
| `mode=alerts` | Alertas da live | Alertas |
| `mode=chat` | Coluna de chat | Chat no OBS |
| `mode=spotify` | Música tocando | Spotify |
| (sem mode) | Todas as camadas juntas | Qualquer uma das telas |

Cada tela do grupo **Overlays do OBS** copia o endereço da sua camada já com o `mode` certo — basta colar numa Fonte de navegador própria e ajustar posição e tamanho sem mexer nas demais. O endereço é estável e não muda ao reabrir o BotLive. Quem já usa uma fonte única, sem o parâmetro, continua vendo tudo como antes.

O exemplo tem fundo transparente, mostra texto por dez segundos e não executa HTML recebido. A simulação do BotLive só registra a ação; não altera o overlay real.

Sem o parâmetro profile, o exemplo pode mostrar eventos de qualquer perfil. Após reiniciar o BotLive, atualize o token na fonte do OBS.

## Overlay de músicas

Acrescente &view=songs à mesma URL. A primeira música da fila é apresentada como Agora, seguida de até duas URLs como A seguir.

A fila é inicializada após autenticação quando existem pedidos. A seleção avança quando você usa **Finalizar primeira música** no painel. Não há reprodução de áudio, leitura de metadados nem sincronização automática com o player.

## Fonte de áudio no OBS: rota /audio

A tela **Áudio no OBS** copia um endereço próprio para uma Fonte de navegador dedicada — a **Áudio do BotLive**, a saída única dos sons do bot. A página servida em `GET /audio` não mostra nada: ela só toca som. O endereço tem a forma:

```text
http://127.0.0.1:9876/audio#token=TOKEN_DO_OVERLAY&profile=UUID_DO_PERFIL
```

Como nas camadas do overlay, o `token` e o `profile` autorizam o acesso e só arquivos do perfil entram. Marque **Controlar áudio pelo OBS** na fonte para a reprodução automática.

### WebSocket da fonte (modo audio)

A página abre o mesmo WebSocket local, autenticando com o modo `audio`:

```json
{"token":"TOKEN_DO_OVERLAY","mode":"audio"}
```

Resposta: `{"type":"audio.claim"}` quando esta fonte passa a ser a dona da reprodução do perfil, ou `{"type":"audio.standby","payload":{"profileId":"UUID","reason":"duplicate"}}` quando outra fonte já reproduz. A fonte em espera não toca nada; se a dona desconectar, a de espera assume e recebe `audio.claim`.

Mensagens recebidas pela fonte:

| type | Conteúdo principal de payload |
|---|---|
| audio.play | profileId, id, kind (`sound`, `alert`, `tts`, `preview`, `test`), src (endereço local do áudio), volume (0–1), nickname e issuedAt (ms) |
| audio.stop | sem payload — para tudo e limpa a fila |
| audio.claim / audio.standby | papel desta conexão na reprodução do perfil |

A fonte confirma a reprodução enviando **apenas** `audio.ack`:

```json
{"type":"audio.ack","id":"ID_DO_SOM","phase":"started"}
```

com `phase` em `started`, `done`, `error` ou `stopped` e `detail` opcional (até 200 caracteres). A confirmação vira a notificação `audio.feedback` — é o que diferencia **som enviado** de **reprodução iniciada**. Outras mensagens vindas da fonte são ignoradas.

Eventos `audio.play` com `issuedAt` antigo (mais de 10 segundos) são descartados — na chegada **e de novo no momento de tocar** —: reconectar o OBS não repete sons antigos. A fila é serial, aceita até 5 pendentes; um som tem 15 segundos para começar e teto de 60 segundos quando já iniciado.

### Rota de TTS

`GET /tts?token=…&id=…` devolve uma fala gerada pelo Windows (SAPI) e guardada em cache pelo BotLive (32 hexadecimais + `.wav`, grupo `media` da pasta de dados). Assim como `GET /asset?token=…&id=…`, exige token de overlay válido, só serve arquivos do perfil autorizado e fica só em 127.0.0.1.

## Protocolo versão 1

### Abrir e autenticar

Conecte a ws://127.0.0.1:9876, ou à porta configurada. Em até cinco segundos, envie o primeiro frame:

```json
{"token":"TOKEN_EXIBIDO_NAS_CONFIGURACOES"}
```

Resposta:

```json
{"type":"ready","version":1}
```

Token inválido encerra a conexão. O servidor admite até dezesseis clientes simultâneos e mensagens de até 64 KiB. Eventos têm limite adicional de tamanho do texto.

### Enviar um evento de teste

```json
{
  "type": "event",
  "event": {
    "id": "teste-local",
    "profileId": "UUID_DO_PERFIL",
    "kind": "chat",
    "user": "Pessoa de teste",
    "userId": "teste-local",
    "role": "everyone",
    "message": "!oi",
    "data": null,
    "simulated": true
  }
}
```

| Campo | Descrição |
|---|---|
| id | Obrigatório no formato; o servidor substitui por um novo UUID |
| profileId | UUID de um perfil existente |
| kind | Tipo do evento, por exemplo chat ou custom |
| user | Nome visível para {{user}} |
| userId | Identificador usado em intervalos e módulos |
| role | Forçado a everyone pelo servidor |
| message | Texto usado por gatilhos, condições e {{message}} |
| data | Dados JSON adicionais |
| simulated | true evita efeitos externos e alterações de economia, memória e estatísticas — o evento entra no histórico marcado como ensaio |

Para integrações reais, simulated pode ser false. Confirme o fluxo antes: isso pode publicar mensagens, debitar pontos ou chamar webhooks.

Resposta de enfileiramento:

```json
{"type":"ack"}
```

Falha:

```json
{"type":"error","message":"Descrição do problema"}
```

ack confirma apenas a entrada na fila, não a conclusão das ações. O processamento é assíncrono. Acompanhe as notificações de atividade e não repita automaticamente um envio só porque a ação demorou. A API substitui IDs externos, então reenviar o mesmo JSON não produz deduplicação pelo seu id original.

### Escutar notificações

Formato geral:

```json
{"type":"overlay","payload":{"profileId":"UUID_DO_PERFIL","text":"Olá, comunidade!"}}
```

| type | Conteúdo principal de payload |
|---|---|
| activity | id do registro, profileId, timestamp, kind, message e status |
| connection | profileId e status da conexão |
| platform-event | Evento normalizado pelo motor |
| overlay | profileId e text |
| captions | profileId e text (fala finalizada da escuta contínua) |
| audio.play | profileId, id, kind, src, volume, nickname, issuedAt e sink (`obs` quando a fonte do OBS reproduz; `local` quando o aplicativo toca) |
| audio.stopLocal | profileId — o aplicativo deve parar e limpar a fila local |
| audio.feedback | profileId, id, phase (`started`, `done`, `error`, `stopped`) e detail |
| audio.state | profileId, clients (fontes conectadas) e claimed (se há fonte dona) |
| community | profileId; aviso de atualização, sem todo o estado |
| songs | profileId e queue, com itens user e url |

Notificações podem chegar antes ou depois de ack. Ao autenticar, uma fila de músicas existente também pode ser enviada. O cliente deve diferenciar mensagens pelo campo type, e não assumir que cada leitura é a resposta do último envio.

### Limites operacionais

A fila de entrada suporta 512 eventos e o motor processa até dezesseis eventos simultaneamente. Mensagens de evento acima de 16000 bytes são rejeitadas. Clientes lentos podem perder notificações quando o buffer de transmissão for ultrapassado.

A API não fornece comandos administrativos, consulta de segredos nem leitura completa do banco. Papéis de streamer/moderador não podem ser atribuídos por ela. Não existe endpoint HTTP REST de administração.

## Ponte de plataforma

Uma ponte Kick deve verificar assinatura e autenticidade na entrada pública, mapear o evento para um perfil e então encaminhá-lo pelo canal local autenticado. Essa ponte não está incluída como serviço hospedado.

Não exponha diretamente o WebSocket local como substituto de um receptor oficial de webhooks. Seu token permite disparar automações e observar atividade. O bloqueio da interface não encerra os clientes da API.

## Diagnóstico do overlay

- Aguardando BotLive: confira aplicativo aberto, porta e token da sessão.
- Sem texto: confira profile, ação Atualizar overlay e disparo real.
- Texto de outro canal: acrescente o UUID no parâmetro profile.
- Músicas sem áudio: o overlay só mostra URLs.
- Mudou a porta: reinicie o app e atualize a URL da fonte.
- Reiniciou o app: atualize o token na fonte e recarregue-a.


## Batimentos via Pulsoid

O conector Pulsoid usa o mesmo overlay dedicado e os mesmos eventos de batimentos da ponte Bluetooth. A chave do fornecedor nunca é a chave da fonte do OBS. As operações de conexão são internas e exclusivas do proprietário; não ampliam a API pública. Veja conexão pelo Android, requisito BRO/teste, formato do estado e operações em [Batimentos da live](15-BATIMENTOS.md).
