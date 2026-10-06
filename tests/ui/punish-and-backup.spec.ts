import {test,expect} from '@playwright/test';

test('ação de punição escolhe modo, duração e alvo e avisa quem executa',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto('/');
 await page.getByRole('button',{name:'Criar primeiro bot',exact:true}).click();
 await page.getByLabel('Nome do perfil',{exact:true}).fill('Moderação da live');
 await page.getByRole('button',{name:'Salvar perfil',exact:true}).click();
 await page.getByRole('dialog').getByRole('button',{name:'Fechar',exact:true}).click();
 await page.getByRole('button',{name:'Automações',exact:true}).click();
 await page.getByRole('button',{name:'Novo fluxo',exact:true}).click();

 // gatilho: o comando que dispara a punição
 await page.getByLabel('Texto que dispara',{exact:true}).fill('!silenciar');
 // bloco de ação existente
 await page.locator('.react-flow__node').nth(1).click();
 await page.getByRole('combobox',{name:'Tipo de etapa',exact:true}).selectOption('punish');
 await expect(page.getByRole('combobox',{name:'O que aplicar',exact:true})).toHaveValue('timeout');
 await expect(page.getByLabel('Duração do silêncio (segundos)',{exact:true})).toHaveValue('60');
 await expect(page.getByRole('combobox',{name:'Quem leva a punição',exact:true})).toHaveValue('sender');
 await expect(page.getByRole('textbox',{name:'Motivo (vai para a Twitch e para o Histórico)',exact:true})).toBeVisible();
 await expect(page.getByText('Só executa em perfil Twitch com a conta do canal autorizada.',{exact:false})).toBeVisible();
 await expect(page.getByText('A ação pune quem disparou o gatilho.',{exact:false})).toBeVisible();

 // duração aparece só quando o modo usa tempo
 await page.getByRole('combobox',{name:'O que aplicar',exact:true}).selectOption('ban');
 await expect(page.getByLabel('Duração do silêncio (segundos)',{exact:true})).toHaveCount(0);
 await page.getByRole('combobox',{name:'O que aplicar',exact:true}).selectOption('warn');
 await expect(page.getByLabel('Duração do silêncio (segundos)',{exact:true})).toHaveCount(0);
 await page.getByRole('combobox',{name:'O que aplicar',exact:true}).selectOption('timeout');
 await expect(page.getByLabel('Duração do silêncio (segundos)',{exact:true})).toBeVisible();

 // alvo pelo primeiro argumento explica como escrever
 await page.getByRole('combobox',{name:'Quem leva a punição',exact:true}).selectOption('first');
 await expect(page.getByText('Escreva o alvo depois do comando, como !silenciar @alvo.',{exact:false})).toBeVisible();
 await page.getByRole('combobox',{name:'Quem leva a punição',exact:true}).selectOption('sender');

 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.getByText('Automação salva.',{exact:true})).toBeVisible();
 expect(errors).toEqual([]);
});

test('comando aceita variações separadas por vírgula e explica nos dois editores',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto('/');
 await page.getByRole('button',{name:'Criar primeiro bot',exact:true}).click();
 await page.getByLabel('Nome do perfil',{exact:true}).fill('Variações');
 await page.getByRole('button',{name:'Salvar perfil',exact:true}).click();
 await page.getByRole('dialog').getByRole('button',{name:'Fechar',exact:true}).click();
 await page.getByRole('button',{name:'Comandos',exact:true}).click();
 await page.getByRole('button',{name:'Novo comando',exact:true}).click();
 await expect(page.getByText('Variações com vírgula valem aqui: !whislist, !whishlist — quem erra o comando também dispara.',{exact:true})).toBeVisible();
 await page.getByLabel('Nome',{exact:true}).fill('Whishlist');
 await page.getByLabel('Comando',{exact:true}).fill('!whislist, !whishlist, !wishlist');
 await page.getByRole('textbox',{name:'Resposta',exact:true}).fill('Lista atualizada!');
 await page.getByRole('button',{name:'Salvar comando',exact:true}).click();
 await expect(page.getByText('Automação salva.',{exact:true})).toBeVisible();

 // o editor visual mostra a mesma orientação no gatilho
 await page.getByRole('button',{name:'Editar Whishlist',exact:true}).click();
 await page.getByRole('button',{name:'Abrir no editor visual',exact:true}).click();
 await expect(page.getByText('Separe variações com vírgula quando o povo erra o comando: !whislist, !whishlist, !wishlist. Qualquer uma delas dispara.',{exact:true})).toBeVisible();
 await page.getByRole('dialog').getByRole('button',{name:'Fechar',exact:true}).click();
 expect(errors).toEqual([]);
});

test('resposta com variável local sem origem avisa no editor e some com texto alternativo',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto('/');
 await page.getByRole('button',{name:'Criar primeiro bot',exact:true}).click();
 await page.getByLabel('Nome do perfil',{exact:true}).fill('Doação');
 await page.getByRole('button',{name:'Salvar perfil',exact:true}).click();
 await page.getByRole('dialog').getByRole('button',{name:'Fechar',exact:true}).click();
 await page.getByRole('button',{name:'Comandos',exact:true}).click();
 await page.getByRole('button',{name:'Novo comando',exact:true}).click();
 const dialog=page.getByRole('dialog');
 const resposta=dialog.getByRole('textbox',{name:'Resposta',exact:true});
 await resposta.fill('A IA disse {{local.aiResponse}}.');
 await expect(dialog.getByRole('status')).toContainText('local.aiResponse');
 await expect(dialog.getByRole('status')).toContainText('Variável ausente');
 await resposta.fill('A IA disse {{local.aiResponse|default:0}}.');
 await expect(dialog.getByRole('status')).toHaveCount(0);
 await page.getByRole('dialog').getByRole('button',{name:'Fechar',exact:true}).click();
 expect(errors).toEqual([]);
});

test('cartão de backup explica pasta, retenção e importação mesmo fora do desktop',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto('/');
 await page.getByRole('button',{name:'Criar primeiro bot',exact:true}).click();
 await page.getByLabel('Nome do perfil',{exact:true}).fill('Backups');
 await page.getByRole('button',{name:'Salvar perfil',exact:true}).click();
 await page.getByRole('dialog').getByRole('button',{name:'Fechar',exact:true}).click();
 await page.getByRole('button',{name:'Configurações',exact:true}).click();

 await expect(page.getByRole('heading',{name:'Backup do bot',exact:true})).toBeVisible();
 await expect(page.locator('code').filter({hasText:'.botlivebak'})).toHaveCount(1);
 await expect(page.getByText('O cofre de credenciais fica no computador e nunca entra no arquivo.',{exact:false})).toBeVisible();
 await expect(page.getByLabel('Pasta dos backups')).toHaveValue('Nenhuma pasta escolhida');
 await expect(page.getByRole('switch',{name:'Backup automático diário',exact:true})).toBeVisible();
 await expect(page.getByLabel('Guardar por dias',{exact:true})).toHaveValue('7');
 await expect(page.getByLabel('Semanas',{exact:true})).toHaveValue('4');
 await expect(page.getByLabel('Meses',{exact:true})).toHaveValue('12');
 await expect(page.getByRole('button',{name:'Fazer backup agora',exact:true})).toBeDisabled();
 // fora do aplicativo desktop nada é gravado e o cartão explica isso
 await expect(page.getByText('Esta prévia não grava arquivos: use o aplicativo desktop.',{exact:true})).toBeVisible();
 await page.getByRole('switch',{name:'Backup automático diário',exact:true}).click();
 await expect(page.getByText('Abra o aplicativo desktop para configurar o backup.',{exact:true})).toBeVisible();
 expect(errors).toEqual([]);
});
