import {test,expect,type Page} from '@playwright/test';
import {join} from 'path';
import {OUT} from './ux-helpers';

/** Shoutout e ganks (Comunidade): escolher como a raid é anunciada — destaque,
 *  Anúncio ou mensagem normal, texto fixo ou IA — salva uma automação de raid
 *  de verdade, visível depois em Automações. */
async function mockDesktop(page:Page){
 await page.addInitScript(()=>{
  const saved=JSON.parse(sessionStorage.getItem('estado')||'{}');
  const state:{flows:Record<string,unknown>;profile:Record<string,unknown>|null}={flows:saved.flows||{},profile:saved.profile||null};
  const flush=()=>sessionStorage.setItem('estado',JSON.stringify(state));
  const base={id:'p1',name:'Canal de teste',platform:'twitch',channel:'cactus',channelId:'1',botId:'9',clientId:'cid',blocklist:[],topics:[],editors:[],ai:{},modules:{}};
  const community={balances:{},names:{},queue:[],songs:[],raffle:{open:false,keyword:'',price:0,entries:{},winner:null},prediction:{open:false,title:'',options:[],bets:{},result:null},shop:[],redemptions:[],trivia:{}};
  Object.assign(window,{isTauri:true,__TAURI_EVENT_PLUGIN_INTERNALS__:{registerListener:()=>{},unregisterListener:()=>{}},
   __TAURI_INTERNALS__:{transformCallback:()=>1,unregisterCallback:()=>{},invoke:async(cmd:string,payload:{op:string;args?:Record<string,unknown>})=>{
    if(cmd!=='rpc')return 1;
    const op=payload.op;const args=payload.args||{};
    if(op==='snapshot')return {...base,...(state.profile?{profile:undefined}:{}),profiles:[{...(state.profile||base)}],flows:[],notes:{},presets:[],theme:'dark',logs:[],statuses:{},apiPort:9876,dataDir:'teste'};
    if(op==='flows')return Object.values(state.flows);
    if(op==='flow.save'){const f=args.flow as {id:string};state.flows[f.id]=f;flush();return f}
    if(op==='flow.delete'){delete state.flows[String(args.id)];flush();return null}
    if(op==='profile.save'){state.profile=args.profile;flush();return args.profile}
    if(op==='community')return community;
    if(op==='command.counters')return {};
    if(op==='settings.get')return {theme:'dark',apiPort:9876,apiToken:'',updateEndpoint:'',updatePublicKey:'',autoUpdate:false};
    return null;
   }}});
 });
}

test('Shoutout e ganks anuncia a raid como Anúncio com mensagem da IA',{tag:'@ux'},async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await mockDesktop(page);
 await page.setViewportSize({width:1100,height:860});
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Comunidade',exact:true}).click();
 await page.getByRole('button',{name:'Abrir Shoutout e ganks',exact:true}).click();

 // Antes de ativar, o painel avisa que nada dispara na live.
 await expect(page.getByText('Quando outra pessoa traz a raid')).toBeVisible();
 await expect(page.getByText(/o anúncio não dispara na live/)).toBeVisible();
 await expect(page.getByLabel('Destaque de canal (shoutout nativo)')).toBeChecked();
 await page.screenshot({path:join(OUT,'shoutout-ganks-1100.png'),fullPage:true});

 // Ativar o módulo cria a automação de raid na hora.
 await page.getByLabel('Ativar Shoutout e ganks').check();
 await expect(page.getByText(/Shoutout ativado: a próxima raid é anunciada/)).toBeVisible();
 let estado=await page.evaluate(()=>JSON.parse(sessionStorage.getItem('estado')||'{}'));
 let fluxos=Object.values(estado.flows||{}) as {trigger:{kind:string};fnId?:string;enabled:boolean}[];
 expect(fluxos).toHaveLength(1);
 expect(fluxos[0].fnId).toBe('gank');
 expect(fluxos[0].trigger.kind).toBe('raid');
 expect(fluxos[0].enabled).toBe(true);

 // Escolha: Anúncio da Twitch + mensagem aleatória pela IA (com reserva).
 await page.getByRole('button',{name:'Anúncio da Twitch',exact:true}).click();
 await page.getByRole('button',{name:'Aleatória por IA',exact:true}).click();
 await expect(page.getByLabel('Instrução para a IA')).toBeVisible();
 await page.getByLabel('Reserva (se a IA falhar)').fill('Valeu pela raid, {{raider.name}}!');
 await page.getByRole('button',{name:'Salvar anúncio',exact:true}).click();
 await expect(page.getByText('Shoutout salvo.')).toBeVisible();

 estado=await page.evaluate(()=>JSON.parse(sessionStorage.getItem('estado')||'{}'));
 fluxos=Object.values(estado.flows||{}) as {trigger:{kind:string};sendType?:string;enabled:boolean;actions:{kind:string;twOp?:string;aiFallback?:string}[]}[];
 expect(fluxos).toHaveLength(1);
 expect(fluxos[0].enabled).toBe(true);
 expect(fluxos[0].trigger.kind).toBe('raid');
 expect(fluxos[0].sendType).toBe('announce');
 expect(fluxos[0].actions.map(a=>a.kind)).toEqual(['twitch','ai.generate','chat']);
 expect(fluxos[0].actions[0].twOp).toBe('shoutout');
 expect(fluxos[0].actions[1].aiFallback).toBe('Valeu pela raid, {{raider.name}}!');
 await expect(page.getByText(/Como pode sair/)).toBeVisible();

 // Ao voltar para a lista de Comandos, a automação não aparece como comando.
 await page.locator('.sidebar').getByRole('button',{name:'Comandos',exact:true}).click();
 await expect(page.getByText('Gank/Raid')).toHaveCount(0);

 await page.setViewportSize({width:1600,height:900});
 await page.locator('.sidebar').getByRole('button',{name:'Comunidade',exact:true}).click();
 await page.getByRole('button',{name:'Abrir Shoutout e ganks',exact:true}).click();
 await page.screenshot({path:join(OUT,'shoutout-ganks-larga.png'),fullPage:true});
 expect(errors).toEqual([]);
});
