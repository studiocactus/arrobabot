# Timers e contadores por comando

Use **Comandos → Timers** para publicar lembretes periódicos. Use **Contar usos deste comando** para manter um total independente em cada comando. Um timer dispara pelo tempo; um contador aumenta quando um comando é aceito. Nenhum dos dois é uma contagem regressiva.

## Receita: divulgar a comunidade a cada dez minutos

1. Selecione o perfil do canal e abra **Comandos**.
2. Clique em **Novo timer**.
3. Em Nome, escreva Lembrete da comunidade.
4. Em Resposta, escreva: Participe da nossa comunidade! Digite !discord para receber o convite.
5. Em **Repetir a cada (segundos)**, informe **600**.
6. Clique em **Salvar timer** e confira se está ativo na aba **Timers**.
7. Use o botão **Simular timer Lembrete da comunidade**. Confira a resposta no Histórico; nada é publicado nesse teste.
8. Conecte o perfil. O primeiro envio real ocorre depois do intervalo completo; os seguintes repetem o intervalo.

**Resultado esperado:** enquanto o perfil estiver conectado, o bot publica o lembrete aproximadamente a cada dez minutos. Não depende de alguém escrever um comando.

| Tempo desejado | Valor em segundos |
|---|---|
| 30 segundos, para teste curto | 30 |
| 5 minutos | 300 |
| 10 minutos | 600 |
| 30 minutos | 1800 |
| 1 hora | 3600 |

Cada timer tem nome, mensagem, intervalo e chave de ativação próprios. São aceitos de 30 segundos a 86400 segundos (24 horas). Você pode ter um lembrete da comunidade a cada dez minutos e outro sobre as regras a cada quinze.

## Pausar, editar e reconectar

- Desligue **Ativar Nome do timer** para pausar sem apagar.
- Use **Editar Nome do timer** para alterar texto ou intervalo e salve. A edição reinicia a espera.
- Ao desconectar, o agendador deixa de executar timers desse perfil. Ao reconectar, ele espera novamente o intervalo inteiro. Fechar o aplicativo também encerra a espera.
- Não há recuperação de disparos perdidos nem uma sequência de mensagens acumuladas após reconexão. Um timer ainda em execução não inicia outra cópia em paralelo.
- A pausa é observada pelo agendador a cada segundo. Uma mensagem já enviada não é removida; uma chamada externa já iniciada pode terminar.

**Conectado não significa ao vivo.** Se o perfil permanecer conectado depois da transmissão, os timers continuam. Desconecte ou pause ao encerrar a live. O Kick, que depende de ponte externa e não tem conexão nativa marcada como online, não inicia esses timers nesta versão.

O intervalo é aproximado: fila de eventos, espaçamento de mensagens e serviços externos podem atrasar a publicação. Uma tentativa que falhar aguarda o próximo intervalo, sem repetição imediata. Confira os erros no Histórico.

## Variáveis e automações nos timers

Na resposta, use **Nome do canal**, data ou horário pelo catálogo de variáveis. Exemplo: São {{time}} no canal {{channel}}. Hora de beber água!

Um timer não representa um espectador: o nome do evento é BotLive, o ID da pessoa e a mensagem recebida ficam vazios. Não use variáveis salvas por pessoa ou parâmetros de comando nesse contexto. Para combinar ações, edite o timer em **Automações**. O gatilho é **Timer periódico**; as ações seguem as mesmas regras dos demais fluxos.

## Receita: contador de mortes

1. Abra **Comandos → Novo comando**.
2. Nome: Mortes. Comando: **!mortes**.
3. Ative **Contar usos deste comando**.
4. Na Resposta, escreva **O streamer já morreu **, clique em **+ Contagem do comando** e complete com ** vezes.**
5. Escolha **Só o streamer** ou **Moderadores** em Quem pode usar. Assim, espectadores comuns não aumentam o total.
6. Defina o intervalo entre usos para evitar cliques repetidos, por exemplo cinco segundos.
7. Clique em **Salvar comando**.

O texto salvo será:

```text
O streamer já morreu {{commandCount}} vezes.
```

**Primeiro uso autorizado:** O streamer já morreu 1 vezes.

**Segundo uso autorizado, após o intervalo:** O streamer já morreu 2 vezes.

Para evitar a diferença entre singular e plural, você pode usar **Mortes registradas: {{commandCount}}.** O exemplo é um contador de usos aceitos; o bot não detecta mortes no jogo automaticamente.

## Receita: contador de vitórias separado

Crie outro comando chamado Vitórias, com **!vitorias**, ative o contador e use **Vitórias registradas: {{commandCount}}.** Mesmo usando o mesmo marcador, cada comando tem seu próprio total. Três usos de !mortes e um de !vitorias produzem totais 3 e 1.

## Conferir, corrigir ou zerar

Na lista de comandos, a coluna **Contador** mostra o total. Clique em **número · Ajustar**, digite o **Novo total** e confirme. Para iniciar outra sessão do zero, defina zero. O próximo uso aceito contará um. Essa operação altera somente o comando escolhido e não publica mensagem.

O total permanece após fechar o BotLive, editar o nome ou mudar a resposta. Desativar o contador conserva o total; reativá-lo continua de onde parou. Apagar o comando apaga seu contador. Criar outro comando com o mesmo nome cria outro total.

O ajuste exige acesso ao perfil no aplicativo. Números inteiros de zero a 9007199254740991 são aceitos. Não existe ajuste de total pelo chat nesta versão: o comando configurado sempre soma um por uso aceito.

## O que conta e o que não conta

| Situação | Efeito no total |
|---|---|
| Comando ativo, permissão e intervalos liberados | Soma 1 antes de executar as ações |
| Pessoa sem permissão ou uso dentro do intervalo | Não soma |
| Mensagem barrada pela moderação | Não soma |
| Uma resposta TXT ou módulo de comunidade consumiu a mensagem primeiro | Não soma |
| Simular evento | Mostra o próximo total na resposta, sem salvar |
| Conferir variáveis no editor | Mostra o total atual, sem somar |
| Ação falha depois de o comando ser aceito | O uso continua contado |
| Dois comandos diferentes possuem o mesmo gatilho | Cada fluxo elegível conta e executa; evite duplicar gatilhos |

O contador mede usos aceitos, não mensagens entregues com sucesso. Para ter um comando público apenas de consulta e outro de incremento, use as variáveis avançadas salvas no perfil, descritas em [Variáveis](VARIAVEIS.md).

## Testar antes da live

Simule o comando com um papel autorizado, leia a resposta no Histórico e confirme que o total real na lista não mudou. Depois, teste no chat com outra conta autorizada: as mensagens da própria conta do bot são ignoradas.

Para o timer, comece com 30 segundos em um canal de teste, confira uma publicação e pause. Depois ajuste para o intervalo da live. A simulação não comprova autorização ou entrega na plataforma.
