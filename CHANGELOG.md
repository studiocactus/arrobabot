# Histórico de atualizações

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
