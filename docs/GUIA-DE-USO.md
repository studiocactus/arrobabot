# Primeiro uso — seu primeiro comando

O BotLive organiza a automação de uma transmissão em perfis. Comece com um perfil e uma resposta simples; configure IA e módulos depois que essa primeira etapa funcionar.

Para escolher uma função pelo resultado que você deseja, abra [Exemplos práticos](EXEMPLOS-DE-USO.md). Não precisa ler os capítulos técnicos para usar o bot.

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
5. Em Resposta, escreva: Olá, $user! Bem-vindo ao canal $channel.
6. Escolha Todo mundo e mantenha os intervalos iniciais.
7. Clique em **Salvar comando**.
8. Confira se a chave **Ativo** está ligada.

Digite apenas o comando no campo Comando, sem espaços. As variáveis $user e $channel são substituídas pelo nome da pessoa e pelo nome do canal.

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
| Lembretes automáticos em intervalos | Comandos → Timers |
| Contador próprio de mortes ou vitórias | Comandos → Contar usos deste comando |
| Respostas de TXT e sons de espectadores | Comandos → Respostas TXT e sons |
| Respostas geradas por um modelo | Inteligência artificial |
| Registrar o jogo, as regras e fatos da comunidade | Memórias |
| Pontos, sorteios, música e jogos | Comunidade |
| Reutilizar uma configuração | Biblioteca de presets |
| Entender um erro | Histórico |
| Ver atividade acumulada | Estatísticas |
| Mudar tema ou proteger o painel | Configurações |

## Uma rotina simples para cada live

Antes de começar: abra o desktop, escolha o perfil, conecte o canal, confira o histórico e teste um comando. Atualize a nota de contexto da transmissão, se usar IA.

Durante a live: acompanhe erros, cuide das filas e finalize sorteios e previsões pelo painel.

Ao terminar: resolva ou cancele apostas e sorteios abertos, desconecte os canais e feche o aplicativo. Faça backup regularmente com o BotLive fechado.

## Atalhos

| Atalho | Função |
|---|---|
| Ctrl + K | Abrir busca de telas |
| Ctrl + Shift + N | Abrir a criação de comando |
| Ctrl + Shift + S | Ir para a biblioteca de presets |
| Escape | Fechar um modal aberto |
| Delete / Backspace no editor de nós | Remover o elemento selecionado |

O atalho de presets abre a biblioteca; ele não salva automaticamente a configuração atual. Salve textos e fluxos antes de trocar de tela.
