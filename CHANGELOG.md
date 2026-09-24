# Histórico de atualizações

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
