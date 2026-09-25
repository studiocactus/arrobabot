# Solução de problemas

## Primeiro diagnóstico

1. Confirme que está no aplicativo desktop.
2. Confira o perfil ativo.
3. Reproduza a falha uma vez e anote o horário.
4. Abra Histórico e procure o primeiro erro relacionado.
5. Siga a orientação correspondente abaixo.

Não precisa apagar o perfil para testar uma configuração. Excluir pode destruir notas, comandos e saldos úteis ao diagnóstico.

## Abertura e instalação

| Sintoma | O que verificar |
|---|---|
| O atalho informa que não encontrou o executável | Confira entrega/BotLive.exe; o atalho deve ficar na raiz do projeto |
| Janela não aparece ou fecha imediatamente | Use o pacote corrigido da entrega; confirme Windows x64 e WebView2; registre a versão e o erro |
| Faixa PRÉVIA NO NAVEGADOR | Abra o executável; a prévia não executa o motor nativo |
| Dados da prévia não aparecem no desktop | São armazenamentos separados, sem migração automática |
| Porta local ocupada | Feche outra instância, ou altere a porta em Configurações e reinicie |

## Contas e conexões

| Mensagem ou sintoma | Próximo passo |
|---|---|
| Informe o Client ID | Use o identificador do aplicativo registrado, não o nome do canal |
| Conta errada vinculada | Refaça a autorização, conferindo a conta no site |
| Autorização pendente | Termine a etapa no navegador e aguarde antes de clicar novamente |
| Porta 43827 ocupada | Termine a outra autorização ou feche a instância duplicada; tente novamente |
| Tempo de autorização terminou | Inicie uma nova autorização e conclua em até três minutos |
| A plataforma recusou a autorização | Confira cliente, segredo, retorno e habilitação da conta de teste |
| Autorização expirou | Reautorize a conta correspondente |
| Reconectando repetidamente | Consulte o erro; confira internet, IDs, autorização e permissões |
| Teste envia, mas !oi não recebe resposta | Conecte a recepção e teste com uma conta diferente da conta do bot |
| YouTube sem chat ativo | Confira vídeo da live atual, chat habilitado e IDs; reconecte |
| Kick exige webhook | A recepção precisa da ponte pública externa; o botão de conectar não a cria |

## Comandos e fluxos

| Sintoma | O que verificar |
|---|---|
| Não dispara | Perfil, chave Ativo, nome exato, papel do usuário e intervalos |
| Funciona em simulação, mas não publica | Autorização de envio, conexão de entrada e permissões da plataforma |
| Fluxo importado não dispara | Fluxos importados começam desativados; revise e ative |
| Só parte das ações executa | O primeiro erro interrompe as próximas; confira condições opcionais |
| Editor recusa salvar | Conecte todos os blocos em uma única sequência, sem ciclos ou ramificações |
| !pontos personalizado não executa | O módulo de pontos trata seu comando pronto antes dos fluxos |
| Espera inválida | O campo é em milissegundos; máximo 30000 |
| Resposta bloqueada | Revise a blocklist; a comparação encontra trechos dentro de palavras maiores |

## IA e memória

| Sintoma | O que fazer |
|---|---|
| Ollama indisponível | Inicie o serviço e confira endereço/porta |
| Ollama sem modelos | Instale um modelo no Ollama e repita Detectar modelos locais |
| Escolha um modelo de IA | Preencha o identificador exato do modelo |
| HTTP 401 ou 403 do provedor | Confira chave e permissões da conta |
| HTTP 404 | Confira endereço base e nome do modelo |
| HTTP 429 | Verifique limites e quota do serviço; aumente os intervalos do fluxo |
| O provedor mudou | Salve a chave novamente para autorizar a nova origem |
| Resposta bloqueada pelo filtro de assuntos | Revise assuntos proibidos; a classificação adicional precisa responder no formato esperado |
| A IA não lembra de uma nota | Confira perfil, salvamento e palavras relacionadas na nota; busca lexical não lê tudo |
| Caminho de nota recusado | Use .md, / e apenas as pastas aceitas do vault |
| Mudança de caminho deixou duas notas | Salvar em outro caminho cria outro arquivo; não é renomeação |
| Copiar para Obsidian está desativado | Selecione uma nota e salve as alterações |
| Obsidian não responde | Confira instância aberta, plugin, endereço local, chave e certificado confiável quando usar HTTPS |

Erros HTTP são indícios, não diagnósticos completos. Verifique também o painel do serviço. Nunca envie a chave da API junto com uma captura de erro.

## Comunidade, áudio e OBS

| Sintoma | O que verificar |
|---|---|
| Pontos não aumentam a cada mensagem | Existe intervalo por pessoa; mensagens rejeitadas não são participação elegível |
| Ajustar saldo não altera a pessoa esperada | O campo exige ID do espectador, não o nome de exibição |
| Saldo insuficiente | Confira saldo, preço/bilhetes/aposta e débitos anteriores |
| Sorteio ou previsão não abre | Finalize ou cancele a operação aberta |
| Pedido de música com link é ignorado | Verifique o módulo de moderação, que bloqueia links de participantes comuns |
| Música não toca | BotLive gerencia a fila; abra/reproduza no serviço original |
| Trivia automática não publica | Módulo e opção ativos, perguntas válidas, canal conectado, atividade anterior, intervalo cumprido e nenhuma pergunta aberta |
| Bingo não reconhece um emote | A detecção compara termos textuais; representação da plataforma pode ser diferente |
| TTS sem som | Teste a voz local, confira áudio do app, voz instalada, limite e filtro |
| Voz não reconhece | Confira microfone autorizado, servidor Whisper local e rota /inference |
| Overlay aguarda BotLive | Confira instância aberta, porta e token da sessão atual |
| Overlay de outro perfil | Preencha o UUID correto no parâmetro profile |
| Discord falha (webhook) | Confira webhook salvo, módulo ativo, permissões do canal e erro HTTP |
| Convite !discord inválido | Use convite HTTPS de discord.gg ou discord.com |
| Bot do Discord não conecta | Token de bot (não de aplicativo), servidor escolhido, **Ativar o bot do Discord** ligado e o motivo no Histórico |
| Bot do Discord não lê as mensagens | Ative **Message Content Intent** na página do aplicativo do Discord |
| Contador de membros não muda | Canal do contador preenchido, **Server Members Intent** ativa e rótulo só com letras, números e hífen |
| Comando slash não aparece no servidor | Conecte e clique em **Registrar comandos slash**; a publicação usa o servidor escolhido |

Motivos de conexão, intents e permissões do bot aparecem no **Histórico** com a categoria `discord`. O passo a passo e as limitações desta versão estão no capítulo [Discord](09-DISCORD.md).

## Acesso e backups

**SESSION_LOCKED:** entre com o acesso local. owner é o nome do proprietário, não o nome da conta de streaming.

**Somente o proprietário:** use owner para gerenciar contas, credenciais, exclusão de perfil e configurações gerais.

**Você não tem permissão para editar este perfil:** o proprietário precisa incluir o nome local na lista de editores desse perfil.

**Backup restaurado pede senha desconhecida:** o banco guarda a ativação do login; as senhas ficam no cofre do sistema original. Preserve a cópia e busque manutenção técnica. Não há recuperação automática por e-mail.

## Ao solicitar ajuda técnica

Informe versão, sistema operacional, tela/ação, perfil/plataforma afetados, horário, mensagem de erro e se aconteceu no desktop ou na prévia. Descreva o resultado esperado e os passos para reproduzir.

Retire tokens, chaves, webhooks privados e informações pessoais de qualquer material compartilhado. Não envie o banco inteiro quando uma descrição ou trecho do erro for suficiente.

## Timer ou contador não funcionou

- Timer: confirme perfil conectado, timer ativo, intervalo salvo e espera do período completo. Conexão online não exige transmissão ao vivo; pause ao terminar. Não há envios retroativos. No Kick com ponte externa, timers nativos não iniciam.
- Contador: confirme Contar usos deste comando, papel autorizado e intervalos liberados. Moderação, respostas TXT e comandos de comunidade podem consumir a mensagem antes. Simular não muda o total real.
- Contador aumentou, mas não respondeu: a contagem ocorre antes das ações; confira falha de envio no Histórico. Ajuste o total pela lista se necessário.
- Exemplos completos: [Timers e contadores](TIMERS-E-CONTADORES.md) e [Receitas por função](EXEMPLOS-DE-USO.md).
