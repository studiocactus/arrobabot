# Auditoria visual e de usabilidade — 0.13.0

Data: 18/10/2026. Base auditada: `2b83cf2` (BotLive 0.12.0), com a simplificação do Voicemod (0.12.0) como ponto de partida. Método: leitura de `AGENTS.md`, dos componentes compartilhados (`src/components.tsx`), dos estilos globais (`src/styles.css`, `src/overlay-studio.css`) e das notas da versão atual; inventário das telas pela navegação (`tests/ui/ux-helpers.ts`); inspeção do aplicativo rodando no navegador com captura em três larguras; auditoria geométrica automatizada (`tests/ui/auditPage` nas cinco larguras de sempre).

## Telas e estados inspecionados

- **23 telas da navegação** em 1100 px (Visão geral, Perfis, Discord, Rótulos OBS, Comandos, Timers, Respostas, Sons de Live, Contadores, Automações, OBS Studio, Áudio no OBS, Voicemod, Spotify, Meta, Alertas, Chat no OBS, IA, Memórias, Comunidade, Presets, Histórico, Estatísticas) + **Comandos por voz** e **Configurações**.
- **6 telas densas em 1600 px** (Automações, Meta, Spotify, Memórias, Voicemod, Áudio no OBS) e **5 abaixo de 960 px** (Automações, Meta, Spotify, Voicemod, Comandos).
- **Editores e modais**: editor de fluxos (canvas + inspetor em coluna única), modal de perfil, modal de simulação de evento.
- **Estados**: listas cheias e vazias, botões desabilitados (teste de voz sem voz escolhida), avisos de integração desligada (Meta, Spotify), prévia web sem núcleo desktop, erro de console (o único, caçado abaixo).
- **Leitura visual detalhada** (uma a uma): Visão geral, Comandos, Sons de Live, Voicemod, Automações, Comunidade, Meta, Spotify, editor de fluxos. As demais telas passaram pela auditoria geométrica e pelas capturas, sem leitura individual — ver **Limitações**.

## Problemas encontrados e prioridade

| # | Problema | Prioridade | Evidência |
|---|---|---|---|
| 1 | Histórico mostrava **“Invalid Date”** no lugar da hora dos registros: o mock de dados usava o campo `at` enquanto o backend emite `timestamp` (`db.rs`), e o formatador não tinha defesa para dado antigo. Na prévia, todo registro aparecia com a data quebrada. | **P1** | `auditoria-antes/01-visao-geral-1100.png` |
| 2 | Intervalo ilegível nas tabelas: **“0.08 min”** para 5 segundos, ainda quebrando em duas linhas por coluna estreita. | **P1** | `auditoria-antes/10-automacoes-1100.png`, `05-comandos-1100.png` |
| 3 | Ações de linha quebrando em duas linhas (o ícone de apagar caía para a linha de baixo em Automações), com colunas de largura imprevisível. | **P1/P2** | `auditoria-antes/10-automacoes-1100.png` |
| 4 | Seletores sem o valor salvo (arquivo de som/TXT apagado) renderizavam **linha em branco**, sem explicar que o item não existe mais. | **P2** | Código: `Sounds/ChatExtras/Alertas/ChatObs` |
| 5 | Formulário com **Salvar** explícito (Sons por espectador) sem indicar alterações pendentes. | **P2** | `auditoria-antes/08-sons-de-live-1100.png` |
| 6 | Amostras de cor escuras invisíveis no Estúdio da Meta (“Cor da borda”, “Fundo do painel”) sem o valor hexadecimal visível. | **P2** | `auditoria-antes/15-meta-1100.png` |
| 7 | `button.danger` só mudava a cor do texto (ação destrutiva pouco diferenciada) e `button.ghost` (Spotify) não tinha estilo próprio — caía no botão comum de 40 px. | **P2** | Código: `styles.css`, `Spotify.tsx` |
| 8 | **Erro de console** em toda carga: `404` no favicon (o `index.html` não declarava ícone). | **P2** | Log do console na captura |
| 9 | Texto de ajuda desatualizado no Estúdio de Alertas (“tela Áudio”), apontando para a navegação antiga. | **P2** | `Alertas.tsx` |
| 10 | Rótulos mínimos de 8 px (`.eyebrow`, `.nav-group`) no limite da leitura. | **P2** | `styles.css` |
| 11 | Erros de ação apareciam só no rodapé/toast, não junto do campo que os causava (ex.: chave da Control API vazia). | **P2** | `Voicemod.tsx` |

O que foi conferido e **não** tem problema: hierarquia de ações principais por tela (um `primary` por contexto), estados vazios com explicação e ação, modais com foco preso e Escape, `aria-label`s em botões de ícone e campos, `.form-grid` com colunas de 240 px, inspetor de fluxos em coluna única, `.lp-studio` com prévia sticky em Meta/Alertas/Chat/Spotify, `prefers-reduced-motion` global e zoom do Spotify fora do canvas lógico.

## Melhorias implementadas

1. **Data do histórico** — formatador defensivo `fmtTime` (traço `—` para dado inválido, nunca “Invalid Date”) e mock corrigido para o campo real `timestamp` (o contrato do mock passa a ser o do backend). Aceita o campo antigo `at` sem quebrar dados legados.
2. **Intervalos legíveis** — `fmtDuration` em Comandos, Automações, Timers e resumos de Respostas/Sons: **“5 s”, “15 min”, “1 h 30 min”** (o “A cada 15 min” dos timers é preservado); coluna de intervalo sem quebra.
3. **Ações de linha** — classe `.row-actions` (uma linha, alinhada à direita) e `title` (tooltip) em todos os botões de ícone da linha; ação destrutiva com borda/tinta vermelha própria (`button.danger`).
4. **Preservação de valor salvo** — componente `MissingOption`: quando o arquivo/voz escolhido não existe mais, o seletor mostra o valor com a explicação “salvo indisponível — escolha outro”, sem substituir em silêncio (Sons por espectador, Respostas TXT/som, Alertas, Chat no OBS).
5. **Estado de salvamento** — pílula “Alterações não salvas / Tudo salvo” no formulário de Sons por espectador (padrão já usado em Memórias).
6. **Amostras de cor** — valor hexadecimal visível em Meta, Alertas e Chat no OBS (mesmo padrão do Spotify); a amostra segue redonda, 34×34, sem encolher.
7. **Botões** — `button.danger` diferenciado de verdade; `button.ghost` padronizado como botão de ícone (32 px).
8. **Console limpo** — favicon embutido (data-URI, sem arquivo servido): zero erros de console nas capturas finais.
9. **Campos e erros** — `Field` ganhou `error` (mensagem junto do campo, com `role="alert"`); aplicado na chave da Control API da tela Voicemod.
10. **Tipografia mínima** — `.eyebrow` e `.nav-group` de 8 px para 9 px.

## Padrões visuais adotados

- Reaproveitamento dos tokens existentes (`--s1..--s7`, `--r-*`, `--h-*`, `--t-*`); nenhuma correção isolada por tela, nenhuma margem negativa, nenhum seletor específico novo fora do bloco único da auditoria em `styles.css`.
- Novas regras compartilhadas: `.row-actions`, `.nowrap`, `.save-state`, `.field-error`/`.field--error`, `.swatch-hex`, `button.danger`, `button.ghost`, `MissingOption`.
- Ações destrutivas sempre com ícone + cor + tooltip; ação principal única por contexto preservada; textos de apoio continuam fora da linha dos botões (`.form-footer`).

## Evidências

- **Antes**: `%TEMP%\botlive-ux\auditoria-antes\` (35 capturas: `NN-tela-1100.png`, `*-1600.png`, `*-900.png`).
- **Depois**: `%TEMP%\botlive-ux\auditoria-depois\` (35 capturas + `editor-fluxos-1100.png`, `editor-fluxos-inspetor-1100.png`, `modal-perfil-1100.png`, `modal-simular-1100.png`).
- Os pares mais úteis: `01-visao-geral` (data), `05-comandos` e `10-automacoes` (intervalo e ações de linha), `15-meta` (amostras de cor), `08-sons-de-live` (estado de salvamento).

## Testes executados

- `npm run check` (tsc): **limpo**.
- `npm test` (vitest): **142 testes aprovados em 16 arquivos** — inclui os novos `src/format.test.ts` (duração legível e hora nunca quebrada).
- Playwright (navegação real, Edge): **90 testes aprovados** — inclui `tests/ui/history.spec.ts` (histórico sem “Invalid Date”; seletor preservando valor salvo), mais a auditoria geométrica nas cinco larguras sem achados.
- Console do navegador: **zero erros** nas capturas finais (antes: 1× 404 do favicon).
- Os scripts de captura usados na auditoria foram **removidos** ao final (não viram teste: reproduziriam detalhe de implementação).

## Pendências e limitações

- **Não é uma auditoria “completa” nem a interface está “perfeita”**: as telas de Discord, Rótulos OBS, Contadores, Respostas, OBS Studio, Inteligência artificial, Presets, Histórico e Estatísticas foram capturadas e passaram pela auditoria geométrica, mas **não** tiveram leitura visual individual; estados com dados reais muito longos, listas com centenas de itens e janelas em 4K não foram exercitados.
- **Contraste**: conferido por leitura (texto `--muted` sobre fundos escuros/claros e variantes de status no tema claro), **sem ferramenta de medição WCAG** — AA é a intenção do sistema de tokens, não uma medição registrada.
- **Teclado**: modais têm foco preso e Escape (código revisado), mas não foi feita varredura tab-a-tab de todas as telas.
- **Zoom por scroll do Spotify** preservado como estava (controles visíveis, canvas lógico intocado); o comportamento não foi revalidado mecanicamente nesta rodada.
- Erros junto do campo (`Field error`) começaram na tela Voicemod; as demais telas ainda reportam por toast/rodapé — evolução incremental, sem reescrever formulários inteiros.
- Nada aqui atesta funcionamento de integrações reais (Twitch, OBS, Voicemod, Spotify): as capturas usam os mocks da prévia, que são distinguíveis do desktop (banner de prévia e ausência de núcleo).
