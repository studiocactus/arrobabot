import {test,expect,type Page} from '@playwright/test';
import {join} from 'path';
import {OUT} from './ux-helpers';

/** Receitas reutilizam os editores e preservam configurações existentes. */
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


const openFollowRecipe=async(page:Page)=>{
 await page.getByRole('button',{name:'Receitas prontas',exact:true}).click();
 await page.getByRole('button',{name:'Tempo de follow',exact:true}).click();
 await page.getByRole('button',{name:/Configurar receita|Editar configuração existente/,exact:true}).click();
};

test('receitas substituem Comandos prontos e reabrem o existente sem duplicar',async({page})=>{
 await mockDesktop(page);await page.setViewportSize({width:1100,height:950});await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Comandos',exact:true}).click();
 await expect(page.getByRole('button',{name:'Comandos prontos',exact:true})).toHaveCount(0);
 await expect(page.getByRole('button',{name:'Resenha com IA',exact:true})).toHaveCount(0);
 await openFollowRecipe(page);
 const dialog=page.getByRole('dialog',{name:'Configurar comando',exact:true});
 await expect(dialog.getByLabel('Comando',{exact:true})).toHaveValue('!followers');
 await expect(dialog.getByLabel('Alias (opcional)')).toHaveValue('!followage');
 await expect(dialog.getByRole('switch',{name:'Escrever com IA no meu jeito'})).toHaveAttribute('aria-checked','false');
 await expect(page.locator('.react-flow')).toHaveCount(0);
 await dialog.getByRole('button',{name:'Salvar comando',exact:true}).click();
 await expect(page.getByText('Comando salvo.',{exact:true})).toBeVisible();
 const before=await page.evaluate(()=>sessionStorage.getItem('flows'));
 const saved=Object.values(JSON.parse(before||'{}')) as {enabled:boolean}[];
 expect(saved).toHaveLength(1);expect(saved[0].enabled).toBe(false);
 await openFollowRecipe(page);
 await expect(dialog.getByLabel('Comando',{exact:true})).toHaveValue('!followers');
 expect(await page.evaluate(()=>sessionStorage.getItem('flows'))).toBe(before);
 await dialog.getByRole('button',{name:'Fechar',exact:true}).click();
});

test('a receita e a edição da função usam o mesmo formulário',async({page})=>{
 await mockDesktop(page);await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Comandos',exact:true}).click();
 await openFollowRecipe(page);
 await page.getByRole('button',{name:'Salvar comando',exact:true}).click();
 await page.getByRole('button',{name:/Editar Tempo de follow/}).click();
 await expect(page.getByText('Personalizar no editor avançado')).toBeVisible();
 await expect(page.locator('.react-flow')).toHaveCount(0);
 await page.getByRole('textbox',{name:'Resposta',exact:true}).fill('{{user}} tá no time desde {{local.followSince}}!');
 await page.getByRole('button',{name:'Salvar comando',exact:true}).click();
 const saved=Object.values(JSON.parse(await page.evaluate(()=>sessionStorage.getItem('flows'))||'{}')) as {actions:{text:string}[]}[];
 expect(saved).toHaveLength(1);expect(saved[0].actions.some(action=>action.text.includes('tá no time'))).toBe(true);
});

test('conflito de comando é recusado no formulário sem sobrescrever o existente',async({page})=>{
 await mockDesktop(page);await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Comandos',exact:true}).click();
 await page.getByRole('button',{name:'Criar comando personalizado',exact:true}).click();
 await page.getByRole('button',{name:'Salvar comando',exact:true}).click();
 const before=await page.evaluate(()=>sessionStorage.getItem('flows'));
 await openFollowRecipe(page);
 await page.getByLabel('Comando',{exact:true}).fill('!oi');
 await page.getByRole('button',{name:'Salvar comando',exact:true}).click();
 await expect(page.getByRole('alert')).toContainText('Já existe o comando');
 expect(await page.evaluate(()=>sessionStorage.getItem('flows'))).toBe(before);
});
