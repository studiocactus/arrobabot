# Perfis, contas e conexões

## O que um perfil separa

Cada perfil tem suas próprias contas, comandos, fluxos, restrições, IA, notas e estado de comunidade. Nome e tema do painel não identificam uma autorização: Client ID, ID do canal e ID do bot têm funções diferentes.

A biblioteca de presets, a aparência e as configurações gerais são compartilhadas no aplicativo. Importar um preset permite reaproveitar configurações entre perfis.

## Criar ou editar

Abra **Perfis de bot → Novo perfil** ou **Configurar** em um cartão existente. Preencha os campos e use **Salvar perfil**. Em telas com **Perfil ativo**, confira o destino antes de alterar dados.

| Campo | O que preencher |
|---|---|
| Nome do perfil | Nome interno, como Canal principal |
| Plataforma | Twitch, YouTube ou Kick |
| Nome do canal | Nome usado na interface e na variável $channel |
| Client ID do aplicativo | Identificador obtido ao registrar um aplicativo na plataforma |
| Client secret | Segredo OAuth, quando exigido; use Salvar segredo no cofre |
| ID do canal | Twitch/Kick: identificador numérico do canal/proprietário |
| ID do vídeo ao vivo | No YouTube, o ID do vídeo da transmissão, não a URL completa |
| ID da conta do bot | Identificador usado para reconhecer o bot; no YouTube é o ID do canal da conta que publica |

O BotLive não registra automaticamente um aplicativo de desenvolvedor para você. Não coloque Client secret no campo Client ID, nem senhas nas descrições do perfil.

## Twitch — passo a passo

1. Registre um aplicativo próprio no portal da Twitch e obtenha o Client ID. O BotLive usa o fluxo de autorização por dispositivo para cliente público.
2. Salve o perfil com esse identificador.
3. Clique em **Autorizar conta do bot**.
4. Clique em **Abrir autorização**. No site da Twitch, confira a conta conectada e informe o código apresentado, se solicitado.
5. Conceda a autorização no site e volte ao BotLive.
6. Clique em **Já autorizei**. Aguarde a confirmação. Se a autorização ainda estiver pendente, aguarde alguns segundos antes de tentar novamente.
7. Repita com **Autorizar conta do canal**, entrando na conta do proprietário do canal.
8. Confira os IDs preenchidos após a vinculação e salve o perfil.
9. Faça o teste de envio e depois clique em **Conectar**.

A autorização de uma conta não autoriza a outra. É possível usar uma conta dedicada para o bot. Se o navegador estiver na conta errada, troque a conta no site antes de conceder o acesso.

O código tem conectores para chat, follow, inscrição, bits, raid e resgates. As permissões concedidas e as condições da plataforma determinam quais assinaturas de eventos são aceitas. Avisos, timeout e banimento usam a autorização da conta do canal.

Referências oficiais: [registro e aplicativos](https://dev.twitch.tv/docs/authentication/register-app/), [fluxos OAuth](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/) e [EventSub WebSocket](https://dev.twitch.tv/docs/eventsub/handling-websocket-events/).

## YouTube — preparação e autorização

1. Prepare um projeto com YouTube Data API habilitada e uma configuração OAuth compatível com aplicativo local.
2. Configure os usuários de teste ou a disponibilização do aplicativo conforme as regras da sua conta.
3. No BotLive, selecione YouTube e informe Client ID.
4. Quando seu cliente exigir segredo, preencha Client secret e clique em **Salvar segredo no cofre**.
5. O retorno usado pelo BotLive é exatamente http://127.0.0.1:43827/callback. A configuração OAuth precisa aceitar esse retorno.
6. Clique em **Autorizar conta do bot** e conclua a autorização no navegador. Não execute duas autorizações ao mesmo tempo.
7. Preencha **ID do vídeo ao vivo** e **ID da conta do bot** manualmente.
8. Salve, teste o envio e conecte.

Exemplo: em uma URL de vídeo com watch?v=ABC123, o campo do vídeo recebe apenas ABC123. Use um vídeo da transmissão atual com chat disponível. Ao iniciar uma transmissão com outro ID, atualize o perfil e reconecte.

A leitura é por consultas periódicas e respeita o intervalo retornado pelo serviço. Não há promessa de sincronização instantânea. O primeiro lote existente é ignorado para não executar novamente comandos antigos.

Referências oficiais: [OAuth para aplicativos nativos](https://developers.google.com/identity/protocols/oauth2/native-app) e [mensagens do chat ao vivo](https://developers.google.com/youtube/v3/live/docs/liveChatMessages/list). A compatibilidade do cadastro OAuth e as quotas precisam ser testadas com seu projeto real.

## Kick — disponibilidade nesta versão

A autorização pelo navegador e o envio usam a API oficial. Informe Client ID, segredo quando exigido, retorno http://127.0.0.1:43827/callback e os IDs numéricos correspondentes. Depois de autorizar, o teste de envio pode ser usado para verificar a publicação.

Receber eventos exige uma ponte de webhook com endereço público. O BotLive não hospeda essa ponte e o botão Conectar informa essa dependência; ele não mantém uma conexão de entrada pronta com a Kick. Não interprete o envio de uma mensagem como prova de recebimento.

Uma implementação de ponte deve validar as assinaturas oficiais antes de encaminhar eventos à [API local](API-LOCAL.md). Eventos dessa API entram com papel público, sem conceder permissões de moderador. A configuração não é uma etapa de um clique para iniciantes.

Referências oficiais: [documentação Kick](https://docs.kick.com/) e [segurança de webhooks](https://github.com/KickEngineering/KickDevDocs/blob/main/events/webhook-security.md).

## Discord — bot no servidor

O Discord não usa OAuth de perfil: ele é conectado com o token de um bot criado no portal do desenvolvedor. Salve o token em **Discord → Token do bot**, informe o servidor e ative **Ativar o bot do Discord**. O token fica no cofre do sistema e não é exportado com as configurações.

Ative **Server Members Intent** e **Message Content Intent** na página do aplicativo antes de conectar; sem elas o bot não conta entradas nem lê o texto. Depois de conectar, use **Descobrir servidor e canais** para preencher as listas de canais e cargos.

A tela **Discord** cobre entrada e saída, contador de membros, espelho de chat, notificações, auto-moderação, auditoria, sorteios, XP, aniversários, vínculo de identidade e comandos slash. O passo a passo completo está no capítulo [Discord](09-DISCORD.md).

## Estados e reconexão

| Estado | Interpretação |
|---|---|
| Desconectado | Sem recepção ativa |
| Conectando | Inicialização da conexão |
| Conectado | Adaptador conectado; ainda confira o recebimento de uma mensagem |
| Reconectando | A conexão caiu ou falhou; o aplicativo tentará novamente |

Se o estado não estabilizar, consulte Histórico antes de repetir a autorização. Alterar o Client ID ou a plataforma desconecta o perfil e remove suas autorizações anteriores. Para mudanças de identificação do canal, salve e desconecte/conecte novamente.

## Restrições e exclusão

Em **Restrições de conteúdo**, coloque uma expressão por linha. A busca de palavras é por ocorrência no texto, sem diferenciar maiúsculas e minúsculas; uma expressão curta pode bloquear palavras maiores que a contenham. Os assuntos proibidos são usados pela IA.

Apagar um perfil remove comandos, notas, economia e credenciais desse perfil. Não há lixeira. Faça backup antes. Os demais perfis têm dados separados.

Nenhuma plataforma foi homologada com contas reais do usuário nesta entrega. Os procedimentos descrevem o fluxo implementado; mensagens de recusa de autorização, quota e escopo devem ser verificadas no Histórico.

## Permissão para sons na entrada silenciosa

A partir de 0.1.5, a autorização da conta do bot Twitch também solicita chat:read para o monitor opcional de entradas por IRC. Contas já autorizadas precisam de nova autorização para esse monitor; o chat por EventSub continua independente. Configure e salve os gatilhos no capítulo [Respostas TXT e sons](RESPOSTAS-E-SONS.md).
