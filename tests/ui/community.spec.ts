import {test,expect} from '@playwright/test';
import {auditPage,fmt} from './ux-helpers';
test('Match do chat: ativa, configura e mantém a configuração ao reabrir',async({page})=>{
 await page.goto('/');await page.getByRole('button',{name:'Criar primeiro bot',exact:true}).click();await page.getByLabel('Nome do perfil',{exact:true}).fill('Match');await page.getByRole('button',{name:'Salvar perfil',exact:true}).click();await page.getByRole('dialog').getByRole('button',{name:'Fechar',exact:true}).click();
 await page.getByRole('button',{name:'Comunidade',exact:true}).click();
 await page.getByRole('switch',{name:'Ativar Match do chat',exact:true}).click();
 await expect(page.getByText('Módulo ativado.')).toBeVisible();
 await page.getByRole('button',{name:'Abrir Match do chat',exact:true}).click();
 await page.getByText('Avançado',{exact:true}).click();
 await expect(page.getByLabel('Palavras-gatilho',{exact:true})).toHaveValue('amor, paixão, crush, casal, beijo, namorar');
 await expect(page.getByLabel('Comando do par',{exact:true})).toHaveValue('!match');
 await page.getByLabel('Palavras-gatilho',{exact:true}).fill('amor, casal');
 await page.getByLabel('Comando do par',{exact:true}).fill('!casal');
 await page.getByRole('combobox',{name:'A live como par',exact:true}).selectOption('on');
 await page.getByRole('textbox',{name:'Mensagens por faixa',exact:true}).fill('Primeira faixa {{user1}} {{user2}} {{pct}}\nSegunda faixa');
 await page.getByRole('button',{name:'Salvar configuração avançada',exact:true}).click();
 await expect(page.getByText('Configuração salva.')).toBeVisible();
 // o aviso flutuante não pode entrar na captura nem na auditoria de layout
 await page.getByRole('button',{name:'Fechar aviso',exact:true}).click();
 await expect(page.getByRole('status')).toBeHidden();

 // a tela cabe em janela estreita e larga, sem espremer o painel do módulo
 await page.setViewportSize({width:1100,height:950});await page.waitForTimeout(250);
 const narrow=await page.evaluate(auditPage);
 expect(fmt(narrow),`${narrow.length} achado(s) em 1100px`).toEqual([]);
 await page.screenshot({path:'artifacts/comunidade-1100.png',fullPage:true});
 await page.setViewportSize({width:1680,height:1000});await page.waitForTimeout(250);
 const wide=await page.evaluate(auditPage);
 expect(fmt(wide),`${wide.length} achado(s) em 1680px`).toEqual([]);
 await page.screenshot({path:'artifacts/comunidade-largo.png',fullPage:true});

 // reabrir mantém o que foi salvo
 await page.reload();await page.getByRole('button',{name:'Comunidade',exact:true}).click();
 await page.getByRole('button',{name:'Abrir Match do chat',exact:true}).click();
 await page.getByText('Avançado',{exact:true}).click();
 await expect(page.getByLabel('Palavras-gatilho',{exact:true})).toHaveValue('amor, casal');
 await expect(page.getByLabel('Comando do par',{exact:true})).toHaveValue('!casal');
 await expect(page.getByRole('combobox',{name:'A live como par',exact:true})).toHaveValue('on');
 await expect(page.getByRole('textbox',{name:'Mensagens por faixa',exact:true})).toHaveValue('Primeira faixa {{user1}} {{user2}} {{pct}}\nSegunda faixa');
});
