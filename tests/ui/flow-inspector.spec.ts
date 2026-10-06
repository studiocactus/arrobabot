import {test,expect} from '@playwright/test';
test('editor de automações agrupa campos em seções, avisa a troca para o visual e explica a ordem por conexão',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto('/');
 await page.getByRole('button',{name:'Criar primeiro bot',exact:true}).click();
 await page.getByLabel('Nome do perfil',{exact:true}).fill('Automações amigáveis');
 await page.getByRole('button',{name:'Salvar perfil',exact:true}).click();
 await page.getByRole('dialog').getByRole('button',{name:'Fechar',exact:true}).click();
 await page.getByRole('button',{name:'Automações',exact:true}).click();
 await page.getByRole('button',{name:'Novo fluxo',exact:true}).click();

 // gatilho: Quando e Como sai abertos, Comportamento recolhido
 await expect(page.getByRole('combobox',{name:'Evento',exact:true})).toBeVisible();
 await expect(page.getByLabel('Texto que dispara',{exact:true})).toBeVisible();
 await expect(page.getByRole('combobox',{name:'Como enviar na Twitch',exact:true})).toBeVisible();
 await expect(page.getByRole('combobox',{name:'Tocar áudio ao disparar',exact:true})).toBeVisible();
 await expect(page.getByRole('switch',{name:'Contar usos deste comando',exact:true})).toBeHidden();
 await page.getByText('Comportamento',{exact:true}).click();
 await expect(page.getByRole('switch',{name:'Contar usos deste comando',exact:true})).toBeVisible();
 await page.getByText('Comportamento',{exact:true}).click();
 await page.getByLabel('Texto que dispara',{exact:true}).fill('!oi2');
 await expect(page.getByRole('switch',{name:'Contar usos deste comando',exact:true})).toBeHidden();
 await page.getByLabel('Texto que dispara',{exact:true}).fill('!oi');

 // lista de opções no gatilho de mensagem contém
 await expect(page.getByText('Separe várias palavras ou frases com vírgula:',{exact:false})).toHaveCount(0);
 await page.getByRole('combobox',{name:'Evento',exact:true}).selectOption('contains');
 await expect(page.getByText('Separe várias palavras ou frases com vírgula: o gatilho passa quando uma delas aparece na mensagem.',{exact:true})).toBeVisible();
 await page.getByRole('combobox',{name:'Evento',exact:true}).selectOption('command');

 // a ordem vem das conexões, não da posição dos blocos
 await expect(page.locator('.node-inspector > p.help')).toContainText('A ordem é a das setas');

 // recusa salvar com erro em português, sem detalhe técnico
 await page.getByLabel('Texto que dispara',{exact:true}).fill('');
 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.getByRole('alert')).toContainText('Preencha o nome e o comando.');
 await expect(page.getByText('Detalhes técnicos',{exact:true})).toHaveCount(0);
 await page.getByLabel('Texto que dispara',{exact:true}).fill('!oi');
 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.getByText('Automação salva.',{exact:true})).toBeVisible();

 // ação: conteúdo visível, condição recolhida em Comportamento
 await page.getByRole('button',{name:'Novo fluxo',exact:true}).click();
 await page.getByRole('button',{name:'Adicionar etapa',exact:true}).click();
 await expect(page.getByRole('textbox',{name:'Mensagem / conteúdo',exact:true})).toBeVisible();
 await expect(page.getByLabel('Executar só se a mensagem contiver',{exact:true})).toBeHidden();
 await page.getByText('Comportamento',{exact:true}).click();
 await expect(page.getByLabel('Executar só se a mensagem contiver',{exact:true})).toBeVisible();
 await page.getByRole('dialog').getByRole('button',{name:'Fechar',exact:true}).click();

 // comando simples pode ser aberto no editor visual, com aviso
 await page.getByRole('button',{name:'Comandos',exact:true}).click();
 await page.getByRole('button',{name:'Novo comando',exact:true}).click();
 await page.getByLabel('Nome',{exact:true}).fill('Alô');
 await page.getByLabel('Comando',{exact:true}).fill('!alo');
 await page.getByRole('textbox',{name:'Resposta',exact:true}).fill('Oi, {{user}}!');
 await page.getByRole('button',{name:'Abrir no editor visual',exact:true}).click();
 await expect(page.locator('.flow-intro')).toContainText('Trocou para o editor visual');
 await expect(page.locator('.react-flow__node')).toHaveCount(2);
 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.getByText('Automação salva.',{exact:true})).toBeVisible();

 // salvar no visual não muda o comando: ele continua abrindo no modo simples
 await page.getByRole('button',{name:'Editar Alô',exact:true}).click();
 await expect(page.getByRole('button',{name:'Salvar comando',exact:true})).toBeVisible();
 await expect(page.getByRole('button',{name:'Abrir no editor visual',exact:true})).toBeVisible();
 expect(errors).toEqual([]);
});
