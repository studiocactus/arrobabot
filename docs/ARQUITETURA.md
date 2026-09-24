# Arquitetura

A interface React chama o comando RPC do Tauri. Rust concentra persistência, permissões, integrações, segredos e execução. O navegador isolado oferece apenas prévia com armazenamento local.

## Fluxo de eventos

Adaptadores normalizam eventos para o motor. Uma fila limitada e um semáforo controlam concorrência. O motor aplica deduplicação, filtros de perfil, permissões e cooldowns, executa ações em ordem e registra resultados. Mensagens de saída são serializadas por perfil. Scripts Rhai têm limites e não expõem execução de comandos do sistema.

## Persistência

SQLite armazena perfis, fluxos, presets, configurações, histórico e estado dos módulos. O histórico retém aproximadamente 10 mil registros. Notas Markdown ficam em diretórios separados por UUID de perfil, com validação de caminhos. Segredos usam o gerenciador de credenciais do sistema; não há fallback em texto puro. Alterar plataforma ou Client ID exige nova autorização.

## Organização

- `src/App.tsx`: navegação e estado da interface. O grupo Seu espaço separa Comandos, Timers, Respostas e sons, Contadores e Automações.
- `src/FlowTools.tsx`: diálogo de criação/edição, simulação, ajuste de contador e ações de linha compartilhados por essas telas.
- `src/Timers.tsx`, `src/Counters.tsx`, `src/ChatExtras.tsx`: listas de timers, totais por comando e respostas TXT/sons.
- `src/FlowEditor.tsx`: edição visual e validação da sequência.
- `src-tauri/src/lib.rs`: despacho RPC e autorização.
- `engine.rs`, `model.rs`: eventos e ações.
- `platforms.rs`, `oauth.rs`: serviços externos.
- `db.rs`, `vault.rs`, `secrets.rs`: armazenamento.
- `modules.rs`: economia e jogos, alterações atômicas.
- `ai.rs`: provedores, memória e filtros de resposta.
- `access.rs`: proprietário e moderadores locais.
- `local_api.rs`: WebSocket autenticado no loopback.

Estatísticas são derivadas do histórico retido, não representam métricas oficiais completas das plataformas. Integrações usam endpoints oficiais e falham explicitamente quando configuração ou capacidade está ausente.

## Motor de variáveis

O módulo conversation.rs mantém, por perfil, até 12 falas recentes com janela de cinco minutos e até 500 caracteres por fala. O motor captura uma cópia anterior ao evento atual, após moderação e antes dos comandos de comunidade. Saídas confirmadas entram como falas do bot; eventos de chat cujo userId coincide com botId são ignorados para evitar auto-respostas. Simulações não leem nem alteram esse contexto. Ações ai e ai.generate enviam mensagem atual e contexto como dados JSON; personalidade e regras de geração permanecem separadas. ai.generate grava somente destinos locais, além de local.aiResponse e local.aiSuccess. O RPC ai.preview aplica autorização de perfil, usa conversa de teste e não publica nem registra memória.

variables.rs mantém um contexto isolado por execução e faz expansão de uma passagem. Variáveis compartilhadas são carregadas no início do fluxo; incrementos usam transação SQLite e retornam o valor atualizado ao contexto. A tabela variables persiste dados por perfil, escopo e usuário qualificado pela plataforma; session_variables é temporária à conexão. A migração aditiva usa user_version 2. Os RPCs variables.list, variables.change e variables.preview passam pela autorização de perfil. Prévia e simulação usam cópias locais dos valores e não gravam os escopos compartilhados.
