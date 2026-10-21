import {test,expect,type Page} from '@playwright/test';
import {join} from 'path';
import {OUT} from './ux-helpers';

/** Experiência unificada de comandos: Comandos prontos cria o comando completo
 *  numa tela só — sem abrir o editor de automações nem criar variáveis — e
 *  nomes repetidos são recusados, nunca sobrescritos. */
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

test('Comandos prontos cria Tempo de follow em uma tela só e recusa nome repetido',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await mockDesktop(page);
 await page.setViewportSize({width:1100,height:860});
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Comandos',exact:true}).click();

 // Caminho de aceitação: Comandos → Comandos prontos → Tempo de follow → salvar.
 await page.getByRole('button',{name:'Comandos prontos',exact:true}).click();
 const dialog=page.getByRole('dialog');
 await expect(dialog.getByText('Escolha a função, confira o nome e a resposta e salve')).toBeVisible();
 await expect(dialog.getByLabel('Função')).toHaveValue('followAge');
 await expect(dialog.getByLabel('Comando',{exact:true})).toHaveValue('!followers');
 await expect(dialog.getByLabel('Alias (opcional)')).toHaveValue('!followage');
 await page.screenshot({path:join(OUT,'comandos-prontos-followage.png'),fullPage:true});
 await dialog.getByRole('button',{name:'Salvar e ativar',exact:true}).click();

 // Nasce pronto e ativado, sem abrir o editor de automações.
 await expect(page.getByText(/criado e ativado/)).toBeVisible();
 await expect(page.getByText('Tempo de follow (!followers)')).toBeVisible();
 await expect(page.getByRole('dialog')).toHaveCount(0);
 await page.screenshot({path:join(OUT,'comandos-prontos-criado.png'),fullPage:true});

 // Nome repetido é recusado e nada é sobrescrito.
 await page.getByRole('button',{name:'Comandos prontos',exact:true}).click();
 await page.getByRole('dialog').getByRole('button',{name:'Salvar e ativar',exact:true}).click();
 await expect(page.getByText(/Já existe o comando/)).toBeVisible();
 await page.getByRole('dialog').getByRole('button',{name:'Cancelar',exact:true}).click();

 await page.setViewportSize({width:1600,height:900});
 await page.screenshot({path:join(OUT,'comandos-prontos-larga.png'),fullPage:true});
 expect(errors).toEqual([]);
});
