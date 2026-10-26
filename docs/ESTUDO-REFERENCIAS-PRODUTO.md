# Evolução do BotLive: StreamElements, Streamlabs e Streamer.bot

Data da pesquisa: 25/10/2026. Base local: BotLive v0.20.15, commit d2dbae4. Prioridade escolhida pelo usuário: **organizar telas e facilitar configuração**.

## 1. Direção proposta

O BotLive deve oferecer entrada por resultado, configuração consistente e operação verificável: a pessoa escolhe o que quer fazer, configura o essencial, testa e encontra uma explicação quando algo falha. A potência avançada continua disponível no mesmo produto.

A limitação percebida tem duas origens distintas: recursos existentes difíceis de descobrir e capacidades de operação/automação que ainda precisam evoluir. Reorganizar o menu resolve parte da primeira; não equivale a implementar a segunda. A proposta é sequenciar ambas, com QA como requisito de cada entrega.

Não propomos copiar interfaces, marcas ou catálogos inteiros. Aplicamos princípios identificados nas referências à arquitetura local do BotLive.

## 2. Método e limites da comparação

Pesquisa em documentação oficial pública, contrastada com componentes, serviços, testes e estudos anteriores do repositório. As referências foram consultadas nesta data. Não houve login em contas dos concorrentes, benchmark de desempenho, compra de planos nem teste comparativo com usuários.

**Documentado** significa recurso explicado pelo fabricante. **Interpretação** significa avaliação de produto nossa. **Proposta** significa comportamento a construir no BotLive. Não atribuímos notas numéricas de facilidade ou robustez sem medi-las.

“Organizado como Streamlabs” é a preferência do usuário, não conclusão de superioridade universal. Recursos podem variar por plataforma/versão. Este estudo não compara preços nem promete paridade de integrações.

## 3. O que aproveitar de cada referência

### StreamElements: tarefas prontas e configuração progressiva

**Documentado:** o guia de início conduz por conta, painel, overlay, OBS e chatbot. O chatbot separa comandos padrão/personalizados e oferece módulos ativados por canal. Fontes: [início guiado](https://docs.streamelements.com/selive/guides/youtube-quickstart), [comandos](https://docs.streamelements.com/chatbot/commands), [módulos](https://docs.streamelements.com/chatbot/modules).

**Interpretação:** um caminho explícito e recursos com nomes de tarefa reduzem a necessidade de aprender a estrutura interna antes de começar.

**Aplicação:** entrada por objetivo, receitas prontas e essencial antes de avançado. O BotLive já possui comandos prontos, presets e formulários simples; precisamos conectá-los em jornadas compreensíveis. Não há motivo para exigir grafo para uma resposta fixa.

### Streamlabs: recursos agrupados e configuração junto do resultado

**Documentado:** Cloudbot apresenta configuração por comandos, moderação, fidelidade e módulos; widgets possuem páginas próprias com endereço para uso como fonte de navegador. Fontes: [configuração Cloudbot](https://streamlabs.com/content-hub/post/quick-guide-setting-up-streamlabs-cloudbot), [widgets no OBS](https://streamlabs.com/content-hub/post/how-to-locate-and-use-streamlabs-widget-urls).

**Interpretação:** a organização por finalidade e a ligação entre ajuste e saída visual ajudam a pessoa a prever o resultado.

**Aplicação:** grupos coerentes no menu e padrão de estúdio já existente, com prévia, ajustes e teste. O endereço de saída e o diagnóstico precisam estar no contexto certo. Não precisamos criar um segundo editor de overlays nem substituir o OBS para obter essa clareza.

### Streamer.bot: execução observável e composição

**Documentado:** ações reúnem subações e gatilhos; filas possuem comportamento configurável; o histórico permite inspecionar estado, duração e variáveis antes/depois da execução. Fonte: [ações e filas](https://docs.streamer.bot/guide/core/actions). O [catálogo de subações](https://docs.streamer.bot/api/sub-actions) mostra controles de filas e execução.

**Interpretação:** robustez inclui saber o que executou, o que está esperando e qual dado cada etapa recebeu.

**Aplicação:** evoluir o acompanhamento de testes do BotLive para uma central de execuções reais, depois oferecer filas por recurso e ações reutilizáveis. Repetição deve respeitar idempotência; reexecutar cegamente um resgate ou mensagem pode duplicar efeitos.

## 4. Inventário e lacunas reais do BotLive

| Capacidade | Evidência local | Limite atual | Evolução proposta |
|---|---|---|---|
| Comandos simples e prontos | Commands, FunctionForm, commandModels | Descoberta depende de saber o nome da tela | Buscar por intenção e entrar por objetivo |
| Respostas e timers | ChatExtras, Timers | A pessoa precisa distinguir três conceitos de mensagem | Jornada de chat com exemplos e destinos claros |
| Estúdios de overlay | Alertas, ChatObs, LivePix, Spotify; padrão lp-studio | Configuração e instalação da fonte ainda exigem leitura | Roteiro de instalação com teste verificável |
| Voz e sons | AudioHub, Voicemod, serviços separados | Conexão não mede qualidade acústica | Continuar QA de voz real e evidência por caminho |
| Automações | Editor visual, filtros, variáveis, simulação | Complexidade alta para quem só quer um resultado | Receitas simples com acesso ao editor avançado |
| Acompanhamento | Painel de teste e Histórico | Resultado de produção disperso em logs | Central de execuções correlacionadas |
| Concorrência | engine usa semáforo; vozes e sons têm filas próprias | Não há demonstração de uma UI unificada de filas por recurso | Política explícita, limites, pausa/cancelamento confirmados |
| Conhecimento e IA | Personalidade, memórias e contexto | Relação entre provedor, ação e conhecimento exige aprendizado | Formulário orientado a tarefa e diagnóstico de dependências |
| Comunidade e Discord | Módulos existentes, configuração por perfil | Descoberta e disponibilidade são pouco explicadas em conjunto | Catálogo organizado com requisitos reais |
| Confiabilidade | Testes, cofre, perfis, CI assinada | Não substitui homologação de equipamento e serviços reais | Matriz operacional por versão e critério de regressão |

Esta tabela não afirma ausência de recursos apenas porque uma tela não os destaca. A fila de voz, por exemplo, já existe; a lacuna é generalizar o controle e observabilidade com segurança.

## 5. Arquitetura da experiência

### Encontrar

Menu por finalidade e busca por tarefas, com termos do usuário. Um destino mantém seu identificador e nome; não há versões duplicadas da mesma configuração em telas diferentes.

### Configurar

Roteiro apresenta os passos e abre o editor existente. Recursos básicos mostram campos essenciais; controles adicionais ficam em Avançado, com os valores preservados. Cada formulário informa como salva, como testa e o que precisa estar conectado.

### Operar

Uma futura Central da live reúne estado real de conexões, últimos eventos, execuções e filas. Estado desconhecido deve aparecer como desconhecido. Configuração salva, autorização válida, conexão ativa e entrega confirmada não podem receber a mesma indicação de sucesso.

### Investigar

Uma execução deve ligar evento → regra → etapa → chamada externa → resultado. Mostrar o motivo de não disparar também é essencial: permissão, intervalo, condição ou conflito. Variáveis com conteúdo sensível precisam ser omitidas/redigidas.

## 6. Primeira etapa implementada nesta rodada

1. Menu reorganizado em **Seu canal**, **Chat**, **Visual do OBS**, **Áudio e voz**, **Automação e IA**, **Comunidade** e **Acompanhamento**. Mantidos todos os 23 destinos, mais Configurações.
2. Visão geral recebe **O que você quer configurar?**, com seis objetivos e passos que abrem as telas existentes.
3. Busca Ctrl+K entende tarefas: resgate, recompensa, áudio, erro, backup, entre outras; aceita consulta sem acentos.
4. Estado sem resultados explica como procurar novamente.
5. Primeiro uso explica a necessidade de perfil, oferece criação e impede atalhos que terminariam em tela vazia.
6. Tela ativa marcada semanticamente na navegação.
7. Roteiro ocupa o lugar do banner decorativo da Visão geral; o atalho de criar automação continua disponível.

Não foram criados estados falsos de “etapa concluída”. O roteiro não autoriza contas, ativa módulos ou executa efeitos ao ser aberto. O conteúdo e as permissões continuam nos serviços existentes.

Esta etapa melhora descoberta e orientação; ainda não elimina diferenças entre formulários nem acrescenta filas gerais.

## 7. Próximas entregas propostas, em ordem

| Etapa | Escopo concreto | Critério de aceite |
|---|---|---|
| A — Organização | Menu, busca e entrada por objetivo | Todo destino preservado; tarefas encontráveis; primeiro uso sem atalhos vazios; teclado e layout aprovados |
| B — Configuração consistente | Inventário de padrões de salvar/testar; requisitos junto ao campo; essencial/avançado nos formulários mais usados | Criar comando, timer, alerta e regra de voz sem abrir editor técnico; valores preservados ao alternar opções |
| C — Receitas prontas | Boas-vindas, raid, som por resgate, voz por resgate, contador e lembrete | Prévia explica gatilho, resultado, permissões e teste; instalação em rascunho sem publicar ou ativar ações inesperadas |
| D — Central de execuções | Histórico por execução com etapas, tempos, erro e contexto redigido | Um evento rastreável até resultado; cancelamento confirmado; sem logs de credenciais; limites de retenção |
| E — Filas e ações reutilizáveis | Política por recurso, limites, espera, falha, timeout e composição | Voz/OBS concorrentes previsíveis; não bloquear chat por áudio; testes de queda, recuperação e duplicação |
| F — Central da live | Visão operacional sobre capacidades anteriores | Nenhum sucesso presumido; atalhos de recuperação; reconexão sem replay e sem captura duplicada |

B e C têm mais retorno imediato para a prioridade escolhida. D e E aproximam a operação da potência desejada; não devem ser prometidas como efeito de uma mudança visual.

### Dependências técnicas para D/E

- Modelar execução persistente ou com retenção limitada, identificada por perfil/evento/ação.
- Publicar eventos estruturados em vez de inferir estado por texto de log.
- Definir comportamento após reinício: tarefas externas não podem ser repetidas por padrão.
- Separar erros recuperáveis de efeitos que podem já ter ocorrido.
- Propagar cancelamento e timeout aos serviços; não declarar cancelado enquanto uma operação continuar.
- Criar ações reutilizáveis com limites de profundidade/ciclo e contrato de variáveis.
- Preservar compatibilidade dos fluxos existentes por migração versionada e backup.

## 8. QA e métricas de aceitação

Não há tempos medidos de usuários neste estudo. Propomos medir cinco tarefas: encontrar a configuração de voz, criar comando, montar alerta, configurar resgate e investigar falha.

Registrar: conclusão sem ajuda, tempo, escolhas incorretas, retornos ao menu e passos que exigem suporte. Comparar a mesma tarefa antes/depois; o sucesso não será medido apenas por quantidade de cliques.

Para cada entrega:
- Testar perfis vazio/existente, permissões, erro, carregamento e perda de conexão.
- Preservar configuração e ações avançadas.
- Navegar por teclado; foco visível, Escape e retorno de foco.
- Validar 1100 px e janela larga; auditoria adicional em tamanhos menores.
- Testes de domínio quando houver semântica nova; testes de navegador nas jornadas alteradas.
- Documentar o que foi executado e o que depende de equipamento/serviço real.
- Manter QA acústico anterior como pendência de homologação: organização não corrige estalos por si só.

## 9. Decisões preservadas

O produto continua local, com cofre de credenciais e isolamento por perfil. A experiência simples usa os mesmos serviços da avançada. Não haverá uma segunda base de automações “básicas” divergindo da primeira.

Não vamos priorizar mais opções apenas para aumentar uma lista de recursos. O primeiro ganho é deixar acessível o que existe; os seguintes são capacidades verificáveis de execução e recuperação.

## 10. Validação desta implementação

Resultados desta rodada: 172 testes de lógica aprovados, verificação de tipos aprovada e build de produção executado nos testes de navegador. Na implementação final, 18 testes de navegador aprovados: app, configuração por objetivo, manual e auditoria em cinco larguras. Um teste adicional aprovado em npm run dev, com configuração temporária de QA ignorando ferramentas/target no observador, produziu capturas de 1100/1600 px, inspecionadas visualmente. Manual gerado com 30 capítulos. A primeira tentativa do teste de atalho ocorreu antes de a interface carregar; o teste foi corrigido para aguardar a tela e passou. Permanece o aviso de build sobre chunk acima de 500 kB. A primeira etapa está implementada no checkout local, ainda sem commit ou publicação. Pesquisa documental não equivale a teste de contas reais dos concorrentes; inferências de usabilidade precisam de validação com pessoas.


## 11. Segunda etapa — formulários consistentes

Implementação local seguinte à organização: contrato compartilhado de salvar/testar/requisitos nos formulários simples de comando e timer, no Estúdio de alertas e nas regras de voz. Os modos de gravação são explícitos: comandos/timers/voz usam botão; alertas mantêm autosave.

Comando e timer recolhem opções secundárias preservando os valores. Regras de voz usam rascunho e gravação explícita do conjunto: corrigido o envio a cada tecla. Falha mantém a edição, e navegação/troca de perfil pede descarte quando há rascunho de voz. A gravação mescla as regras com os ajustes atuais de fila/OBS para não sobrescrever edições em outra seção.

Os formulários simples tratam erro de gravação e impedem pedidos simultâneos. Alertas oferecem nova tentativa de autosave, impedem teste de configuração ainda pendente e mostram falha de carregamento sem liberar edição de valores padrão desconhecidos. Receitas e central de execuções continuam propostas, não implementadas nesta etapa.


### Evidências e limites da segunda etapa

- Corrigida sugestão ambígua de função: uma etapa genérica de mensagem não identifica “Total de seguidores”, e timer não deve ser convertido em comando por esse formato. Funções com identidade explícita continuam no formulário próprio.
- Salvamento explícito de voz é mudança de interação: vale para inclusão, alteração, remoção e ativação de regras. A confirmação de saída protege esse rascunho; ele não é persistido como recuperação de sessão.
- Comandos e timers mantêm a regra existente de aplicar somente ao salvar. O aviso de descarte adicionado nesta etapa atende às regras de voz; não foi generalizado para todo editor.
- Alertas continuam com debounce de 700 ms: aguarde Salvo antes de sair. Não há promessa de recuperação de uma edição perdida por encerramento abrupto do processo.
- Backend, dispositivos e mecanismo acústico não foram alterados. Testes de interface usam serviços simulados; não comprovam emissão pública, qualidade de áudio ou permissões reais.
- Mantida a implementação local da etapa anterior. Nenhuma destas duas etapas foi publicada nesta rodada.


### Validação executada da segunda etapa

- npm test: **173 aprovados**; npm run check aprovado.
- Suíte geral de navegador: **108 cenários executados, 105 aprovados e 3 falhas**. Uma falha identificou erro duplicado no editor visual, corrigido no produto; as outras exigiram ajustar os testes ao novo estado de salvamento e às opções recolhidas. A repetição direcionada dos três cenários passou (**3/3**). Não se declara aqui uma segunda execução completa sem falhas.
- Validação final em npm run dev com configuração temporária de QA: **7/7 aprovados**, cobrindo falha/repetição de gravação, opções preservadas, navegação com rascunho e capturas. Após posicionar Salvar no rodapé de voz, os dois testes de rascunho foram reaprovados; uma repetição adicional gerou a captura a partir do topo, sem aviso cobrindo os campos.
- Build de produção executado pelas rodadas de navegador; permanece aviso de chunk acima de 500 kB. Manual gerado com 30 capítulos; npm run update:check aprovado. Não houve alteração de versão, commit ou push.
- Capturas de comando/timer/voz em 1100 e 1600 px; alertas em 1100 e janela larga. Inspeção visual realizada. Arquivos de teste permanecem fora do versionamento.
- Núcleo Rust não foi alterado nem retestado nesta etapa. Sem homologação de áudio real e sem execução pública em plataformas.

Capturas da implementação local:
- [Comando — 1100 px](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-form-comando-1100.png) · [1600 px](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-form-comando-1600.png)
- [Timer — 1100 px](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-form-timer-1100.png) · [1600 px](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-form-timer-1600.png)
- [Voz — 1100 px](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-form-voz-1100.png) · [1600 px](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-form-voz-1600.png)
- [Alertas — 1100 px](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-form-alertas-1100.png) · [larga](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-form-alertas-largo.png)


## 12. Terceira etapa — receitas substituindo caminhos duplicados

Conforme a orientação de substituir em vez de acumular, **Receitas prontas** toma o lugar do formulário de criação de Comandos prontos e do botão separado de Resenha com IA. Não há outra página na navegação nem outro motor de execução.

- As cinco funções anteriores (follow, seguidores, música, doação e clipes) abrem o mesmo formulário usado para editá-las; a criação antiga foi removida.
- Boas-vindas, contador de mortes, lembrete de água e som por resgate são rascunhos dos fluxos/timers atuais.
- Agradecer raid abre Comunidade → Shoutout e ganks. Voz por resgate abre Voicemod → Etapa 3. Não cria outras regras, módulos ou filas.
- Resenha usa o formulário de criação existente; se já estiver configurada, abre o fluxo atual para edição.
- Os rascunhos novos são salvos desativados. Configurações existentes preservam sua ativação e seus passos.
- A identificação é persistida no campo de identidade que já existe nos fluxos; receitas novas continuam reconhecíveis após renomear. Configurações antigas sem identidade usam somente correspondências conservadoras (comando/alias ou nome e tipo), sem migração automática.
- Ao escolher uma receita, os fluxos do perfil são consultados novamente. Uma falha de leitura não é interpretada como ausência de configuração.
- Comandos em conflito são recusados no formulário. A receita de som exige recompensa específica e arquivo de áudio e verifica outro som já ligado à mesma recompensa.
- Biblioteca de presets mantém sua função distinta de guardar/importar/exportar pacotes do usuário; o catálogo não replica esse mecanismo.

### Limites desta etapa

Não há substituição automática de dados salvos nem exclusão de fluxos. Se uma configuração legada foi renomeada e não tem identidade, não há evidência suficiente para decidir que ela equivale à receita. Fluxos personalizados continuam editáveis e não são reduzidos ao modelo.

As conferências de conflito desta etapa ocorrem na interface, antes do salvamento. Não constituem uma transação de exclusividade entre duas instâncias que gravem ao mesmo tempo. Raid e voz continuam sujeitos aos seus serviços e testes operacionais existentes.


### Validação executada da terceira etapa

- **176 testes de lógica aprovados**, incluindo rascunhos desativados, identificação após renomear, preservação de passos personalizados, aliases e isolamento por perfil.
- Primeira rodada dirigida de navegador: **20/20 aprovados**, com comandos existentes, novos editores, destinos de raid/voz, timer e validação de som por recompensa.
- Suíte completa: **113 cenários executados; 112 aprovados e um teste ainda procurando o botão removido de Resenha com IA**. O teste foi adaptado ao caminho Receitas prontas e reaprovado na verificação final. Não se declara uma segunda execução completa sem falhas.
- Verificação final em **npm run dev: 7/7 aprovados**, incluindo esse caso, catálogo, edição e conflitos de comandos, reaproveitamento de timer renomeado e encaminhamento aos painéis existentes. Um teste adicional gerou as capturas finais sem transições de hover.
- npm run check, npm run docs (30 capítulos) e npm run update:check aprovados. Build de produção executado pelas rodadas de navegador, mantendo aviso de chunk acima de 500 kB.
- Capturas em 1100 e 1600 px, com inspeção visual; nenhum artefato de teste foi incluído no versionamento.
- Nenhum backend Rust, dispositivo de áudio, dado real de usuário ou credencial foi alterado. Os testes usam prévia ou serviços simulados e não homologam áudio real nem ações públicas.
- As etapas A–C permanecem locais, sem commit, alteração de versão ou push nesta rodada.

[Catálogo — 1100 px](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-receitas-1100.png) · [1600 px](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-receitas-1600.png)


## 13. Central de execuções — primeira parte da etapa D

Histórico foi substituído por uma central com Execuções e Eventos gerais. A lista usa flow.runs e os mesmos estados traduzidos do teste de fluxo; não há segundo motor, outro histórico persistente ou nova entrada no menu. Eventos gerais preserva os registros e a busca anteriores.

A central consulta o perfil ativo a cada dois segundos, permite busca por nome/estado/identificador e filtros de falhas, andamento e testes. Ao selecionar uma execução, mostra cada etapa e o erro devolvido pelo motor. Parada por condição continua distinta de falha. Consultas em voo são descartadas ao sair ou trocar de perfil. Erro de consulta não vira ausência de execuções: dados anteriores são identificados como possivelmente desatualizados e há nova tentativa.

Esta entrega conclui a consulta central da primeira parte de D, não a etapa D inteira: o motor mantém até 30 registros recentes por perfil na sessão, sem persistência após reiniciar. Tempos, contexto redigido do evento e cancelamento central continuam pendentes. Não foi adicionado botão de repetição ou cancelamento nesta tela; o cancelamento do teste permanece no editor. Backend e áudio não foram alterados.

Validação: 176 testes de lógica aprovados; verificação de tipos, geração do manual (30 capítulos) e update:check aprovados. Dois testes novos de navegador em npm run dev aprovados, cobrindo falha/recuperação de consulta, filtros, condição falsa, detalhes, preservação dos eventos e ausência de chamadas para executar efeitos. A primeira rodada encontrou nome acessível inadequado do filtro e instabilidade de preparação do mock; ambos corrigidos e reaprovados. Inspeção das capturas revelou altura fixa herdada nos botões da lista, corrigida e coberta por verificação de conteúdo sem transbordamento; repetição final 2/2 aprovada. Capturas de 1100 e 1600 px inspecionadas. Não houve nova suíte geral de navegador nem testes acústicos reais nesta etapa. Implementação local, sem commit, versão nova ou push.


## 14. Central de execuções — tempos e cancelamento

A central existente passa a mostrar início, tempo total e duração de cada etapa, medidos pelo núcleo Rust. O relógio monotônico mede duração sem depender de ajustes do relógio do Windows; o horário de início é apresentado em português. Durações ficam congeladas ao terminar. Etapas não iniciadas não recebem duração inventada. Corrigida a transição de Em espera para Executando ao iniciar a próxima etapa.

Cancelar execução usa a mesma operação flow.cancel do teste de fluxo. Há confirmação pelo nome, trava de pedidos simultâneos, erro com nova tentativa e estado explícito de solicitação pendente. Sucesso do pedido não significa cancelamento concluído: a tela acompanha o resultado final publicado pelo motor. Corrigido também o isolamento no núcleo: outro perfil não pode cancelar a execução, mesmo conhecendo seu identificador. O registro de eventos agora diz Cancelamento solicitado, sem antecipar a confirmação.

O cancelamento é cooperativo: ações externas já concluídas não são desfeitas, e a ação atual pode terminar ou atingir seu timeout antes de o fluxo parar. Se o pedido chegar durante a última ação e ela concluir, o motor pode registrar Concluída. Não há replay, nova fila ou execução duplicada. Retenção em sessão continua igual; persistência e contexto redigido do evento permanecem pendentes. Áudio real não foi homologado nesta etapa.

### Validação desta rodada

- 176 testes de lógica frontend aprovados; verificação de tipos, build de produção, geração do manual (30 capítulos) e update:check aprovados. Permanecem avisos de chunk grande no build e atribuição não utilizada já existente em twitch_ops.rs.
- Quatro cenários da central aprovados em produção e novamente em npm run dev: leitura/filtros, recuperação de consulta, confirmação de cancelamento sem envio duplicado e recuperação de falha no pedido. Treze cenários existentes de teste de fluxo/eventos gerais aprovados em desenvolvimento. Seletores foram atualizados para o texto com duração e para Eventos gerais; uma rodada teve inicialização de página indisponível, repetida com sucesso. Não foi repetida a suíte geral de navegador.
- Seis testes unitários do acompanhamento Rust aprovados. Suíte nativa completa com acesso a servidores locais simulados: 393 aprovados, uma falha por porta de autorização Spotify ocupada e dois ignorados. O teste de Spotify passou ao ser repetido isoladamente. A primeira tentativa no sandbox foi interrompida após falhas e esperas em serviços locais simulados; não constitui homologação dessa execução.
- O teste de integração do cancelamento verifica recusa por outro perfil, confirmação pelo motor, etapa seguinte não executada e campos de tempo na API. Nenhum teste desta rodada executou ações públicas nem certificou qualidade acústica.
- Capturas de tempos e confirmação em 1100 e 1600 px, inspecionadas visualmente. Código e documentação permanecem locais, sem commit, nova versão ou push.

[Tempos — 1100 px](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-execucoes-1100.png) · [1600 px](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-execucoes-1600.png)

[Cancelamento — 1100 px](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-cancelamento-1100.png) · [1600 px](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-cancelamento-1600.png)


## 15. Origem das execuções e correlação de eventos

A central existente passa a explicar qual tipo de evento iniciou a execução e distingue live, teste rápido, ensaio e simulação. A busca inclui tipo, modo e referência do evento. Eventos de ensaio e simulação também entram no filtro de testes, sem serem apresentados como execuções da live. Registros anteriores sem contexto mostram explicitamente origem não registrada.

O núcleo anexa o contexto uma única vez, ao iniciar o motor, conferindo o perfil. O contrato permite somente tipo de evento conhecido, modo e referência. Não copia mensagem, nome, identificador de pessoa, papel, payload arbitrário, variáveis ou credenciais. Tipos desconhecidos viram Outro evento. A referência usa 16 caracteres hexadecimais de SHA-256 derivado do perfil e do identificador do evento, com prefixo evt-; o identificador original não é devolvido nesse contexto. Eventos iguais dentro do perfil têm a mesma referência; o mesmo identificador em outro perfil gera referência diferente. Essa referência reduzida é para diagnóstico, não é chave de autorização, deduplicação ou garantia de anonimização. Eventos sem identificador não ganham identidade inventada.

A mudança reutiliza o mesmo registro de execução e a API flow.runs; não cria histórico ou motor paralelos. A política anterior de eventos gerais, nomes de fluxos e erros não foi reescrita por esta entrega: a minimização descrita aqui se aplica ao novo campo de contexto. Retenção continua em memória; persistência entre sessões ainda está pendente. Tempos do motor não medem latência acústica e não certificam a qualidade do Voicemod.

Validação desta rodada: 177 testes frontend aprovados; sete testes unitários Rust do acompanhamento aprovados; suíte nativa completa com servidores locais simulados: 395 aprovados, zero falhas e dois ignorados. O teste de integração verifica tipo, modo e referência devolvidos pela API para a mesma execução do motor. Seis testes de navegador em npm run dev aprovados, incluindo busca por evento/referência, origem ausente, rótulo de ensaio, cancelamento, recuperação e preservação dos eventos gerais. Capturas de 1100 e 1600 px inspecionadas. Verificação de tipos, geração do manual de 30 capítulos e update:check aprovados. Não foi repetida a suíte geral de navegador nem executada homologação de áudio real. Alterações continuam locais, sem commit, nova versão ou push.

[Origem — 1100 px](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-contexto-1100.png) · [1600 px](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-contexto-1600.png)


## 16. Retenção entre sessões e execuções interrompidas — 26/10/2026

O mesmo acompanhamento passa a conservar seus registros no banco SQLite atual; não há segundo histórico nem rotina de retomada. O esquema é ampliado de forma aditiva, com chave estrangeira para o perfil e índice por perfil/horário. A central mantém a mesma API flow.runs e mostra até 30 finalizações por perfil mais as execuções ativas. A retenção também é aplicada após recuperar um encerramento com muitas execuções ainda abertas.

Cada mudança de estado salva um retrato com duração observada, etapas, origem e erro já existentes. Ao reabrir, RUNNING/WAITING viram INTERRUPTED, etapas em curso ficam com resultado não confirmado e as pendentes aparecem como não iniciadas. O aviso explica que efeitos externos podem ter ocorrido e que nenhuma ação foi repetida. Não é uma promessa de exactly-once dos serviços externos. Os tempos recuperados são os últimos observados antes do encerramento, sem contar tempo fechado. Registros finalizados não são recalculados nem mudam de resultado.

A recuperação não recria registros ativos no motor, não inicia ações e não oferece cancelamento de registros antigos. Outro perfil não lê nem cancela os registros; excluir um perfil remove seus diagnósticos por cascata. Falha de persistência é reavaliada ao consultar: a central avisa e permite nova tentativa, em vez de fingir lista vazia. As gravações de diagnóstico são tentadas durante a execução sem transformar falha de armazenamento em repetição de efeitos; portanto um encerramento com falha de disco pode perder o último estado não gravado.

O banco é compartilhado com o runtime por Arc, mantendo uma conexão e o mesmo controle de transações. Restaurar um backup faz a troca dos dados e o descarte do acompanhamento anterior sob o mesmo bloqueio; se a operação falhar, mantém o acompanhamento. Tarefas antigas que já estiverem em uma ação externa seguem a limitação cooperativa de cancelamento, sem desfazer o que ocorreu. O histórico detalhado é diagnóstico local limitado e não entra nos pacotes .botlivebak; Eventos gerais continua no backup. Documentação de operação e solução de problemas atualizada com essa distinção.

### Validação

- 177 testes frontend aprovados; verificação de tipos aprovada.
- Dez testes unitários do acompanhamento Rust aprovados, incluindo reabrir banco, estabilidade de resultados/durações, ausência de replay, retenção após interrupção, isolamento, exclusão de perfil, falha de gravação e nova tentativa, troca atômica de base.
- Suíte nativa completa com serviços locais simulados: 398 aprovados, zero falhas e dois ignorados.
- Dezoito cenários de navegador em npm run dev aprovados: central, eventos gerais e teste de fluxo. Cinco cenários da central reaprovados no build de produção, incluindo estado Interrompida, ausência de controles para repetir/cancelar registros antigos e capturas de 1100/1600 px inspecionadas.
- Build de produção, manual de 30 capítulos e update:check aprovados. Permanecem os avisos já existentes de chunk grande e atribuição não utilizada em twitch_ops.rs. Não houve nova execução da suíte geral de navegador.
- A recuperação foi testada por reabertura real do banco temporário, não por interrupção forçada de uma live real. Nenhum arquivo de usuário ou credencial foi alterado. Teste acústico prolongado adiado por decisão do usuário, sem declarar qualidade de estúdio.
- Implementação local, ainda sem commit, versão nova ou push.

[Histórico preservado — 1100 px](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-historico-persistente-1100.png) · [1600 px](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-historico-persistente-1600.png)


## 17. Primeira parte de E — concorrência e restauração do OBS

A revisão encontrou um defeito no serviço existente: temporários iguais releriam o estado já alterado como origem, permitindo que a última restauração mantivesse o efeito. A trava por perfil também permitia que dois perfis alterassem simultaneamente o mesmo recurso do mesmo OBS. A implementação antiga foi substituída, sem outro caminho de execução.

Comandos, automações e testes manuais usam o serviço único. Há admissão serial por destino OBS (endereço/porta), com uma operação atendida e até oito aguardando; espera de até dez segundos, fila cheia/timeout explícitos e liberação dos lugares por RAII ao abandonar o pedido. O fluxo mostra Em espera durante a admissão e Executando ao ser atendido. Cancelamento durante a espera ou antes do envio da mutação impede a alteração; pedidos já enviados continuam sujeitos à confirmação do serviço externo. O prazo das respostas também é limitado no conjunto da chamada, evitando que mensagens alheias ao pedido renovem a espera indefinidamente.

No recurso, temporários iguais compartilham a primeira origem e contam os períodos ativos, sem reaplicar o efeito. Pedidos conflitantes — inclusive imediatos — são recusados enquanto o temporário estiver ativo. A última liberação usa a origem original e confirma o resultado por nova leitura. A cena/item da fonte e o destino OBS são os do início, mesmo que o programa troque de cena ou a configuração de endereço mude. Gerações impedem temporizador antigo de liberar um efeito novo.

A restauração primeiro confere o estado atual. Se mudou fora do efeito, preserva a alteração. Se o pedido foi aplicado mas sua resposta se perdeu, a origem continua protegida e a liberação agenda uma tentativa de restauração, inclusive quando a tarefa abandona a espera de confirmação. Falta de informação do estado anterior recusa o temporário antes da mutação: não assume mudo, volume zero ou fonte visível. Falha ao restaurar mantém o recurso pendente e informa no Histórico o alvo e a origem; novos efeitos só são liberados após a conferência do estado corrigido manualmente no OBS.

### Limites e próxima parte de E

A fila é por destino OBS, não uma política geral de filas configuráveis. O recurso permanece protegido durante todo o efeito, mas a admissão de outros recursos ao mesmo servidor ocorre entre as chamadas. Não houve alteração das filas de voz ou sons do bot. Ações reutilizáveis, composição e contratos de variáveis continuam pendentes.

A recuperação de um recurso pendente é manual nesta etapa; não há novo botão ou rotina de repetição automática. Travas e temporizadores não são retomados após reiniciar o aplicativo: confira o OBS manualmente se ele encerrou com efeito ativo. Não há garantia de distinguir alterações externas que terminem exatamente no mesmo valor do efeito. A identidade da fonte usa cena/item/nome, sem reconciliar coleções de cenas substituídas. Endereços localhost/127.0.0.1 são normalizados, mas aliases DNS arbitrários não são conciliados: use o mesmo endereço nos perfis do mesmo OBS. Respostas de rede e espera pela admissão podem estender o tempo até concluir a restauração.

### Validação executada

- 177 testes frontend aprovados; verificação de tipos aprovada.
- Suíte nativa completa com serviços locais simulados: **408 aprovados, zero falhas, dois ignorados**. Treze testes do serviço OBS, incluindo origem entre perfis, concorrência e temporizadores reais, conflitos, estado anterior ausente, troca de cena, alteração manual, troca de destino, geração antiga, recusa de restauração, perda de confirmação, limite de fila, timeout e cancelamento sem mutação.
- Suíte completa de navegador: **118/118 aprovados**, cobrindo as mudanças acumuladas de navegação, formulários, receitas e central, além dos recursos anteriores. Build de produção executado nessa rodada.
- Manual de 30 capítulos, update:check e diff --check aprovados. Permanecem avisos já existentes de chunk grande no build e atribuição não utilizada em twitch_ops.rs.
- Nenhum layout novo nesta etapa, sem nova página ou editor; a auditoria de layout existente passou na suíte completa. Testes de OBS usam WebSocket local simulado e não homologam conexão do OBS real, qualidade acústica ou ações públicas.
- Documentação de uso e solução de problemas atualizada. Alterações ainda locais, sem commit, nova versão ou push. A validação acústica prolongada continua adiada pelo usuário.


## 18. Fechamento funcional de v0.20.16

A entrega inclui etapas individuais reutilizáveis por perfil e a Central da live no lugar do cartão repetido Comece por aqui após criar perfil. A biblioteca usa o inspetor existente, sem segundo formulário de configuração de ações. O original é escolhido por ID estável; atualizar uma etapa vale para os próximos disparos. A preparação resolve todas as referências uma vez antes dos efeitos e preserva o contexto do fluxo. Não há recursão: uma referência não pode ser guardada como original. A exclusão é recusada enquanto houver vínculos em fluxos salvos. Presets exportados contêm cópias resolvidas, com posições mantidas, e backups incluem a biblioteca no estado do perfil.

A Central da live consulta somente estados locais dos serviços existentes, sem conectar, repetir ações ou capturar áudio. Mostra último instante de leitura, falha explícita e atalhos às telas atuais. Estado desconhecido não é tratado como sucesso e conexão de fonte de áudio não comprova emissão sonora. O pré-voo continua estático e distinto da consulta operacional.

Limites desta versão: etapas individuais, sem grupos ou parâmetros próprios; variáveis são as do fluxo que chama. Alterações simultâneas do original seguem a última gravação, sem controle de revisão entre editores. A biblioteca permite até 64 originais por perfil. O teste rápido acompanha as ações resolvidas, mas os campos de valores de exemplo do diálogo continuam baseados no fluxo aberto: variáveis citadas somente dentro do original podem exigir etapa local anterior. Políticas gerais configuráveis de filas e recuperação automática do OBS após reiniciar não foram incluídas. O teste acústico prolongado foi adiado pelo usuário. As demais validações e o estado de publicação estão registrados nas notas da versão.


## 19. Validação final de v0.20.16

Código congelado com 177 testes frontend e 412 testes Rust aprovados (dois ignorados), nove testes de atualização/manifesto aprovados e verificação de tipos aprovada. A suíte de navegador executou 120 cenários: 119 passaram e a auditoria identificou bases desalinhadas dos cartões da Visão geral em duas larguras. O CSS foi corrigido sem flexibilizar o teste; auditoria e jornadas novas foram reaprovadas em npm run dev (4/4). Não houve segunda rodada local completa sem falhas. Build de produção executado, manual de 30 capítulos gerado e update:check/diff --check aprovados.

A primeira compilação das novas provas exigiu derivar Debug no tipo de biblioteca; seletores receberam nomes acessíveis explícitos. As capturas finais de biblioteca e central em 1100/1600 px foram inspecionadas. O commit inclui o registro detalhado em updates/0.20.16.md; a publicação de instaladores e manifesto assinados depende da validação do workflow no GitHub, sem alteração da chave atual. Nenhum dado das gravações acústicas ou artefato de teste é incluído no versionamento.

[Central — 1100 px](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-v0.20.16-central-1100.png) · [1600 px](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-v0.20.16-central-1600.png)

[Biblioteca — 1100 px](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-v0.20.16-biblioteca-1100.png) · [1600 px](/C:/Users/conta/.codex/visualizations/2026/10/23/01a169bf-cdca-7f23-b502-e48542c891a2/botlive-v0.20.16-biblioteca-1600.png)
