# Instalação e abertura

## Escolha uma forma de abrir

| Arquivo | Para que serve |
|---|---|
| entrega/BotLive_0.1.0_x64-setup.exe | Instalador Windows x64, com opções de idioma |
| entrega/BotLive_0.1.0_x64_en-US.msi | Instalador alternativo no formato MSI |
| entrega/BotLive.exe | Executável para abrir diretamente, sem instalar o aplicativo |
| INICIAR-BOTLIVE.cmd | Atalho do projeto: procura o executável da entrega ou o build local |
| entrega/SHA256SUMS.txt | Hashes para conferir a integridade dos três binários |

Escolha um instalador; não é necessário instalar o NSIS e o MSI. O executável direto não é um pacote de dados portátil: ele continua usando a pasta de dados do usuário no Windows.

## Requisitos de uso

- Windows x64 e Microsoft Edge WebView2 Runtime.
- Acesso à internet para conectar plataformas, usar IA remota e consultar atualizações.
- Contas e permissões da plataforma desejada.
- Para IA local, Ollama funcionando e um modelo já instalado.
- Para voz local, um servidor compatível com Whisper; ele não acompanha o instalador.

O uso do aplicativo compilado não exige Node.js, Rust ou Visual Studio. Essas ferramentas são necessárias apenas para desenvolver ou compilar o código.

O instalador usa o mecanismo de provisionamento do WebView2 do Tauri; uma máquina sem esse runtime pode precisar de internet durante a instalação. O executável direto depende de WebView2 já disponível.

## Instalar e abrir

1. Feche outras instâncias do BotLive.
2. Execute o instalador escolhido.
3. Siga as opções apresentadas por ele.
4. Abra o BotLive pelo atalho instalado.
5. Confirme que aparece a Visão geral, sem a faixa de prévia de navegador.
6. Siga [Primeiro uso](GUIA-DE-USO.md).

A versão distribuída localmente não tem assinatura de um publicador comercial configurada. Confirme a origem do arquivo; este manual não orienta a desativar proteções do Windows.

## Onde os dados ficam

Abra **Configurações → Pasta de dados** e copie o caminho exibido. Ele é a referência para o seu computador. Não use a pasta entrega como origem de backup.

Dentro da pasta de dados ficam o banco botlive.sqlite e o diretório vaults, com uma subpasta por perfil. Credenciais ficam no cofre do sistema operacional, separado desses arquivos.

Duas cópias do aplicativo executadas pelo mesmo usuário podem acessar os mesmos dados e disputar a mesma porta local. Use uma instância por vez.

## Atualizar, reinstalar e desinstalar

Antes de trocar a versão, feche o aplicativo e faça [backup](07-OPERACAO-E-BACKUP.md). Os dados não ficam junto ao executável, mas preserve uma cópia própria; não dependa do comportamento de um instalador para protegê-los.

A verificação interna de atualizações depende de um endereço de manifesto e de uma chave pública fornecidos pelo distribuidor. Não existe um feed público pronto nesta entrega.

Para desinstalar, use o recurso de aplicativos instalados do Windows. A remoção do programa e a exclusão dos dados são operações diferentes; confira a pasta de dados se quiser conservar ou remover seu espaço.

## Estado dos pacotes

Executável e instaladores Windows foram gerados. A abertura nativa foi verificada no computador de desenvolvimento. A instalação em máquina limpa e os pacotes macOS/Linux ainda não foram homologados. Consulte [Validação](VALIDACAO.md).
