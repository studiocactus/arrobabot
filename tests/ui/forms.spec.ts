import {test,expect} from '@playwright/test';
import {join} from 'path';
import {mock,OUT} from './ux-helpers';

test('comando preserva edição e opções após falha ao salvar',async({page})=>{
 await mock(page);await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Comandos',exact:true}).click();
 await page.getByRole('button',{name:'Criar comando personalizado',exact:true}).click();
 const dialog=page.getByRole('dialog',{name:'Configurar comando',exact:true});
 await expect(dialog.getByRole('note')).toContainText('Editar ou testar não salva');
 await dialog.getByLabel('Nome',{exact:true}).fill('Comando QA');
 await dialog.getByLabel('Comando',{exact:true}).fill('!qa');
 await dialog.getByRole('textbox',{name:'Resposta',exact:true}).fill('Resposta preservada');
 await expect(dialog.getByRole('combobox',{name:'Como enviar na Twitch',exact:true})).toBeHidden();
 const options=dialog.getByText('Opções de envio, som e contagem',{exact:true});
 await options.click();
 await dialog.getByRole('combobox',{name:'Como enviar na Twitch',exact:true}).selectOption('announce');
 await options.click();await options.click();
 await expect(dialog.getByRole('combobox',{name:'Como enviar na Twitch',exact:true})).toHaveValue('announce');
 await page.evaluate(()=>{
  const w=window as unknown as {__TAURI_INTERNALS__:{invoke:(cmd:string,payload:{op:string})=>Promise<unknown>}};
  const original=w.__TAURI_INTERNALS__.invoke;let fail=true;
  w.__TAURI_INTERNALS__.invoke=async(cmd,payload)=>{
   if(payload?.op==='flow.save'){
    sessionStorage.setItem('qaFlowAttempts',String(Number(sessionStorage.getItem('qaFlowAttempts')||0)+1));
    if(fail){fail=false;throw Error('Falha controlada ao salvar comando')}
   }
   return original(cmd,payload);
  };
 });
 await dialog.getByRole('button',{name:'Salvar comando',exact:true}).click();
 await expect(dialog.getByRole('alert')).toContainText('Falha controlada ao salvar comando');
 await expect(dialog.getByRole('textbox',{name:'Resposta',exact:true})).toHaveValue('Resposta preservada');
 await expect(dialog.getByRole('combobox',{name:'Como enviar na Twitch',exact:true})).toHaveValue('announce');
 await dialog.getByRole('button',{name:'Salvar comando',exact:true}).dblclick();
 await expect(dialog).toHaveCount(0);
 expect(await page.evaluate(()=>sessionStorage.getItem('qaFlowAttempts'))).toBe('2');
});

test('essencial e avançado de comando e timer cabem em janela estreita e larga',async({page})=>{
 await mock(page);await page.goto('/');
 for(const type of ['comando','timer']){
  await page.locator('.sidebar').getByRole('button',{name:type==='timer'?'Timers':'Comandos',exact:true}).click();
  await page.getByRole('button',{name:type==='timer'?'Novo timer':'Criar comando personalizado',exact:true}).click();
  const dialog=page.getByRole('dialog',{name:type==='timer'?'Configurar timer':'Configurar comando',exact:true});
  await expect(dialog.getByText('Abrir no formulário dessa função',{exact:true})).toHaveCount(0);
  if(type==='timer'){
   await expect(dialog.getByLabel('Repetir a cada (minutos)',{exact:true})).toBeVisible();
   await expect(dialog.getByRole('combobox',{name:'Categoria',exact:true})).toBeHidden();
   await dialog.getByText('Avançado do timer',{exact:true}).click();
   await dialog.getByRole('combobox',{name:'Prioridade',exact:true}).selectOption('alta');
   await dialog.getByText('Avançado do timer',{exact:true}).click();
   await dialog.getByText('Avançado do timer',{exact:true}).click();
   await expect(dialog.getByRole('combobox',{name:'Prioridade',exact:true})).toHaveValue('alta');
  }
  for(const width of [1100,1600]){
   await page.setViewportSize({width,height:1000});
   expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
   await page.screenshot({path:join(OUT,`formulario-${type}-${width}.png`),fullPage:true});
  }
  await dialog.getByRole('button',{name:'Fechar',exact:true}).click();
 }
});
