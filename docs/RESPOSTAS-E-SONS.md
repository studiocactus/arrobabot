# Respostas TXT e sons por espectador

A partir da versão 0.7.0, respostas e sons por espectador moram em **duas telas separadas**, ambos no grupo **Seu espaço** do menu lateral:

- **Respostas** — as regras de resposta em texto (TXT), cada uma com um som opcional da biblioteca.
- **Sons de Live** — a biblioteca de áudios e os sons por espectador, com a sessão de sons.

O botão **Respostas** em **Comunidade** abre a tela de Respostas. Escolha o perfil correto em ambas. Cada tela tem seu próprio botão de salvar (**Salvar respostas** e **Salvar sons por espectador**); as alterações só entram em vigor ao salvar, e a tela mostra **Alterações não salvas** / **Tudo salvo** enquanto você edita. O navegador mostra os controles, mas os arquivos e a execução dependem do aplicativo desktop.

Nas duas telas, cada regra ou pessoa aparece como uma linha compacta com um ponto de situação, o nome, os resumos em pastilhas, o controle de ativar e uma seta. Clique no título para abrir ou fechar o formulário; **Expandir todos** e **Recolher todos** trabalham na lista inteira, e o contador informa quantos cadastros existem. Com mais de cinco itens, o campo **Filtrar** localiza pelo nome ou arquivo. Um cadastro recém-adicionado já abre sozinha, e o conteúdo fechado sai da tabulação do teclado. Quando a lista está vazia, o botão **Adicionar** fica logo abaixo da explicação.

Somente o proprietário pode vincular arquivos e salvar; moderadores autorizados podem consultar e testar arquivos já cadastrados.

## Responder a uma palavra usando um TXT

A resposta sai no fio de quem falou: na Twitch, presa à mensagem da pessoa; no Discord, citando a mensagem original. Cada regra tem o interruptor **Responder no fio da pessoa** para ligar ou desligar esse comportamento. Regras antigas ganham o fio ligado sozinhas; desligue por regra se preferir a mensagem solta. No Kick e no YouTube não há fio, e o Histórico avisa que a mensagem saiu comum.

Ao vincular um TXT, o BotLive confere as variáveis de todas as linhas na hora e mostra até 20 avisos com o número da linha. Uma variável que não existe no sistema aparece como `{{randomViewr}} não existe — quis dizer {{randomViewer}}?`. O **Conferir linhas** repete a conferência quando quiser. A lista de variáveis válidas está em [Variáveis](VARIAVEIS.md); `R$100` e `$desconhecido` não são variáveis e ficam como texto.

1. Clique em **Adicionar resposta TXT** e escreva a palavra ou expressão, por exemplo `café`.
2. Escolha **Palavra / expressão inteira** para evitar que café combine com cafés, ou **Qualquer trecho da mensagem** para permitir esse caso. Maiúsculas e minúsculas não alteram a busca; acentos são preservados.
3. Clique em **Vincular TXT** e escolha um arquivo UTF-8 com uma resposta por linha. Linhas vazias são ignoradas. Não existe sintaxe especial de comentários.
4. Escolha **Aleatória, sem repetir a última** ou **Em sequência**. Uma única linha pode repetir; a sequência volta ao início ao terminar.
5. Ajuste os intervalos da regra e por pessoa, em minutos. Os padrões são 0,5 e 1 minuto. A regra exige pelo menos um segundo; zero desativa somente o intervalo por pessoa.
6. Use **Conferir linhas** para visualizar as primeiras dez respostas e a quantidade total, sem publicar.
7. Ative a regra e **Ativar respostas TXT**, depois clique em **Salvar respostas**.

### Som junto com a resposta

Cada regra pode tocar um áudio da biblioteca no mesmo instante da resposta. No campo **Som junto com a resposta (opcional)**, escolha um som já importado em **Sons de Live** (ou **Sem som**). O áudio toca pela saída única dos sons do bot (a fonte **Áudio do BotLive** no OBS, tela [Áudio no OBS](13-AUDIO.md)). Use **Testar som** para ouvir pelo mesmo caminho da live e ajuste o **Volume do som** (aparece quando um som está escolhido). Escolher som é opcional: regras sem som respondem em silêncio.

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

## Sons de Live: biblioteca e sons por espectador

A tela **Sons de Live** tem duas partes. A **Biblioteca de sons** é o depósito central de áudios: adicione WAV, MP3 ou OGG (até 5 MiB), ouça para conferir e apague quando não servir mais. Tudo que usa som — comandos, timers, automações, resgates de pontos, alertas e respostas — escolhe daqui. Um som apagado fica sem áudio onde estiver em uso, até você escolher outro. A saída de todos eles é a mesma, a fonte **Áudio do BotLive** no OBS (tela [Áudio no OBS](13-AUDIO.md)).

Abaixo fica o cartão **Sons por espectador**: para cada pessoa, o áudio e quando ele toca.

Clique em **Adicionar pessoa** e preencha:

- **Nome no chat**: nome exato recebido da plataforma, sem depender de maiúsculas ou do @ inicial.
- **Apelido**: como você deseja identificar a pessoa na lista; não substitui a identificação da conta.
- **ID da pessoa (opcional)**: quando informado, a identificação usa esse ID. Recomendado para nomes exibidos iguais. Correspondências por ID têm prioridade sobre nome.
- **Som da pessoa**: escolha um áudio da biblioteca; toca na saída dos sons do bot. **Testar som** ouve antes da live, mesmo com a função desativada.
- **Disparar som**: quando enviar mensagem, quando entrar mesmo sem falar, ou ao entrar ou enviar mensagem.
- **Quando tocar**: uma vez por sessão ou repetir com intervalo, em minutos. O intervalo individual mínimo é 5 minutos; o padrão é um minuto.
- **Volume**: ajuste a intensidade do áudio daquela pessoa.

Ative a pessoa e **Ativar sons por espectador**, depois clique em **Salvar sons por espectador**. Para suspender, desligue o controle geral ou o individual e salve. Remover uma pessoa também exige salvar. Mudanças valem para novos eventos; áudio já em reprodução ou na fila termina normalmente.

**Reiniciar sessão de sons** libera novamente as pessoas deste perfil para tocar de novo, sem mexer nas regras TXT. Há um intervalo geral de 5 segundos entre sons; uma pessoa ignorada nesse intervalo poderá tocar na próxima participação. A sessão reinicia ao fechar o aplicativo. Mensagens barradas pela moderação e mensagens da própria conta do bot não disparam sons. O player toca um som por vez, aceita até cinco solicitações pendentes e interrompe cada áudio após quinze segundos. Falhas de reprodução aparecem no aviso da interface; **Som solicitado** no Histórico confirma o pedido, não comprova áudio na transmissão. Um pedido já emitido consome a primeira participação, mesmo se a reprodução falhar; após corrigir, reinicie a sessão.

### Usar um som em um comando, timer ou automação

Os sons importados na biblioteca também ficam disponíveis em **Comandos**, **Timers** e **Automações**: no campo **Tocar áudio ao disparar**, escolha **Escolher som** para importar ou selecione um som já cadastrado, ajuste o **Volume** e use **Testar som**. O som toca quando o fluxo executa, independentemente de **Ativar sons por espectador**. Veja o passo a passo em [Comandos e automações](03-COMANDOS-E-AUTOMACOES.md).

### Entrada silenciosa na Twitch

1. Ative **Monitorar entradas silenciosas na Twitch** (no cartão Sons por espectador) e escolha **Quando entrar, mesmo sem falar** ou **Ao entrar ou enviar mensagem** para a pessoa.
2. Em **Nome no chat**, informe o login atual da conta Twitch (não o nome decorativo). Mesmo com ID preenchido, esse login é necessário para localizar a entrada; o ID informado continua sendo conferido.
3. Salve. No perfil, autorize novamente a conta do bot para conceder a permissão chat:read, depois conecte o perfil.
4. Confira no Histórico o registro **Monitor de entradas ativo**. Se faltar autorização, o Histórico explica como corrigir; as mensagens continuam pelo conector normal.
5. Teste com a pessoa entrando depois que o monitor estiver ativo. A lista inicial de pessoas já presentes não toca. Reconectar o bot não limpa os sons já consumidos na sessão.

A Twitch informa conexões ao chat por IRC; isso não comprova que alguém está assistindo ao vídeo. Entradas podem atrasar ou não ser informadas, e a ordem dos eventos não é garantida. Para maior previsibilidade, escolha **Quando enviar mensagem**. O monitor é opcional e não envia mensagens pelo IRC. Ao desativar e salvar, ele encerra a observação; uma tentativa em andamento pode demorar até o limite de rede. Veja a [documentação Twitch](https://dev.twitch.tv/docs/chat/irc/).

No YouTube, use mensagem: o conector não fornece presença silenciosa. Uma ponte autorizada pode enviar join com identidade pela API local em outras plataformas. Entradas contam para a mesma sessão de sons; com **Uma vez por sessão**, uma mensagem posterior não toca novamente. Cadastros antigos assumem **Quando enviar mensagem**, sem habilitar observação silenciosa automaticamente.

## Fazer o som sair na live

A saída dos sons é a fonte **Áudio do BotLive** no OBS, configurada na tela [Áudio no OBS](13-AUDIO.md) — não há seletor de aparelho nesta tela nem naquela. Use **Testar som** daqui para confirmar que o áudio segue o mesmo caminho da live.

No OBS, a fonte de navegador **Áudio do BotLive** já é esse caminho: confira o medidor dela e faça uma gravação local para validar volume e roteamento. Evite capturar o mesmo áudio duas vezes (se sobrou uma fonte antiga apontando para o cabo, remova-a). O BotLive não altera o mixer da transmissão.

Há até 200 pessoas e 300 arquivos cadastrados por perfil (TXT e sons somados). Remover regras ou pessoas mantém os arquivos registrados para reutilização; não apaga originais nem cópias do áudio. Arquivos de áudio corrompidos podem ser recusados pelo player mesmo com extensão válida.

## Backup e migração

As configurações ficam no banco. Preserve também a pasta **media** dentro da pasta de dados, que contém as cópias dos áudios. Os TXT ficam nos caminhos externos escolhidos e precisam de cópia separada. Estes recursos não são transportados pelos presets. Ao mudar de computador ou reinstalar o OBS, crie de novo a fonte **Áudio do BotLive** no OBS com o endereço da tela **Áudio no OBS**: a fonte vive na configuração do OBS daquela máquina. Ao mudar a pasta de dados ou de computador, vincule novamente os arquivos para atualizar os caminhos registrados. Apagar um perfil remove seu cadastro do banco, mas conserva suas cópias de áudio no disco; elas não são executadas sem cadastro.

## Resolver problemas

- Nenhuma resposta: confira controles ativados, palavra/acentos, intervalos, perfil e arquivo existente. Uma regra anterior pode ter respondido primeiro.
- Variável ausente: revise o marcador. Variáveis locais de outro fluxo, como local.aiResponse, não estão disponíveis no TXT.
- Som não toca: confira nome recebido ou ID, controle geral/individual, sessão já consumida, volume e avisos da interface. Use Testar som, depois Reiniciar sessão de sons se necessário.
- Som só no computador: ajuste a captura no OBS e confira o medidor. O BotLive não altera o mixer da transmissão.
- Arquivo recusado: TXT exige UTF-8; áudio exige WAV/MP3/OGG. Confira também tamanho e limites indicados acima.
