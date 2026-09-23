# Central de documentação — BotLive

Esta documentação descreve a versão implementada do BotLive. Os exemplos usam nomes fictícios. A documentação de planejamento está em ../../documentação; para saber o que efetivamente funciona nesta versão, use este manual e a matriz de aceite.

## Por onde começar

Abra [MANUAL.html](MANUAL.html) para ler todos os capítulos em uma página, com busca, navegação e impressão. Não precisa iniciar servidor nem instalar ferramentas.

1. [Primeiro uso](GUIA-DE-USO.md) — do aplicativo aberto ao primeiro comando.
2. [Instalação](01-INSTALACAO.md) — escolher pacote, abrir e localizar os dados.
3. [Perfis e integrações](INTEGRACOES.md) — contas, canais e autorização.
4. [Comandos e automações](03-COMANDOS-E-AUTOMACOES.md) — respostas, eventos, editor visual e simulação.
5. [IA e memória](04-IA-E-MEMORIA.md) — provedores, personalidade, notas e Obsidian.
6. [Comunidade](05-COMUNIDADE.md) — pontos, sorteios, músicas, jogos, voz e moderação.
7. [Presets e aparência](06-PRESETS-E-APARENCIA.md) — reutilização e personalização.
8. [Acesso, backup e rotina](07-OPERACAO-E-BACKUP.md) — proteger e manter o espaço.
9. [OBS e API local](API-LOCAL.md) — overlays e integrações externas.
10. [Solução de problemas](08-SOLUCAO-DE-PROBLEMAS.md) — diagnóstico por sintoma.

## Para quem mantém o projeto

- [Arquitetura](ARQUITETURA.md): componentes, dados e fluxo de execução.
- [Desenvolvimento e distribuição](DISTRIBUICAO.md): dependências, testes e builds.
- [Validação realizada](VALIDACAO.md): evidências e limites dos testes.
- [Matriz de aceite](MATRIZ-DE-ACEITE.md): implementações e pendências.

## Vocabulário

| Termo | Significado no BotLive |
|---|---|
| Perfil | Espaço independente de um canal, com contas, comandos, IA, memória e módulos |
| Conta do bot | Conta que publica as mensagens automáticas |
| Conta do canal | Conta do proprietário do canal, usada nas operações que exigem suas permissões |
| Gatilho | Evento ou texto que inicia uma automação |
| Ação | Etapa executada após um gatilho, como responder ou registrar memória |
| Fluxo | Gatilho seguido de uma sequência de ações |
| Cooldown / intervalo | Tempo mínimo antes de permitir outra execução |
| Vault | Pasta de notas Markdown de um perfil |
| Preset | Configuração reutilizável; não é backup de todos os dados |
| OAuth | Autorização concedida no site da plataforma, sem informar a senha ao BotLive |
| Client ID | Identificação do aplicativo registrado na plataforma; não é o nome do canal |
| Webhook | Requisição enviada de um serviço para outro quando algo acontece |
| Loopback | Endereço deste computador, como 127.0.0.1 |
| Homologação | Teste de funcionamento com contas, serviços e ambiente reais |

## Variáveis avançadas

Consulte [Variáveis do BotLive](VARIAVEIS.md) para catálogo, escopos, contadores, filtros e receitas. No manual HTML, esse conteúdo está no capítulo 16.

## Atualizações e publicação

[Atualizações e commits](ATUALIZACOES.md): procedimento obrigatório por commit, notas de versão, canal GitHub e assinatura dos pacotes.
