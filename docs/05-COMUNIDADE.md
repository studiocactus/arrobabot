# Ferramentas de comunidade

O botão **Respostas TXT e sons** permite cadastrar espectadores com apelido e áudio personalizado, ativar/desativar cada pessoa e escolher mensagem, entrada silenciosa na Twitch ou ambos, uma vez por sessão ou com intervalo. Consulte o [guia completo](RESPOSTAS-E-SONS.md), incluindo a seleção de saída e captura do áudio no OBS. Para receitas de cada módulo, abra [Exemplos práticos](EXEMPLOS-DE-USO.md).

Abra **Comunidade**, confira o perfil e ligue a chave do módulo desejado. Clique no nome do cartão para ver seu painel. Abrir um cartão não ativa o módulo. Desativar conserva os dados; não equivale a cancelar uma operação em andamento.

Os comandos dos participantes funcionam no chat real. A simulação de fluxos não executa jogos, apostas ou concessão de pontos.

## Pontos e loja

Ao ativar **Pontos & loja**, uma loja vazia recebe três exemplos: Escolher o próximo desafio, Mensagem em destaque e Jogar com o streamer.

Em **Avançado**, configure:

| Campo | Padrão | Intervalo aceito |
|---|---|---|
| Nome da moeda | pontos | Até 30 caracteres na interface |
| Pontos por participação | 5 | 0 a 10000 |
| Intervalo por pessoa | 60 segundos | 10 a 86400 segundos |
| Multiplicador para assinantes | 2 | 1 a 10 |

A concessão acontece quando chega uma mensagem elegível e o intervalo daquela pessoa terminou. Não há contagem de tempo de espectadores silenciosos. O multiplicador usa a hierarquia de papéis: moderadores e streamer também passam pela condição de assinante.

Use **Ajustar saldo** para conceder ou remover pontos. O campo pede **ID do espectador**, não apelido. Saldo negativo é rejeitado. Não há conversão desses pontos em dinheiro no aplicativo.

### Loja

Clique em **Novo item**, informe nome, custo e estoque. !loja mostra as posições e !resgatar 1 compra o primeiro item. Um resgate desconta saldo, reduz estoque e aparece em Resgates recentes.

O resgate registra um pedido; ele não executa automaticamente o benefício descrito no nome. Cabe à equipe cumprir “Jogar com o streamer”, por exemplo. Não há editor completo de itens, reposição ou reembolso de resgate na interface desta versão.

## Sorteios

1. Ative Sorteios.
2. Clique em **Abrir sorteio**.
3. Use uma palavra sem espaços, em minúsculas, como !sorteio.
4. Defina custo 0 para participação gratuita ou um custo por bilhete.
5. Confirme e anuncie a regra aos participantes.
6. Aguarde entradas e clique em **Sortear vencedor**.
7. Confira o vencedor no painel.

No sorteio gratuito, cada pessoa tem uma entrada. No pago, !ticket 2 compra dois bilhetes. O peso de participantes reconhecidos como assinantes ou acima é dobrado; isso altera a chance, não o preço.

**Cancelar e reembolsar** devolve o custo dos bilhetes de um sorteio aberto. Depois de sortear, não existe cancelamento com devolução pelo mesmo botão. Não há etapa separada de fechar entradas e sortear mais tarde, nem modalidade “primeiro a digitar”.

## Previsões

1. Ative Previsões e abra **Nova previsão**.
2. Informe a pergunta e de duas a seis opções separadas por vírgula.
3. Oriente o chat: !bet 1 25 aposta 25 pontos na primeira opção.
4. Cada pessoa pode registrar uma aposta; não há edição pelo chat.
5. Quando souber o resultado, clique em **Encerrar: número. opção**.
6. Confirme o resultado na janela.

O total apostado é dividido proporcionalmente entre os vencedores, em valores inteiros. Sem vencedores, as apostas são devolvidas. **Cancelar e devolver pontos** cancela uma previsão aberta.

Exemplo: Ana apostou 25 na opção vencedora e foi a única vencedora; outras pessoas apostaram 75 nas demais. Ela recebe os 100 pontos do total. O resultado é final nessa interface, sem desfazer automático.

Desligar o módulo não reembolsa apostas nem bilhetes. Cancele pelo painel antes de desativar quando quiser devolver os pontos.

## Fila para jogar

Ative Fila para jogar. !entrar adiciona a pessoa, ou informa sua posição se ela já estiver na fila. !sair remove a própria entrada. !fila lista os participantes.

No painel, **Chamar próximo** remove a primeira pessoa da fila. Combine com sua comunidade como anunciar a chamada; o botão não inicia uma partida ou integração com jogo.

## Pedidos de música

Ative Pedidos de música. São aceitos links HTTPS dos domínios youtube.com, www.youtube.com, youtu.be e open.spotify.com.

| Comando | Resultado |
|---|---|
| !musica link | Adiciona o pedido |
| !musicas | Mostra até três próximas URLs |
| !minhamusica | Informa a posição do primeiro pedido da pessoa |
| !removermusica | Remove todos os pedidos daquela pessoa |

Em Avançado, o padrão é dois pedidos por pessoa e cinquenta na fila. É possível configurar de 1 a 20 por pessoa e de 1 a 500 no total.

**Abrir** abre o link no serviço original. **Finalizar primeira música** remove a primeira e avança a fila. O BotLive não reproduz áudio, consulta duração nem controla o player. A posição não é uma previsão de minutos de espera.

O [overlay](API-LOCAL.md) mostra a primeira URL como seleção atual e as próximas. Ele não detecta automaticamente quando uma música começou ou terminou.

A moderação desta versão bloqueia links de participantes comuns. Com ela ativada, pedidos por link podem ser interrompidos antes de chegar ao módulo de música; não há lista de domínios permitidos configurável. Considere esse conflito ao escolher os módulos da live.

## Jogos de chat

Ative Jogos de chat e, para os participantes acumularem saldo, Pontos & loja.

### Roleta

!roleta 10 aposta dez pontos. O resultado é aleatório, com chances iguais. Em caso de vitória, o retorno é o dobro da aposta debitada; em caso de derrota, a aposta é perdida. O intervalo por pessoa é trinta segundos.

### Duelo

!duelo ana 20 desafia Ana. O alvo precisa já ter participado do chat. Apenas a pessoa desafiada pode usar !aceitar, em até sessenta segundos.

O saldo de ambos é verificado no aceite. O vencedor aleatório recebe as duas apostas. Há apenas um desafio aberto por perfil; desafio expirado é descartado quando o chat volta a ser processado. Não há votação do vencedor.

### Bingo

Clique em **Abrir bingo de emotes**. !bingo gera uma cartela com três emotes do conjunto inicial. Os termos são marcados quando aparecem como palavras no chat. A primeira cartela completa detectada recebe cinquenta pontos.

A correspondência é textual; representações especiais de emotes de cada plataforma podem não coincidir com o nome da cartela. O painel usa o conjunto predefinido e não oferece editor de emotes.

### Trivia manual

Clique em **Criar pergunta de trivia**, informe pergunta, resposta e recompensa. A pergunta fica aberta no painel. Anuncie-a na transmissão ou no chat; a criação manual não publica a pergunta automaticamente.

A primeira mensagem igual à resposta, ignorando maiúsculas/minúsculas e espaços nas pontas, ganha a recompensa. A comparação não aceita automaticamente sinônimos ou respostas aproximadas. Criar outra pergunta substitui a atual.

### Trivia automática

1. Em Jogos de chat, abra **Avançado**.
2. Escolha **Publicar durante baixa atividade**.
3. Configure silêncio mínimo e intervalo entre perguntas.
4. Cadastre uma pergunta por linha, separada da resposta pelo caractere de barra vertical.
5. Salve a configuração.

Exemplo: Quanto é 2 + 2? | 4

Padrões: cinco minutos sem novas mensagens e quinze minutos entre perguntas. O verificador roda a cada trinta segundos, exige canal conectado e pelo menos uma atividade anterior registrada. Ele não inicia outra pergunta enquanto houver uma aberta.

A pergunta automática é publicada no chat, vale cinquenta pontos e tem prazo de cinco minutos, verificado pelo agendador enquanto o recurso permanece ativo e conectado.

## Texto para voz

1. Ative Texto para voz.
2. Escolha uma voz disponível no sistema.
3. Ajuste velocidade e limite de caracteres.
4. Clique em **Testar voz local** e confira o áudio.
5. Clique em **Salvar configuração**.
6. Em uma automação, use a ação **Ler em voz alta**.

O fluxo pode ser disparado por chat ou por um evento de resgate suportado pela plataforma. A leitura usa a interface aberta e a saída de áudio do sistema. Limite: 1 a 500 caracteres; padrão: 300. A velocidade oferecida vai de 0,5 a 2.

A lista depende das vozes instaladas. Não há download automático de voz nem provedor TTS remoto configurável nesta versão. O teste de voz só toca no computador; configure a captura de áudio do OBS separadamente, se quiser transmiti-lo.

## Controle por voz

Ative Controle por voz e informe um endpoint local compatível com whisper-server, inicialmente http://127.0.0.1:8080/inference.

Clique em **Gravar comando**, autorize o microfone quando solicitado e fale. A captura dura até oito segundos; **Parar e reconhecer** encerra antes. O servidor local recebe o áudio e devolve texto, que dispara os fluxos de Comando de voz.

Exemplo: configure um gatilho de voz com Texto que dispara igual a boas-vindas e uma ação de chat. A detecção usa um trecho do texto reconhecido; não exige transcrição idêntica da frase toda.

É captura por botão, não escuta contínua nem palavra de ativação. O servidor Whisper é externo ao instalador. O destino do reconhecimento deve ser local.

## Discord

Ative Discord, informe o webhook do canal e clique em **Guardar webhook**. Crie uma automação com **Enviar ao Discord**. O webhook é guardado no cofre e menções em massa são desativadas no envio.

Em Avançado, informe um convite HTTPS oficial de discord.gg ou discord.com para o comando !discord. Convite e webhook são diferentes: o convite permite entrar no servidor; o webhook publica mensagens.

## Moderação

Ative Moderação, escolha a ação e salve:

| Ação | Comportamento |
|---|---|
| Só ignorar e registrar | Interrompe o processamento da mensagem e registra a ocorrência |
| Enviar aviso nativo | Envia aviso Twitch |
| Aplicar timeout | Timeout Twitch, com duração configurada |
| Banir | Banimento Twitch |

São detectadas expressões proibidas, links com http://, https:// ou www. e repetição idêntica em quinze segundos. Streamer e moderadores ficam isentos da detecção automática. Avisos e sanções nativas estão implementados apenas para Twitch e exigem a autorização apropriada do canal.

“Só ignorar” significa que o bot ignora a mensagem; não significa apagar a mensagem na plataforma. A blocklist de saída continua sendo aplicada às respostas do bot, independentemente da sanção escolhida.

## Referência rápida de comandos

| Comando | Módulo |
|---|---|
| !pontos, !ranking | Pontos |
| !transferir nome valor | Pontos; o destinatário precisa ter participado |
| !loja, !resgatar número | Pontos e loja |
| Palavra do sorteio, !ticket quantidade | Sorteios |
| !bet opção valor | Previsões |
| !entrar, !sair, !fila | Fila para jogar |
| !musica link, !musicas, !minhamusica, !removermusica | Música |
| !roleta valor, !duelo nome valor, !aceitar, !bingo | Jogos |
| !discord | Discord |

Esses comandos dependem da ativação dos módulos. Valores de aposta, transferência e quantidades interpretadas pelos comandos devem ser inteiros positivos, até 1.000.000, sujeitos a saldo, estoque e demais limites.
