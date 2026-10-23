# Auditoria QA do BotLive

## 1. Versão, commit e ambiente auditados

- **Base auditada:** v0.20.12 publicada (`latest.json` conferido) + este commit (v0.20.13) com as correções.
- **Checkout:** `main`, limpo antes e depois das alterações; nenhum executável ativo foi substituído durante a auditoria.
- **Método:** leitura do caminho completo evento → normalização → regras → execução → chamada externa, com testes automatizados reproduzindo os eventos pelo **payload real do EventSub**. Nenhuma mensagem pública foi enviada, nenhum ponto gasto, nenhuma moderação executada, nenhuma raid criada.
- **Evidências de produção (sua live):** `!followage` respondeu o follow real de 09/11/2020 (v0.20.9); o teste de fluxo do `!followers` fez consulta real à Twitch e respondeu `not_following` (validação do caminho de API).

## 2. P0 — Raid recebida sem destaque

**Causa encontrada (comprovada):** o destaque nativo **não usava quem mandou a raid**. O alvo era resolvido por @menção na mensagem, nomes vistos no chat ou alvo fixo — e a mensagem de um evento de raid é **vazia** → "Não entendi quem…" → nenhum destaque. Os dados corretos (`from_broadcaster_user_login`, o canal de **origem**) até chegavam no evento e não eram usados. Fatores agravantes: a permissão do gatilho ("Quem pode usar") era aplicada a um evento que chega como `everyone` (regra restrita nunca casava, sem aviso) e o destaque não cobria o escopo `moderator:manage:shoutouts` na conferência.

**Correções:**
- O destaque em evento de raid usa **quem mandou a raid** (canal de origem), com menção explícita continuando vencedora; nada é presumido de nomes — o login vem do evento (`event_actor_login`, testado).
- `shoutout` passa a exigir `moderator:manage:shoutouts` na conferência de token (erro diz o que reautorizar).
- Teste de ponta a ponta com o payload real do `channel.raid`: a regra de raid dispara **exatamente uma vez** (reentrega do EventSub deduplicada), a resposta usa quem raidou e o Histórico correlaciona a execução ao evento.

**Como configurar (sem mensagens automáticas):** Automações → Novo fluxo → gatilho **Raid** → etapa **Ação na Twitch → Destaque de canal** (deixe o alvo vazio: será quem raidou) → opcionalmente **Enviar mensagem** com `{{raider.name}}`/`{{raid.viewers}}`. Nada é publicado até você ativar o fluxo.

**Evidência:** aprovado em teste automatizado (payload real + dedupe + alvo). **Pendente:** verificação durante live real (o destaque público depende da sua autorização de teste).

## 3. P0 — Menções ao bot sem resposta

**Causas encontradas (comprovadas no código):**
1. **Consumidores silenciosos antes das automações:** regras por palavra (Respostas TXT) e comandos de módulos **consomem a mensagem e voltam** sem deixar rastro — a automação de menção nunca roda e nada explica. É o cenário do "frequentemente não responde".
2. **`!setgame`/`!settitle` para moderadores:** um `return` incondicional contradizia o próprio comentário — quem criou um fluxo com o mesmo comando nunca o executava. Corrigido: o comando nativo só volta quando **não** há automação própria.
3. **Sem rastro de "nenhuma automação casou":** mensagens de comando sem automação correspondente não geravam nenhuma linha.
4. **Nomes do bot digitados à mão** (o login real nem é armazenado) — um nome divergente do login nunca casa, silenciosamente.

**Correções:**
- Consumo por regra TXT, módulo ou comando natural agora **fica registrado** no Histórico ("Regra por palavra respondeu; automações não processaram esta mensagem", etc.).
- Mensagens de comando sem automação registradas ("Nenhuma automação ativa casou com !xyz").
- Correlação evento→execução: a linha da execução traz o id do evento (`evento 8f3a12cd`), para ligar "o que aconteceu" ao evento recebido.
- O caminho de menção já tem **resposta de reserva de IA** (v0.20.10/11) — falha do provedor não silencia mais o comando quando a reserva está preenchida.

**Evidência:** aprovado em teste automatizado (menção: palavra inteira, caixa e pontuação já cobertas por teste) + comprovado por código (consumidores). **Pendente:** observação em live real; a política de menção (responder sempre que marcado) depende da regra criada com os nomes corretos do bot.

## 4. Demais problemas encontrados (severidade)

| Problema | Severidade | Situação |
|---|---|---|
| Consumo silencioso por TXT/módulos | Alta (causa dos P0s) | ✅ corrigido (rastro no Histórico) |
| `!setgame`/`!settitle` engolindo fluxos próprios | Média | ✅ corrigido |
| Sem log "nenhuma automação casou" | Média | ✅ corrigido |
| Sem correlação evento→execução | Média | ✅ corrigido (id do evento na linha) |
| Shoutout sem escopo cobrado | Média | ✅ corrigido |
| Fallback de IA de **perfil** é campo morto (nunca publicado) | Baixa | 📋 pendente (o da ação funciona; remover ou ligar o do perfil) |
| Eventos sem `metadata.message_id` descartados em silêncio | Baixa | 📋 pendente |
| `custom` só casa com `kind=="custom"` literal | Baixa | 📋 pendente (documentar) |
| Gatilhos `live`/`unlive`/`category` existem como evento, não como gatilho | Baixa | 📋 pendente (catalogar) |

## 5. Matriz de funções (estado de cobertura)

Conexão/reconexão (EventSub + chat) — reconexão com backoff testada em código; **live real pendente**. Perfis e isolamento — aprovado (testes). Comandos/aliases/permissões — aprovado (93+ testes de tela/aceitação). Modelos prontos e consultas à Twitch — aprovado + evidência real (`!followage`). Respostas TXT — consumo agora rastreado. IA/menções/memória — aprovado (fallback testado); observação real pendente. Timers — aprovado (testes de tela). Automações/filtros por evento — raid/follow/sub/cheer via `kind`; matrizes nos testes. Recompensas (reward id) — aprovado (v0.17). Sons/TTS/OBS — aprovado; audição real é sua. Voicemod fila/restauração — aprovado (v0.20.x). Alertas/metas/chat/Spotify — aprovado. Contadores/pontos/loja — aprovado (módulos). Discord — aprovado (testes). Presets/duplicação — aprovado. Persistência/backup — testes de restore existem; **restauração em ambiente isolado pendente**.

## 6. Correções realizadas (resumo)

1. Destaque de raid usa quem mandou a raid (canal de origem) — com menção explícita vencendo.
2. `shoutout` cobra `moderator:manage:shoutouts`.
3. Consumo por TXT/módulo/comando natural registrado no Histórico.
4. `!setgame`/`!settitle` deixam os fluxos próprios executarem.
5. "Nenhuma automação ativa casou com !comando" registrado.
6. Execuções trazem o id do evento (correlação).

## 7. Testes e evidências

- **Rust: 389 aprovados**, incluindo `o_destaque_da_raid_usa_quem_mandou_a_raid` e `raid_dispara_o_fluxo_de_raid_uma_vez_so` (payload real do EventSub, disparo único com reentrega, resposta por quem raidou, correlação no Histórico).
- **Interface: 161** e **tela: 96** inalterados (nenhuma tela mudou nesta versão).
- Classificação: correções **aprovadas em teste automatizado**; caminho de API da Twitch **verificado com serviço real** (evidência do `!followage`); raid/menção **pendentes de validação operacional na live**.

## 8. Limitações e pendências

- Mocks não comprovam entrega pública: o destaque real precisa de um teste seu, com sua autorização.
- Fallback de IA do perfil é campo morto — decisão pendente (remover ou ligar).
- Diagnóstico por interface ("Por que não respondeu?") usa o Histórico; uma leitura dedicada fica para a próxima entrega.

## 9. Roteiro curto antes da próxima live

1. Atualize para esta versão e confirme **Perfis**: conta do **bot** reautorizada (inclui `moderator:manage:shoutouts` e leitura de chatters).
2. Crie (ou confira) o fluxo de raid: gatilho **Raid** → **Destaque de canal** (alvo vazio) → mensagem com `{{raider.name}}` — e **ative**.
3. Confira a regra de menção com os **logins reais** do bot (ex.: `ArrobaSrv`), separados por vírgula.
4. Teste de conta diferente do bot: `@ArrobaSrv` no chat → a resposta sai; se não sair, o Histórico agora diz **por quê** (regra por palavra, módulo, cooldown, sem automação ou erro de envio).
5. Para a raid: aguarde (ou peça) uma raid de teste com a live ligada e observe o destaque — é a validação operacional que falta.
