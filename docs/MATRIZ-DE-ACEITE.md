# Matriz de aceite

Esta matriz acompanha as fases dos documentos originais em ../documentação. "Implementado" descreve código existente; não equivale à homologação com serviços e máquinas externas.

| Ordem | Área | Entregue no código | Aceite ainda necessário |
|---|---|---|---|
| 1 | Motor de eventos | Fila assíncrona, gatilhos, permissões, sequência de ações, variáveis, cooldowns, histórico, simulação e API WebSocket | Receber todos os eventos em canal Twitch real e testar carga prolongada |
| 2 | Perfis e contas | Dados e memória isolados; OAuth bot/canal; credenciais no cofre; proprietário e moderadores locais | Autorizar contas reais, revogar tokens e validar renovação e permissões nas plataformas |
| 3 | IA | Ollama, Chat Completions e Anthropic; personalidade; fallback; filtros e busca de memória | Homologar os modelos escolhidos e o desempenho na máquina de uso |
| 4 | Memória | Editor Markdown, pastas por perfil, busca lexical e sincronização manual para Obsidian | Testar com o plugin e vault reais; busca semântica vetorial não implementada |
| 5 | Interface | Dashboard, roteiro de configuração, modo simples, editor de nós, temas claro/escuro, atalhos e adaptação à janela | Teste de primeiro uso com pessoa externa e acessibilidade completa com leitor de tela |
| 6 | Presets | JSON versionado, biblioteca, filtros, prévia, conflitos, dados de perfil sem contas, configurações de módulos, tema claro/escuro e cor personalizada | Importação entre dois computadores |
| 7 | Distribuição | Configuração Tauri, pipeline Windows/macOS/Linux, atualizador com verificação de assinatura | Executável e instaladores Windows gerados; faltam teste em máquina limpa, builds macOS/Linux, certificados e feed publicado |
| 8 | Discord nativo | Gateway v10 e API REST sem terceiros; token no cofre; servidor, canais e cargos; entrada, saída, autorole e contador de membros; espelho de chat; notificações da Twitch; auto-moderação com as regras da Twitch; auditoria única com desfazer e exportação; sorteios, XP, aniversários, vínculo de identidade e comandos slash | Conectar um bot real a um servidor Discord, conferir intents privilegiadas, permissões de cargo e limites de requisição |

## Módulos adicionais

- Pontos por participação, multiplicador para assinantes, nome da moeda, loja, ranking e transferências. Contagem de tempo de espectadores silenciosos e multiplicadores específicos por evento não implementados.
- Sorteio por palavra-chave ou bilhetes, peso de assinantes e reembolso ao cancelar. Modalidade "primeiro a digitar" não implementada.
- Palpites com divisão proporcional e devolução sem vencedores.
- Fila de músicas, limites configuráveis, consulta e remoção; overlay OBS mostra seleção atual e próximas URLs. Reprodução, metadados e controle dos players não integrados.
- Roleta, duelo aleatório, bingo de emotes e trivia manual ou automática por inatividade. Duelo por votação não implementado.
- TTS do sistema, voz local via Whisper, Discord por webhook e convite, moderação Twitch, fila de interação e estatísticas locais. Motores TTS remotos não implementados. A moderação nativa do Discord (aviso, timeout, expulsão, banimento, apagar mensagem, cargo, modo lento e auto-moderação) está implementada pela tela **Discord**; a moderação nativa de Kick e YouTube continua pendente.

## Dependências externas

Kick precisa de ponte pública de webhooks; ela não é hospedada pelo aplicativo. Twitch e YouTube precisam de aplicativos OAuth registrados. IA remota requer chave e modelo. A voz local depende de servidor Whisper. Atualizações públicas exigem hospedagem e assinatura próprias.

Não considerar a especificação inteira homologada enquanto os itens pendentes desta matriz não forem resolvidos.

## Variáveis

Implementados catálogo, prévia, contexto por execução, argumentos de comando, campos JSON do evento, filtros, variáveis persistentes por perfil/pessoa, variáveis de sessão e incrementos atômicos. Testes cobrem isolamento, concorrência, expansão sem recursão e simulação sem gravação. Não há compatibilidade integral com ações C#, funções inline ou todo o catálogo de eventos do Streamer.bot. Consulte VARIAVEIS.md.

## Funções do Streamer.bot com uso simplificado

Diretriz solicitada: ampliar as funções do BotLive usando seleção visual, nomes compreensíveis, configurações guiadas e prévia. A tabela é um mapa de evolução, não uma declaração de paridade concluída. A referência externa é o [catálogo oficial de ações do Streamer.bot](https://docs.streamer.bot/api/sub-actions).

| Área de referência | Situação no BotLive | Forma simples adotada ou proposta |
|---|---|---|
| Argumentos e variáveis | Implementado, com diferenças de sintaxe e catálogo | Botões por nome, categorias, inserção no cursor, etiquetas e escolha de formato |
| Variáveis persistentes e por pessoa | Implementado | Escolher onde guardar, nome e tipo de valor |
| Inspeção de valores | Prévia implementada; sem inspector completo de execuções passadas | Testar mensagem e expandir detalhes quando necessário |
| Condicionais e laços | Apenas condição por trecho de mensagem | Pendente: condições por variável, Se/Senão e repetição limitada |
| Chamar outra ação e controlar filas | Fila interna implementada; gestão visual e chamada entre fluxos pendentes | Proposto: selecionar automação e política de fila |
| Formatação, cálculos e sorteio de números | Filtros básicos e contadores implementados; expressões e aleatórios genéricos pendentes | Proposto: controles numéricos e operações selecionáveis |
| Arquivos e HTTP | Vault e POST de webhook; leitura genérica e consulta com captura de resposta pendentes | Proposto: seleção de arquivo e requisição guiada |
| OBS | Overlay local implementado; controle de cenas e fontes pendente | Proposto: escolher cena/fonte pelo nome |
| Temporizadores e atalhos | Trivia automática e atalhos do painel; disparadores gerais pendentes | Proposto: frequência ou combinação de teclas |
| Som e voz | TTS do sistema e Whisper; player de sons genérico pendente | Proposto: selecionar áudio ou voz |
| Plataformas de live | Adaptadores parciais; não cobrem todas as ações e eventos da referência | Proposto: seletores específicos após conectar a conta |
| Scripts e integrações de terceiros | Rhai limitado; sem C# ou cobertura geral de plugins | Proposto: receitas visuais para tarefas frequentes |

Ordem proposta das próximas ampliações: condições por variável; chamadas entre automações e filas; temporizadores; OBS; mais ações e eventos das plataformas; arquivos/HTTP; integrações adicionais. Cada entrega deve trazer interface simples, exemplo no manual e validação própria. Os itens pendentes não foram implementados nesta revisão de usabilidade das variáveis.
