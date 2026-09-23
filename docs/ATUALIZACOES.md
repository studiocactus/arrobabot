# Atualizações e commits

## Canal oficial

Código e histórico: [studiocactus/arrobabot](https://github.com/studiocactus/arrobabot). Pacotes e notas: [Releases](https://github.com/studiocactus/arrobabot/releases). O aplicativo consulta o manifesto latest.json da última release publicada e verifica a assinatura antes de instalar.

A versão 0.1.1 estabelece esse canal e inclui a chave pública. Quem usa um executável antigo sem a configuração precisa instalar essa primeira versão manualmente ou preencher a URL do manifesto e a chave pública oficial nas Configurações. A chave pública está em src-tauri/tauri.conf.json. Nenhuma senha ou chave privada é necessária para o usuário instalar uma atualização.

O serviço só terá um manifesto disponível depois de a primeira release concluir a compilação e publicação. Até lá, a consulta pode informar falha. A assinatura do atualizador é diferente de um certificado Authenticode do Windows; não significa que os avisos do SmartScreen desaparecerão.

## O que cada commit precisa trazer

1. Código, testes e documentação relacionados à mudança.
2. Uma versão nova no formato X.Y.Z, inclusive para correções de documentação.
3. Notas em updates/X.Y.Z.md com O que mudou, Como usar, Validação e Limitações.
4. updates/latest.json e CHANGELOG.md atualizados pelo script.
5. Versão sincronizada em package.json, package-lock.json, Cargo.toml, Cargo.lock e tauri.conf.json.

Não é necessário editar o código do script a cada commit. O script prepara e verifica os registros daquela atualização. As notas da versão são a descrição detalhada que acompanha o commit e a release.

## Preparar o próximo commit

Após clonar, execute:

```text
npm ci
npm run repo:setup
```

Copie updates/TEMPLATE.md para um arquivo de trabalho fora de updates, preencha todas as seções e escolha a próxima versão. Por exemplo:

```text
npm run update:prepare -- 0.1.2 minhas-notas.md
npm run update:check
npm run check
npm test
npm run test:updates
cargo test --manifest-path src-tauri/Cargo.toml --locked --lib -j 1
npm run docs
```

Revise os arquivos, selecione os que serão incluídos com git add e faça o commit. O hook verifica os arquivos staged, não apenas o que está salvo no diretório. Se você corrigir as notas depois de preparar a versão, mantenha a mesma seção em CHANGELOG.md e adicione ambos ao stage.

```text
git commit -m "feat: descreva a alteração concreta"
git push origin main
```

O hook pode ser removido ou ignorado por alguém com acesso local. A CI repete a validação e bloqueia a publicação sem registro válido; ela não impede fisicamente um push manual. Proteção de branch obrigatória não foi presumida nem configurada silenciosamente. Em outros clones, execute repo:setup para ativar a proteção local.

## O que acontece no GitHub

O workflow Validar e publicar atualização verifica os commits recebidos, tipos, testes e build da interface. No Windows executa testes Rust e de interface. Para main, gera instaladores NSIS/MSI, assinaturas e manifesto em uma release inicialmente privada como rascunho. Após conferir o manifesto e anexar o manual e as notas, publica a release.

Pull requests são verificados sem publicação nem acesso às chaves. As execuções da mesma branch são serializadas. Se vários commits forem enviados juntos, todos precisam de registro, mas o pacote publicado corresponde à versão final daquele push. Para um pacote de cada versão intermediária, envie uma versão por push.

As releases automáticas deste fluxo são Windows x64. O código mantém estrutura Tauri para outros sistemas, mas macOS e Linux ainda precisam de compilação, assinatura e homologação próprias antes de entrar no canal oficial.

## Chaves e recuperação

Os secrets TAURI_SIGNING_PRIVATE_KEY e TAURI_SIGNING_PRIVATE_KEY_PASSWORD ficam no GitHub Actions. A cópia local foi criada na pasta .private, ignorada pelo Git. Guarde uma cópia segura dessa pasta fora do repositório; o GitHub não permite recuperar o conteúdo de um secret pela interface.

Não gere outra chave para cada atualização: a chave pública embutida nos aplicativos deve continuar verificando os pacotes seguintes. Perder ou trocar a chave exige planejar a migração dos aplicativos existentes. Não envie a pasta .private por commit nem como asset da release.

## Falhas e conferência

O manifesto gerado pelo empacotador pode usar endereços da API do GitHub. Antes da publicação, o script confere cada endereço contra os arquivos reais do rascunho e o converte para o download público da mesma versão. A assinatura é preservada. Arquivos desconhecidos, assinaturas ausentes e versões divergentes interrompem a publicação. O script também recusa modificar uma release que já esteja publicada.

Consulte a aba Actions do repositório. Uma falha nos testes ou na assinatura impede a publicação final. Confira os assets, notas, tag, commit e latest.json antes de anunciar a versão. Uma release em rascunho não é disponibilizada pelo endpoint latest.

Para uma falha de infraestrutura antes de publicar, é possível reexecutar o workflow do mesmo commit. Para corrigir código, crie outro commit com outra versão e novas notas. Não use uma versão publicada para distribuir conteúdo diferente.
