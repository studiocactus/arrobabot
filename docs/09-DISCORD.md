# Discord: seu servidor administrado junto com a live

O BotLive fala com o Discord por dois caminhos oficiais: o **Gateway** (`wss://gateway.discord.gg`, versão 10) para receber eventos e a **API REST** (`https://discord.com/api/v10`) para executar ações. Não existe ponte de terceiros, extensão do navegador nem robô de conta pessoal. Tudo acontece com o token de um bot criado por você no portal do desenvolvedor do Discord.

A tela **Discord**, em **SEU ESPAÇO**, concentra conexão, moderação, entrada e saída, espelho de chat, sorteios, XP, aniversários, vínculo de identidade e comandos slash.

**Escolha rápida:** a tela **Comunidade** modera quem fala na Twitch e passa aviso nas duas casas; a tela **Discord** administra o servidor. As duas compartilham a lista de palavras bloqueadas, a auto-moderação e a auditoria.

## O que você precisa ter

| Item | Onde pegar | Para que serve |
|---|---|---|
| Aplicativo (bot) | Portal do desenvolvedor do Discord | Sua identidade no servidor |
| Token do bot | Página do aplicativo → Bot → Reset Token | Autenticar as chamadas do Gateway e da API |
| ID do servidor | Configurações do Discord → Avançado → Modo desenvolvedor | Apontar qual guild será administrado |
| Permissões de bot | Convite gerado na própria tela | Garantir banir, moderar, gerenciar canais e cargos |

O token é salvo no cofre do sistema e sai da gravação em texto: ele não aparece nas configurações exportadas nem volta para a tela depois de salvo. Nunca publique o token em prints, repositórios ou tickets.

## Conectar o bot

1. Crie o aplicativo no portal do desenvolvedor do Discord e copie o token.
2. Abra **Discord** no perfil desejado e clique em **Salvar token**.
3. Ative **Server Members Intent** e **Message Content Intent** na página do aplicativo. Sem elas, o bot não lê o texto nem conta quem entrou.
4. Clique em **Descobrir servidor e canais**. Os servidores onde o bot já está aparecem na lista.
5. Escolha o **Servidor do Discord** e os canais que forem usar.
6. Ligue **Ativar o bot do Discord** e clique em **Salvar configuração**.
7. Clique em **Convite do bot** e autorize no servidor, se ainda não tiver convidado.
8. Por fim, clique em **Conectar**. O rótulo passa a mostrar **Conectado**.

**Se a conexão falhar:** confirme que o token é de bot e não de aplicativo OAuth, que o servidor foi informado e que as intents privilegiadas estão ligadas. Os motivos aparecem no **Histórico**, com a categoria `discord`.

## Servidor, canais e cargos

| Campo | O que faz | Observação |
|---|---|---|
| Servidor do Discord | Define o guild administrado | Pode ser escolhido na lista ou colado como ID |
| Canal de logs de auditoria | Recebe entradas, saídas, banimentos e ações do bot | Fica vazio se o canal não existir mais |
| Cargo automático na entrada | Concede um cargo a quem chega | Exige que o bot esteja acima do cargo |
| Canal do contador de membros | Renomeia o canal para `membros-1234` | Use um canal criado só para isso |
| Rótulo do contador | Prefixo do nome, como `presenca` | Vira `presenca-1234`; só letras, números e hífen |
| Cargo de moderador / assinante | Traduz cargos do Discord para as regras do bot | É o que conecta permissão do servidor à ação do bot |

As listas de canais e cargos só aparecem depois de **Descobrir servidor e canais**. Antes disso, os campos aceitam o ID digitado.

## Entrada, saída e contador de membros

- **Mensagem de boas-vindas** aceita `{user}` e `{name}`; sem texto, o bot escreve `Bem-vindo(a) ao servidor, {user}!`.
- **Mensagem de despedida** usa as mesmas variáveis e é publicada quando alguém sai.
- O **contador de membros** é um canal cujo nome acompanha o total. Ao entrar, sair ou ao conectar, o BotLive lê `member_count` do servidor e reescreve o nome.
- O **cargo automático** é enviado em paralelo; se falhar, o erro entra no **Histórico** sem impedir a mensagem de boas-vindas.

O contador depende do intent de membros. Com o intent desligado, o total fica congelado no valor lido ao conectar.

## Espelho do chat e notificações

Com **Espelho do chat** ativo e um canal escolhido, a conversa circula nos dois sentidos:

- A mensagem da Twitch é publicada no canal do Discord.
- A mensagem do Discord é publicada no canal da Twitch.

Os interruptores **Espelhar Twitch para o Discord** e **Espelhar Discord para a Twitch** controlam cada sentido separadamente, para você espelhar só a direção que interessa.

**Notificações da Twitch** publica seguidores, inscrições, bits e raids no canal escolhido, usando o mesmo formato já usado nos demais canais do bot.

## Auto-moderação

O interruptor **Auto-moderação do Discord** usa exatamente as mesmas regras da Twitch, já configuradas em **Comunidade**:

- Lista de expressões bloqueadas do perfil.
- Bloqueio de links.
- Bloqueio de repetição em sequência.

A mensagem violadora é apagada pelo Discord e a entrada entra na auditoria. Quando existe vínculo de identidade, o aviso também é contado para quem falou na Twitch.

## Moderação em dupla e auditoria única

Busque o membro, escolha o motivo e aplique. O motivo fica gravado junto com a ação.

| Ação | Efeito | Pode ser desfeita |
|---|---|---|
| Timeout | Silencia o membro por um período | Sim, remove o silenciamento |
| Expulsar | Remove do servidor | Não; a expulsão não tem reversão na API do Discord |
| Banir | Impede a entrada no servidor | Sim, devolve o acesso |
| Desbanir | Libera a entrada | Sim, volta a banir |
| Avisar nos dois chats | Registra aviso e soma no total do perfil | Sim, remove o aviso |
| Ver avisos | Lista os avisos daquele membro | — |

A página **Auditoria única** mostra o registro com a plataforma em que a ação aconteceu (`discord`, `twitch` ou os dois) e o botão **Desfazer**, que só aparece quando existe operação reversível. **Exportar CSV** baixa o histórico inteiro para conferência.

> Punir pela tela **Comunidade** continua valendo: se a identidade estiver vinculada, o aviso ou banimento aplicado na Twitch também é registrado no Discord com a plataforma combinada.

## Identidade unificada Twitch e Discord

1. Em **Identidade unificada Twitch e Discord**, informe o ID do membro no Discord.
2. Clique em **Gerar código**. O BotLive devolve um código de seis dígitos.
3. Peça para a pessoa digitar `!vincular CODIGO` no chat da Twitch.
4. O vínculo aparece na lista e pode ser removido a qualquer momento.

Com o vínculo, o bot reconhece a mesma pessoa nas duas casas: aviso, punição, XP e resposta a comando valem nos dois lugares.

## Sorteios, XP e aniversários

- **Sorteios:** informe canal, prêmio, minutos e quantidade de vencedores. A inscrição acontece reagindo com 🎉. **Encerrar** sorteia os vencedores; **Sortear de novo** refaz o sorteio sem repetir quem já ganhou.
- **XP e níveis:** quem conversa no Discord ganha XP a cada mensagem, com intervalo de um minuto por membro. O ranking mostra posição, nível e XP, e a linha também informa quanto falta para o próximo nível.
- **Aniversariantes:** registre o ID e a data em `DD/MM` ou `DD/MM/AAAA`. O BotLive comemora no dia, com a idade calculada a partir do ano informado.

Todos os três têm interruptor próprio e canal próprio, para você ligar só o que usa.

## Comandos slash próprios

Clique em **Registrar comandos slash** depois de conectar. O BotLive publica o catálogo dentro do servidor escolhido.

| Comando | O que faz | Quem pode usar |
|---|---|---|
| `/painel` | Mostra o estado do bot neste servidor | Qualquer pessoa |
| `/convite` | Devolve o link de convite | Qualquer pessoa |
| `/ranking` | Mostra o ranking do servidor | Qualquer pessoa |
| `/xp` | Mostra o nível de um membro | Qualquer pessoa |
| `/avisos` | Lista os avisos de um membro | Qualquer pessoa |
| `/vincular` | Usa o código gerado na tela do Discord | Qualquer pessoa |
| `/aniversario` | Registra um aniversário com data em `DD/MM/AAAA` | Qualquer pessoa |
| `/limpar` | Apaga até 100 mensagens do canal | Moderação |
| `/ban` | Bane o membro citado | Moderação |
| `/mute` | Silencia o membro citado | Moderação |
| `/warn` | Registra aviso nas duas casas | Moderação |
| `/sorteio` | Cria um sorteio com prêmio e duração | Moderação |

Os comandos marcados como **Moderação** conferem a permissão do autor antes de executar. Sem o servidor escolhido na tela **Discord**, qualquer comando devolve um aviso pedindo a configuração.

## Limitações desta versão

- **A conexão real não foi homologada com uma conta Discord nesta versão.** A compilação, os testes do núcleo e a tela da interface foram verificados; a primeira conexão com servidor real depende do seu token e do seu servidor.
- A prévia no navegador não abre conexões: ela mostra a tela, os formulários e as listas com dados de demonstração. O Gateway e a API só funcionam no aplicativo desktop.
- O token é de bot criado no portal do desenvolvedor. Usar token de conta pessoal (selfbot) viola os termos do Discord e não é suportado.
- As intents de membros e de conteúdo são privilegiadas e precisam ser ligadas manualmente no portal.
- Limites de requisição do Discord se aplicam. Em 429, o BotLive espera o tempo indicado e tenta de novo até três vezes por chamada.
- Canal de contador, canais de log e cargos devem existir; canais apagados geram erro no **Histórico** e nada mais quebra.
- O espelho de chat não traduz nem formata mensagem longa: textos acima de 2000 caracteres são cortados no limite do Discord.
