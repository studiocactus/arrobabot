# OBS, overlays e API local

## Visão geral

O BotLive expõe um WebSocket em 127.0.0.1, somente neste computador. Ele serve para receber eventos de ferramentas externas e emitir atividade para overlays. No outro sentido, o BotLive também fala com o OBS Studio desta máquina pelo WebSocket do OBS (ação **Ação no OBS** e tela **OBS Studio**).

## Controlar o OBS pelo BotLive

Na tela **OBS Studio**: ative a integração, confira endereço (padrão `127.0.0.1`) e porta (padrão `4455`) e salve a senha se o OBS pedir (fica no cofre, nunca em log). **Testar conexão** mostra versão do OBS, versão do WebSocket e cena atual, além das listas de cenas, entradas e fontes.

No OBS, ative em Ferramentas → WebSocket do OBS. A porta 4455 nunca é exposta à internet: tudo acontece nesta máquina.

Nas automações, a ação **Ação no OBS** oferece mutar/desmutar/alternar entrada, volume, mostrar/esconder/alternar fonte e trocar de cena, com alvos em dropdown vindos do próprio OBS. Duração em segundos torna o efeito temporário e restaura o estado anterior no fim (se já estava mutado, continua mutado). Alvo ocupado por outro efeito é recusado no Histórico; o botão **Testar no OBS** executa na hora.

Dicas de uso: use nomes estáveis nas fontes; temporário curto (5–15 s) para zoeiras de resgate; intervalo no gatilho para não empilhar efeitos; confira o Histórico (tipo `obs`) a cada teste.

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

O exemplo tem fundo transparente, mostra texto por dez segundos e não executa HTML recebido. A simulação do BotLive só registra a ação; não altera o overlay real.

Sem o parâmetro profile, o exemplo pode mostrar eventos de qualquer perfil. Após reiniciar o BotLive, atualize o token na fonte do OBS.

## Overlay de músicas

Acrescente &view=songs à mesma URL. A primeira música da fila é apresentada como Agora, seguida de até duas URLs como A seguir.

A fila é inicializada após autenticação quando existem pedidos. A seleção avança quando você usa **Finalizar primeira música** no painel. Não há reprodução de áudio, leitura de metadados nem sincronização automática com o player.

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
| simulated | true evita efeitos externos e alterações de economia/memória |

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
| tts | profileId, text, voice e rate |
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
