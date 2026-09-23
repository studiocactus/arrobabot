# Acesso local, backup e rotina de operação

## Dois tipos de permissão

Papéis do chat controlam quem dispara comandos. Acessos locais controlam quem pode editar perfis neste computador. Cadastrar um moderador local não transforma essa pessoa em moderadora da Twitch.

O controle local é uma restrição da interface e das operações do aplicativo. Ele não criptografa notas nem impede um usuário com acesso aos arquivos do sistema de lê-los.

## Habilitar login local

1. Abra **Configurações → Permissões de edição**.
2. Mantenha o nome **owner**, reservado ao proprietário.
3. Escolha uma senha com pelo menos seis caracteres.
4. Clique em **Salvar acesso**.
5. Guarde a senha de forma segura.
6. Use **Bloquear painel** para verificar o login.

Após configurar owner, as próximas aberturas pedem acesso local. As senhas ficam no cofre do sistema. Não há recuperação de senha por e-mail, tela de redefinição sem autenticação ou botão de desativar o login nessa versão.

Enquanto estiver autenticado como proprietário, salvar novamente o mesmo nome atualiza sua senha. Se perder o acesso, não exclua o banco para tentar recuperar: preserve os dados e procure manutenção técnica.

## Criar um moderador local

1. Com owner autenticado, informe outro nome em Configurações.
2. Use letras minúsculas sem acentos, números, hífen ou sublinhado; sem espaços, até quarenta caracteres.
3. Defina a senha e salve.
4. Abra **Perfis de bot → Configurar → Quem pode editar este perfil?**
5. Acrescente o nome em uma linha e salve.
6. Repita nos perfis autorizados.
7. Bloqueie o painel e entre com o novo acesso para conferir.

O proprietário continua com acesso completo. Moderadores não podem trocar contas e permissões, gerenciar credenciais/OAuth, apagar perfis ou alterar configurações gerais. Podem operar recursos do perfil para os quais estão autorizados.

A biblioteca de presets é compartilhada entre acessos locais. Não a trate como um cofre privado. Para retirar acesso a um perfil, remova o nome da lista desse perfil. Não existe uma tela de exclusão de contas locais nesta versão.

## Bloquear, desconectar e fechar

**Bloquear painel** protege a edição; não desliga o motor de automação. **Desconectar** encerra a recepção do perfil. Fechar o aplicativo encerra sua execução.

TTS depende dos eventos recebidos pela interface. Para interromper completamente o funcionamento, não use apenas o bloqueio do painel.

## Backup completo

1. Abra Configurações e copie o caminho de **Pasta de dados**.
2. Termine ou cancele operações que você não quer conservar em aberto.
3. Feche todas as instâncias do BotLive.
4. Copie a pasta de dados inteira para uma pasta com data.
5. Confira que a cópia contém botlive.sqlite e vaults.
6. Guarde também os presets exportados e a versão do aplicativo utilizada.

Copie a pasta inteira; podem existir arquivos auxiliares do SQLite. Não copie apenas um arquivo de banco enquanto o app está aberto.

O backup conserva configurações, histórico retido, notas e estados da comunidade, incluindo operações abertas. Ele não inclui as credenciais do cofre do sistema, nem os dados da prévia de navegador.

## Restaurar no mesmo computador

1. Feche o BotLive.
2. Faça uma cópia adicional da pasta atual antes de substituir qualquer coisa.
3. Restaure a pasta de dados completa do backup no caminho usado pelo aplicativo.
4. Abra preferencialmente a mesma versão ou uma versão com compatibilidade verificada.
5. Confira perfis, notas, saldos e operações abertas.
6. Reautorize serviços se as credenciais não estiverem mais disponíveis.

Não misture manualmente dois bancos e não copie somente tabelas entre versões sem um procedimento técnico de migração.

## Levar para outro computador

Para reaproveitar configurações, prefira presets e reautorize as contas no destino. Copie as notas separadamente, se necessário, para o vault do perfil correto.

Uma migração completa do banco transporta também identificadores e o estado de login local, mas não transporta as senhas do cofre. Se o backup usa proteção local, restaurá-lo em outra máquina pode deixar o painel bloqueado. Essa migração exige procedimento técnico para restabelecer credenciais; não é uma operação automática oferecida pelo aplicativo.

Presets não transportam saldos e apostas. Quando precisar preservar esses dados entre máquinas, prepare a migração completa com manutenção técnica e teste em cópia.

## Histórico e estatísticas

O Histórico da interface mostra até trezentos registros recentes. O banco retém aproximadamente dez mil no total, compartilhados entre perfis. Registros mais antigos são descartados quando esse limite é ultrapassado.

Estatísticas usam o que está nesse histórico: mensagens registradas, ações concluídas, eventos de follow e início dos fluxos. Não são um relatório oficial de audiência, espectadores únicos ou total histórico de seguidores.

O gráfico agrupa até vinte e quatro horas com registros, não necessariamente as últimas vinte e quatro horas contínuas. Simulações também geram histórico e podem influenciar as contagens. O histórico exibe horários convertidos pela interface; as marcações armazenadas usam UTC.

## Rotina recomendada

Antes da live: conferir perfil, conexão, fluxo ativo, contexto de memória, saldo/estoque, áudio e overlay.

Durante: acompanhar erros e evitar alterar a plataforma/Client ID de um perfil em uso. Confirmar resultados de previsões com cuidado, pois não há desfazer automático.

Depois: resolver operações abertas, desconectar, salvar notas e fechar antes do backup. Ao atualizar o programa, conserve a cópia anterior até verificar os dados na nova versão.
