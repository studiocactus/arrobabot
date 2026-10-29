# Estudo da Visão geral do BotLive

Data: 29/10/2026. Base: v0.20.17, commit a3093e4. Escopo: diagnóstico e proposta, sem alterar o aplicativo ou publicar versão.

## Conclusão

A Visão geral deve responder, nesta ordem: estou ao vivo? Quantas pessoas estão assistindo? O bot e os serviços estão funcionando? O que exige minha atenção? O que aconteceu e o que preciso fazer agora? Hoje a tela prioriza configuração, inventário e registros, deixando a operação abaixo da primeira dobra. A mudança necessária envolve hierarquia e confiabilidade dos dados, além da aparência.

## Método e limites

Leitura do código frontend/backend, execução do cenário existente da Central no navegador em npm run dev, capturas de 1100 e 1600 pixels e inspeção visual da captura larga. O cenário passou. As capturas usam mocks identificados; seus números, falhas e diferenças de estado não descrevem a live do usuário. Não houve conexão a Twitch/OBS reais, pesquisa com usuários ou medição de tempo de tarefa. As conclusões de usabilidade são avaliação heurística, a validar em uso. Documentação oficial de Twitch e OBS e referência de divulgação progressiva consultadas em 29/10/2026.

## Diagnóstico com evidência

| Prioridade | Constatação | Evidência atual | Consequência |
|---|---|---|---|
| Alta | Configuração ocupa o lugar da operação | App.tsx monta SetupHub antes das métricas, ChatNow, pré-voo e Central | O streamer precisa rolar para investigar problemas durante a live |
| Alta | Conexão do bot e transmissão não são estados distintos na apresentação geral | ChatNow recebe statuses[perfil] === online; LiveConsole chama esse estado de Canal | Pode haver bot conectado com live fora do ar; a conexão também não prova todas as assinaturas de eventos funcionando |
| Alta | Não há contador de audiência real na Visão geral | ChatPresence usa labels.viewers; labels.rs consulta chat/chatters | Pessoas no chat e espectadores são métricas diferentes |
| Alta | A métrica Eventos registrados conta apenas linhas kind=chat | Expressão em App.tsx filtra logs e conta chat, sem janela de sessão | Pode incluir mensagens de saída e outros registros; não mede total de eventos nem mensagens recebidas da live |
| Alta | Contadores não têm base temporal uniforme | Snapshot traz logs retidos; eventos novos limitam a lista no cliente a 300; stats usa histórico local até 10 mil | Quantidades mudam com retenção e reinício, sem representar a sessão de transmissão |
| Alta | Falha de API pode parecer zero no chat | labels::chatters reconhece status JSON 401/403, mas outros retornos sem total/data usam zero/lista vazia | Pelo código, 429/500 com JSON de erro podem aparentar chat vazio; hipótese técnica a cobrir com teste |
| Média | Vários blocos disputam a mesma prioridade | SetupHub, quatro métricas, ChatNow, pré-voo, Seus bots, Central, atividade | Mais informação não resulta em melhor leitura rápida |
| Média | Mistura de escopo global e perfil | Canais conectados soma todos; automações e logs seguem perfil; Voicemod é serviço da máquina; OBS segue destino | Pode atribuir ao canal atual uma condição de outro escopo |
| Média | Falhas recentes não têm janela de tempo explícita | liveConsole.get conta FAILED/INTERRUPTED do histórico disponível, sem filtro por sessão/data | Uma falha antiga pode parecer pendência atual |
| Média | Indicadores de saúde são parciais | OBS health conta temporários/restauração; áudio conta clientes | Zero pendências não prova OBS conectado; fonte conectada não comprova áudio transmitido |
| Média | Há leituras independentes de presença | ChatPresence e ChatNow usam a mesma lógica em instâncias distintas; endpoint faz consulta direta | Pode duplicar chamadas e produzir leituras de momentos diferentes |
| Média | Estado de IA é configuração, não disponibilidade | Card verifica apenas profile.ai.model | Modelo selecionado não comprova serviço funcionando ou voz audível |

Não tratar a diferença entre o card Seus bots e a Central da captura de teste como falha comprovada de uma live: o cenário injeta dados diferentes. O risco real é manter fontes e horários independentes sem comunicar sua idade.

## Composição proposta

1. Cabeçalho compacto: avatar/nome do canal e plataforma, status da transmissão confirmado pela plataforma, título e categoria, início/duração. Ações Alterar título/categoria abrem o caminho já existente, sem um segundo editor.
2. Quatro indicadores principais: espectadores agora, duração da live, mensagens por minuto e participantes ativos no chat nos últimos cinco minutos. Mensagens e participantes são observados pelo BotLive; indicar cobertura parcial quando ele entrou depois do início.
3. Área Precisa de atenção: problemas atuais, ordenados por impacto, com horário e uma ação clara. Até três itens visíveis, com acesso aos demais. Uma restauração pendente de voz deve aparecer antes de estatísticas. Sem problema, usar uma linha compacta; evitar cartão vazio gigante.
4. Corpo em duas colunas: audiência e atividade à esquerda (aproximadamente 2/3), conexões e saúde à direita (1/3). Na largura estreita, empilhar preservando ordem e prioridade; mínimos de 240 px para cards/formulários.
5. Faixa operacional compacta: última solicitação do Arroba e resultado, resgate/efeito atual, fila de voz e execuções em andamento. Mostrar confirmação pendente de banimento com prazo, sem executar ações ao consultar.
6. Atividade recente: eventos compreensíveis, resultado e horário local. Filtros Todos, Comunidade, Resgates, Bot e Falhas. Agrupar execução e confirmação para não gerar linhas repetidas. Histórico técnico continua no destino atual.
7. Configuração e pré-voo: resumo recolhível no fim, em destaque apenas no primeiro uso, antes da live ou quando detectar pendências. Seus bots vira seletor/resumo do perfil; gerenciamento continua em Perfis.

A organização troca a composição atual; não cria uma segunda Visão geral ou Central paralela. SetupHub permanece acessível de forma contextual, preservando suas seis jornadas. Stats continua sendo o destino de análise histórica; o dashboard exibe o recorte operacional.

## O que cada dado significa e de onde vem

| Informação | Fonte / situação atual | Trabalho necessário | Regra de apresentação |
|---|---|---|---|
| Live no ar, título, categoria, início, espectadores | Twitch Get Streams; já há usos pontuais em platforms, labels e discord | Criar coleta compartilhada e estado estruturado por perfil | Mostrar horário da coleta; falha não vira offline ou zero |
| Título/categoria fora do ar | Twitch Get Channel Information | Integrar ao estado do dashboard | Identificar como configuração do canal, sem afirmar transmissão |
| Pessoas conectadas ao chat | Get Chatters já usado | Corrigir erros, paginação quando necessária, reutilizar coleta | Não chamar de espectadores; lista detalhada sob demanda e acesso autorizado |
| Mensagens/minuto e participantes ativos | Eventos recebidos já existem; live_state guarda até 40 mensagens e 20 s | Agregador dedicado com janelas e exclusão de bots/simulações | São sinais observados, não audiência oficial nem total retroativo |
| Follows, subs, gifts, bits e raids da sessão | EventSub já integrado; histórico retido é insuficiente como contrato de sessão | Agregar por ID de transmissão, deduplicar eventos e persistir | Subs, renovações e presentes com definições separadas; sem transformar bits em dinheiro estimado |
| Curva de audiência | Não há série persistente na Visão geral | Amostrar e armazenar valores da API | Buracos de coleta ficam vazios; não interpolar como medição real |
| Pico e média | Derivados da série nova | Média ponderada pelo tempo/cobertura, definição do pico | Rotular como pico/média observados pelo BotLive, não oficiais da Twitch |
| Chat e recebimento de eventos | statuses é agregado; há logs de falhas por assinatura | Saúde separada por conta, conexão e assinatura esperada | Bot conectado não implica eventos de subs/follows disponíveis |
| OBS transmitindo, gravando, cena, FPS e quadros perdidos | OBS atual executa ações; health não coleta telemetria de transmissão | Leitura dedicada GetStreamStatus/GetRecordStatus/GetStats/GetCurrentProgramScene | Conexão WebSocket, transmissão OBS e live da plataforma são três estados |
| Bitrate OBS | GetStreamStatus oferece bytes/duração, não taxa pronta nessa resposta | Derivar diferença de bytes por tempo entre amostras válidas | Chamar taxa observada/estimada; descartar primeiro ponto e reset de contadores |
| Sons/TTS | Fonte conectada e confirmações audio.ack existentes | Agregar última entrega, início/fim/erro e idade da confirmação | Reproduzido pela fonte não equivale a som recebido pelo público |
| Voicemod | health e fila existentes | Expor estado e recuperação no painel compacto | Identificar serviço da máquina; evitar prometer medição de qualidade acústica |
| Arroba | Resposta administrativa/voz já existe | Resultado estruturado para ação/chat/voz e pendência de confirmação | “Pedido executado”, “Chat enviado” e “Voz entregue” separados |
| Meta/doações | Integração Meta/Livepix existente, requer checar contrato antes da implementação | Adaptador opcional com fonte/período/moeda | Não somar bits/subs/doações como receita oficial ou saldo disponível |

A primeira integração de audiência deve ser Twitch. Outras plataformas exibem suporte indisponível até validação de suas APIs, sem números simulados em produção. Serviços globais/mesmo endpoint não devem parecer isolados por perfil quando são compartilhados.

## Contrato de confiabilidade

Cada domínio precisa devolver valor, fonte, perfil/escopo, observedAt, lastSuccessAt, estado e erro recuperável. Estados distintos: carregando, atualizado, desatualizado, sem permissão, não configurado, indisponível e não suportado. Zero é reservado a uma resposta válida com zero. Dado ausente é “—”; não usar verde no desconhecido. Um erro de uma fonte não apaga as demais. O cabeçalho informa há quanto tempo os dados da plataforma foram obtidos.

Proposta inicial de coleta: Twitch 30 s, presença de chat 60 s ou sob demanda, OBS 2–5 s quando dashboard visível, eventos do bot em tempo real. Valores são decisões propostas, a ajustar com limites/respostas da API e teste de carga. Uma coleta compartilhada por perfil/destino, sem sobreposição, com timeout, cancelamento na troca de perfil, descarte de respostas tardias, backoff/jitter em falhas e respeito a rate limit/Retry-After. A telemetria não deve competir com ações de voz/OBS na fila operacional.

Ao reabrir o app durante uma live, recuperar ID/início da plataforma; não reiniciar a duração porque o bot reconectou. Associar os agregados ao ID da transmissão; marcar intervalos sem observação e não inventar eventos durante desconexão. Sessão do módulo de rótulos não equivale automaticamente a sessão da transmissão. Nunca extrair os valores estruturados analisando frases de logs.

## Direção visual e interação

Manter a identidade escura e verde atual, reduzir cartões decorativos e textos de preparação na área principal. Um número relevante por card, rótulo, período e atualização; unidades explícitas. Usar gráfico de audiência de leitura simples, com eixo temporal e lacunas reais, e não dezenas de minigráficos. Não usar velocímetro de “saúde 98%” sem fórmula confiável.

Cor acompanha texto/ícone: conectado, atenção, erro, desconhecido. Layout estável durante atualização, números sem saltos de largura e sem reposicionar ações sob o mouse. Contraste adequado, navegação por teclado, foco visível e movimento reduzido. Atualizações a cada poucos segundos não devem ser anunciadas incessantemente por leitor de tela; anunciar apenas mudanças relevantes. Modo privacidade para ocultar nomes e valores ao compartilhar a tela.

Antes da live: destacar pré-voo e pendências. Ao vivo: audiência, duração e alertas. Depois da live: resumo observado e preparação da próxima. Transições só com estado confirmado; não ocultar conteúdo porque uma consulta falhou. Configurações detalhadas e listas grandes ficam acessíveis por expansão ou pelas telas existentes, seguindo divulgação progressiva.

## Ordem de implementação

1. Fundação dos dados e correções de significado: separar transmissão/conexão, corrigir falhas que parecem zero, padronizar escopo e validade. Sem esta etapa, um visual novo preserva os erros.
2. Substituir a composição da Visão geral: cabeçalho de live, quatro indicadores, atenção prioritária, saúde compacta e atividade. Reaproveitar painéis/editores; recolher jornadas e pré-voo. Primeira entrega já inclui audiência real da Twitch.
3. Sessão e gráficos: coleta persistida, mensagens/minuto, pico/média observados e eventos da sessão, com cobertura explícita.
4. Telemetria OBS e áudio/Arroba: estado confirmado, quadros perdidos, entregas de voz, recuperação nas telas existentes. Não incluir botões perigosos de encerrar live/banir como atalhos sem contexto.
5. Personalização leve: escolher cards secundários e ocultar integrações não utilizadas. Adiar arrastar widgets e layouts livres até validar a hierarquia padrão.

As etapas são fronteiras de escopo, não estimativas de prazo ou promessa de funcionalidades já prontas.

## Plano de QA e aceite

- Um streamer identifica canal, ao vivo/fora do ar, espectadores e problema prioritário sem rolar em 1100×900 e 1600×1000. Testar também zoom 125%/150%, nomes longos, listas vazias, teclado e contraste.
- Bot conectado + live offline; live online + bot desconectado; OBS transmitindo + plataforma ainda não confirmada; OBS indisponível + plataforma online: combinações independentes e mensagens corretas.
- Respostas 401, 403, 429, 500, JSON inválido, timeout e internet perdida não viram zero, offline confirmado ou sucesso. Dados antigos continuam identificados com horário.
- Trocar perfil durante resposta lenta não mostra números do perfil anterior. Atualizar/reabrir/entrar duas vezes na tela não multiplica polling. Não revelar tokens, endpoints sensíveis ou dados de outros perfis.
- Reiniciar/reconectar com live ativa preserva a duração da plataforma e o recorte da sessão. Lacunas, eventos duplicados e fora de ordem não inflacionam métricas. Simulações e testes nunca entram nos números reais.
- Chatters 1000+ permite total distinto de lista paginada; falta de permissão não parece sala vazia. Não converter usuários do chat em audiência.
- Áudio: distinguir fonte conectada, arquivo entregue, reprodução iniciada/finalizada, erro e timeout. Não afirmar qualidade de estúdio ou áudio audível com base só em telemetria.
- OBS: reset/reconexão de contadores não gera bitrate negativo ou falso alerta; queda momentânea não vira falha persistente. Telemetria não atrasa restaurações temporárias.
- Medir tempo de tarefas com o usuário: descobrir quantos assistem, detectar áudio desconectado e chegar à recuperação da voz. Proposta de meta inicial: identificar estado/problema em até 5 s e chegar à tela de solução em até 2 interações, a confirmar por teste, sem alegar resultado medido.
- Antes de publicar implementação: testes de contrato/mocks, testes nativos pertinentes, navegador estreito/largo com capturas, gravação real OBS/Twitch de teste quando autorizada, notas detalhadas e versão incremental via update:prepare. Este estudo não equivale à homologação dessas futuras mudanças.

## Fontes e arquivos examinados

Código: src/App.tsx, src/SetupHub.tsx, src/Preflight.tsx, src/LiveConsole.tsx, src/ChatPresence.tsx, src/Stats.tsx, src-tauri/src/lib.rs, platforms.rs, labels.rs, live_state.rs, stats.rs, obs.rs e trecho de consulta de stream em discord.rs.

- Twitch Get Streams: https://dev.twitch.tv/docs/api/reference/#get-streams
- Twitch Get Chatters: https://dev.twitch.tv/docs/api/reference/#get-chatters
- Twitch Get Channel Information: https://dev.twitch.tv/docs/api/reference/#get-channel-information
- OBS WebSocket protocolo: https://github.com/obsproject/obs-websocket/blob/master/docs/generated/protocol.md
- Nielsen Norman Group, Progressive Disclosure: https://www.nngroup.com/articles/progressive-disclosure/

Os valores da prévia visual anexa são fictícios. Ela apresenta hierarquia proposta, não dados do canal ou funcionalidade já implementada.


## Requisito adicional: recuperar o banner original

Pedido do usuário: preservar o banner antigo no topo da Visão geral. Identificado no pai de 0ad0182, retirado na v0.20.16: “Menos tarefas. Mais momentos ao vivo.”, com robô, órbitas, chips e chamada para configurar/criar automação. Reutilizar o original, sem redesenhar a arte.

Na implementação atual da restauração local, aparece sem perfil ou enquanto o bot selecionado não está online, antes do roteiro. Não inferir transmissão offline a partir desse estado. Na nova coleta do dashboard, a regra final proposta é: sem perfil OU bot não conectado OU transmissão confirmada fora do ar. Transmissão desconhecida/desatualizada não deve ser tratada como offline; durante operação confirmada e bot conectado, ocultar o banner para priorizar métricas. Não fazer o banner piscar em cada atualização de dados.

Restauração local separada do redesenho completo: validar 1100/larga, arte original, botão, ausência quando conectado e primeiro uso. Publicação e versão somente na entrega de implementação, com notas e demais verificações exigidas pelo repositório.


## Entrega da v0.20.18

A implementação substitui a composição anterior, recupera o banner condicional, usa API Twitch e observação OBS, preserva agregados da transmissão e inclui personalização/privacidade. O guia de uso registra comportamento e limites concretos. O estudo acima é a referência histórica de decisões; não equivale a comprovação de todos os critérios de teste com um canal real. Consulte updates/0.20.18.md para a validação executada.
