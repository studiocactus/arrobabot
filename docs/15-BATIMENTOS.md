# Batimentos da live

## Preparar o relógio

Em Comunidade, ative Batimentos da live e abra o cartão. No Amazfit Bip 6, procure Heart Rate Push nas configurações do relógio ou no Zepp. A disponibilidade depende do firmware e de o relógio anunciar o serviço Bluetooth padrão Heart Rate (180D). A compatibilidade física com o relógio do proprietário ainda precisa ser validada; a implementação não lê o histórico do Zepp.

No computador, habilite Bluetooth e use Chrome ou Edge. Clique em Conectar relógio no navegador e, na página aberta, em Escolher relógio e conectar. Se o navegador padrão não oferecer Bluetooth, copie o link da ponte e abra no Chrome ou Edge. Selecione o relógio no seletor do navegador e mantenha a página aberta durante o uso.

O primeiro valor deve aparecer na ponte e na prévia do aplicativo. Se não houver uma medição recente por mais de dez segundos, o número desaparece. Um dispositivo que não oferece o serviço padrão não será listado. Não é necessário conceder acesso aos seus arquivos ou à conta Zepp.

## Publicar no OBS

A publicação começa desligada. Ligue Publicar batimentos no OBS somente quando quiser mostrar a medição na transmissão. Copie a fonte de batimentos e cole como URL de uma Fonte de navegador no OBS. O fundo é transparente; dimensione e posicione no OBS.

Desligar a publicação oculta o número público, mantendo a prévia local. Desconectar e revogar links encerra a sessão. Desativar o módulo também revoga as chaves. Depois de reconectar, copie o novo endereço para o OBS. Reiniciar o BotLive invalida os links anteriores.

## Criar uma reação

Em Automações, crie um fluxo com o evento Batimentos cardíacos. Escolha Mudou, Cruzou acima ou Cruzou abaixo. Para um limite de 120, Cruzou acima dispara ao passar de 120 ou menos para mais de 120; permanecer acima não repete o cruzamento. Cruzou abaixo faz o inverso. O primeiro sinal e o retorno após perda de sinal não disparam.

Use a variável `{{heartRate.bpm}}` nas etapas seguintes. Configure intervalo entre usos e, para efeitos na cena, duração e recuperação compatíveis com a ação. A entrada admite no máximo cinco atualizações por segundo e gera no máximo um evento de mudança por segundo. Não representa cada amostra do relógio.

O editor preserva o limite de cruzamento ao salvar. Se você criou uma automação de limite na v0.20.19, confira o gatilho e escolha novamente Cruzou acima ou Cruzou abaixo: naquela versão o editor podia apagar o limite durante a gravação. O valor já apagado não pode ser reconstruído automaticamente.

Esses números servem a efeitos da transmissão. O módulo não avalia saúde. O módulo e a ponte não salvam uma série de medições; uma automação configurada por você pode enviar ou guardar o valor em mensagens, variáveis ou serviços externos.

## Referência das operações

Todas as operações abaixo exigem o acesso do proprietário e um `profileId` existente. São operações internas do aplicativo, não comandos do WebSocket público de eventos.

| Operação | Parâmetros adicionais | Resultado |
|---|---|---|
| `heartRate.get` | nenhum | estado, BPM recente ou nulo, idade e publicação |
| `heartRate.prepare` | nenhum | bridgeUrl e overlayUrl com chaves diferentes |
| `heartRate.visibility` | enabled: booleano | atualiza a permissão de publicação |
| `heartRate.stop` | nenhum | revoga a sessão e apaga a leitura em memória |

Estados: fresh significa leitura recente; waiting significa ponte conectada aguardando sinal; disconnected significa sem ponte; hidden significa módulo desligado ou publicação pública desligada. Nunca interprete valor nulo como zero BPM.

A ponte autentica com modo heart-publisher e só envia mensagens heartRate contendo packet, uma lista de bytes da característica Bluetooth 2A37. O servidor valida flags, comprimentos, contato e BPM antes de aceitar a leitura. A fonte autentica com modo heart-overlay e só recebe o estado público do perfil. Nenhuma dessas chaves autoriza comandos gerais ou acesso a outro perfil. Não compartilhe os links.

## Diagnóstico

Se o relógio não aparecer, confira Heart Rate Push, Bluetooth e o navegador; feche outras pontes que disputem a conexão. Se o aplicativo estiver fechado, a página indica conexão encerrada. Se a medição ficar antiga, confira o contato do relógio com o pulso e a conexão. Desconecte antes de preparar uma nova ponte. Após revogar os links, abra uma ponte nova pelo aplicativo.

Fontes: [Web Bluetooth no Chrome](https://developer.chrome.com/docs/capabilities/bluetooth), [manual oficial do Bip 6](https://support.amazfit.com/en/amazfit_bip_6/files/user-manual.pdf.pdf) e [orientação do fornecedor de integração sobre Heart Rate Push](https://blog.hyperate.io/post/amazfit-heart-rate-push-live-bluetooth-heart-rate-streaming-with-zepp-os-hyperate-compatible/).
