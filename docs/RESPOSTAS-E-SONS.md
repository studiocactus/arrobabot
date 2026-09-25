# Respostas TXT e sons por espectador

Abra **Respostas e sons** no menu lateral ou o botão **Respostas TXT e sons** em **Comunidade**. Escolha o perfil correto. As alterações entram em vigor ao clicar em **Salvar respostas e sons**. Somente o proprietário pode vincular arquivos, salvar configurações e reiniciar a sessão; moderadores autorizados podem consultar e testar arquivos já cadastrados. O navegador mostra os controles, mas os arquivos e a execução dependem do aplicativo desktop.

Cada regra e cada pessoa aparecem como uma linha compacta com um ponto de situação, o nome do cadastro, os resumos em pastilhas, o controle de ativar e uma seta. Clique no título da linha para abrir ou fechar o formulário; **Expandir todos** e **Recolher todos** fazem o mesmo com a lista inteira, e o contador ao lado informa quantos cadastros existem. Com mais de cinco itens, o campo **Filtrar** localiza um cadastro pelo nome, pela palavra-chave ou pelo arquivo. Uma regra ou pessoa recém-adicionada já abre sozinha, e o conteúdo fechado sai da ordem de tabulação do teclado. Quando a lista está vazia, use o botão **Adicionar resposta TXT** ou **Adicionar pessoa** logo abaixo da explicação.

## Responder a uma palavra usando um TXT

1. Clique em **Adicionar resposta TXT** e escreva a palavra ou expressão, por exemplo `café`.
2. Escolha **Palavra / expressão inteira** para evitar que café combine com cafés, ou **Qualquer trecho da mensagem** para permitir esse caso. Maiúsculas e minúsculas não alteram a busca; acentos são preservados.
3. Clique em **Vincular TXT** e escolha um arquivo UTF-8 com uma resposta por linha. Linhas vazias são ignoradas. Não existe sintaxe especial de comentários.
4. Escolha **Aleatória, sem repetir a última** ou **Em sequência**. Uma única linha pode repetir; a sequência volta ao início ao terminar.
5. Ajuste os intervalos da regra e por pessoa. Os padrões são 30 e 60 segundos. A regra exige pelo menos um segundo; zero desativa somente o intervalo por pessoa.
6. Use **Conferir linhas** para visualizar as primeiras dez respostas e a quantidade total, sem publicar.
7. Ative a regra e **Ativar respostas TXT**, depois clique em **Salvar respostas e sons**.

Exemplo de arquivo:

```text
{{user}}, o café está servido!
Pausa para o café, {{user}}?
Hoje a energia do canal vem do café!
```

O TXT continua vinculado ao caminho original: editar e salvar fora do bot altera as próximas respostas, sem importar novamente. Se mover ou apagar o arquivo, vincule o novo caminho. São aceitos até 256 KiB, 2000 linhas não vazias e 450 caracteres por linha; a resposta expandida também deve caber em 450 caracteres. Escolha **UTF-8** ao salvar no editor. Arquivos inválidos interrompem a resposta e geram erro no Histórico.

A primeira regra correspondente com intervalo liberado escolhe uma linha e responde. Essa mensagem não segue para comandos de comunidade ou fluxos, evitando respostas duplicadas. Sons são avaliados antes e continuam independentes. A escolha e os intervalos são reservados antes do envio; uma falha de conexão pode consumir aquele turno. Posição da sequência e última escolha ficam na sessão do aplicativo, não são restauradas após fechar.

Expressões bloqueadas no perfil continuam sendo verificadas. Os marcadores do TXT usam o motor de variáveis; valores vindos do chat não são reinterpretados. Não há chamada à IA. **Simular evento** lê o arquivo e mostra a resposta no Histórico sem publicar nem avançar sequência ou intervalos reais. Simulações continuam respeitando regras ativadas.

São permitidas 100 regras por perfil. **Remover regra** remove a configuração após salvar; o arquivo original é preservado.

## Cadastrar um som para uma pessoa

Clique em **Adicionar pessoa** e preencha:

- **Nome no chat**: nome exato recebido da plataforma, sem depender de maiúsculas ou do @ inicial.
- **Apelido**: como você deseja identificar a pessoa no painel e nos registros; não substitui a identificação da conta.
- **ID da pessoa (opcional)**: quando informado, a identificação usa esse ID. Recomendado para nomes exibidos iguais. Correspondências por ID têm prioridade sobre nome.
- **Escolher som**: selecione WAV, MP3 ou OGG, até 5 MiB. O aplicativo guarda uma cópia; mover o original não quebra o som.
- **Disparar som**: quando enviar mensagem, quando entrar mesmo sem falar, ou ao entrar ou enviar mensagem.
- **Quando tocar**: uma vez por sessão ou repetir com intervalo. O intervalo individual mínimo é cinco segundos; o padrão é sessenta.
- **Volume**: ajuste a intensidade e use **Testar som** para ouvir antes da live. O teste reproduz mesmo com a função desativada.

Ative a pessoa e **Ativar sons por espectador**, depois clique em **Salvar respostas e sons**. Para suspender, desligue o controle geral ou o individual e salve. Remover uma pessoa também exige salvar. Mudanças valem para novos eventos; áudio já em reprodução ou na fila termina normalmente.

### Entrada silenciosa na Twitch

1. Ative **Monitorar entradas silenciosas na Twitch** e escolha **Quando entrar, mesmo sem falar** ou **Ao entrar ou enviar mensagem** para a pessoa.
2. Em **Nome no chat**, informe o login atual da conta Twitch (não o nome decorativo). Mesmo com ID preenchido, esse login é necessário para localizar a entrada; o ID informado continua sendo conferido.
3. Salve. No perfil, autorize novamente a conta do bot para conceder a permissão chat:read, depois conecte o perfil.
4. Confira no Histórico o registro **Monitor de entradas ativo**. Se faltar autorização, o Histórico explica como corrigir; as mensagens continuam pelo conector normal.
5. Teste com a pessoa entrando depois que o monitor estiver ativo. A lista inicial de pessoas já presentes não toca. Reconectar o bot não limpa os sons já consumidos na sessão.

A Twitch informa conexões ao chat por IRC; isso não comprova que alguém está assistindo ao vídeo. Entradas podem atrasar ou não ser informadas, e a ordem dos eventos não é garantida. Para maior previsibilidade, escolha **Quando enviar mensagem**. O monitor é opcional e não envia mensagens pelo IRC. Ao desativar e salvar, ele encerra a observação; uma tentativa em andamento pode demorar até o limite de rede. Veja a [documentação Twitch](https://dev.twitch.tv/docs/chat/irc/).

No YouTube, use mensagem: o conector não fornece presença silenciosa. Uma ponte autorizada pode enviar join com identidade pela API local em outras plataformas. Entradas contam para a mesma sessão de sons; com **Uma vez por sessão**, uma mensagem posterior não toca novamente. Cadastros antigos assumem **Quando enviar mensagem**, sem habilitar observação silenciosa automaticamente.

Uma sessão começa ao abrir o aplicativo. **Reiniciar sessão de sons** libera novamente as pessoas deste perfil, sem alterar intervalos TXT. Desconectar e reconectar não reinicia a sessão. Participações enquanto a função está desativada não consomem a primeira reprodução. Existe intervalo global de cinco segundos; uma pessoa ignorada nesse intervalo poderá tocar na próxima participação. Mensagens barradas pela moderação e mensagens da própria conta do bot não disparam sons.

O player toca um som por vez, aceita até cinco solicitações pendentes e interrompe cada áudio após quinze segundos. Falhas de reprodução aparecem no aviso da interface; **Som solicitado** no Histórico confirma o pedido, não comprova áudio na transmissão. Um pedido já emitido consome a primeira participação, mesmo se a reprodução falhar; após corrigir, reinicie a sessão.

## Fazer o som sair na live

Escolha **Dispositivo de saída dos sons** no painel, clique em **Salvar respostas e sons** e use **Testar som**. A seleção é por perfil e aplica-se somente aos sons por espectador, não ao texto para fala. **Padrão do Windows** acompanha a saída padrão. **Atualizar dispositivos** recarrega a lista; nomes e disponibilidade dependem das permissões do Windows/WebView. Se o ambiente não oferecer seleção, use o Mixer de volume do Windows. Se uma saída salva desaparecer ou for recusada, o player informa erro e não muda silenciosamente para outra saída; escolha novamente e salve.

Mantenha o BotLive aberto e desbloqueado e use **Testar som**. No OBS, inclua o áudio do desktop correspondente à saída usada pelo BotLive, ou uma captura de áudio do aplicativo. Confira o medidor e faça uma gravação local para validar volume e roteamento. Esta versão reproduz no computador; não configura o OBS automaticamente nem fornece um player de áudio em fonte de navegador. Evite capturar o mesmo áudio duas vezes.

A simulação registra que tocaria o som, sem reproduzir ou consumir a sessão real. Há até 200 pessoas e 300 arquivos cadastrados por perfil (TXT e sons somados). Remover regras ou pessoas mantém os arquivos registrados para reutilização; não apaga originais nem cópias do áudio. Arquivos de áudio corrompidos podem ser recusados pelo player mesmo com extensão válida.

## Backup e migração

As configurações ficam no banco. Preserve também a pasta **media** dentro da pasta de dados, que contém as cópias dos áudios. Os TXT ficam nos caminhos externos escolhidos e precisam de cópia separada. Estes recursos não são transportados pelos presets. Ao mudar de computador, selecione novamente o dispositivo de áudio: o identificador é local. Ao mudar a pasta de dados ou de computador, vincule novamente os arquivos para atualizar os caminhos registrados. Apagar um perfil remove seu cadastro do banco, mas conserva suas cópias de áudio no disco; elas não são executadas sem cadastro.

## Resolver problemas

- Nenhuma resposta: confira controles ativados, palavra/acentos, intervalos, perfil e arquivo existente. Uma regra anterior pode ter respondido primeiro.
- Variável ausente: revise o marcador. Variáveis locais de outro fluxo, como local.aiResponse, não estão disponíveis no TXT.
- Som não toca: confira nome recebido ou ID, controle geral/individual, sessão já consumida, volume e avisos da interface. Use Testar som, depois Reiniciar sessão de sons se necessário.
- Som só no computador: ajuste a captura no OBS e confira o medidor. O BotLive não altera o mixer da transmissão.
- Arquivo recusado: TXT exige UTF-8; áudio exige WAV/MP3/OGG. Confira também tamanho e limites indicados acima.
