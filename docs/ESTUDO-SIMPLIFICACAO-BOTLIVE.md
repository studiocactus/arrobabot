# Estudo de simplificação do BotLive

Este documento registra o mapeamento das jornadas, as evidências, as prioridades, o que foi implementado nesta entrega e o que ficou pendente — para a operação do BotLive ficar mais simples sem perder o que já funciona.

## 1. Inventário das jornadas (onde começa, o que se pede, quantas telas)

| Tarefa frequente | Onde começa | Informações pedidas | Telas / decisões | Como salva e testa | Como corrige uma falha |
|---|---|---|---|---|---|
| Responder no chat (texto fixo) | Comandos | comando + mensagem | 1 formulário | Salvar comando / Testar fluxo | Histórico + reabrir o comando |
| Consultar informação (follow, seguidores, música) | Comandos prontos | comando + resposta | 1 formulário | Salvar e ativar / Testar comando | Resposta avisa o que falta; Histórico |
| Enviar mensagem periódica | Timers | mensagem + intervalo | 1 formulário + opções | Salvar timer | Histórico |
| Tocar um som | Sons de Live + regra | som + gatilho | 2 telas | Salvar regra | Histórico + Testar |
| Trocar minha voz | Voicemod | acionamento + voz + duração | 1 tela (acordeão) | Salvar regra / Testar esta voz | Fila + Histórico |
| Reagir a evento/resgate | Automações | gatilho + etapas | editor visual | Salvar fluxo | Histórico + Simular evento |
| Comandos falados | Comunidade → voz | microfone + palavras | 1 tela | Ligado por interruptor | Histórico |
| Contadores/comunidade | Comunidade | módulo + ajustes | 1–2 telas | Ativar módulo | textos da própria tela |
| Overlays (OBS) | Meta/Alertas/Chat no OBS | modelo + endereço no OBS | 1 tela (.lp-studio) | Salvar + colar no OBS | prévia + "Gerando o endereço" |
| IA/memória | Inteligência artificial | provedor + tom | 1 tela | Salvar personalidade | Testar resposta contextual |

**Evidências desta análise** (conforme pedido, separadas por tipo):
- **Comprovado por interação:** o comando pronto era fácil de criar mas abria o grafo técnico ao editar (reproduzido no teste de tela "editar a função pronta abre o formulário dela, sem grafo"); o bot ignora as próprias mensagens (reproduzido: teste com a conta do bot não registra nada); erro de permissão de seguidores aparecia com texto cru sem persona (reproduzido no chat).
- **Identificado no código:** modelos com IA no caminho padrão seguiam o contrato "falha da IA interrompe o fluxo" (`engine.rs`), sem resposta de reserva; a identidade do modelo não existia (só nome/textos); conflitos não consideravam comandos de módulos/Voicemod.
- **Hipótese a validar com uso real:** que a entrada "Criar" por resultado reduz indecisão; que a visão unificada de comandos reduz suporte sobre precedência.

## 2. Pontos da análise anterior — confirmação

| Ponto levantado | Situação |
|---|---|
| Modelos abriam o editor visual ao editar | ✅ **Confirmado e corrigido** — agora abrem o formulário da função |
| Tempo de follow e doação incluem IA no caminho padrão | ✅ **Confirmado e corrigido** — IA virou opcional, com resposta determinística padrão |
| Comandos distribuídos entre fluxos, Voicemod e módulos | ✅ Confirmado — visão unificada na tela Comandos |
| Respostas por TXT podem consumir mensagens antes das automações | ✅ Confirmado no código (`chat_extras` responde antes) — explicado na visão unificada |
| Formulários expõem opções secundárias cedo | ✅ Confirmado — modelos agora têm essencial/avançado com resumo |
| Tarefas exigem navegar entre bibliotecas/integrações/editores | ⏳ Parcialmente tratado (som/recompensa ainda vivem em telas próprias) |
| Página inicial sem distinção por intenção | ⏳ Pendente (seção 9) |

## 3. Prioridades (seção 13 do pedido)

1. **Edição simples dos modelos e menos dependências** → implementado nesta entrega.
2. **Descoberta de comandos e conflitos** → implementado (visão unificada + recusa citando a origem).
3. **Criação contextual e presposto de rascunho** → parcial (modelos em um formulário; importar som/recompensa dentro do editor continua pendente).
4. **Consistência das demais funções** → pendente (matriz na seção 7).
5. **Navegação e acabamento** → pendente.

## 4. Percursos antes e depois (medidos)

| Tarefa | Antes | Depois |
|---|---|---|
| Criar resposta fixa | 1 tela (Comandos → Novo comando) | **igual** (1 tela) — clareza maior nos rótulos |
| Criar Tempo de follow | Comandos prontos → 1 tela | **igual** (1 tela), agora com resumo "O que vai acontecer" |
| **Editar Tempo de follow** | abrir **grafo técnico** (2 blocos + conexões) | **formulário da função** (comando, alias, resposta) — 1 tela |
| Usar IA na resposta | sempre (obrigatória) | **opcional** (1 interruptor), com reserva determinística |
| Entender por que não respondeu | ler o Histórico inteiro | resposta do próprio comando + aviso de conflito na criação |
| Achar comandos de módulos/voz | vasculhar telas | lista "Outros comandos do chat" na tela Comandos |

Decisões exigidas para editar Tempo de follow: antes ~6 (abrir, achar o campo no grajo, entender variáveis…); agora 3 (campo, texto, salvar).

## 5. Decisões de produto

- **Identidade explícita e versionada** (`fnId`/`fnVersion` no fluxo): quem escolhe o editor é o tipo da função, nunca o nome do comando ou textos editáveis.
- **Desvio preservado**: se o fluxo foi personalizado no editor avançado e não representa mais o modelo, o app **explica** e abre o avançado — nada é descartado para "voltar ao simples".
- **IA opcional com reserva**: dados verificados (Twitch/Spotify) ficam separados da geração; a frase da IA pode mudar, os números não. Se a IA falhar, a resposta determinística vai ao chat e o motivo real fica no Histórico.
- **"O que vai acontecer?"**: toda criação traz uma frase curta do resultado esperado — salvado ≠ ativado ≠ pronto para executar continua sendo dito na interface.
- **Conflitos**: nomes repetidos (fluxos, módulos, Voicemod) são recusados citando a origem; a precedência atual é explicada (comandos prontos da comunidade antes dos seus; TXT antes das automações).

## 6. Alterações implementadas (v0.20.8)

- Formulário da função para criar **e editar** (Tempo de follow, Total de seguidores, Música atual, Link de doação, Clipadores) com essencial/personalizar/avançado e o resumo do que vai acontecer.
- Respostas determinísticas por padrão; IA opcional com **resposta de reserva** editável.
- Visão unificada "Outros comandos do chat" (módulos + Voicemod) com origem, e recusa de conflito citando a origem.
- Testes: modelo sem IA e com IA, reserva sempre presente, edição sem grafo (aceitação), conflitos com comandos reservados.

**Capturas:** `%TEMP%\botlive-ux\comandos-prontos-followage.png`, `comandos-prontos-criado.png`, `comandos-prontos-larga.png`, `funcao-editor-simples.png` (1100 px e janela larga).

## 7. Pendências (com dependências)

1. **Criação contextual**: importar som/recompensa sem sair do editor (rascunho preservado) — depende de diálogo compartilhado de biblioteca.
2. **Diagnóstico por comando**: "não respondeu porque a regra X processou antes" — depende de rastro de decisão no motor (o Histórico já mostra intervalo e consumidores; falta amarrar por comando).
3. **Navegação por tarefas e página inicial** (seção 9) — depende de redesenho de grupos; os dados de estado por integração já existem.
4. **Padronização de salvamento/teste** (seção 10) — a etiqueta "exemplo/simulação/consulta real/execução real" precisa de componente compartilhado.
5. **Passada nas demais áreas** (Timers/Respostas/Sons/Overlays/Comunidade/Integrações/Presets) — a matriz acima é a base; cada item é uma entrega própria.

## 8. Testes executados

- Rust: **387** aprovados (inclui alias do follow, desfechos da Twitch, reserva de IA).
- Interface: **160** aprovados (modelos determinísticos/IA, reserva, conflitos).
- Tela: **94** aprovados, incluindo a aceitação "editar a função pronta abre o formulário dela, sem grafo" e capturas em 1100 px e janela larga.
- Nada foi enviado ao chat, nenhum ponto gasto, nenhuma moderação executada para validar.

## 9. Limitações desta análise

- Os números de percurso vêm da leitura do código e do teste automatizado; a medição com pessoas reais (tempo, erros) continua sendo o teste na sua operação.
- A IA opcional depende do provedor configurado; sem ele, o caminho determinístico é o que responde — por decisão de produto.
