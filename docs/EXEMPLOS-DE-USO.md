# Exemplos práticos — escolha o que quer fazer

Cada exemplo mostra **onde configurar**, **o que usar** e **o resultado esperado**. Comece com um perfil de teste e uma função por vez. Os resultados abaixo explicam o comportamento configurado; frases geradas por IA podem variar.

## Antes de copiar qualquer exemplo

1. Confira o perfil selecionado no topo do BotLive.
2. Salve a configuração e ative o comando, timer ou módulo correspondente.
3. Para respostas reais, conecte o perfil e teste usando outra conta: o bot ignora as próprias mensagens.
4. Acompanhe **Histórico**. Use simulação para conferir fluxos sem publicar; testes de IA consultam o provedor, e testes de som reproduzem no computador.

## Mensagens, timers e contadores

| Quero fazer | Onde configurar | Exemplo pronto | Resultado esperado |
|---|---|---|---|
| Cumprimentar quem pede | Comandos → Novo comando | !oi → Olá, {{user}}! Bem-vindo ao canal {{channel}}. | Ana escreve !oi e recebe uma saudação com seu nome |
| Compartilhar redes | Comandos → Novo comando | !redes → Nossas redes: coloque aqui seus links públicos. | O chat recebe os links cadastrados |
| Repetir um lembrete | Timers → Novo timer | Nome: Água; intervalo: 900; resposta: Hora de beber água! | Publica aproximadamente a cada 15 minutos, enquanto conectado |
| Citar um espectador no lembrete | Timers → Novo timer | Nome: Minecraft; intervalo: 600; resposta: {{randomViewer\|default:alguém}}, quer jogar com a gente? | Sorteia um nome entre quem já falou no chat; sem ninguém, publica "alguém" |
| Contar mortes | Novo comando → Contar usos deste comando | !mortes → Mortes registradas: {{commandCount}}. | Cada uso autorizado soma um ao total de !mortes |
| Contar vitórias separadamente | Outro comando com contador | !vitorias → Vitórias: {{commandCount}}. | Usa um total independente de !mortes |
| Corrigir o placar | Lista de comandos → número · Ajustar | Novo total: 7 | O próximo uso aceito mostra 8 |
| Evitar spam | Editar comando → intervalos | Global: 10; por pessoa: 60 | No máximo uma execução a cada dez segundos, e uma por minuto para a mesma pessoa |
| Limitar quem altera um contador | Editar comando → Quem pode usar | Moderadores | Só moderadores e streamer disparam o comando |
| Somar a contagem com um som de caixa | Novo comando → Contar usos + Tocar áudio ao disparar | !ifood → O ifood já passou a milhão na rua {{commandCount}} vezes! | Cada uso aceito soma 1 e toca o som escolhido na saída do BotLive |
| Publicar como anúncio colorido | Editar comando → Como enviar na Twitch → Anúncio | Cor do anúncio: Laranja | A mensagem sai como anúncio da Twitch em vez de mensagem comum |

Siga os passos completos em [Comandos e automações](03-COMANDOS-E-AUTOMACOES.md) e [Timers e contadores](TIMERS-E-CONTADORES.md).

## Responder com um arquivo e receber pessoas com sons

### Arquivo de respostas para “café”

**Onde:** Respostas e sons → Adicionar resposta TXT.

Salve um arquivo chamado cafe.txt como UTF-8, com este conteúdo:

```text
{{user}}, pausa para o café!
O café está pronto, {{user}}.
Hoje o combustível da live é café!
```

Configure a palavra **café**, reconhecimento **Palavra / expressão inteira** e escolha **Aleatória, sem repetir a última**. Vincule o arquivo, ative a regra e o controle geral e salve.

**Entrada:** Ana escreve “alguém quer café?”. **Resultado:** uma das linhas é enviada, com o nome de Ana quando houver {{user}}. Editar e salvar o TXT original altera os próximos disparos. “cafés” não combina no modo de palavra inteira.

### Som da Ana ao participar

**Onde:** no mesmo painel → Sons por espectador → Adicionar pessoa.

Nome no chat: login real da Ana. Apelido: Aninha. Escolha um WAV, MP3 ou OGG. Em **Disparar som**, escolha **Quando enviar mensagem**; em **Quando tocar**, **Uma vez por sessão**. Ative a pessoa e o controle geral e salve.

**Entrada:** primeira mensagem da Ana. **Resultado:** o som escolhido é solicitado. Mensagens seguintes não repetem até reiniciar a sessão. **Testar som** ajuda a conferir áudio e volume sem esperar por Ana.

Para entrada silenciosa na Twitch, escolha **Quando entrar, mesmo sem falar**, ative o monitor e autorize novamente a conta do bot. A pessoa deve entrar depois de o monitor estar ativo. Para transmissão, escolha **Dispositivo de saída dos sons** e capture essa saída no OBS. A entrada depende dos eventos de conexão ao chat, não da lista de quem assiste ao vídeo.

Veja limites de arquivos, intervalos, permissões e diagnóstico em [Respostas TXT e sons](RESPOSTAS-E-SONS.md).

## Conversa com IA e memória

### Resenha relacionada à jogada

**Onde:** configure e teste o provedor em Inteligência artificial; depois abra Comandos → Resenha com IA.

Trecho que dispara: **amassando**. Orientação: **Responda em uma frase, com humor leve sobre a jogada mencionada. Use apenas fatos presentes na conversa ou nas memórias.**

**Entrada:** “O Thenees hoje está amassando na play.” **Exemplo possível:** “Hoje a mira acordou inspirada, vamos ver se aguenta até o fim da partida!” A IA deve se relacionar com a mensagem; não é uma resposta fixa garantida.

Use **Gerar resposta de teste** para conferir sem publicar. Esse teste chama o provedor. **Simular evento** apenas mostra um marcador de prévia, sem chamar a IA.

### Usar a mesma resposta no chat e no overlay

Em Automações, conecte **Gerar resposta da IA (variável)** → **Enviar mensagem** → **Atualizar overlay**. Nas duas últimas ações, insira **Resposta da IA** pelo catálogo.

**Resultado:** as duas ações usam o mesmo texto gerado, sem fazer duas consultas ao modelo. Para falar esse texto, acrescente **Ler em voz alta** e ative/configure Texto para voz.

### Dar contexto sobre a live

**Onde:** Memórias → Nova memória. Caminho: **contexto-live/desafio.md**.

```markdown
# Desafio de hoje
Estamos jogando xadrez e tentando vencer três partidas seguidas.
Quando alguém perguntar pelo desafio, explique esse objetivo.
```

Salve e teste uma pergunta sobre o desafio na tela de IA. A busca de notas usa palavras relacionadas; ela não envia todas as memórias em toda chamada.

Para registrar acontecimentos automaticamente, use **Registrar memória** no fluxo, destino **eventos/chegadas.md**, conteúdo **{{user}} participou: {{message}}**. Um evento real acrescenta uma entrada; a simulação não grava.

As instruções de provedor, personalidade, alternativas e Obsidian estão em [IA e memória](04-IA-E-MEMORIA.md).

## Variáveis sem decorar códigos

Clique dentro da resposta e use **+ Nome da pessoa**, **+ Nome do canal** ou **Inserir variável e testar mensagem**. O texto abaixo do editor mostra etiquetas legíveis.

| Exemplo de mensagem | Entrada de teste | Resultado esperado |
|---|---|---|
| Olá, {{user}}! | Pessoa: Ana | Olá, Ana! |
| Seu pedido foi: {{rawInput}} | !pedido jogar com o streamer | Seu pedido foi: jogar com o streamer |
| Olá, {{arg0\|default:amigo}}! | !oi | Olá, amigo! |
| Canal: {{channel\|upper}} | Canal: cactus | Canal: CACTUS |
| Usos deste comando: {{commandCount}} | Comando com contador, terceiro uso aceito | Usos deste comando: 3 |

Para uma variável própria, abra **Comandos → Variáveis**, escolha o escopo salvo no perfil, nome **meta**, valor **10**. Use **{{global.meta}}** numa resposta. Variáveis locais duram só a execução; variáveis por pessoa precisam de ID de usuário. Mais receitas: [Variáveis](VARIAVEIS.md).

## Pontos, loja, sorteios e previsões

### Recompensar participação

**Onde:** Comunidade → ative Pontos & loja → Avançado. Configure cinco pontos por participação e intervalo de sessenta segundos.

**Entrada:** Ana envia uma mensagem elegível. **Resultado:** recebe cinco pontos; outra mensagem logo depois não concede novamente. **!pontos** consulta o saldo e **!ranking** mostra o ranking. Não são pontos por minutos assistidos em silêncio.

**Transferência:** depois de ambas as pessoas participarem, **!transferir ana 10** move dez pontos do remetente para Ana, se houver saldo. O apelido precisa corresponder ao cadastro recebido pelo chat.

### Benefício na loja

No painel da loja, crie **Escolher o próximo desafio**, custo **100**, estoque **2**. Use **!loja** para conferir a posição atual; se for o primeiro item, **!resgatar 1** compra uma unidade.

**Resultado:** saldo cai cem, estoque cai um e o pedido aparece no painel. O streamer entrega o benefício; o bot não altera o jogo automaticamente.

### Sorteio gratuito

Ative Sorteios, abra um sorteio com palavra **!sorteio** e custo **0**. Ana escreve !sorteio. Clique em **Sortear vencedor** quando quiser encerrar.

**Resultado:** Ana participa uma vez. O vencedor aparece no painel. Para bilhetes pagos, configure o custo e use **!ticket 2** para comprar dois. **Cancelar e reembolsar** devolve os custos de um sorteio ainda aberto.

### Prever o resultado da partida

Ative Previsões. Pergunta: **Vamos vencer?** Opções: **Sim, Não**. Ana usa **!bet 1 25** para apostar vinte e cinco pontos em Sim. Ao concluir, selecione **Encerrar: 1. Sim**, se esse foi o resultado.

**Resultado:** o total é distribuído entre vencedores conforme suas apostas. Use **Cancelar e devolver pontos** para anular uma previsão aberta. Desligar o módulo sozinho não devolve valores.

Detalhes e limites: [Comunidade](05-COMUNIDADE.md).

## Fila, música e jogos

| Função | Preparar no painel | Exemplo no chat ou painel | Resultado esperado |
|---|---|---|---|
| Fila para jogar | Ativar Fila para jogar | !entrar, depois !fila | A pessoa entra e consulta a fila; !sair retira a própria entrada |
| Chamar jogador | Abrir a fila | Chamar próximo | Remove a primeira pessoa; combine como anunciar a chamada |
| Pedido de música | Ativar Pedidos de música | !musica seguido de um link HTTPS aceito | Guarda o pedido; não inicia reprodução automática |
| Gerir música | Mesmo módulo | !minhamusica ou !removermusica | Consulta a posição ou remove os pedidos da pessoa |
| Avançar música | Painel de música | Finalizar primeira música | Remove o primeiro pedido; o player externo continua sob seu controle |
| Roleta | Ativar Jogos e Pontos | !roleta 10 | Debita a aposta; uma vitória retorna vinte pontos |
| Duelo | Jogos, Pontos e dois participantes com saldo | !duelo ana 20; Ana usa !aceitar | Resolve o duelo aleatoriamente e concede o total ao vencedor |
| Bingo | Abrir bingo de emotes | !bingo | Gera cartela; termos reconhecidos no chat preenchem as casas |
| Trivia manual | Criar pergunta de trivia | Pergunta: Quanto é 2 + 2?; resposta: 4; recompensa: 50 | A primeira resposta correta recebida ganha cinquenta pontos; anuncie a pergunta |
| Trivia automática | Jogos → Avançado | Linha: Quanto é 2 + 2? \| 4 | Pode publicar durante baixa atividade, conforme silêncio e intervalo configurados |

A moderação de links pode impedir pedidos de música antes de chegarem ao módulo. As regras completas, prazos e permissões estão em [Comunidade](05-COMUNIDADE.md).

## Voz, Discord e moderação

**Texto para voz:** ative o módulo, escolha voz e velocidade e use **Testar voz local**. Crie um comando !fala com ação **Ler em voz alta**, conteúdo **{{rawInput}}**. A mensagem **!fala Boa noite, chat!** solicita essa leitura. Configure limites e permissões; capture o áudio no OBS. A seleção de saída dos sons por espectador não muda o TTS.

**Controle por voz:** configure o servidor local de reconhecimento, ative o módulo e crie um fluxo de **Comando de voz**, trecho **boas-vindas**, ação de chat **Sejam bem-vindos!**. Clique em **Gravar comando**, fale a frase e encerre a captura. O reconhecimento recebido dispara o fluxo. Não há escuta contínua.

**Discord:** guarde o webhook no módulo e use **Enviar ao Discord** numa automação para publicar nesse canal. Para compartilhar um convite no chat, configure o convite no Avançado e use **!discord**. Convite e webhook são campos com funções diferentes.

**Moderação:** configure termos proibidos no perfil e comece com **Só ignorar e registrar**. Uma mensagem de participante comum com termo bloqueado é registrada e não segue para comandos. Isso não apaga a mensagem da plataforma. Para timeout Twitch, escolha essa ação, configure a duração e autorize a conta do canal. Teste com cuidado em um canal de teste: sanções reais alteram a participação da pessoa.

## Discord — bot no servidor

**Conectar o bot:** abra **Discord** em **SEU ESPAÇO**, salve o token do bot, clique em **Descobrir servidor e canais**, escolha o servidor, ligue **Ativar o bot do Discord** e salve. Depois clique em **Conectar**. **Resultado esperado:** o rótulo mostra **Conectado** e os canais e cargos já estão disponíveis nos campos.

**Boas-vindas e contador:** escolha um canal de boas-vindas, escreva Bem-vindo(a) ao servidor, {user}! e ative **Boas-vindas**; informe o ID de um canal criado só para o contador. **Resultado esperado:** quem entra recebe a mensagem e o nome do canal do contador passa a mostrar o total, como membros-1284.

**Moderação em dupla:** ative **Auto-moderação do Discord**, deixe o canal de logs escolhido e peça para alguém escrever um termo da lista bloqueada. **Resultado esperado:** a mensagem é apagada, a ocorrência entra na auditoria com a plataforma `discord` e o botão **Desfazer** aparece quando a operação é reversível.

**Sorteio com reação:** ative **Sorteios liberados**, escolha o canal, preencha prêmio e minutos e clique em **Criar sorteio**. **Resultado esperado:** a mensagem do sorteio aparece no canal e reagir com 🎉 inscreve a pessoa; **Encerrar** sorteia os vencedores.

A tela **Discord** completa os passos, os comandos slash e as limitações estão no capítulo [Discord](09-DISCORD.md).

## Ações avançadas, overlay e integrações

| Ação | Exemplo de configuração | O que observar |
|---|---|---|
| Esperar | 1000 milissegundos entre mensagem e overlay | A próxima etapa começa após cerca de um segundo |
| Atualizar overlay | {{user}} chegou ao canal! | O cliente OBS conectado recebe o texto; é necessário configurar a fonte |
| Ajustar pontos | Valor: 10 | Soma dez pontos à pessoa que disparou, com o módulo ativado |
| Definir variável | Destino: global.meta; texto: 10 | Grava dez como meta do perfil |
| Incrementar variável | Destino: global.acertos; texto: 1 | Soma um; envie {{global.acertos}} numa ação posterior |
| Apagar variável | Destino: global.acertos | Remove o valor; use alternativa ao referenciá-lo depois |
| Chamar webhook | URL HTTPS de uma integração sua; texto: Live iniciada | O destino recebe JSON; use o Histórico para conferir falhas |
| Executar script Rhai | "Olá, " + user + "!" | Devolve uma saudação e a envia ao chat |

Uma automação segue uma única sequência de blocos conectados. Uma condição de ação como **Executar só se a mensagem contiver desafio** pula aquela etapa quando o trecho não aparece; não cria um caminho “senão”.

Para overlay, siga [OBS e API local](API-LOCAL.md): fonte, porta e chave da sessão precisam estar configuradas. Conectar uma fonte de navegador não captura automaticamente os sons tocados no aplicativo. Para enviar eventos de Stream Deck ou de outra integração, use o protocolo documentado; a API local não dá papel de moderador ao evento externo.

## Presets, acesso, backup e manutenção

| Tarefa | Passos resumidos | Como confirmar |
|---|---|---|
| Reutilizar um comando ou timer | Ícone de pacote → Biblioteca de presets → Exportar | Arquivo .botlivepreset salvo; use tipo fluxo para timers |
| Aplicar em outro perfil | Selecione destino → Importar arquivo → revise → Confirmar aplicação | Fluxos entram desativados; confira e ative os desejados |
| Mudar aparência | Configurações → Claro/Escuro e Cor de destaque | O painel muda; não muda a aparência do overlay |
| Proteger o painel | Configurações → crie primeiro a senha de owner → Bloquear painel | A tela pede login; automações conectadas continuam no motor |
| Permitir edição por moderador | Cadastre acesso local e inclua esse nome nos editores do perfil | O moderador local só acessa perfis autorizados |
| Fazer backup | Feche o BotLive e copie pasta de dados, memórias e arquivos TXT externos | A cópia inclui banco e pasta media; credenciais do cofre exigem cuidado separado |
| Atualizar | Configurações → Verificar atualização → leia notas → Instalar | Após concluir e reabrir, confira a versão completa na lateral |
| Investigar falha | Histórico → busque nome do comando ou erro | Diferencie início, intervalo, erro e envio confirmado |
| Acompanhar atividade | Estatísticas no perfil correto | Dados registrados pelo bot; não representam audiência total da plataforma |

Presets não transportam totais dos contadores, saldos ou todo o estado da live. Backup e preset não são equivalentes. Consulte [Presets](06-PRESETS-E-APARENCIA.md), [Acesso e backup](07-OPERACAO-E-BACKUP.md), [Atualizações](ATUALIZACOES.md) e [Solução de problemas](08-SOLUCAO-DE-PROBLEMAS.md).

## Checklist de uma live de exemplo

1. Abra o perfil certo e conecte o canal.
2. Teste !oi usando outra conta.
3. Confira !mortes e !vitorias; zere somente se quiser começar novos totais.
4. Ative os timers Água (900 segundos) e Comunidade (600 segundos).
5. Teste o som de uma pessoa e confira o medidor no OBS.
6. Confira a personalidade da IA e a nota de contexto do dia.
7. Durante a live, acompanhe Histórico, fila e operações de comunidade.
8. Ao terminar, resolva ou cancele operações abertas, pause timers ou desconecte e faça seu backup.
