import {test,expect} from '@playwright/test';
test('atualizador mostra andamento, erro persistente, notas e ausência de atualização',async({page})=>{
 await page.addInitScript(()=>{
  Object.assign(window,{isTauri:true,__TAURI_INTERNALS__:{transformCallback:()=>1,unregisterCallback:()=>{},invoke:async(cmd:string,{op}:{op:string})=>{
   if(cmd!=='rpc')return 1;
   if(op==='snapshot')return {profiles:[],logs:[],statuses:{},theme:'dark',apiPort:9876,dataDir:'teste'};
   if(op==='settings.get')return {updateEndpoint:'https://example.com/latest.json',updatePublicKey:'test',autoUpdate:false};
   if(op==='update.check'){
    await new Promise(resolve=>setTimeout(resolve,500));
    const n=Number(sessionStorage.getItem('checks')||0)+1;sessionStorage.setItem('checks',String(n));
    if(n===1)throw Error('Falha de rede de teste');
    return n===2?{available:true,version:'9.9.9',notes:'Mudança de teste'}:{available:false};
   }
   return null;
  }}});
 });
 await page.goto('/');await page.getByRole('button',{name:'Configurações',exact:true}).click();
 await expect(page.getByRole('button',{name:'Instalar atualização',exact:true})).toBeDisabled();
 await page.getByRole('button',{name:'Verificar atualização',exact:true}).click();await expect(page.getByRole('button',{name:'Verificando…',exact:true})).toBeDisabled();
 await expect(page.getByRole('alert')).toHaveText('Falha de rede de teste');
 await page.getByRole('button',{name:'Verificar atualização',exact:true}).click();await expect(page.getByText('Mudança de teste',{exact:true})).toBeVisible();await expect(page.getByRole('button',{name:'Instalar atualização',exact:true})).toBeEnabled();
 await page.getByRole('button',{name:'Verificar atualização',exact:true}).click();await expect(page.getByText('Você está na versão mais recente disponível neste canal.')).toBeVisible();await expect(page.getByRole('button',{name:'Instalar atualização',exact:true})).toBeDisabled();
});
