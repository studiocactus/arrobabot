# Primeiro uso — seu primeiro comando

O BotLive organiza a automação de uma transmissão em perfis. Comece com um perfil e uma resposta simples; configure IA e módulos depois que essa primeira etapa funcionar.

Para escolher uma função pelo resultado que você deseja, abra [Exemplos práticos](EXEMPLOS-DE-USO.md). Não precisa ler os capítulos técnicos para usar o bot.

## Encontre a configuração pelo objetivo

Na **Visão geral**, use **O que você quer configurar?**: conectar canal, chat, visual da live, sons e voz, comunidade ou automação personalizada. Escolher um objetivo mostra um roteiro; cada passo abre a tela existente. O roteiro não indica conclusão nem executa alterações sozinho. Sem perfil, crie um pelo botão indicado antes de configurar os recursos.

O menu agrupa as mesmas telas por finalidade: **Seu canal**, **Chat**, **Visual do OBS**, **Áudio e voz**, **Automação e IA**, **Comunidade** e **Acompanhamento**. **Configurações** continua no rodapé.

Use **Ctrl+K** para procurar uma tela ou tarefa: “resgate” encontra Voicemod e Automações, “erro” encontra Histórico, “backup” encontra Configurações. A busca aceita palavras sem acentos.

## 1. Abra o aplicativo certo

No diretório do projeto, execute INICIAR-BOTLIVE.cmd. Você também pode abrir entrega/BotLive.exe ou usar o aplicativo instalado. Veja [Instalação](01-INSTALACAO.md).

Se aparecer a faixa **PRÉVIA NO NAVEGADOR**, você está na demonstração da interface. Ela permite editar perfis, comandos e notas em um armazenamento separado, mas não executa conexões, IA, cofre ou o motor de automação. Esses dados não são transferidos automaticamente para o desktop.

## 2. Crie o perfil

1. Na Visão geral, clique em **Criar primeiro bot**.
2. Em **Nome do perfil**, escreva Minha comunidade.
3. Escolha a plataforma.
4. Preencha **Nome do canal**, sem @.
5. Clique em **Salvar perfil**.
6. Aguarde o aviso **Perfil salvo.**

Você já pode criar e simular comandos sem autorizar uma conta. Para responder em um canal real, complete [Perfis e integrações](INTEGRACOES.md).

## 3. Crie uma resposta simples

1. Feche o formulário do perfil e abra **Comandos**.
2. Confira qual perfil está selecionado no topo.
3. Clique em **Novo comando**.
4. Use Nome: Boas-vindas; Comando: !oi.
5. Em Resposta, escreva: Olá, {{user}}! Bem-vindo ao canal {{channel}}.
6. Escolha Todo mundo e mantenha os intervalos iniciais.
7. Clique em **Salvar comando**.
8. Confira se a chave **Ativo** está ligada.

Digite apenas o comando no campo Comando, sem espaços. As variáveis {{user}} e {{channel}} são substituídas pelo nome da pessoa e pelo nome do canal.

## 4. Simule antes de publicar

1. Clique em **Simular evento**.
2. Informe !oi e escolha o papel Todo mundo.
3. Execute a simulação.
4. Abra **Histórico**.
5. Procure a resposta iniciada por [Simulação].

Resultado esperado: o histórico mostra a mensagem de entrada, o início do fluxo e a resposta simulada. Nada foi enviado à plataforma. A simulação também respeita intervalos: aguarde antes de repetir. Outras ações são registradas como “Executaria”, sem chamar IA, webhooks, TTS ou alterar pontos e memória.

## 5. Autorize e teste no canal real

1. Abra **Perfis de bot → Configurar**.
2. Preencha o Client ID e siga a autorização da sua plataforma.
3. Salve o perfil após conferir os identificadores.
4. Abra **Teste de envio ao chat**.
5. Clique em **Enviar mensagem de teste ao canal**.

Esse botão publica uma mensagem real. A mensagem confirma o envio, mas ainda não comprova o recebimento de eventos.

6. Feche o formulário e clique em **Conectar** no cartão do perfil.
7. Aguarde o estado **Conectado**.
8. Use outra conta para enviar !oi no chat da plataforma.
9. Confira a resposta e o Histórico.

Os adaptadores ignoram mensagens da própria conta do bot para evitar respostas em ciclo. Não use essa conta para testar o gatilho.

## 6. Acrescente recursos

| Quero… | Próxima tela |
|---|---|
| Uma resposta com várias etapas | Automações |
| Lembretes automáticos em intervalos | Timers |
| Contador próprio de mortes ou vitórias | Comandos → Contar usos deste comando; totais em Contadores |
| Respostas de TXT e sons de espectadores | Respostas (regras) e Sons de Live (sons por espectador) |
| Respostas geradas por um modelo | Inteligência artificial |
| Registrar o jogo, as regras e fatos da comunidade | Memórias |
| Pontos, sorteios, música e jogos | Comunidade |
| Silenciar, banir ou avisar alguém | Automações → ação Punir na Twitch |
| Backup automático em um arquivo só | Configurações → Backup do bot |
| Reutilizar uma configuração | Biblioteca de presets |
| Entender um erro | Histórico |
| Ver atividade acumulada | Estatísticas |
| Mudar tema ou proteger o painel | Configurações |

## Uma rotina simples para cada live

Antes de começar: abra o desktop, escolha o perfil, conecte o canal, confira o histórico e rode o **pré-voo da live** na Visão geral — o relatório acusa, antes de ir ao ar, alertas e chat desligados, sons apagados e ações sem módulo ou autorização, e cada aviso leva à tela de conserto. Teste um comando (no editor de automações, **Testar comando ao vivo** ensaia o comando pelo funil da live, sem publicar nada). Atualize a nota de contexto da transmissão, se usar IA.

Durante a live: acompanhe erros, cuide das filas e finalize sorteios e previsões pelo painel.

Ao terminar: resolva ou cancele apostas e sorteios abertos, desconecte os canais e feche o aplicativo. Confira no Histórico se o backup automático do dia saiu em **Backup do bot** e faça a cópia manual da pasta de dados com o BotLive fechado quando for trocar de computador.

## Atalhos

| Atalho | Função |
|---|---|
| Ctrl + K | Abrir busca de telas |
| Ctrl + Shift + N | Abrir a criação de comando |
| Ctrl + Shift + S | Ir para a biblioteca de presets |
| Escape | Fechar um modal aberto |
| Delete / Backspace no editor de nós | Remover o elemento selecionado |

O atalho de presets abre a biblioteca; ele não salva automaticamente a configuração atual. Salve textos e fluxos antes de trocar de tela.


## Criar a partir de uma receita

Em Comandos ou Automações, abra **Receitas prontas**. Escolha o resultado, confira os requisitos e use **Configurar receita**. O catálogo abre o editor já usado pela função; se encontrar uma configuração equivalente, oferece **Editar configuração existente**. Abrir uma receita não altera seus dados.

Novas configurações são salvas desativadas para você revisar e ativar na lista. Raid e voz abrem os painéis atuais de Comunidade e Voicemod, preservando suas regras e filas.


## Investigar uma execução

Em **Histórico → Execuções**, escolha um registro para conferir o resultado de cada etapa e o erro informado pelo motor. Busque pelo nome ou identificador e use os filtros de falhas, andamento ou testes. Parada por condição é um resultado da regra, não uma falha técnica.

A consulta se atualiza a cada dois segundos e não repete ações. Se a atualização falhar, a tela avisa que os dados anteriores podem estar desatualizados e oferece nova tentativa. **Eventos gerais** preserva a busca anterior de mensagens, conexões e demais serviços.

O acompanhamento conserva até 30 execuções finalizadas por perfil no banco local, além das que estiverem em andamento. Reiniciar não apaga esses registros. O início, o tempo total e o tempo de cada etapa são informados pelo motor; etapas não iniciadas não têm duração registrada. A medição inclui esperas e fica congelada ao terminar. A origem exibe tipo de evento, modo (live, teste, ensaio ou simulação) e uma referência reduzida que permite relacionar execuções do mesmo evento dentro do perfil. A busca aceita essa referência e o tipo do evento. O contexto inclui somente esses metadados: não copia mensagens, nomes, identificadores de pessoas, payloads ou credenciais. A referência é derivada do identificador e do perfil, sem expor o identificador original; não é uma garantia de anonimização. Eventos sem identificador mostram referência indisponível. Registros anteriores sem contexto são indicados como origem não registrada. Se o aplicativo encerrar sem confirmar o resultado final, o registro aparece como **Interrompida** na próxima abertura. Isso não significa falha nem sucesso da ação externa: ela pode ter ocorrido. Os tempos são os do último registro salvo, sem incluir o período fechado. Etapas que ainda não tinham começado aparecem como Não iniciada. Não há retomada nem repetição automática. O filtro Falhas e interrupções reúne os casos para investigação.

Esse histórico de diagnóstico fica no banco local e não entra nos pacotes de backup do bot; os Eventos gerais continuam incluídos nos backups. Restaurar um backup substitui os dados anteriores e limpa o acompanhamento em memória para não misturar execuções das duas bases. Excluir um perfil também remove seus registros. Falha ao consultar ou gravar o histórico é exibida pela central, com nova tentativa; não é tratada como lista vazia.

Para uma execução em andamento, use **Cancelar execução** e confira o nome antes de confirmar. O pedido não desfaz efeitos já concluídos. A tela aguarda o estado final do motor; a ação atual pode continuar até concluir ou atingir seu limite. O cancelamento impede as próximas etapas e vale somente para o perfil ativo. Se a ação final já estiver concluindo, o resultado pode ser Concluída, não Cancelada. Falha ao enviar o pedido aparece no diálogo e permite nova tentativa. Falhas de dispositivos de áudio exigem diagnóstico no equipamento real.


## Central da live

Com um perfil criado, a **Central da live** substitui o cartão repetido Comece por aqui na Visão geral. Mostra o estado informado pelo núcleo para canal, fontes de áudio, Voicemod, fila de voz, temporários do OBS e execuções recentes. O roteiro por objetivos e o pré-voo continuam disponíveis para configurar e conferir o perfil.

A consulta ocorre a cada cinco segundos e oferece Atualizar estado. Falha mantém a última leitura com aviso de dados possivelmente desatualizados; não transforma ausência de resposta em serviço conectado. Os atalhos levam às telas existentes para corrigir contas, áudio, voz, OBS e execuções. Abrir a central não conecta serviços, aplica efeitos, reinicia filas ou repete ações. Uma fonte de áudio conectada não comprova áudio audível; confirme por gravação. O pré-voo continua sendo conferência estática distinta do estado operacional.

As etapas compartilhadas ficam no próprio editor de automações: configure, guarde e vincule uma etapa; em outro fluxo, selecione Etapa reutilizável. Atualizar um original vale para os próximos disparos e não altera execuções já iniciadas.
