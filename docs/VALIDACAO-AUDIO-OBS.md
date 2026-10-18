# Validação de áudio no OBS — entrega 0.11.0

Data: 18/10/2026. Commit auditado: `a7487b0453bbeaef13290acfa04ee63e8ae92da0`.
Base anterior: 0.10.0, `1a722fb`. Checkout limpo antes desta documentação; versão, commit e notas `updates/0.11.0.md` conferidos. AGENTS.md e documentos de auditoria/handoff consultados.

## Resultado: NÃO HOMOLOGADO para live

A entrega implementa uma fonte de navegador dedicada para os sons do bot, com Voicemod separado para o microfone. Entretanto, há um bloqueio identificado por revisão de código nas URLs dos sons e do TTS, além de riscos na reconexão. Nenhuma gravação real do OBS foi produzida nesta auditoria. Não há garantia de audição pública.

## Achados para o Open Code

1. **Bloqueante: chave serializada com aspas nas URLs de mídia.** `src-tauri/src/audio.rs:221` (`asset_src`) e `src-tauri/src/tts.rs:192` (`url`) interpolam `info["token"]`, um `serde_json::Value`, diretamente em `format!`. O token é uma string JSON de 64 caracteres, criada em `livepix::overlay_info`; sua representação JSON inclui aspas. A query resultante tem 66 caracteres, ou 70 quando as aspas são codificadas pelo navegador. `src-tauri/src/local_api.rs:243` aceita no máximo 64 e não decodifica a query; a autenticação também exige igualdade com a chave original. Consequência prevista pelo código: rejeição das requisições de `/asset` e `/tts`. O bipe inline não exercita essas rotas e não demonstra que arquivos/TTS funcionam. Extrair a string com `as_str()` e tratar ausência/tipo inválido. Adicionar regressões que usem as URLs geradas nas rotas autenticadas reais, com credenciais e arquivos temporários de teste; verificar resposta e conteúdo WAV, não apenas prefixo/sufixo da URL.
2. **Alto: reprodução não é cancelada quando a fonte perde conexão ou entra em espera.** Em `src/audioSourceApp.ts`, `onclose` e `audio.standby` alteram `claimed`, mas não chamam `player.stopAll()`. Áudio já carregado e a fila podem continuar enquanto outra fonte assume ou o backend seleciona reprodução local. Cancelar reprodução e fila ao perder a posse; testar queda durante um som, promoção de segunda fonte e reconexão sem sobreposição ou repetição. Risco identificado no código, não reproduzido em OBS real.
3. **Fila antiga e limites precisam de validação operacional.** `src/audioSource.ts` verifica idade apenas ao receber, não ao iniciar cada item. Sons podem ficar mais de dez segundos na fila e tocar depois. O teto de 15 segundos interrompe sons/TTS longos; cinco pendentes podem ser insuficientes para rajadas de resgates. Definir comportamento explícito e testar idade no momento da reprodução, saturação e interrupção.
4. **Fallback local exige cuidado.** Sem fonte conectada, o backend encaminha para o aplicativo. Isso não comprova presença no fader dedicado do OBS. Validar transição local→OBS durante reprodução e evitar que captura de desktop duplique o áudio. Deixar a interface explícita quando a saída pública não estiver disponível.

## Evidências executadas nesta auditoria

- `npm test -- --run src/audioSource.test.ts src/viewerSound.test.ts`: 14 testes passaram (10 + 4). São testes automatizados; não atestam audição no OBS.
- `npm run check`: concluído sem erro de TypeScript.
- `npm run update:check`: versões e registros válidos.
- Reprodução sintética da formação da query com Node e token fictício de 64 caracteres: representação JSON = 66; query enviada pelo URL parser = 70; limite do servidor rejeita; igualdade com token original falsa. Não foi uma requisição HTTP ao BotLive nem uma execução Rust.
- Revisão dos caminhos `audio::asset_src`, `tts::url`, consumidor da fonte, fila e autenticação local.
- Consulta de processos pelos nomes `BotLive`, `obs64` e `Voicemod` não encontrou processos. Isso não exclui executáveis renomeados. Executável efetivamente usado pelo usuário, versão instalada, dispositivos e cenas não foram confirmados.
- As suítes Rust/Playwright/SAPI informadas nas notas da entrega são evidências declaradas pelo Open Code, não reexecutadas nesta auditoria. Nenhum build, commit ou push realizado; código da entrega preservado.

## Status por origem

| Origem | Evidência atual | Status de audição real |
|---|---|---|
| Resgates com som | Caminho de arquivos revisado; bloqueio na URL | Pendente; bloqueado pelo achado 1 |
| Comandos com som | Caminho compartilhado de arquivos revisado | Pendente; bloqueado pelo achado 1 |
| Timers com som | Arquitetura compartilhada; sem disparo real | Pendente; bloqueado quando usa arquivo |
| Alertas com som | Uso de `audio::asset_src` identificado | Pendente; bloqueado pelo achado 1 |
| TTS | Geração e URL revisadas; sem fala gravada | Pendente; bloqueado pelo achado 1 |
| Bipe de diagnóstico | Caminho inline distinto; testes do consumidor | Pendente; não comprova demais origens |
| Voicemod | Microfone separado previsto na entrega | Pendente: fala, mudança e retorno de voz reais |
| STT | Microfone físico é requisito | Pendente: dispositivo e reconhecimento reais |

Nenhuma origem recebeu aprovação real. Os testes simulados do reprodutor não equivalem a resgate simulado de ponta a ponta.

## Critérios para encerrar a homologação

Após corrigir os achados, confirmar caminho/versão do executável aberto e ausência de live ou gravação em andamento. Com OBS e Voicemod disponíveis e participação do usuário para falar, gravar localmente cada origem isolada e em sequência. Simular resgate pelo mesmo handler/fila de produção, sem gastar pontos nem enviar mensagens públicas. Verificar no arquivo gravado áudio de resgates, comandos, timers, alertas e TTS no mesmo fader; mute e volume da fonte devem afetar todos eles.

Testar troca de cena, segunda fonte, desconexão/reconexão durante reprodução, fila cheia, interrupção, arquivos inválidos e troca de perfil. Confirmar ausência de duplicação, replay e vazamento entre perfis. Gravar microfone Voicemod normal, alterado e restaurado; conferir STT captando microfone físico. Ouvir a gravação e registrar caminho local, configuração de faixas e resultados por origem, sem versionar artefatos ou credenciais.

Pendências humanas: disponibilizar o ambiente OBS/Voicemod e voz para a gravação controlada, além de confirmar o executável usado. Credenciais, se necessárias, devem ser configuradas localmente sem publicação. Não iniciar live como teste.

Este documento é o handoff para o Open Code; nenhuma mensagem foi enviada diretamente a ele. Monitor encerrado após esta entrega para evitar repetir testes do mesmo commit. Retomar a validação após correções e disponibilidade operacional.

## Revisão após as correções — 18/10/2026

Os quatro achados acima foram corrigidos nesta entrega (0.12.0), com testes de regressão que exercitam os caminhos reais — a auditoria original permanece registrada acima como histórico.

1. **URLs de mídia (bloqueante) — corrigido.** O token passou a ser extraído como texto (`audio::overlay_addr`) em `asset_src`, `tts::url` e no endereço da fonte (`audio.get`); sem interpolar o `serde_json::Value`, não há mais aspas na query. Regressão `urls_de_som_e_tts_funcionam_nas_rotas_autenticadas_de_verdade` (integration_tests.rs) sobe a API local numa porta livre e faz **requisições HTTP reais** em `/asset` e `/tts` com arquivos WAV do perfil, conferindo resposta 200 e corpo WAV — além de autenticar o token do endereço da fonte de verdade.
2. **Perda de conexão/standby — corrigido.** `audioSourceApp.ts` chama `player.stopAll()` ao perder a posse (`audio.standby`) e ao cair o WebSocket: som atual e fila param na hora, sem sobreposição com a fonte que assume e sem repetição na reconexão. Coberto pelos testes do reprodutor (“parar tudo pausa o som atual…”, geração invalidando a fila) e pela promoção de segunda fonte testada no backend.
3. **Idade da fila e limites — definido e testado.** A idade (10 s) é conferida na chegada **e de novo no momento de tocar**; o prazo ficou explícito: **15 s para iniciar** e teto de **60 s** para som já iniciado (falas longas de TTS cabem). Saturação (5 pendentes, aviso “Fila de sons cheia”) e interrupção têm teste próprio em `audioSource.test.ts`.
4. **Fallback local — explícito.** A tela Áudio no OBS mostra “Última reprodução” dizendo se o som foi **pela fonte do OBS** ou **pelo aplicativo — a fonte do OBS não estava conectada**; a migração continua mandando remover a captura antiga de cabo para não duplicar áudio. A transição local→OBS durante reprodução está coberta pelo teste de registro de fonte no meio da fila (o app nunca toca junto quando há fonte dona).

**Status da homologação real: segue pendente.** Nenhuma gravação em OBS real nem audição com Voicemod/microfone reais foi executada nesta máquina (sem OBS/Voicemod disponíveis e sem participação humana para falar). Os critérios da seção anterior continuam valendo: nada é declarado homologado antes da gravação controlada com o executável confirmado.