# Batimentos da live

## Conectar pelo Android com Pulsoid

Em Comunidade, ative Batimentos da live e abra o cartão. Selecione Pelo celular · Pulsoid. O caminho é Amazfit Bip 6 → Bluetooth → Android com Pulsoid → internet → BotLive. O computador não precisa de Bluetooth nem de uma página aberta.

1. No relógio, abra a lista de aplicativos e procure Heart Rate Push: um coração branco com ondas em um círculo azul. Ative o compartilhamento. Nome, posição e disponibilidade dependem da versão do relógio.
2. No Pulsoid do Android, escolha Amazfit Bip 6. Confirme um BPM atualizado e os indicadores Device On e Network On. Confira se o sinal continua depois de bloquear a tela do celular.
3. Na página API Clients do Pulsoid, cadastre ou abra seu aplicativo e copie seu Client ID público: https://pulsoid.net/ui/api-clients. Na seção Pelo celular · Pulsoid do BotLive, cole-o em Client ID do Pulsoid.
4. Clique em Autorizar Pulsoid. O BotLive abre a página oficial de consentimento no navegador. Entre na mesma conta usada no Android e permita a leitura dos batimentos. Não é necessário Client Secret nem cadastrar URL de retorno para esse fluxo.
5. Volte ao BotLive e aguarde Recebendo batimentos reais. A autorização fica no cofre do Windows para usar Conectar Pulsoid nas próximas sessões.

Enquanto aguarda, Abrir autorização do Pulsoid permite reabrir a página. O BotLive respeita o intervalo de consulta informado pelo fornecedor. Recusar, expirar ou desconectar encerra a tentativa; uma resposta atrasada da sessão cancelada não grava a chave nem reinicia a conexão.

**A autorização pelo navegador não garante acesso gratuito.** O Pulsoid pode exigir assinatura, inclusive depois do consentimento. O BotLive informa a recusa e não faz assinatura ou compra. O fluxo foi validado com um servidor de teste; a autorização e a disponibilidade da API na conta gratuita real precisam ser conferidas pelo usuário.

Como alternativa, clique em Usar chave pessoal. Abra a página de chaves pelo botão, na mesma conta: https://pulsoid.net/ui/keys. Gere uma chave com `data:heart_rate:read`, cole no campo protegido e use Salvar chave e conectar. **A emissão manual exige BRO ou teste ativo.** Não cole Client ID ou Client Secret no campo de chave pessoal.

A chave fica no cofre do Windows, separada por perfil. Não é devolvida à interface, exposta no overlay, registrada no histórico ou incluída no backup do BotLive. Para reconectar, deixe o campo vazio e use a chave salva. Remover chave salva apaga a credencial local e encerra a sessão Pulsoid; para revogar a autorização no fornecedor, remova a chave também no site do Pulsoid.

Fechar esta área do BotLive mantém a leitura. Fechar o aplicativo interrompe. Após abrir novamente, clique em Conectar Pulsoid; a integração não inicia a coleta automaticamente. O celular e o computador precisam de internet; o relógio usa Bluetooth com o celular. Não é necessário que os dois estejam na mesma rede Wi-Fi.

O usuário confirmou neste atendimento o funcionamento de Bip 6 → Pulsoid no Android, incluindo retorno após tela bloqueada. A integração Pulsoid → BotLive foi validada com servidores de teste locais; a conta real do usuário ainda precisa ser conferida após atualizar.

## Alternativa: Bluetooth do computador

Selecione Bluetooth do computador. Ative Heart Rate Push no Bip 6 e use Chrome ou Edge com Bluetooth no computador. Desconecte o sensor no Pulsoid antes de tentar esta alternativa, para evitar disputa entre receptores. O Zepp pode continuar configurado; não remova seu vínculo nem restaure o relógio.

Clique em Conectar relógio no navegador e, na página aberta, em Escolher relógio e conectar. Se o navegador padrão não oferecer Bluetooth, copie o link da ponte e abra no Chrome ou Edge. Selecione o relógio e mantenha a página aberta durante o uso. Esta alternativa depende de o dispositivo anunciar o serviço padrão Heart Rate (180D), e não lê o histórico do Zepp. O pareamento do Windows sozinho não confirma recebimento de BPM.

Desconecte e revogue a sessão atual antes de trocar entre as duas pontes. Elas alimentam a mesma prévia e o mesmo tipo de overlay, sem criar outra área de configuração.

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

- Chave expirada ou revogada: gere outra no Pulsoid e cole no campo do BotLive.
- Acesso à API recusado: confira BRO/teste e a permissão `data:heart_rate:read`. A mensagem explica recusas HTTP 401, 402 e 403 sem mostrar a credencial.
- Conectado, aguardando sinal: confira Device On e Network On no Android, o relógio no pulso e se celular e chave pertencem à mesma conta.
- Internet interrompida: o BPM é ocultado e o BotLive tenta reconectar, com intervalos crescentes de dois a trinta segundos. Após oito falhas consecutivas, clique em Conectar Pulsoid para tentar novamente. Uma leitura válida reinicia esse limite. A reconexão automática preserva o endereço do overlay.
- Sinal antigo: leituras com mais de dez segundos, duplicadas, fora de ordem ou mais de cinco segundos no futuro não renovam a prévia. O horário do computador precisa estar correto. O BPM aceito está limitado ao formato de 1 a 300, sem interpretação médica.
- Campo removido do cofre: a operação no BotLive não revoga uma chave que já foi copiada para outro aplicativo; faça isso no site do Pulsoid.


Se o relógio não aparecer, confira Heart Rate Push, Bluetooth e o navegador; feche outras pontes que disputem a conexão. Se o aplicativo estiver fechado, a página indica conexão encerrada. Se a medição ficar antiga, confira o contato do relógio com o pulso e a conexão. Desconecte antes de preparar uma nova ponte. Após revogar os links, abra uma ponte nova pelo aplicativo.

Fontes: [Web Bluetooth no Chrome](https://developer.chrome.com/docs/capabilities/bluetooth), [manual oficial do Bip 6](https://support.amazfit.com/en/amazfit_bip_6/files/user-manual.pdf.pdf) e [orientação do fornecedor de integração sobre Heart Rate Push](https://blog.hyperate.io/post/amazfit-heart-rate-push-live-bluetooth-heart-rate-streaming-with-zepp-os-hyperate-compatible/).


## Referência Pulsoid e sessão existente

As operações abaixo são internas ao aplicativo e exigem o proprietário; não são novos comandos HTTP públicos. Todas recebem `profileId`.

| Operação | Entrada adicional | Resultado |
|---|---|---|
| `heartRate.pulsoid.get` | nenhuma | apenas `hasToken`, sem a credencial |
| `heartRate.pulsoid.connect` | `token` opcional; vazio usa o cofre | inicia validação/conexão assíncrona e retorna `overlayUrl`; o retorno não comprova sinal |
| `heartRate.pulsoid.forget` | nenhuma | remove a chave local e encerra a sessão Pulsoid |
| `heartRate.links` | nenhuma | recupera somente o endereço atual do overlay, se houver sessão |

`heartRate.get` mantém os campos anteriores e acrescenta `source` (`bluetooth` ou `pulsoid`), `phase` (`connecting`, `reconnecting`, `waiting`, `error` ou `disconnected`) e mensagem de erro sanitizada. O estado `fresh` só aparece com BPM recente; `phase=waiting` significa transporte conectado, não autorização para inventar uma leitura. O estado público nunca contém a chave Pulsoid nem mensagem privada de erro.

O transporte usa validação Bearer em `https://dev.pulsoid.net/api/v1/token/validate` e leitura WebSocket em `wss://dev.pulsoid.net/api/v1/data/real_time`, com escopo `data:heart_rate:read`. O conector aceita apenas os destinos oficiais em produção. O timestamp `measured_at` é verificado antes de usar `data.heart_rate`. Conexão interrompida limpa a medição anterior; o primeiro sinal após reconexão não dispara cruzamento de limite. A mesma variável e os mesmos gatilhos existentes funcionam com as duas origens.

Fontes Pulsoid: [chave pessoal e requisito BRO](https://docs.pulsoid.net/getting-started/access-token-management/manual-token-issuing.md), [validação de autorização](https://docs.pulsoid.net/getting-started/access-token-management/validate-authorization-token.md), [contrato do cliente oficial](https://github.com/pulsoid-oss/pulsoid-socket) e [Bip 6 no Android](https://blog.pulsoid.net/post/how-to-use-pulsoid-with-your-amazfit-device).


### Referência da autorização pelo navegador

`heartRate.pulsoid.get` também devolve `clientId`, um identificador público salvo no perfil. `heartRate.pulsoid.auth.start` recebe `clientId` (UUID), cria uma sessão de autorização e devolve o endereço do overlay existente. É uma operação interna exclusiva do proprietário; não é um endpoint público.

Enquanto `phase=authorizing`, a leitura privada de `heartRate.get` pode incluir `authorization` com `url` de consentimento e `expiresAt` em segundos Unix. O device_code e o access_token nunca são devolvidos. O estado público do overlay não inclui o consentimento. Ao concluir, recusar, expirar ou desconectar, o endereço de consentimento é removido do estado.

O Client ID é salvo junto da preferência de publicação sem apagá-la. A autorização usa `https://pulsoid.net/oauth2/device_authorization` e `https://pulsoid.net/oauth2/token`, com escopo apenas `data:heart_rate:read`. Não usa segredo de aplicativo. O token recebido passa pela mesma validação e pelo mesmo conector de batimentos. O código de recusa `premium_required` é tratado como requisito de assinatura; não é apresentado como promessa de API gratuita.

Fonte: [fluxo oficial de autorização para desktop](https://docs.pulsoid.net/getting-started/access-token-management/oauth-2-device-authorization-flow.md).
