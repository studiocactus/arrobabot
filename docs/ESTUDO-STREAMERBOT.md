# Evolução do BotLive a partir da API do Streamer.bot

## Escopo e evidências

Consulta em 29/10/2026. Base comparada: BotLive v0.20.18. O inventário estruturado em docs/reference/streamerbot-inventory.json contém nomes, caminhos, identificadores da fonte e estado da revisão individual. Não é uma implementação de compatibilidade com Streamer.bot.

A árvore oficial consultada contém 421 arquivos de subações, 454 de gatilhos e 202 de métodos C#. O índice publicado consultado anuncia 457 subações e 467 gatilhos. São contagens de superfícies diferentes; não foram reconciliadas e não comprovam cobertura equivalente. Os 29 contratos de subações OBS foram consultados; grande parte do restante exige revisão individual. Um item contract-review-required não significa ausência: significa equivalência ainda não certificada.

Fontes primárias: [referência da API](https://docs.streamer.bot/api), [subações](https://docs.streamer.bot/api/sub-actions), [gatilhos](https://docs.streamer.bot/api/triggers), [métodos C#](https://docs.streamer.bot/api/csharp), [WebSocket](https://docs.streamer.bot/api/websocket), [HTTP](https://docs.streamer.bot/api/http), [UDP](https://docs.streamer.bot/api/udp) e [repositório oficial da documentação](https://github.com/Streamerbot/docs/tree/main/streamerbot/3.api).

## O que aproveitar

O principal ganho é combinar um catálogo claro de eventos e ações com execução previsível. Acrescentar opções sem mostrar pré-requisitos, resultados e falhas aumenta a dificuldade de configuração. Cada função nova deve aparecer no editor existente, oferecer nomes e parâmetros claros, usar o mesmo motor de execução e ter uma receita no manual.

| Família | Base encontrada no BotLive | Ampliação necessária |
|---|---|---|
| Motor de automações | fluxos, condições, espera, variáveis, referências, ensaio, intervalos e cancelamento | comparar contratos de filas, grupos, chamadas e argumentos; documentar diferenças |
| OBS | conexão 5.x, descoberta, oito operações, efeitos temporários e telemetria | transformações, mídia, filtros, conteúdo, cenas, transições, grupos, gravação e replay |
| Twitch | chat, eventos, recompensas e diversas ações de canal/moderação | revisar os 95 contratos de subações e os gatilhos individualmente |
| Kick | perfil e conexão existentes | revisar 14 subações, eventos oficiais, moderação e atualização de canal; testar conta real |
| YouTube | perfil e conexão existentes | revisar nove subações, chat e eventos; tratar cotas e permissões |
| Equipamentos | eventos por ponte local | atalhos do Fifine D6 PRO_1B36; sensor cardíaco separado |
| Serviços externos | webhooks e integrações específicas | conectores com autenticação, reconexão e estados explícitos; não presumir SDK disponível |
| API para terceiros | WebSocket local de eventos e fontes HTTP | contratos versionados de requisições, respostas, assinaturas e erros; separar capacidades de leitura e controle |
| Scripts | Rhai | documentar linguagem própria; métodos C# não são automaticamente compatíveis |

Os 202 métodos C# não são 202 recursos independentes necessariamente ausentes. Há métodos que representam funções também expostas por subações. A equivalência deve considerar comportamento e não apenas nomes.

## Primeira implementação no código

O catálogo OBS existente passa a incluir seis comandos de mídia (reproduzir, pausar, parar, reiniciar, próxima e anterior), rotação absoluta e espelhamento horizontal/vertical. Reutiliza fila por destino, cancelamento e bloqueio de recursos com efeitos temporários. Não cria um segundo painel de automação. Os comandos dependem de suporte da fonte; próxima/anterior não equivalem a qualquer playlist arbitrária.

O módulo Batimentos da live usa uma ponte Bluetooth local no navegador, uma chave de publicação e outra de leitura pública, publicação desligada por padrão, ausência de histórico próprio e invalidação de leitura antiga. É uma implementação de protocolo; a compatibilidade física com o Amazfit Bip 6 do proprietário continua pendente. Consulte [Batimentos da live](15-BATIMENTOS.md).

## Próximos blocos e critérios de conclusão

O segundo bloco acrescenta seis operações de filtro/conteúdo, seleção explícita de cena/grupo, descoberta de fontes compatíveis e três receitas com formulário simples, sem duplicar o motor. Consulte [Receitas e controles do OBS](16-OBS-AUTOMACOES.md). Transições, gravação, replay, D6 e ampliação das plataformas continuam pendentes.

1. Completar OBS: filtros com descoberta, edição tipada de texto/imagem/navegador, grupos e seleção de cena explícita; depois transições, gravação e replay. Cada comando deve preservar campos não alterados, propagar erro e respeitar o motor existente.
2. Implementar o controle do D6 por atalhos configuráveis, sem remover o software da Fifine. A [página oficial de software do D6](https://fifinemicrophone.com/pages/download-fifine-d6-software) descreve atalhos e exige o aplicativo do fabricante, mas não confirma SDK público; conexão USB direta não está homologada. Evitar atalhos duplicados, exigir escolha do perfil e impedir disparos durante captura de configuração.
3. Completar Kick e YouTube a partir de APIs oficiais. Eventos por webhook exigem um destino HTTPS acessível; a API local 127.0.0.1 não satisfaz esse requisito. Documentar claramente funções indisponíveis por permissão ou cota.
4. Revisar e classificar os contratos restantes do inventário, com evidência de arquivo, operação equivalente, diferenças, testes e limitações. Não marcar uma família inteira como concluída por ter um conector básico.
5. Expandir o manual por categorias de eventos, ações, variáveis e API. Cada entrada deve conter parâmetros, resultado, permissões, exemplo, erros, teste simulado e teste real pendente.

## Qualidade exigida

Testes de protocolo devem cobrir resposta negativa, reconexão, cancelamento, fila cheia, dados malformados e isolamento entre perfis. Testes visuais devem cobrir 1100px e janela larga, sem rolagem horizontal, mantendo o padrão compartilhado de prévia e ajustes. Ensaio não deve produzir efeitos na live.

Para o relógio: verificar serviço anunciado, contato, perda de conexão, leitura antiga, revogação e diferença entre prévia privada e OBS. Para o D6: verificar combinação de teclas, foco em outro aplicativo, repetição e ausência de disparo duplicado. Para contas: testar permissões mínimas e revogadas. Resultados de simuladores não substituem esses testes físicos.

## Situação

Este estudo é uma base rastreável e uma entrega incremental. A paridade completa com Streamer.bot ainda não está implementada ou homologada. Os blocos futuros acima permanecem trabalho aberto; o inventário não deve ser apresentado como lista de funcionalidades disponíveis no BotLive.
