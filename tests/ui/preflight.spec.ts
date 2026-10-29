import {test,expect,type Page} from '@playwright/test';

/** Pré-voo da live 0.7.8: o card da Visão geral varre as configs que só quebrariam
 * ao vivo, cada item leva à tela de conserto e nada é alterado. */

type Item={message:string;page:string};
type St={items:Item[]|null;fail:boolean;ops:string[];preflightCalls:number};
const profile={id:'p1',name:'Canal de teste',platform:'twitch',channel:'cactus',channelId:'1',botId:'',clientId:'',blocklist:[],topics:[],editors:[],ai:{},modules:{}};

async function mock(page:Page,items:Item[]){
 await page.addInitScript(arg=>{
  const list=arg.items as Item[];
  const st:St={items:list,fail:false,ops:[],preflightCalls:0};
  const w=window as unknown as Record<string,unknown>;
  w.__st=st;
  Object.assign(w,{
   isTauri:true,
   __TAURI_EVENT_PLUGIN_INTERNALS__:{registerListener:()=>{},unregisterListener:()=>{}},
   __TAURI_INTERNALS__:{
    transformCallback:()=>1,
    unregisterCallback:()=>{},
    invoke:async(cmd:string,payload:{op:string;args?:Record<string,unknown>}|null)=>{
     if(cmd!=='rpc')return 1;
     const op=payload?.op||'';
     st.ops.push(op);
     if(op==='snapshot')return {profiles:[arg.profile],logs:[],statuses:{},theme:'dark',apiPort:0,dataDir:'teste'};
     if(op==='preflight.check'){st.preflightCalls++;if(st.fail)throw Error('Falha do pré-voo simulada');return {items:st.items||[]}}
     if(op==='module.config.get')return {};
     if(op==='flows'||op==='logs'||op==='command.counters')return [];
     return null;
    }
   }
  });
 },{items,profile});
 await page.goto('/');
 await page.getByText('Preparação e configuração',{exact:true}).click();
}
const st=async(page:Page)=>page.evaluate(()=>{
 const s=(window as unknown as {__st:St}).__st;
 return {ops:s.ops,preflightCalls:s.preflightCalls};
});

test('pré-voo acusa o que só quebraria e cada item leva à tela de conserto',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await mock(page,[
  {message:'O alerta de seguidor está desligado no overlay.',page:'alertas'},
  {message:'O áudio de “Cumprimento” foi apagado de Sons de Live.',page:'sounds'},
  {message:'“Boas-vindas” lê mensagens em voz alta, mas o módulo Texto para voz está desligado.',page:'community'}
 ]);
 const card=page.locator('.card').filter({hasText:'Pré-voo da live'});
 await expect(card.getByRole('heading',{name:'Pré-voo da live',exact:true})).toBeVisible();
 await expect(card.getByText('Antes de ir ao ar, o pré-voo confere',{exact:false})).toBeVisible();
 await expect(card.getByRole('button',{name:'Rodar pré-voo',exact:true})).toBeVisible();

 // estreito: o relatório não espreme o texto nem empurra rolagem horizontal
 await page.setViewportSize({width:1100,height:900});
 await card.getByRole('button',{name:'Rodar pré-voo',exact:true}).click();
 await expect(card.getByText('O alerta de seguidor está desligado no overlay.',{exact:true})).toBeVisible();
 await expect(card.getByText('Ir para Sons de Live',{exact:true})).toBeVisible();
 await expect(card.getByText('Ir para Comunidade',{exact:true})).toBeVisible();
 const overflow=await page.evaluate(()=>document.documentElement.scrollWidth>document.documentElement.clientWidth);
 expect(overflow,'sem rolagem horizontal a 1100px').toBe(false);
 await page.screenshot({path:'artifacts/preflight-1100.png',fullPage:true});

 // o item leva à tela de conserto — o menu marca a tela aberta
 await card.getByRole('button',{name:'Ir para Alertas',exact:true}).click();
 await expect(page.locator('.nav-item.active')).toContainText('Alertas');
 await page.setViewportSize({width:1440,height:1000});

 // voltar e verificar de novo: o card remonta e o relatório se refaz com um clique
 await page.getByRole('button',{name:'Visão geral',exact:true}).click();
 await page.getByText('Preparação e configuração',{exact:true}).click();
 await card.getByRole('button',{name:'Rodar pré-voo',exact:true}).click();
 await expect(card.getByText('O alerta de seguidor está desligado no overlay.',{exact:true})).toBeVisible();
 await page.screenshot({path:'artifacts/preflight-wide.png',fullPage:true});

 const s=await st(page);
 expect(s.preflightCalls,'um pré-voo por clique').toBe(2);
 const writes=s.ops.filter(op=>/(save|delete|import|connect|disconnect|start|stop|send|rehearse|clear|action|\.config$)/.test(op));
 expect(writes,'o pré-voo só lê: nenhuma operação de escrita — ops: '+s.ops.join(', ')).toEqual([]);
 expect(errors).toEqual([]);
});

test('pré-voo sem problemas delimita a conferência; falha orienta sem quebrar o painel',async({page})=>{
 await mock(page,[]);
 const card=page.locator('.card').filter({hasText:'Pré-voo da live'});
 await card.getByRole('button',{name:'Rodar pré-voo',exact:true}).click();
 await expect(card.getByText('Nenhum problema encontrado nas configurações verificadas. Confira também as conexões e uma gravação de áudio antes da live.',{exact:true})).toBeVisible();
 await expect(card.getByRole('button',{name:'Verificar de novo',exact:true})).toBeVisible();

 // falha de comunicação: o aviso aparece e o botão permite tentar de novo
 // (o relatório anterior não some — só o aviso entra junto)
 await page.evaluate(()=>{(window as unknown as {__st:St}).__st.fail=true});
 await card.getByRole('button',{name:'Verificar de novo',exact:true}).click();
 await expect(card.getByRole('alert')).toContainText('Falha do pré-voo simulada');
 await page.evaluate(()=>{(window as unknown as {__st:St}).__st.fail=false});
 await card.getByRole('button',{name:'Verificar de novo',exact:true}).click();
 await expect(card.getByText('Nenhum problema encontrado nas configurações verificadas. Confira também as conexões e uma gravação de áudio antes da live.',{exact:true})).toBeVisible();
});
