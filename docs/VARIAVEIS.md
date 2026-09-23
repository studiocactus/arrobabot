# Variáveis do BotLive

## Comece aqui

Uma variável troca um marcador pelo valor daquela execução. Em **Comandos → Novo comando**, escreva `Olá, {{user}}! Você pediu: {{rawInput}}`. Se Ana enviar `!oi novidades`, a resposta será `Olá, Ana! Você pediu: novidades`.

Você não precisa digitar os marcadores. Os botões **+ Nome da pessoa**, **+ Nome do canal** e **+ Texto do pedido** inserem essas informações na posição do cursor. Se houver um trecho selecionado, o botão substitui somente aquele trecho.

## Montar uma mensagem sem decorar códigos

1. Escreva a parte fixa da mensagem, como Olá, !.
2. Clique entre a vírgula e a exclamação.
3. Clique em **+ Nome da pessoa**.
4. Confira a leitura logo abaixo: o nome técnico aparece como uma etiqueta **Nome da pessoa**, dentro da mensagem.
5. Para outras informações, abra **Inserir variável e testar mensagem**.
6. Busque por uma descrição ou escolha uma categoria: Pessoa, Mensagem, Canal, Data e hora, Execução, Minhas variáveis ou Neste fluxo.
7. Clique na informação desejada. Em **Se não houver valor, mostrar**, informe uma alternativa opcional, como amigo.
8. Em **Como mostrar**, escolha texto normal, maiúsculas, minúsculas, número ou outra opção disponível.
9. Clique em **Inserir na mensagem**. O aplicativo monta a sintaxe automaticamente e devolve o cursor ao texto.

A leitura com etiquetas serve para entender a composição; não é o resultado de uma execução. O campo de edição conserva os marcadores para manter a compatibilidade e permitir ajustes manuais. No catálogo, **Mostrar códigos técnicos** é opcional e começa desmarcado.

**Minhas variáveis** carrega os valores cadastrados no perfil no aplicativo desktop. **Atualizar minhas variáveis** refaz a consulta. Os nomes por pessoa correspondem ao ID informado nos dados avançados do teste. **Neste fluxo** mostra os destinos definidos ou incrementados pelas ações; a definição deve ocorrer antes do uso. Para um nome ainda não listado, abra **Usar uma variável personalizada**, escolha onde o valor está guardado e informe apenas seu nome. Campos de eventos usam Dados do evento e um caminho como reward.title.

## Conferir o resultado antes da live

Abra **Testar como a mensagem vai ficar**, informe mensagem e nome de teste e clique em **Conferir variáveis** no aplicativo desktop. O resultado mostra o texto calculado e os valores disponíveis. **Dados avançados do teste** reúne ID da pessoa e JSON do evento, para quem precisa desses detalhes. No editor visual, a prévia percorre a sequência conectada e calcula as ações de variável sem executar serviços ou gravar mudanças.

## Sintaxe e compatibilidade

O formato recomendado é `{{nome}}`. Os marcadores antigos `$user`, `$message` e `$channel` continuam funcionando. Também é possível usar `$userId`, `$arg0` e outros nomes simples disponíveis. O formato `%userName%` aceita nomes simples como alternativa familiar a quem usa Streamer.bot.

Nomes diferenciam maiúsculas e minúsculas: `userId` não é `userid`. `$userId` é um marcador inteiro, nunca uma substituição parcial de `$user`. Variáveis ausentes em `{{...}}` e `%...%` interrompem a ação com uma mensagem no Histórico; use `{{arg0|default:amigo}}` para valores opcionais. Um `$nome` desconhecido é mantido literalmente para preservar textos antigos.

Para escrever marcadores literalmente, acrescente uma barra invertida: `\{{user}}`, `\$user` ou `\%userName%`. O valor recebido do chat nunca é interpretado novamente como um modelo: se alguém enviar `{{global.contador}}`, o uso de `{{message}}` mostrará aquele texto, sem ler a variável interna.

## Catálogo de contexto

| Marcador | Conteúdo |
|---|---|
| {{user}} / {{userName}} | Nome exibido de quem disparou |
| {{userId}} | ID da pessoa informado pelo evento |
| {{role}} | everyone, subscriber, moderator ou broadcaster |
| {{isModerator}} | true para moderador ou streamer |
| {{isBroadcaster}} | true para streamer |
| {{isSubscriber}} | true somente quando o papel informado é subscriber; não é consulta à assinatura |
| {{message}} | Mensagem completa |
| {{command}} | Primeira palavra da mensagem |
| {{rawInput}} | Texto após a primeira palavra, preservando espaços internos |
| {{arg0}}, {{arg1}}, ... | Argumentos após a primeira palavra, começando em zero |
| {{argCount}} | Quantidade de argumentos |
| {{args}} | Lista dos argumentos em JSON |
| {{channel}} / {{channelId}} | Canal e ID cadastrados |
| {{platform}} | twitch, youtube ou kick |
| {{profileName}} / {{profileId}} | Nome e UUID do perfil |
| {{eventType}} / {{eventId}} | Tipo e ID do evento |
| {{actionName}} / {{actionId}} | Nome e ID do fluxo; disponíveis durante um fluxo |
| {{simulated}} | true em simulação e prévia |
| {{date}} | Data local no formato AAAA-MM-DD |
| {{time}} | Hora local HH:MM:SS |
| {{unixtime}} | Instante Unix em segundos |
| {{lf}} | Quebra de linha |

Data e hora são capturadas no início de cada fluxo. Os argumentos são separados por espaços em branco: aspas não agrupam várias palavras. Para eventos sem texto, command/rawInput ficam vazios e argCount é zero. Nem todo evento possui usuário; um ID vazio impede gravar variáveis por pessoa.

## Dados específicos do evento

Use `{{data.caminho}}` para acessar o objeto data recebido do adaptador ou da ponte externa. Exemplo: com `{"reward":{"title":"Hidratar"},"items":[{"amount":10}]}`, `{{data.reward.title}}` resulta em Hidratar e `{{data.items.0.amount}}` resulta em 10.

Os campos dependem do evento real e do adaptador. Não há garantia de que reward, amount ou qualquer outro campo exista em todas as plataformas. Teste usando **Dados do evento (JSON)** e use um padrão, como `{{data.reward.title|default:Resgate}}`. Campos de data não sobrescrevem user, role ou outros valores do contexto. Pontos em nomes de chaves não têm sintaxe de escape nesta versão.

## Escopos e duração

| Escopo | Exemplo | Duração e isolamento |
|---|---|---|
| local | {{local.resposta}} | Somente a execução atual do fluxo |
| global | {{global.meta}} | Persistente e compartilhado pelos fluxos do mesmo perfil |
| user | {{user.visitas}} | Persistente por perfil, plataforma e ID da pessoa |
| session | {{session.contador}} | Compartilhado no perfil até fechar o núcleo do aplicativo |
| sessionUser | {{sessionUser.visitas}} | Por perfil, plataforma e pessoa até fechar o núcleo |

Global significa compartilhado dentro do perfil, não entre todos os bots. Alterar o nome exibido da pessoa não muda seu registro; mudar o ID ou a plataforma muda o conjunto de variáveis por pessoa. Fechar só um painel não encerra a sessão do núcleo.

As variáveis compartilhadas são lidas no início de cada fluxo. Mudanças feitas pelo próprio fluxo ficam disponíveis nas próximas ações. Outro fluxo em execução pode ter uma leitura anterior. Para contadores concorrentes, sempre use **Incrementar variável**, que lê e grava atomicamente o valor atual; não faça uma leitura seguida de Definir com `add:1`.

## Criar, editar e apagar valores

Abra **Comandos → Variáveis** ou **Automações → Variáveis**. Escolha o escopo, informe um nome como contador e o valor. Para escopos por pessoa, informe o ID da plataforma, não o nome exibido. Clique em **Carregar variáveis** depois de alterar o ID. A lista mostra as variáveis do perfil e as daquela pessoa.

Nomes aceitam letras ASCII, números e sublinhado, com até 64 caracteres; não podem começar por número. São válidos contador, meta_live e visitas2026. Não são válidos minha meta, 1contador e nome.com.ponto.

Escolha **Tipo de valor**: Texto, Número / contador, Sim ou não, ou Lista ou objeto (JSON avançado). Texto mantém exatamente o que você digitou, inclusive números escritos como texto. Número / contador é a escolha para incrementar valores. Sim ou não usa um seletor; listas e objetos exigem JSON válido. Não guarde senhas ou tokens: esses dados não vão para o cofre e podem aparecer no chat, nas prévias ou no backup.

Editar preenche os campos para uma nova gravação. Apagar pede confirmação e não oferece lixeira. Variáveis locais são criadas pelas ações do fluxo e não aparecem no gerenciador persistente.

## Ações de variável

No editor visual, escolha **Definir variável**, **Incrementar variável** ou **Apagar variável**. Em **Guardar o valor**, escolha a duração e o compartilhamento; em **Nome do valor**, digite apenas contador, por exemplo. Para guardar global.contador, selecione Salva no perfil e escreva contador. O destino é um nome fixo; não recebe interpolação.

No conteúdo das ações Definir/Incrementar, a interpretação continua sendo texto ou JSON: 10 é número, true é booleano e texto comum é uma string. Para preservar um número como texto nesse campo, escreva-o entre aspas. A seleção explícita de tipo fica no gerenciador **Variáveis do perfil**.

| Ação | Conteúdo | Resultado |
|---|---|---|
| Definir variável | Texto, JSON ou modelo com variáveis | Cria ou substitui o valor |
| Incrementar variável | Número positivo ou negativo | Soma atomicamente; começa em zero se ainda não existir |
| Apagar variável | Ignorado | Remove o valor; apagar um nome ausente é permitido |

Texto numérico entre aspas é texto e não pode ser incrementado. Resultados fora do limite numérico produzem erro sem alterar o valor. Cada ação concluída é independente: uma falha posterior não desfaz incrementos anteriores.

**Gerar resposta da IA (variável)** disponibiliza `{{local.aiResponse}}` sem publicar. Na próxima ação, clique em **+ Resposta da IA** para usar o texto em uma mensagem, voz ou overlay. **Responder com IA** também disponibiliza o valor, mas já o envia ao chat. O catálogo tem a categoria **IA**, com **Resposta da IA** e **IA respondeu com sucesso?** (`{{local.aiSuccess}}`).

O conteúdo é gerado a partir da mensagem atual, conversa recente do perfil, personalidade, orientação da ação e memórias recuperadas. Cada execução tem sua própria resposta. Um nome personalizado como `resenha` cria também `{{local.resenha}}`. A geração deve acontecer antes de usar o valor; inserir o marcador sozinho não chama a IA. Marcadores dentro da resposta gerada são texto literal, sem segunda interpretação.

Na falha do provedor, a resposta alternativa configurada ocupa a variável e aiSuccess fica false. Na prévia e simulação, nenhum provedor é chamado: o texto **[Prévia: resposta contextual da IA]** ocupa o valor e aiSuccess fica false. Para avaliar uma resposta real sem publicar, use **Testar resposta contextual**. Consulte a receita **Resenha com IA** no [capítulo de inteligência artificial](04-IA-E-MEMORIA.md).

## Filtros

Separe filtros com a barra vertical dentro do marcador. Eles rodam da esquerda para a direita. Exemplo: `{{arg0|default:amigo|upper}}`.

| Filtro | Exemplo | Efeito |
|---|---|---|
| default:texto | {{arg0\|default:amigo}} | Substitui ausente, null ou string vazia; preserva false e zero |
| upper / lower | {{user\|upper}} | Converte maiúsculas/minúsculas |
| trim | {{rawInput\|trim}} | Remove espaços nas extremidades |
| length | {{rawInput\|length}} | Conta caracteres Unicode; em arrays/objetos conta itens/chaves |
| number:N | {{arg0\|number:2}} | Formata de zero a seis casas, com ponto decimal |
| add:N | {{arg0\|add:5}} | Soma um número; não grava contador |
| multiply:N | {{arg0\|multiply:2}} | Multiplica; não grava o valor |
| url | {{rawInput\|url}} | Codifica um componente de URL |
| json | {{user\|json}} | Serializa como JSON, incluindo aspas para texto |

A tabela usa barras verticais na sintaxe dos marcadores. Não há escape para usar uma barra vertical dentro do texto de default. Não há avaliação arbitrária de código nos filtros, nem suporte à sintaxe C# de formatação ou às funções inline do Streamer.bot.

## Receita: contador de chegadas por pessoa

Crie um fluxo com comando !cheguei e conecte:

1. Incrementar variável: Guardar o valor → Salva por pessoa; Nome do valor → visitas; Quanto somar → 1.
2. Incrementar variável: Guardar o valor → Salva no perfil; Nome do valor → chegadas; Quanto somar → 1.
3. Enviar mensagem: `Olá, {{user}}! Esta é sua visita {{user.visitas}}. O canal registrou {{global.chegadas}} chegadas.`

Configure os intervalos para evitar repetições. Abra a prévia e informe sempre o mesmo ID para conferir a leitura de uma pessoa. Na simulação, os incrementos são calculados somente dentro da execução; repetir uma simulação não aumenta o saldo real. Um disparo real grava os contadores.

## Receita: resposta personalizada

Crie um comando !saudar com resposta `Olá, {{arg0|default:comunidade|upper}}! Mensagem de {{user}} no canal {{channel}}.`. Para !saudar Ana, o primeiro argumento é Ana. Para !saudar sem argumentos, o padrão é comunidade.

Para uma mensagem longa após o comando, use rawInput em vez de arg0. Exemplo: `Você sugeriu: {{rawInput|default:nenhuma sugestão}}`.

## Onde os modelos funcionam

Modelos são resolvidos no conteúdo das ações de chat, IA, memória, overlay, webhook, Discord, TTS e definição/incremento de variáveis. O caminho da ação de memória também aceita modelos, sujeito às restrições do vault. O campo URL de webhook, destinos de variáveis, condição “mensagem contém” e campos numéricos não são interpolados.

Scripts Rhai mantêm seu próprio contexto user, message e channel e não interpolam modelos no código. A biblioteca de presets transporta as ações e seus nomes de variável, mas não os valores globais ou por pessoa: inicialize-os no perfil de destino ou inclua passos apropriados de inicialização. Não redefina um contador a zero a cada execução.

## Limites, backup e diagnóstico

Um modelo ou resultado aceita até 64 KiB; um valor armazenado aceita até 16000 bytes de JSON. O conteúdo das ações continua limitado a 32768 bytes. São permitidos até 10000 registros persistentes e 10000 de sessão por perfil, somando os usuários. A expansão é de uma passagem, sem recursão.

Variáveis persistentes ficam na tabela variables do SQLite e acompanham o backup do banco. As de sessão usam tabela temporária e não são restauradas. Excluir um perfil remove seus valores. Os acessos locais existentes protegem listagem, edição e prévia por perfil.

Se ocorrer “Variável ausente”, confira o nome, a ordem dos blocos, o escopo, o ID e a presença dos campos no evento. Se ocorrer “contador precisa ser numérico”, edite o valor para um número JSON. Prévia não valida OAuth, limites da plataforma nem efeitos externos; use-a para verificar o conteúdo calculado.

## Referência de comparação

O Streamer.bot documenta argumentos locais, variáveis globais e por usuário, inspeção e funções inline em sua [documentação oficial de variáveis](https://docs.streamer.bot/guide/core/variables). O BotLive adota escopos explícitos, catálogo no editor, prévia sem gravação e expansão sem reinterpretação do texto recebido. Isso não representa compatibilidade integral nem superioridade geral: não importa ações C#, não implementa todas as funções inline e não oferece todos os campos de todas as integrações do Streamer.bot.

A direção do projeto é oferecer funções equivalentes por controles simples, descrições em português e exemplos. A [matriz de aceite](MATRIZ-DE-ACEITE.md) registra a comparação por área e a ordem proposta para as funções que ainda faltam.
