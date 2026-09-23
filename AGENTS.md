# Manutenção do BotLive

- Todo commit precisa de uma versão superior à anterior e de notas detalhadas em `updates/<versão>.md`.
- Use `npm run update:prepare -- X.Y.Z caminho/das-notas.md`; não altere versões isoladamente.
- As notas devem explicar o que mudou, como usar, validação realmente executada e limitações. Atualize também o capítulo de uso afetado.
- Execute `npm run repo:setup` após clonar. O hook verifica o conteúdo staged; a CI verifica cada commit recebido.
- Nunca versione `.private`, `.tools`, dados de usuários, credenciais, instaladores ou artefatos de teste. Binários são publicados em Releases.
- Um push em `main` inicia validação e publicação Windows assinada. Não reutilize uma versão já publicada nem reescreva uma release existente.
- Preserve a chave de assinatura atual. Nunca imprima, copie para o código ou faça commit da chave privada e sua senha.
- Antes do push, execute os testes pertinentes e `npm run update:check`. Registre falhas e limitações sem afirmar uma homologação que não ocorreu.
