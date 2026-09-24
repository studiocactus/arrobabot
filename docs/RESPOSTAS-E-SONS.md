# Respostas TXT e sons por espectador

Abra **Comandos → Respostas TXT e sons** ou o mesmo botão em **Comunidade**. Escolha o perfil correto. As alterações entram em vigor ao clicar em **Salvar respostas e sons**. Somente o proprietário pode vincular arquivos, salvar configurações e reiniciar a sessão; moderadores autorizados podem consultar e testar arquivos já cadastrados. O navegador mostra os controles, mas os arquivos e a execução dependem do aplicativo desktop.

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
- **Quando tocar**: primeira mensagem da sessão ou mensagens com intervalo. O intervalo individual mínimo é cinco segundos; o padrão é sessenta.
- **Volume**: ajuste a intensidade e use **Testar som** para ouvir antes da live. O teste reproduz mesmo com a função desativada.

Ative a pessoa e **Ativar sons por espectador**, depois clique em **Salvar respostas e sons**. Para suspender, desligue o controle geral ou o individual e salve. Remover uma pessoa também exige salvar. Mudanças valem para novos eventos; áudio já em reprodução ou na fila termina normalmente.

“Chegar” significa enviar a primeira mensagem recebida pelo bot. Os adaptadores atuais não detectam espectadores silenciosos. Uma ponte autorizada pode enviar um evento `join` com identidade pela API local; ele usa as mesmas regras, sem privilégio de moderador. Um som no join conta como a primeira participação da sessão.

Uma sessão começa ao abrir o aplicativo. **Reiniciar sessão de sons** libera novamente as pessoas deste perfil, sem alterar intervalos TXT. Desconectar e reconectar não reinicia a sessão. Participações enquanto a função está desativada não consomem a primeira reprodução. Existe intervalo global de cinco segundos; uma pessoa ignorada nesse intervalo poderá tocar na próxima participação. Mensagens barradas pela moderação e mensagens da própria conta do bot não disparam sons.

O player toca um som por vez, aceita até cinco solicitações pendentes e interrompe cada áudio após quinze segundos. Falhas de reprodução aparecem no aviso da interface; **Som solicitado** no Histórico confirma o pedido, não comprova áudio na transmissão. Um pedido já emitido consome a primeira participação, mesmo se a reprodução falhar; após corrigir, reinicie a sessão.

## Fazer o som sair na live

Mantenha o BotLive aberto e desbloqueado e use **Testar som**. No OBS, inclua o áudio do desktop correspondente à saída usada pelo BotLive, ou uma captura de áudio do aplicativo. Confira o medidor e faça uma gravação local para validar volume e roteamento. Esta versão reproduz no computador; não configura o OBS automaticamente nem fornece um player de áudio em fonte de navegador. Evite capturar o mesmo áudio duas vezes.

A simulação registra que tocaria o som, sem reproduzir ou consumir a sessão real. Há até 200 pessoas e 300 arquivos cadastrados por perfil (TXT e sons somados). Remover regras ou pessoas mantém os arquivos registrados para reutilização; não apaga originais nem cópias do áudio. Arquivos de áudio corrompidos podem ser recusados pelo player mesmo com extensão válida.

## Backup e migração

As configurações ficam no banco. Preserve também a pasta **media** dentro da pasta de dados, que contém as cópias dos áudios. Os TXT ficam nos caminhos externos escolhidos e precisam de cópia separada. Estes recursos não são transportados pelos presets. Ao mudar a pasta de dados ou de computador, vincule novamente os arquivos para atualizar os caminhos registrados. Apagar um perfil remove seu cadastro do banco, mas conserva suas cópias de áudio no disco; elas não são executadas sem cadastro.

## Resolver problemas

- Nenhuma resposta: confira controles ativados, palavra/acentos, intervalos, perfil e arquivo existente. Uma regra anterior pode ter respondido primeiro.
- Variável ausente: revise o marcador. Variáveis locais de outro fluxo, como local.aiResponse, não estão disponíveis no TXT.
- Som não toca: confira nome recebido ou ID, controle geral/individual, sessão já consumida, volume e avisos da interface. Use Testar som, depois Reiniciar sessão de sons se necessário.
- Som só no computador: ajuste a captura no OBS e confira o medidor. O BotLive não altera o mixer da transmissão.
- Arquivo recusado: TXT exige UTF-8; áudio exige WAV/MP3/OGG. Confira também tamanho e limites indicados acima.
