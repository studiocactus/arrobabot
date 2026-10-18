# Auditoria do BotLive — 17/10/2026

Base inspecionada: versão 0.9.0. Auditoria local de código, testes e contratos de integração. Não representa homologação de contas reais nem garantia de ausência de defeitos. Nenhuma live, punição, mensagem externa, release ou configuração pessoal foi acionada.

## Resultado executivo

O projeto tem cobertura automatizada útil e compila no frontend. Esta auditoria corrigiu os achados de concorrência do Spotify e da voz, os achados médios da tela Spotify e parte dos avisos de dependências. Resta como limitação estrutural a restauração de backup não atômica (item Alta descrito abaixo, documentado no manual) e a validação com ambientes reais (contas, OBS, Voicemod), que não faz parte de uma auditoria local. As correções do estúdio Spotify — zoom de 10% a 100%, modelos do cartão, seções e corretor de cor — estão neste checkout, validadas na seção abaixo.

## Correções desta auditoria (concluídas na junta Spotify/auditoria, versão 0.10.0)

- **Concorrência Spotify (`spotify.rs`):** uma leitura por perfil em voo (trava com liberação automática mesmo se a tarefa cair), geração de sessão capturada antes da rede: `poll_once` descarta a resposta que chega depois de desconectar/trocar de conta (sem repovoar a faixa nem publicar evento atrasado) e `token` não reescreve credenciais apagadas quando a renovação termina tarde. `spotify.connect`, `spotify.disconnect` e `delete_profile` avançam a geração. Coberto por teste unitário da trava e por cenas de integração no teste do Spotify (consulta em voo sem chamada extra, desconexão no meio, renovação interrompida sem regravar o cofre).
- **Captura de voz por botão (`voice.rs`):** a submissão do evento confere a geração da sessão; transcrição que chega depois de parar a escuta ou trocar de perfil é descartada, como já acontecia na escuta contínua.
- **Carregamento e autosave entre perfis (`Spotify.tsx`):** resposta da tela trocada é descartada (só o perfil ativo escreve na tela) e o autosave guarda o perfil de origem — perfil anterior não contamina o salvamento do novo.
- **ResizeObserver (`Spotify.tsx`):** o efeito agora observa o quadro quando a configuração aparece (dependência `[cfg]`), em vez de rodar uma vez com ref possivelmente nula.
- **Zoom e estúdio (`Spotify.tsx`, `overlay-studio.css`, `spotifyOverlay.ts`):** zoom de 10% a 100% com roda (ponto sob o cursor estável), indicador, −/+, "Ajustar à área" e "100%", arraste e barras internas sem rolagem horizontal da página, estado do zoom fora do design e persistido em `localStorage` (o overlay do OBS nunca recebe escala); modelos do cartão (Só a música, Uma linha, Compacto, Completo, Personalizado sem travar edição), seções Conteúdo/Layout/Aparência/Comportamento/Ajustes avançados, corretor circular com hex, amostra identificada como demonstração e prévia fiel ao OBS pelo renderizador compartilhado.
- **Dependências:** `npm audit fix` (sem mudança major) corrigiu o `source-map-js`. Vitest segura em 3.2.7: as três ocorrências restantes pertencem à cadeia de testes e exigiriam major 4.x, decisão registrada nas notas da 0.10.0.

## Correções locais desta auditoria

- **Retenção de backup:** o reconhecimento anterior aceitava qualquer nome iniciado por `botlive-` e uma data, inclusive `botlive-2000-01-01-notas.txt`. Esse arquivo podia entrar na retenção e ser apagado. Agora exige a extensão completa, sufixo numérico válido quando presente e arquivo regular. Testes cobrem nomes semelhantes e arquivos alheios à retenção.
- **Restauração sem cópia de segurança:** a operação prosseguia quando `safety_copy` retornava None. Agora cancela antes da troca do banco. Isso não torna a restauração de banco e arquivos uma transação única.
- **Tempos de audiência omitidos:** `watch_time`, usada pelos rótulos, não fazia parte do backup. Agora integra o formato 2, com teste de restauração. Backups de formato 1 continuam aceitos, sem inventar os tempos que nunca foram exportados.
- **Handshake local fragmentado:** a inspeção WebSocket usava `peek` como se consumisse os bytes, acumulando repetidamente o mesmo trecho. Agora cada inspeção substitui o buffer, tem prazo total e cede execução entre tentativas. Teste local envia o cabeçalho em duas partes e verifica que os bytes permanecem disponíveis.
- **Desconexão Twitch:** as sessões bot/canal eram tarefas destacadas; abortar a tarefa pai não as encerrava. Agora são futures pertencentes à conexão, canceladas junto com ela. Ainda requer homologação de desconexão/reconexão em conta real.
- **Auditoria visual:** os achados eram somente escritos em relatório, sem asserção de falha. Agora problemas geométricos, erros de página e falhas de navegação reprovam o teste. Spotify foi incluído na lista e no contrato simulado.

Arquivos: `src-tauri/src/backup.rs`, `local_api.rs`, `platforms.rs`, `tests/ui/layout.spec.ts`, `tests/ui/ux-helpers.ts` e capítulo de backup.

## Achados ainda pendentes

| Prioridade | Achado e evidência | Consequência / próxima ação |
|---|---|---|
| Alta | `backup.rs`: write_tables confirma a transação antes de write_files, que remove pastas e regrava arquivos. As conexões são encerradas somente depois. | **PENDENTE.** Falha de disco/permissão pode deixar banco novo com arquivos incompletos. Preparar conteúdo em staging, interromper atividades e implementar recuperação coordenada antes de declarar restauração atômica. Limitação já consta no manual e nas notas da 0.10.0. |
| Alta | `spotify.rs`: run cria uma tarefa por perfil a cada cinco segundos; o cliente HTTP admite chamadas de até 30 segundos. poll_once publica e token grava credenciais sem geração de sessão. | **CORRIGIDO na 0.10.0:** exclusão por perfil (trava com liberação automática), geração de sessão em consulta e renovação, descarte após desconexão/troca de conta. Teste unitário da trava e cenas de integração (consulta em voo sem chamada extra, desconexão no meio, renovação interrompida sem regravar o cofre). |
| Alta | `voice.rs`: transcrição pelo botão submete evento após a chamada HTTP sem conferir a geração da sessão usada em `listen.rs`. | **CORRIGIDO na 0.10.0:** a submissão confere a geração; resultado atrasado é descartado, como na escuta contínua. Homologação com microfone real continua pendente. |
| Média | `Spotify.tsx`: carregamento assíncrono sem descarte da resposta do perfil anterior; autosave compartilha refs entre perfis. | **CORRIGIDO na 0.10.0:** flag de perfil ativo descarta resposta invertida e o autosave guarda o perfil de origem. Testes de interface cobrem troca de perfil. |
| Média | `Spotify.tsx`: ResizeObserver em efeito com dependências vazias, enquanto o elemento da prévia só aparece depois da configuração. | **CORRIGIDO na 0.10.0:** o efeito observa o quadro quando a configuração chega (`[cfg]`); o zoom manual (10% a 100%) foi implementado no mesmo trabalho. |
| Média | npm audit encontrou quatro pacotes sinalizados, todos na árvore de desenvolvimento. | **PARCIALMENTE CORRIGIDO na 0.10.0:** `npm audit fix` sem major corrigiu o `source-map-js`; restam 3 ocorrências da cadeia do Vitest (2 críticas, 1 moderada), mantidas de propósito por exigirem major 4.x. Árvore de produção sem alertas. |
| Média | Auditoria geométrica não detecta todo problema semântico ou estético. | **ATENDIDO na 0.10.0:** testes específicos do Spotify (zoom, presets, seções, prévia fiel ao OBS) somam-se à auditoria de layout; a revisão visual humana continua recomendada. |

Os riscos de concorrência acima são conclusões da leitura dos caminhos de código, não reproduções contra contas reais. As condições e correções propostas estão descritas para orientar regressões determinísticas.

## Validação executada

| Checagem | Resultado |
|---|---|
| `npm run check` | Aprovado (reexecutado na base 0.10.0). |
| `npm test` | 129 testes, 15 arquivos, aprovados (era 124 na base 0.9.0). |
| `npm run test:updates` | 9 testes aprovados. |
| `npx playwright test` | 85 testes aprovados; contratos desktop simulados, navegador Edge. |
| `npx playwright test tests/ui/spotify.spec.ts` | 5 testes aprovados — zoom 10–100% (roda, teclado, persistência fora do design), presets e seções, prévia fiel ao OBS, conexão/leitura/variáveis com autosave completo, erro de autorização. |
| `npx playwright test tests/ui/layout.spec.ts` | Auditoria de layout em cinco larguras (1280, 1440, 1152, 922 e 680 px), zero achados geométricos, zero erros de página. |
| Build de produção | Executado pelo webServer do Playwright, aprovado; aviso de chunk superior a 500 kB. |
| Rust | `node .tools/run.cjs cargo test --manifest-path src-tauri/Cargo.toml -j 1`: **336 testes aprovados, 1 ignorado, 0 falhas** — inclui os testes novos de concorrência do Spotify (trava por perfil, consulta em voo sem chamada extra, descarte de resposta tardia e renovação interrompida). O executável cargo não está no PATH padrão; o wrapper força modo offline. |
| `npm audit` | 3 ocorrências (2 críticas, 1 moderada), todas na cadeia do Vitest em árvore de desenvolvimento, após `npm audit fix` sem major. Classificações do registro, não confirmação de exploração no BotLive. |
| `npm audit --omit=dev` | Nenhum alerta na árvore npm de produção. Não cobre crates Rust, dependências nativas ou vulnerabilidades ainda não catalogadas. |
| `npm run update:check` | Executado antes do commit da 0.10.0. |

Os testes de interface cobrem perfis, comandos, timers, variáveis, automações, ensaio, alertas, chat/overlay, áudio, voz, Spotify, Discord, backup, atualização, manual, pré-voo e navegação. Usam dados de teste; não validam OAuth real, microfone ou reprodução nativa.

## Avisos de dependências

- `@vitest/mocker` 3.2.7 / Vitest 3.2.7: https://github.com/advisories/GHSA-82fw-gwwq-j7x9
- `tinypool` 1.1.1, via Vitest: https://github.com/advisories/GHSA-5gmw-xhrv-c9v3 e https://github.com/advisories/GHSA-85c8-ppgw-ccpr

Vitest herda a classificação crítica de suas dependências; três ocorrências não significam três falhas independentes. O registro sugere Vitest 4.1.12, uma atualização major que exige validar configuração e testes — mantida em 3.2.7 nesta junta, decisão registrada nas notas da 0.10.0. O `source-map-js` (via Vite/PostCSS, GHSA-68fv-2mgg-jv7q) foi corrigido por `npm audit fix` sem mudança major.

## Homologação online/nativa pendente

1. Twitch: OAuth de bot/canal, renovação/revogação, desconexão das duas sessões, EventSub duplicado, perda de rede e recuperação. Resgates devem usar recompensa de teste autorizada; moderação deve permanecer simulada até autorização específica.
2. Spotify: OAuth, faixas reais, pausa/seek/troca, 204, 401, 429 com Retry-After, respostas lentas e desconexão durante renovação.
3. Voicemod e voz: aplicativo real, chave válida, microfone físico, efeitos, restauração, queda/reconexão, STT local e descarte de falas antigas.
4. OBS: fonte real, WebView2, microfone virtual, ausência de duplicação e áudio com/sem rede. O navegador de testes não prova qualidade ou latência de áudio.
5. Discord, YouTube, Kick e LivePix: contratos reais, permissões, limites e reconexão; a ponte pública de Kick continua dependência externa.
6. IA: provedores configurados, indisponibilidade, tempo limite, limites de uso e isolamento do conteúdo. Não foram feitas chamadas pagas.
7. Backup/atualizador: disco cheio, acesso negado, restauração interrompida, máquina limpa, instalador assinado e recuperação. Não houve instalação nem publicação.

## Artefatos e manutenção

- Relatórios npm: `artifacts/auditoria-npm.json` e `artifacts/auditoria-npm-producao.json` (ignorados pelo Git).
- Capturas Spotify a 1100px e 1440px: `artifacts/spotify-1100.png` e `artifacts/spotify-wide.png`.
- Relatório geométrico e capturas das telas: `%TEMP%/botlive-ux/` (fora do repositório, como manda a casa).
- Versão 0.10.0 preparada com `npm run update:prepare` e notas em `updates/0.10.0.md`; `npm run update:check` executado antes do commit local. **Não houve push nem publicação** — a release é decisão do dono do repositório.

A limpeza foi limitada a correções com impacto identificável. Reformatação ampla, remoção de código sem prova de desuso e atualização forçada de dependências não são evidência de estabilidade.
