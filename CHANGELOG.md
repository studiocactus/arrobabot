# Histórico de atualizações

# BotLive 0.1.6

# BotLive 0.1.6 — manual prático, timers e contadores individuais

## O que mudou

- Manual organizado em Comece aqui, Configure o bot, Cuide da sua live e Referência técnica. Vinte capítulos, sumários locais, atalhos por tarefa, busca por capítulo e botões para copiar exemplos. Âncoras antigas preservadas.
- Novo catálogo de receitas cobre comandos, timers, contadores, variáveis, IA contextual, memória, TXT, sons, saída de áudio, pontos, loja, sorteios, previsões, fila, música, jogos, voz, Discord, moderação, ações, overlay, presets, acesso, backup e atualização. Cada receita explica configuração, entrada e resultado esperado, distinguindo simulação e uso real.
- Aba Timers e formulário simples de mensagem periódica por perfil, com nome, texto, intervalo de 30 segundos a 24 horas, ativação e simulação. Disponível também no editor visual como Timer periódico.
- Agendador aguarda o intervalo completo, executa somente em perfil conectado, mantém períodos independentes, pula sobreposições e não acumula disparos perdidos. Pausa/desconexão/edição reiniciam a espera; eventos antigos de uma configuração editada são descartados. Eventos externos reais não podem forçar timers pelo mesmo gatilho.
- Contar usos deste comando ativa um total persistente individual. A variável commandCount e o atalho Contagem do comando inserem o valor nas respostas. A lista mostra o total e permite ajustar ou zerar com confirmação.
- Contagem transacional respeita permissão, cooldown e processamento anterior. Simulações mostram o próximo valor sem gravar. O total é incrementado antes das ações e sobrevive a falhas posteriores. Contadores não compartilham dados entre perfis ou comandos.
- Banco cria tabela de contadores com remoção em cascata; comandos antigos continuam com contador desativado. Presets carregam a opção e os intervalos, sem exportar totais.
- Barra de ações adaptada para janelas menores, sem rolagem horizontal causada pelos novos botões.

## Como usar

Abra o manual e comece por Exemplos práticos. Para um lembrete, Comandos → Novo timer: nome Água, resposta Hora de beber água!, intervalo 900. Salve, ative e conecte o perfil. Aguarde quinze minutos para o primeiro envio real. Simular timer registra a prévia no Histórico sem publicar. Pause ou desconecte ao terminar a live.

Para um contador: Novo comando, nome Mortes, gatilho !mortes. Ative Contar usos deste comando e escreva Mortes registradas: {{commandCount}}. Escolha Moderadores ou Só o streamer para limitar quem incrementa. Crie !vitorias da mesma forma para outro total. Na coluna Contador, Ajustar permite definir zero ou corrigir o valor; confirme o novo total. Cada uso aceito soma um. O bot não detecta mortes ou vitórias no jogo automaticamente.

Variáveis por pessoa não se aplicam a timers sem usuário. Use canal, data e horário. Configurações de timer também podem ser combinadas com ações no editor visual; o agendamento não depende de uma mensagem no chat.

## Validação

Compilação TypeScript/Vite aprovada. Executados dez testes Vitest, 36 testes Rust, nove testes Playwright e sete testes de scripts de atualização. Os testes novos cobrem concorrência/persistência/isolamento de contadores, permissões, cooldown, simulação, timer por destino, bloqueio offline/externo, descarte após edição e intervalos sem recuperação acumulada. Após ajustes finais, os testes Rust e os dois testes específicos de interface/manual foram repetidos e passaram.

Manual gerado com vinte capítulos; verificadas âncoras, grupos, busca, tabelas, impressão, tela de 390 pixels e aparência desktop. A suíte detectou inicialmente overflow de três pixels na barra em 720 pixels; corrigido e validado. A compilação inicialmente bloqueada pelo sandbox foi repetida com permissão e passou. Não restaram falhas de teste conhecidas.

## Limitações

Timers exigem aplicativo aberto e perfil online; online não equivale a transmissão ao vivo. O Kick por ponte externa não inicia timers nativos. O agendador verifica a cada segundo; filas, limites de envio e serviços podem atrasar execução. Mensagens já enviadas não são desfeitas e uma chamada externa iniciada pode terminar após pausa. Não há calendário, horário fixo, contagem regressiva ou requisito de número mínimo de mensagens.

Contadores medem usos aceitos antes das ações, não entregas confirmadas nem eventos do jogo. Simulação não altera o total. Ajuste pelo painel é permitido a acessos autorizados ao perfil; não há comando de chat para definir ou subtrair o total. Excluir o comando exclui seu contador. Ao substituir configuração via preset, um comando existente conserva o contador associado ao seu identificador.

Os testes locais não comprovam envio periódico com contas reais nem homologação sobre a instalação aberta do usuário. Funções que dependem de IA, Twitch, OBS, áudio e serviços externos mantêm as limitações documentadas. A build instalada só recebe as mudanças depois de atualizar e reabrir.

---

# BotLive 0.1.5

# BotLive 0.1.5 — atualização visível, entrada silenciosa e saída de áudio

## O que mudou

- A lateral e o rodapé exibem a versão completa do pacote, eliminando o texto fixo v0.1.
- Configurações mostra a versão instalada, andamento da consulta, resultado persistente, erros e notas da versão. Só habilita instalação após encontrar atualização. A consulta aguarda configurações carregadas e não afirma que instalou quando nenhuma atualização foi encontrada.
- Consulta do manifesto limitada a 30 segundos e download a dez minutos. Assinatura e canal oficial preservados.
- Sons por pessoa oferecem gatilhos separados: mensagem, entrada silenciosa ou ambos; frequência uma vez por sessão ou com intervalo. Configurações antigas assumem mensagem, sem ativar monitoramento automaticamente.
- Monitor opcional Twitch IRC recebe entradas e saídas, responde keepalive, reconecta e cancela junto com o perfil. Não duplica mensagens EventSub nem executa pontos ou automações gerais. Ignora a lista inicial e deduplica entradas observadas. Solicita chat:read na autorização do bot.
- Saída de áudio dos sons selecionável por perfil, com atualização da lista e aplicação antes da reprodução. Saída indisponível gera erro sem desviar silenciosamente o som. Ambiente sem suporte orienta o Mixer do Windows.
- Manual atualizado nos capítulos de conexões, atualização e respostas/sons.

## Como usar

Em Configurações, clique em Verificar atualização, leia o resultado e use Instalar atualização quando disponível. Faça isso fora da live; o instalador pode fechar o aplicativo. Se uma versão antiga não atualizar internamente, use o instalador oficial na mesma pasta, preservando os dados.

Abra Comandos ou Comunidade → Respostas TXT e sons. Cadastre a pessoa, seu login, apelido e áudio. Escolha Disparar som e Quando tocar. Para entrada silenciosa na Twitch, ative Monitorar entradas silenciosas, salve, autorize novamente a conta do bot e conecte o perfil. Confira Monitor de entradas ativo no Histórico. O login atual é necessário mesmo quando o ID está preenchido.

Escolha Dispositivo de saída dos sons, salve e use Testar som. Capture essa saída no OBS. A seleção não altera o texto para fala. Reiniciar sessão de sons libera uma pessoa cujo primeiro disparo já foi consumido.

## Validação

Executados localmente: compilação TypeScript/Vite; dez testes Vitest, incluindo ordem de seleção/reprodução e dispositivo removido; 32 testes Rust, incluindo parsing IRC, gatilhos e migração de configuração antiga; sete testes Playwright, incluindo consulta com erro, andamento, notas, versão sem atualização e controles de som; sete testes dos scripts de atualização/publicação. Compilação e testes Node inicialmente encontraram bloqueio de criação de processos no sandbox; repetidos com permissão, passaram. Nenhuma falha de teste permaneceu.

Confirmado que o aplicativo aberto no PC estava em 0.1.3 e que a release anterior 0.1.4 tinha manifesto e instaladores públicos acessíveis. A consulta na interface foi testada com transporte simulado; não foi realizada instalação sobre o aplicativo em execução.

## Limitações

Entrada silenciosa representa conexão ao chat, não audiência do vídeo. A Twitch pode atrasar ou omitir eventos. O monitor requer nova autorização chat:read e ainda precisa de homologação com conta Twitch real. No YouTube, use mensagem; outras plataformas dependem de uma ponte join.

Dispositivos disponíveis e seleção dependem do Windows/WebView e das permissões. Reprodução física e captura no OBS não foram homologadas nesta execução. Dispositivo salvo é local ao computador. Sons mantêm limite de quinze segundos, fila de cinco, intervalo geral de cinco segundos e exigem app aberto/desbloqueado. Falha de reprodução pode consumir a primeira participação; reinicie a sessão após corrigir.

A versão instalada só muda após concluir a instalação. O erro original do atualizador instalado não foi reproduzido diretamente; foram corrigidos ausência de feedback persistente, versão fixa, falta de limite de espera e confirmação indevida de instalação. As verificações locais não substituem um teste real de atualização e áudio no ambiente da live.

---

# BotLive 0.1.4

## O que mudou

- Novo painel Respostas TXT e sons em Comandos e Comunidade, com controles de ativação geral e individual.
- Respostas automáticas por palavra ou expressão inteira ou por trecho. Arquivos TXT UTF-8 continuam vinculados externamente e são relidos no disparo; escolha aleatória sem repetição imediata ou sequencial, variáveis e intervalos por regra/pessoa.
- Cadastro de espectadores por nome ou ID, apelido, áudio WAV/MP3/OGG, volume e modo primeira participação da sessão ou mensagens com intervalo. Reinício manual da sessão de sons por perfil.
- Áudios copiados para a pasta de dados, com fila de reprodução no aplicativo, intervalo global, limite de tamanho e duração. Teste individual sem depender de uma mensagem real.
- Simulação sem publicação, reprodução ou consumo dos intervalos reais. Regras isoladas por perfil, validação de arquivos e restrições de acesso para importação e configuração.
- Manual agora com 18 capítulos, incluindo configuração dos recursos, exemplo TXT, captura de áudio no OBS, backup e limitações de presença.

## Como usar

Abra Comandos ou Comunidade > Respostas TXT e sons no desktop. Adicione uma regra, preencha a palavra, vincule o TXT, escolha o modo e ative o controle geral. Para sons, adicione a pessoa com nome do chat ou ID, apelido e áudio, ajuste volume e modo e use Testar som. Ative o controle individual e geral e clique em Salvar respostas e sons. Para a live, capture o áudio do BotLive ou da saída do desktop no OBS. O arquivo examples/respostas-cafe.txt serve como modelo inicial.

## Validação

Build TypeScript/Vite aprovado, 30 testes Rust, nove testes Vitest e seis cenários Playwright passaram localmente. Cobertura inclui leitura UTF-8 e limites, arquivo editado/apagado, seleção sequencial/aleatória, cooldown, identidade por ID/nome, cópia de áudio, controles de ativação, simulação, isolamento e permissões. Player testado com áudio controlado em memória para fila, volume, timeout e falha. O primeiro teste de interface falhou por seletor de label do select; corrigido para nome acessível, com suíte completa aprovada. Sem mensagens enviadas a contas reais durante a validação.

## Limitações

Entrada silenciosa não é detectada pelos adaptadores atuais: usa-se a primeira mensagem; evento join depende de ponte autorizada. Reprodução exige aplicativo aberto e desbloqueado. Captura e mixagem no OBS precisam ser configuradas e homologadas pelo operador; não foram testadas numa transmissão real. TXT externo precisa de backup separado e caminho válido; áudios copiados ficam em media. Recursos não são transportados pelos presets. Remover cadastros não apaga arquivos originais ou cópias de mídia. A build Windows desta versão será gerada pelo workflow após o push.

---

# BotLive 0.1.3

## O que mudou

- IA agora recebe a mensagem atual, autor e até 12 falas recentes dos últimos cinco minutos no mesmo perfil, além da personalidade e das memórias recuperadas.
- Nova ação Gerar resposta da IA (variável), sem envio automático, com nome local personalizável e marcadores local.aiResponse e local.aiSuccess. Respostas não são reinterpretadas como comandos ou variáveis.
- Botão Resenha com IA cria uma automação pronta com gatilho por trecho, orientação de humor e intervalos de 60 segundos por fluxo e 120 por pessoa.
- Campos de mensagem incluem atalhos Resposta da IA e Mensagem do chat; o catálogo tem categoria IA e nomes legíveis.
- Teste contextual permite informar mensagem e conversa anterior e chamar o provedor sem enviar ao chat ou gravar memórias. Simulação gratuita usa um marcador explícito, sem chamar o provedor.
- Contexto isolado por perfil e execução, limitado em tamanho e idade; eventos do próprio bot são ignorados quando o ID está configurado. Manual atualizado com receita, variáveis, exemplos e limites.

## Como usar

Configure e salve o provedor e a personalidade em Inteligência artificial. Abra Comandos > Resenha com IA, escolha o trecho que dispara (exemplo: amassando), ajuste o humor e use Testar resposta contextual no desktop. Salve para criar a geração seguida do envio. No editor visual, reutilize a resposta nas próximas ações clicando em + Resposta da IA; também pode encaminhar para voz ou overlay. Para citar partidas ou acontecimentos anteriores, registre esse contexto nas memórias ou forneça falas reais do chat.

## Validação

Build TypeScript/Vite aprovado; sete testes Vitest e cinco cenários Playwright passaram. Os testes Rust cobrem 26 casos, incluindo provedor HTTP local controlado, contexto por perfil, expiração, limite de histórico, variável local, texto sem expansão recursiva, fallback, simulação, prévia e autorização. O primeiro cenário novo de interface falhou por seletor de rótulo do textarea; corrigido para consultar o nome acessível do campo, com suíte completa aprovada. Não houve chamada a um modelo pago nem envio a um chat real nesta validação.

## Limitações

A qualidade, coerência e humor dependem do modelo e da personalidade; instruções não garantem ausência de alucinações. Testes com provedor controlado validam integração, não qualidade linguística de um modelo real. Requer provedor configurado no desktop. Provedores remotos recebem a conversa enviada na geração e podem cobrar por uso. Contexto recente não sobrevive ao fechamento do app; histórico normal e memórias seguem as regras existentes. A nova compilação Windows é produzida pelo workflow após o push; não há homologação de instalação em máquina limpa.

---

# BotLive 0.1.2

## O que mudou

- Corrigida a etapa final de publicação: o empacotador forneceu URLs da API do GitHub e a validação anterior recusou o manifesto da versão 0.1.1.
- O script agora consulta os arquivos reais do rascunho e converte as URLs reconhecidas para downloads públicos da mesma release, preservando as assinaturas.
- Publicação rejeita arquivos desconhecidos, assinaturas ausentes, versão/tag divergente e releases já publicadas.
- Incluídos testes de regressão no comando test:updates e explicação no manual de atualizações.

## Como usar

Baixe o instalador Windows x64 na página Releases do repositório. Nas versões com canal oficial configurado, use Configurações > Verificar atualização. Para manter o projeto, continue preparando versão e notas com npm run update:prepare antes de cada commit; a conversão dos links ocorre automaticamente na publicação.

## Validação

Os sete testes do sistema de atualização passaram localmente, incluindo links da API e URLs temporárias de rascunho, preservação da assinatura, rejeição de destinos desconhecidos e proteção de releases publicadas. Na execução anterior, os testes de aplicação e a geração dos instaladores assinados 0.1.1 passaram; a falha ocorreu somente na validação final do endereço do manifesto. O rascunho 0.1.1 foi recuperado com este script, publicado e conferido sem autenticação: manifesto e downloads MSI/NSIS acessíveis. A nova versão passará novamente pelo workflow completo após o push.

## Limitações

Canal automático disponível para Windows x64. Assinaturas do atualizador não são certificados Authenticode. Instalação em máquina limpa e atualização completa em uma instalação real ainda precisam de homologação. Esta correção valida a correspondência dos arquivos e mantém a verificação criptográfica de instalação a cargo do atualizador Tauri.

---

# BotLive 0.1.1

## O que mudou

- Publicação inicial do projeto BotLive no repositório studiocactus/arrobabot, com interface React, núcleo Rust/Tauri, perfis, automações, comunidade, IA e manual offline.
- Sistema de variáveis com escopos local, perfil, pessoa e sessão; filtros, argumentos do comando, dados de eventos e incrementos atômicos.
- Editor simplificado com inserção no cursor, etiquetas, categorias, opções de formato e tipos de valor no gerenciador.
- Canal oficial de atualização pelo GitHub, chave pública embutida e configuração automática do manifesto para novas instalações.
- Workflow Windows para verificar commits, executar testes e gerar releases assinadas com instaladores, manual e notas detalhadas.
- Scripts de preparação e validação de versões, hook de pre-commit e registro obrigatório de cada atualização.

## Como usar

Instale a primeira versão 0.1.1 pela página Releases quando o workflow terminar. Abra Configurações e use Verificar atualização; o endereço e a chave pública oficial já vêm preenchidos. Para variáveis, abra Comandos e use os botões abaixo da resposta. Para desenvolvimento, execute npm run repo:setup e prepare cada versão com npm run update:prepare. Consulte docs/ATUALIZACOES.md para o procedimento completo.

## Validação

Validação local aprovada: 22 testes Rust, 7 Vitest, 4 cenários Playwright e 3 testes do script de atualização. TypeScript e build Vite aprovados. A revisão do stage excluiu chaves privadas, instaladores e dados de execução. Os testes de interface agora usam um servidor de produção isolado na porta 1421; o servidor de desenvolvimento anterior não respondeu no primeiro ensaio. A publicação remota continua condicionada aos testes do workflow.

## Limitações

O canal automático inicial é Windows x64. Instalação em máquina limpa, homologação de contas reais, certificados Authenticode e pacotes macOS/Linux não foram concluídos. Clientes anteriores sem canal configurado precisam da primeira instalação manual. O endpoint fica disponível após a publicação bem-sucedida da primeira release. Assinatura do atualizador não substitui certificado Windows. As funções ainda não equivalentes ao Streamer.bot estão listadas na matriz de aceite.
