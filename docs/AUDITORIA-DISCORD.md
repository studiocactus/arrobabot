# Auditoria da integração Discord

## Escopo e diagnóstico

Esta revisão examinou a conexão Gateway, os eventos de membros, a tela Discord, a moderação, os cargos e os aniversários. A aplicação usa um bot oficial do Discord, com token guardado no cofre do sistema. Nenhuma credencial ou servidor real foi acessado durante a revisão.

| Área | Situação encontrada | Correção nesta revisão |
|---|---|---|
| Conexão | Exigia clique em Conectar após abrir o aplicativo | Perfis ativados e com token passam a iniciar o Gateway na abertura; salvar servidor ou intents reconecta |
| Estado | A tela não acompanhava os eventos de conexão | Indicador acompanha conexão e desconexão em tempo real; tarefas encerradas deixam de aparecer como ativas |
| Reconexão | Heartbeat não marcava o ACK como pendente | Falta de confirmação força a reconexão com espera progressiva |
| Cargo automático e boas-vindas | Entrada lia `id` na raiz do evento, mas o membro chega em `user.id` | Entrada e saída leem o identificador do usuário no campo correto |
| Cargos manuais | API de atribuir e remover existia, mas não havia controle na tela | Seleção e botões para atribuir/remover ao membro escolhido, com auditoria |
| Moderação | Bloqueio de expressões, links e repetição era um único interruptor | Cada regra pode ser configurada no Discord; repetição usa registro separado da Twitch |
| Aniversários | O dia seguia UTC e aceitava datas impossíveis | Usa o horário local do computador e valida o calendário |

## Recursos já presentes

Boas-vindas e despedidas com texto, cargo automático, contador de membros, espelho de chat, notificações da Twitch, limpeza de mensagens, timeout, expulsão, banimento, avisos, auditoria, comandos slash, XP, sorteios, vínculo Twitch/Discord e aniversários. Esses recursos dependem das permissões e intents correspondentes no servidor.

## Próximos pontos de produto

1. Homologar com um servidor Discord de teste: entrada de um membro novo, atribuição de cargo, boas-vindas, remoção de mensagem e aniversário no fuso local. A revisão atual não afirma que essa validação real ocorreu.
2. Adicionar uma página de diagnóstico de permissões e hierarquia de cargos para explicar por que uma ação foi recusada.
3. Oferecer uma prévia local das boas-vindas e da mensagem de aniversário antes de publicar.
4. Estudar cargos por reação e cargos por vínculo com a Twitch, com regras explícitas de consentimento e remoção.
5. Permitir ajustar a política de moderação por canal, com exceções para canais de links e registro dos resultados.

## Verificação executada

- `npm run check`: passou.
- Testes Rust filtrados por `discord`: 22 passaram.
- `npx playwright test tests/ui/discord.spec.ts`: passou, incluindo a seleção de cargo e as novas opções de moderação.
- Integração com servidor real: pendente, pois exige token e servidor do proprietário.

A verificação automatizada não substitui a instalação da versão atualizada e o teste em um servidor Discord real.
