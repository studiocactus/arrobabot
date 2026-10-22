import {test,expect,type Page} from '@playwright/test';
import {join} from 'path';
import {OUT} from './ux-helpers';

/** Quem está no chat agora, na Visão geral: números e nomes em tempo real,
 *  somente durante a live — sem live, o card some. */
async function mockDesktop(page:Page,status:'online'|'offline'){
 await page.addInitScript(([status])=>{
  Object.assign(window,{isTauri:true,__TAURI_EVENT_PLUGIN_INTERNALS__:{registerListener:()=>{},unregisterListener:()=>{}},
   __TAURI_INTERNALS__:{transformCallback:()=>1,unregisterCallback:()=>{},invoke:async(cmd:string,payload:{op:string})=>{
    if(cmd!=='rpc')return 1;
    const op=payload.op;
    if(op==='snapshot')return {profiles:[{id:'p1',name:'Canal de teste',platform:'twitch',channel:'cactus',channelId:'1',botId:'',clientId:'',blocklist:[],topics:[],editors:[],ai:{},modules:{}}],flows:[],notes:{},presets:[],theme:'dark',logs:[],statuses:{p1:status},apiPort:9876,dataDir:'teste'};
    if(op==='labels.viewers')return {total:3,users:['Ana','Bia','Caio']};
    if(op==='flows')return [];
    if(op==='command.counters')return {};
    if(op==='settings.get')return {theme:'dark',apiPort:9876,apiToken:'',updateEndpoint:'',updatePublicKey:'',autoUpdate:false};
    return null;
   }}});
 },[status]);
}

test('Visão geral mostra quem está no chat com números e nomes',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await mockDesktop(page,'online');
 await page.setViewportSize({width:1100,height:860});
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Visão geral',exact:true}).click();
 const card=page.getByText('Quem está no chat agora');
 await expect(card).toBeVisible();
 await expect(page.getByText('com o chat aberto mesmo sem falar')).toBeVisible();
 for(const nome of ['Ana','Bia','Caio'])await expect(page.getByText(nome,{exact:true}).first()).toBeVisible();
 await page.screenshot({path:join(OUT,'inicio-quem-esta-no-chat.png'),fullPage:true});
 await page.setViewportSize({width:1600,height:900});
 await page.screenshot({path:join(OUT,'inicio-quem-esta-no-chat-larga.png'),fullPage:true});
 expect(errors).toEqual([]);
});

test('sem live, o card da Visão geral some',async({page})=>{
 await mockDesktop(page,'offline');
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Visão geral',exact:true}).click();
 await expect(page.getByText('Quem está no chat agora')).toHaveCount(0);
});
