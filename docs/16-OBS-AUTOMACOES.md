# Receitas e controles do OBS

## Começar pelo resultado

Em Comandos ou Automações, abra Receitas prontas e procure OBS. Escolha Comando atualiza texto no OBS, Resgate mostra uma imagem ou Resgate ativa um filtro no OBS. Escolher não salva nem executa: a receita abre desativada para revisão.

O formulário reúne três passos: quando acontece, o que aparece no OBS e a confirmação no chat. Você configura a mesma automação utilizada pelo editor completo. Ao reabrir a receita, o BotLive encontra a configuração existente em vez de criar outra. Se você personalizar o desenho ou acrescentar etapas no editor completo, continuará editando por lá para manter todos os passos visíveis.

## Conectar e preparar as fontes

Abra OBS Studio no BotLive, ative a integração e salve endereço, porta e senha do WebSocket. No OBS, o WebSocket fica em Ferramentas. Testar conexão mostra a cena e as versões. Consulte [OBS e API local](API-LOCAL.md) para a configuração existente.

No OBS, em Fontes → +, crie uma fonte Texto (GDI+ ou FreeType), Imagem ou Navegador conforme o resultado desejado. Crie os filtros nas propriedades da fonte. O BotLive lista as fontes existentes e só mostra tipos compatíveis nas operações de conteúdo; ele não cria ou apaga essas fontes.

Clique em Atualizar cenas e fontes se fizer mudanças no OBS com o painel aberto. Se a conexão falhar, os nomes salvos continuam preservados e a tela mostra o problema. Uma lista vazia não significa que a configuração anterior foi apagada.

## Texto enviado pelo chat

1. Escolha Comando atualiza texto no OBS.
2. Confira o comando, inicialmente !tela, e a permissão, inicialmente moderadores e streamer.
3. Escolha onde está a fonte e selecione a fonte de texto.
4. Mantenha Texto depois do comando como conteúdo, ou personalize a mensagem pelo catálogo de informações.
5. Ajuste a confirmação no chat e salve. Ative a automação na lista quando estiver pronta.
6. No chat, envie !tela Boa noite, comunidade! O conteúdo da fonte será atualizado; depois da confirmação do OBS, o bot enviará a mensagem de confirmação no chat.

A receita inicial ignora pedidos sem texto depois do comando. Atualizar texto preserva fonte, cor, tamanho e os demais ajustes; a fonte passa a usar texto direto, desativando a leitura de arquivo. Testar automação ensaia a sequência sem alterar o OBS. Testar efeito no OBS faz uma alteração real na fonte sem salvar nem enviar confirmação no chat. Quando o conteúdo tem variáveis, informe um conteúdo de exemplo nesse painel: ele substitui o texto apenas no teste e não muda a receita.

## Imagem por recompensa

Escolha Resgate mostra uma imagem, selecione uma recompensa específica da Twitch e a fonte Imagem existente no OBS. O modelo mostra essa fonte durante dez segundos e restaura a visibilidade anterior; se já estava visível, continuará visível. Ajuste o tempo e a mensagem de agradecimento, salve e depois ative na lista.

Essa receita usa a imagem já configurada na fonte. Para trocar seu arquivo, use a operação Trocar imagem no painel OBS Studio ou no editor completo. Clique em Escolher imagem e selecione PNG, JPG, GIF, BMP ou WebP. O arquivo permanece no computador, sem upload. A operação confere caminho absoluto, existência e extensão; a decodificação final é responsabilidade do OBS. O caminho precisa ser acessível ao OBS no destino configurado.

## Filtro por recompensa

Escolha Resgate ativa um filtro no OBS, selecione uma recompensa específica, a fonte ou cena e o filtro existente. Use Ativar, Desativar ou Alternar. O filtro conserva seus parâmetros. A seleção fica preservada ao salvar e reabrir.

Essas operações são permanentes: não têm restauração temporária automática nesta etapa. O formulário informa isso; para uma sequência diferente, abra Editar etapas no editor completo. A confirmação só ocorre após o OBS aceitar a operação. Falha no OBS interrompe os passos seguintes, em vez de enviar uma confirmação de sucesso.

## Cena atual, cena fixa e grupos

Mostrar, esconder, girar e espelhar atuam no item da cena ou grupo selecionado. Cena atual da transmissão acompanha a cena de programa no momento do disparo. Uma cena ou grupo escolhido pelo nome fixa o destino, mesmo se a live trocar de cena.

Para um item dentro de um grupo, escolha esse grupo em Onde está a fonte e selecione o item. A lista mostra os itens diretos do contêiner escolhido; não achata grupos e cenas aninhadas em uma única lista. Se um mesmo nome de fonte aparecer repetidamente no mesmo contêiner, a seleção por nome conserva o comportamento anterior de usar a primeira ocorrência.

Efeitos temporários conservam o destino e o estado anterior para a restauração, inclusive em grupos. Ações que disputem o mesmo item durante o efeito são recusadas. Uma mudança manual de estado no OBS continua sendo respeitada pelo mecanismo de restauração existente.

Mudar conteúdo ou filtro afeta a fonte compartilhada: todas as cenas que reutilizam essa fonte verão o resultado. Esse comportamento é diferente da visibilidade ou transformação de um item específico.

## Fonte de navegador

Selecione Atualizar endereço de navegador, escolha uma fonte Navegador e informe a URL completa com http:// ou https://. A operação preserva largura, altura, CSS e os demais ajustes, passando do modo arquivo local para endereço. Endereços com usuário ou senha embutidos, javascript e file são recusados.

Texto e Imagem são fontes nativas do OBS: você não precisa copiar uma URL do BotLive para controlá-las. Os overlays do BotLive, como Alertas, Chat e Batimentos, continuam oferecendo os botões de copiar fonte em seus próprios painéis.

## Referência e compatibilidade

As operações internas obs.execute continuam aceitando profileId, op, target, num e secs. Os campos opcionais scene, filter e content ampliam a chamada. Sem scene, a visibilidade e transformação mantêm a cena atual, como antes. Nenhuma operação nova habilita chamadas arbitrárias ao OBS.

| Operação | Campos específicos | Resultado |
|---|---|---|
| filter_enable / filter_disable / filter_toggle | target e filter | altera estado do filtro existente |
| set_text | target e content | atualiza GDI+/FreeType em modo texto direto |
| set_image | target e content | troca arquivo da fonte Imagem |
| set_browser_url | target e content | troca URL da fonte Navegador |
| obs.targets | profileId e scene opcional | lista cenas, grupos, entradas e itens do contêiner |
| obs.filters | profileId e target | lista nomes e estados dos filtros |

Nos arquivos de automação, obsScene e obsFilter são opcionais e vazios por padrão; o conteúdo usa text, com as variáveis preenchidas na execução. Os arquivos antigos não precisam ser reescritos. O motor continua usando a mesma fila, cancelamento, intervalo e tratamento de erro. O ensaio mantém as ações OBS bloqueadas.

Foram acrescentados testes de protocolo e interface. Não há homologação visual na instalação física do OBS do proprietário nesta etapa. A confirmação de protocolo significa que o OBS aceitou o pedido; confira na cena real se fonte, carregamento da imagem/página e posição produzem o resultado desejado.

Referências primárias: [operações OBS do Streamer.bot](https://docs.streamer.bot/api/sub-actions/obs-studio) e [protocolo OBS WebSocket](https://github.com/obsproject/obs-websocket/blob/master/docs/generated/protocol.md).
