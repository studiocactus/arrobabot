# Presets e aparência

## O que um preset guarda

| Tipo | Conteúdo |
|---|---|
| Comandos | Um ou todos os fluxos de gatilho por comando selecionados |
| Fluxo de automação | Uma ou todas as automações selecionadas |
| Personalidade de IA | Provedor, modelo, personalidade, fallback e restrições, sem chave |
| Perfil completo | Fluxos, IA, restrições, módulos e configurações selecionadas de módulos, tema e cor |
| Tema visual | Modo claro/escuro e cor de destaque do painel |

Um preset de perfil não inclui notas, histórico, saldos, filas, apostas, senhas ou contas vinculadas. Ele também não transporta todas as configurações externas: endpoints de voz/Obsidian e segredos precisam ser configurados no destino.

O tema é global no aplicativo. Aplicar um preset de perfil com tema pode alterar a aparência de todo o painel.

## Criar na biblioteca

1. Selecione o perfil de origem.
2. Abra **Biblioteca de presets → Salvar como preset**.
3. Dê um nome.
4. Escolha o tipo.
5. Para comando/fluxo, selecione uma automação ou todas as desse tipo.
6. Informe tags separadas por vírgula.
7. Clique em **Salvar preset**.

A busca combina nome, tipo e tags. Exemplos de tags: boas-vindas, xadrez, sorteio.

Também há atalhos: ícone de pacote na lista de comandos/fluxos; **Salvar como preset** na tela de IA; **Salvar tema como preset** nas Configurações. O atalho do tema depende de existir um perfil selecionado.

## Exportar

No cartão do preset, clique em **Exportar**, escolha uma pasta e salve a extensão .botlivepreset. O arquivo é JSON versionado, limite de dois megabytes.

A exportação retira campos de contas vinculadas e não lê as chaves do cofre. Há verificações para campos de credenciais e padrões conhecidos de segredo. Endereços de webhook em ações podem impedir a exportação porque podem conter credenciais.

Textos livres podem conter informações pessoais que você tenha escrito. Revise o conteúdo antes de compartilhar. O validador não é um detector universal de todo segredo possível.

## Importar e aplicar

1. Selecione o perfil de destino.
2. Clique em **Importar arquivo** e escolha o arquivo.
3. Confira nome, tipo, destino e quantidade de automações.
4. Se houver conflitos, escolha a política.
5. Clique em **Confirmar aplicação**.
6. Revise os fluxos e ative os que quiser usar.

| Opção de conflito | Efeito |
|---|---|
| Escolha antes de continuar | Não permite aplicar enquanto houver conflito sem decisão |
| Manter configurações atuais | Pula as automações conflitantes |
| Substituir as configurações em conflito | Substitui os fluxos identificados como conflitantes |

A comparação de conflito considera nome e, para comandos importados, padrão do comando. Não trata todos os casos de comandos equivalentes como duplicatas; revise diferenças de capitalização e sobreposição.

A política trata conflitos de automação. Em presets de perfil/personalidade, as configurações daquele tipo são aplicadas mesmo quando você escolhe manter automações atuais. Faça backup ou exporte a configuração anterior se precisar voltar.

Fluxos importados ficam desativados. Conta, Client ID e credenciais existentes no destino são preservados. Salvar uma configuração na biblioteca e aplicá-la ao perfil são etapas diferentes.

## Exemplos fornecidos

- examples/boas-vindas.botlivepreset: comando !ola com resposta e overlay.
- examples/assistente-ia.botlivepreset: personalidade de comunidade em português.

O preset de IA deixa o modelo vazio: selecione um modelo existente antes de testar.

## Personalizar o painel

Em **Configurações → Do seu jeito**, selecione Escuro ou Claro. Escolha **Cor de destaque** para os elementos que usam a cor principal. O texto dos botões principais ajusta o contraste em relação à cor escolhida.

**Restaurar padrão** remove a cor personalizada e volta à cor do modo claro/escuro. Não apaga perfis nem configurações de módulos.

O botão de sol/lua na barra superior alterna o modo. A preferência fica salva. Não há editor completo de CSS ou de todas as cores da interface.

## O que não confundir

- Preset: reutiliza configurações.
- Backup: preserva banco, memórias e estado operacional.
- Exportar: grava um arquivo para guardar ou compartilhar.
- Aplicar: altera o perfil selecionado.
- Tema do painel: aparência do aplicativo; não é o visual completo do overlay OBS.

## Timers e contadores nos presets

Timers são fluxos: exporte como Fluxo de automação ou Perfil completo. O intervalo e a opção de contar usos acompanham a configuração; os totais dos contadores não acompanham. Fluxos importados ficam desativados. Confira destino e intervalos antes de ativar.
