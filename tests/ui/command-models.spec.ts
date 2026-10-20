import {test,expect,type Page} from '@playwright/test';
import {join} from 'path';
import {OUT} from './ux-helpers';

/** Modelos de comando: Link de doação e Clipadores. Os dois nascem prontos e
 *  ativados; o dos clipadores ainda conta por pessoa em {{user.clips}}. */
async function mockDesktop(page:Page){
 await page.addInitScript(()=>{
  const flows:Record<string,unknown>=JSON.parse(sessionStorage.getItem('flows')||'{}');
  Object.assign(window,{isTauri:true,__TAURI_EVENT_PLUGIN_INTERNALS__:{registerListener:()=>{},unregisterListener:()=>{}},
   __TAURI_INTERNALS__:{transformCallback:()=>1,unregisterCallback:()=>{},invoke:async(cmd:string,payload:{op:string;args?:Record<string,unknown>})=>{
    if(cmd!=='rpc')return 1;
    const op=payload.op;const args=payload.args||{};
    if(op==='snapshot')return {profiles:[{id:'p1',name:'Canal de teste',platform:'twitch',channel:'cactus',channelId:'1',botId:'',clientId:'',blocklist:[],topics:[],editors:[],ai:{},modules:{}}],flows:[],notes:{},presets:[],theme:'dark',logs:[],statuses:{},apiPort:9876,dataDir:'teste'};
    if(op==='flows')return Object.values(flows);
    if(op==='flow.save'){const f=args.flow as {id:string};flows[f.id]=f;sessionStorage.setItem('flows',JSON.stringify(flows));return f}
    if(op==='flow.delete'){delete (flows as Record<string,unknown>)[String(args.id)];sessionStorage.setItem('flows',JSON.stringify(flows));return null}
    if(op==='command.counters')return {};
    if(op==='settings.get')return {theme:'dark',apiPort:9876,apiToken:'',updateEndpoint:'',updatePublicKey:'',autoUpdate:false};
    return null;
   }}});
 });
}

test('modelos de comando criam !pix e os clipadores prontos para usar',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await mockDesktop(page);
 await page.setViewportSize({width:1100,height:860});
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Comandos',exact:true}).click();
 await page.getByRole('button',{name:'Modelos',exact:true}).click();
 const dialog=page.getByRole('dialog');
 await expect(dialog.getByRole('heading',{name:'Link de doação'})).toBeVisible();
 await expect(dialog.getByRole('heading',{name:'Clipadores (!clip)'})).toBeVisible();
 await dialog.getByLabel('Seu link de doação',{exact:true}).fill('livepix.gg/thenees');
 await page.screenshot({path:join(OUT,'modelos-comandos.png'),fullPage:true});

 // O dos clipadores nasce pronto: clipe na hora, contagem por pessoa e aviso.
 await dialog.getByLabel('Comando',{exact:true}).nth(1).fill('!clip, !clipe');
 await dialog.getByRole('button',{name:'Criar comando',exact:true}).nth(1).click();
 await expect(page.getByText(/criado e ativado/)).toBeVisible();
 await expect(page.getByText('Clipadores (!clip)')).toBeVisible();
 await page.screenshot({path:join(OUT,'modelos-clipadores-criado.png'),fullPage:true});
 expect(errors).toEqual([]);
});
