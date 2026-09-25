import {test,expect} from '@playwright/test';
test('tela do Discord configura servidor, canais, engajamento e auditoria',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto('/');await page.getByRole('button',{name:'Criar primeiro bot',exact:true}).click();
 await page.getByLabel('Nome do perfil',{exact:true}).fill('Servidor da casa');await page.getByRole('button',{name:'Salvar perfil',exact:true}).click();
 await page.getByRole('dialog').getByRole('button',{name:'Fechar',exact:true}).click();
 const nav=(name:string)=>page.locator('.sidebar').getByRole('button',{name,exact:true});
 await nav('Discord').click();
 await expect(page.getByRole('heading',{name:'Discord',exact:true})).toBeVisible();
 await expect(page.getByRole('heading',{name:'Conexão com o Discord',exact:true})).toBeVisible();

 await page.getByLabel('Token do bot',{exact:true}).fill('1234567890.abcdefghijklmnopqrstuvwxyz');
 await page.getByRole('button',{name:'Salvar token',exact:true}).click();
 await expect(page.getByText('Token do bot salvo no cofre do sistema.',{exact:true})).toBeVisible();
 await page.getByRole('button',{name:'Descobrir servidor e canais',exact:true}).click();
 await expect(page.getByText('Servidor, canais e cargos carregados.',{exact:true})).toBeVisible();

 await page.getByRole('combobox',{name:'Servidor do Discord',exact:true}).selectOption({label:'Servidor da comunidade'});
 await page.getByRole('combobox',{name:'Canal de logs de auditoria',exact:true}).selectOption({label:'#geral'});
 await page.getByRole('combobox',{name:'Cargo automático na entrada',exact:true}).selectOption({label:'Moderador'});
 await page.getByRole('combobox',{name:'Canal de boas-vindas',exact:true}).selectOption({label:'#avisos'});
 await page.getByRole('combobox',{name:'Canal do espelho',exact:true}).selectOption({label:'#sorteios'});
 for(const label of ['Ativar bot do Discord','Boas-vindas','Espelho do chat','Auto-moderação do Discord','XP e níveis','Mensagens de aniversário','Sorteios liberados'])await page.getByRole('switch',{name:label,exact:true}).click();
 await page.getByRole('button',{name:'Salvar configuração',exact:true}).click();
 await expect(page.getByText('Configuração do Discord salva.',{exact:true})).toBeVisible();
 await page.screenshot({path:'artifacts/discord-018.png',fullPage:true});

 await page.reload();await nav('Discord').click();
 await expect(page.getByRole('combobox',{name:'Servidor do Discord',exact:true})).toHaveValue('123456789012345678');
 await expect(page.getByRole('combobox',{name:'Canal de logs de auditoria',exact:true})).toHaveValue('111111111111111111');
 await expect(page.getByRole('combobox',{name:'Canal de boas-vindas',exact:true})).toHaveValue('222222222222222222');
 await expect(page.getByRole('switch',{name:'Ativar bot do Discord',exact:true})).toHaveAttribute('aria-checked','true');
 await expect(page.getByRole('switch',{name:'Auto-moderação do Discord',exact:true})).toHaveAttribute('aria-checked','true');

 await page.getByLabel('Buscar membro no Discord',{exact:true}).fill('apoiador');
 await page.getByRole('button',{name:'Buscar',exact:true}).click();
 await page.getByRole('button',{name:'Apoiador · 999999999999999999',exact:true}).click();
 await page.getByLabel('Motivo registrado na auditoria',{exact:true}).fill('Prova de auditoria');
 await page.getByRole('button',{name:'Avisar nos dois chats',exact:true}).click();
 await expect(page.getByText('Aviso registrado nas duas casas.',{exact:true})).toBeVisible();
 await page.getByRole('button',{name:'Ver avisos',exact:true}).click();
 await expect(page.getByText('Aviso de Apoiador',{exact:true})).toBeVisible();
 await expect(page.getByText('Nível 1 · 120 XP',{exact:true})).toBeVisible();
 await expect(page.getByText('Faltam 280 XP para o próximo nível',{exact:true})).toBeVisible();
 await expect(page.getByText('Apoiador',{exact:true})).toBeVisible();
 await expect(page.getByText('Nitro',{exact:true})).toBeVisible();

 await expect(page.getByText('warn · Apoiador',{exact:true})).toBeVisible();
 await expect(page.getByText('discord + twitch · Prova de auditoria · 25/09/2026 10:00',{exact:true})).toBeVisible();
 await page.getByRole('button',{name:'Desfazer warn de Apoiador',exact:true}).click();
 await expect(page.getByText('Ação desfeita no Discord.',{exact:true})).toBeVisible();

 await page.getByLabel('ID do membro vinculado',{exact:true}).fill('999999999999999999');
 await page.getByRole('button',{name:'Gerar código',exact:true}).click();
 await expect(page.getByText('482913',{exact:true})).toBeVisible();
 await expect(page.getByRole('button',{name:'Remover vínculo 999999999999999999',exact:true})).toBeVisible();

 await page.getByRole('button',{name:'Registrar comandos slash',exact:true}).click();
 await expect(page.getByText('Comandos slash registrados no servidor.',{exact:true})).toBeVisible();
 await expect(page.getByText('Nenhum vínculo criado ainda.')).toHaveCount(0);
 expect(errors).toEqual([]);
});
