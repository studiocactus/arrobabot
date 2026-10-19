import {test,expect,type Page} from '@playwright/test';
import {auditPage,fmt} from './ux-helpers';
import {CHAT_EVENT_SAMPLES,DEFAULT_CHAT} from '../../src/chatOverlay';

const profile={id:'p1',name:'Canal de teste',platform:'twitch',channel:'cactus',channelId:'1',botId:'',clientId:'',blocklist:[],topics:[],editors:[],ai:{},modules:{}};

/** Mock desktop: `chat.get` devolve a config e o endereço do overlay; o op
 *  `chat.test` obedece os mesmos interruptores do backend e entrega o payload
 *  pelo evento do Tauri (`chat`/`chat-event`), como o aplicativo real faz —
 *  a prévia só recebe pelo listener, igual ao que acontece em live. */
async function setup(page:Page){
 await page.addInitScript(p=>{
  const handlers:Record<number,(e:unknown)=>void>={};
  const byEvent:Record<string,number[]>={};
  let nextId=1;
  const emit=(event:string,payload:unknown)=>{
   for(const id of byEvent[event]||[]){const h=handlers[id];if(h)h({event,id,payload})}
  };
  // Mesmo preenchimento de {chaves} que o motor do chat faz no texto da caixa.
  const fill=(tpl:string,vars:Record<string,unknown>)=>{
   let out=tpl;
   for(const k of ['name','amount','months','tier','sender','message']){
    const v=vars[k];
    out=out.split(`{${k}}`).join(v===undefined||v===null?'':String(v));
   }
   return out;
  };
  const label=(kind:string)=>(({subscription:'sub',resub:'renovação',gift:'sub de presente',giftbulk:'presentes em lote',cheer:'bits'} as Record<string,string>)[kind])||kind;
  const samples=p.samples as Record<string,Record<string,string|number>>;
  const evDefaults=p.evDefaults as Record<string,{text:string;color:string}>;
  // Última configuração salva do chat (o backend leria do perfil).
  let chatCfg:Record<string,any>={enabled:false,eventsOn:false,evShowMsg:true,ev:evDefaults};
  Object.assign(window,{isTauri:true,
   __TAURI_EVENT_PLUGIN_INTERNALS__:{registerListener:()=>{},unregisterListener:()=>{}},
   __TAURI_INTERNALS__:{
    transformCallback:(cb:(e:unknown)=>void,once?:boolean)=>{
     const id=nextId++;
     handlers[id]=once?(e:unknown)=>{delete handlers[id];cb(e)}:cb;
     return id;
    },
    unregisterCallback:()=>{},
    invoke:async(cmd:string,payload:any)=>{
     if(cmd==='plugin:event|listen'){const l=payload as {event:string;handler:number};(byEvent[l.event]||=[]).push(l.handler);return l.handler}
     if(cmd==='plugin:event|unlisten'){const l=payload as {event:string;handler:number};const list=byEvent[l.event]||[];const i=list.indexOf(l.handler);if(i>=0)list.splice(i,1);return null}
     if(cmd!=='rpc')return 1;
     const op=payload.op;const args=payload.args||{};
     if(op==='snapshot')return {profiles:[p.profile],flows:[],notes:{},presets:[],theme:'dark',logs:[],statuses:{},apiPort:9876,dataDir:'teste'};
     if(op==='flows')return [];
     if(op==='command.counters')return {};
     if(op==='settings.get')return {theme:'dark',apiPort:9876,apiToken:'',updateEndpoint:'',updatePublicKey:'',autoUpdate:false};
     if(op==='chat.get')return {config:null,overlay:{token:'tok',port:9876,url:'http://127.0.0.1:9876/overlay#token=tok&profile=p1'}};
     if(op==='chatExtras.get')return {config:{audioDeviceId:''},assets:[{id:'snd1',kind:'sound',name:'ping.wav'}]};
     if(op==='module.config'){
      if(args.key==='chatOverlay')chatCfg=args.value as Record<string,any>;
      sessionStorage.setItem('chatSave',JSON.stringify({key:args.key,value:args.value}));
      return null;
     }
     if(op==='chat.test'){
      if((args.mode||'message')==='message'){
       if(chatCfg.enabled!==true)return {fired:false,reason:'Ligue o chat no overlay: em live a mensagem não apareceria.'};
       emit('chat',{...(args.sample||{}),profileId:'p1',rehearsal:true});
       return {fired:true};
      }
      const kind=String(args.kind||'');
      if(chatCfg.enabled!==true)return {fired:false,reason:'Ligue o chat no overlay: em live o evento não apareceria.'};
      if(chatCfg.eventsOn!==true)return {fired:false,reason:'Ligue os eventos de canal no chat: em live nada apareceria.'};
      const ev=(chatCfg.ev||{})[kind];
      if(!ev||ev.on!==true)return {fired:false,reason:`Ligue o evento de ${label(kind)} no chat: em live ele não apareceria.`};
      const vars=samples[kind]||{};
      let text=fill(String(ev.text||evDefaults[kind]?.text||''),vars);
      const msg=String(vars.message||'');
      if(chatCfg.evShowMsg!==false&&msg)text+=`<span class="bl-evmsg">${msg}</span>`;
      emit('chat-event',{profileId:'p1',kind,color:String(ev.color||evDefaults[kind]?.color||''),text,rehearsal:true});
      return {fired:true};
     }
     return null;
    }}});
  sessionStorage.removeItem('chatSave');
 },{profile,samples:CHAT_EVENT_SAMPLES,evDefaults:DEFAULT_CHAT.ev});
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Chat no OBS',exact:true}).click();
}

const saved=async(page:Page)=>JSON.parse(await page.evaluate(()=>sessionStorage.getItem('chatSave')||'null'));

/** Liga o interruptor mestre e espera a configuração salvar: a partir daí os
 *  testes seguem o caminho da live (antes disso, o aviso explica a parada). */
async function enableChat(page:Page){
 const master=page.getByRole('switch',{name:'Ligar chat no overlay'});
 await master.click();
 await expect.poll(async()=>(await saved(page))?.value?.enabled,{timeout:3000}).toBe(true);
}

test('estúdio de chat empilha mensagens de teste pelo motor real e salva sozinho',async({page})=>{
 await setup(page);

 await expect(page.getByRole('heading',{name:'Chat no overlay'})).toBeVisible();
 const col=page.locator('.ch-stage .bl-chat-col');
 const badge=page.locator('.ch-stage .lp-ensaio');

 // Com o chat desligado, o teste não empilha nada: o aviso diz o que a live faria.
 await page.getByRole('button',{name:/Comum/}).click();
 await expect(page.locator('.toast')).toContainText('Ligue o chat no overlay');
 await expect(col.locator('.bl-chat-msg')).toHaveCount(0);
 await expect(badge).toHaveCount(0);

 // Ligar o interruptor mestre salva a configuração inteira e libera os testes.
 const master=page.getByRole('switch',{name:'Ligar chat no overlay'});
 await expect(master).toHaveAttribute('aria-checked','false');
 await master.click();
 await expect.poll(async()=>(await saved(page))?.value?.enabled,{timeout:3000}).toBe(true);

 // Mensagem comum entra com nome, separador e texto — pelo caminho da live,
 // com o selo ENSAIO no palco.
 await page.getByRole('button',{name:/Comum/}).click();
 await expect(col).toContainText('Ana');
 await expect(col).toContainText('Boa noite, gente!');
 await expect(badge).toBeVisible();

 // Moderador chega com badge desenhada e destaque por papel.
 await page.getByRole('button',{name:/Moderador/}).click();
 await expect(col).toContainText('ModeradorAna');
 await expect(col.locator('.bl-chat-msg.hl-bar')).toHaveCount(1);
 await expect(col.locator('.bl-chat-badge svg').first()).toBeVisible();

 // Só emotes usa a CDN da Twitch com o texto alternativo do emote.
 await page.getByRole('button',{name:/Só emotes/}).click();
 await expect(col.locator('img.bl-chat-emote[alt="Kappa"]').first()).toBeVisible();

 // Limpar zera a coluna.
 await page.getByRole('button',{name:'Limpar'}).click();
 await expect(col.locator('.bl-chat-msg')).toHaveCount(0);

 // Direção e some-apois também entram no mesmo salvamento.
 await page.getByRole('button',{name:'Novas no topo'}).click();
 await expect.poll(async()=>(await saved(page))?.value?.direction,{timeout:3000}).toBe('down');
});

test('filtros do estúdio escondem os exemplos correspondentes',async({page})=>{
 await setup(page);
 await enableChat(page);

 // Esconder comandos não afeta as mensagens de teste; esconder o bot, sim.
 await page.getByRole('switch',{name:'Mostrar as mensagens do bot'}).click();
 await expect.poll(async()=>(await saved(page))?.value?.showBot,{timeout:3000}).toBe(false);
 await page.getByRole('button',{name:/Do bot/}).click();
 await expect(page.locator('.ch-stage .bl-chat-col .bl-chat-msg')).toHaveCount(0);

 // Re ligando, o mesmo botão volta a empilhar.
 await page.getByRole('switch',{name:'Mostrar as mensagens do bot'}).click();
 await page.getByRole('button',{name:/Do bot/}).click();
 await expect(page.locator('.ch-stage .bl-chat-col')).toContainText('BotLive');
});

test('estúdio de chat cabe em janela de 1100px e ampla sem espremer texto',async({page})=>{
 await page.setViewportSize({width:1100,height:950});
 await setup(page);
 await enableChat(page);
 // Padrão Estúdio: a prévia fica fixa enquanto os ajustes rolam ao lado.
 await expect(page.locator('.lp-preview')).toHaveCSS('position','sticky');
 await page.getByRole('button',{name:/Moderador/}).click();
 await page.getByRole('button',{name:/Em emote/}).click();
 await page.getByRole('button',{name:/Primeira vez/}).click();
 const col=page.locator('.ch-stage .bl-chat-col');
 await expect(col).toContainText('ModeradorAna');
 await col.scrollIntoViewIfNeeded();
 const narrow=await page.evaluate(auditPage);
 expect(fmt(narrow),`${narrow.length} achado(s) em 1100px`).toEqual([]);
 await page.screenshot({path:'artifacts/estudio-chat-1100.png'});

 await page.setViewportSize({width:1680,height:1000});
 await page.waitForTimeout(250);
 const wide=await page.evaluate(auditPage);
 expect(fmt(wide),`${wide.length} achado(s) em 1680px`).toEqual([]);
 await page.screenshot({path:'artifacts/estudio-chat-largo.png'});
 const overflow=await page.evaluate(()=>document.documentElement.scrollWidth-document.documentElement.clientWidth);
 expect(overflow).toBeLessThanOrEqual(1);
});

test('origem da cor do nome em 3 vias e prefixo antes das badges',async({page})=>{
 await setup(page);
 await enableChat(page);

 // Personalizadas e Fixa salvam; Fixa revela o seletor de cor.
 await page.getByRole('button',{name:'Personalizadas',exact:true}).click();
 await expect.poll(async()=>(await saved(page))?.value?.nameMode,{timeout:3000}).toBe('custom');
 await page.getByRole('button',{name:'Fixa',exact:true}).click();
 await expect(page.getByLabel('Cor do nome',{exact:true})).toBeVisible();
 await expect.poll(async()=>(await saved(page))?.value?.nameMode,{timeout:3000}).toBe('fixed');

 // Prefixo: desligar some, ligar volta com o texto editado.
 const prefix=page.getByRole('switch',{name:'Prefixo antes do nome'});
 await prefix.click();
 await expect.poll(async()=>(await saved(page))?.value?.prefixOn,{timeout:3000}).toBe(false);
 await prefix.click();
 await page.getByLabel('Texto do prefixo').fill('» ');
 await expect.poll(async()=>(await saved(page))?.value?.prefix,{timeout:3000}).toBe('» ');

 // A mensagem de teste entra com o prefixo antes das badges, como o widget.
 await page.getByRole('button',{name:/Moderador/}).click();
 const col=page.locator('.ch-stage .bl-chat-col');
 await expect(col.locator('.bl-chat-prefix')).toHaveText('» ');
 const order=await col.locator('.bl-chat-msg').first().evaluate(el=>{
  const html=el.innerHTML;
  return html.indexOf('bl-chat-prefix')<html.indexOf('bl-chat-badge');
 });
 expect(order).toBe(true);
});

test('eventos na coluna: caixa por evento com cor, texto, som e interruptor',async({page})=>{
 await setup(page);
 await enableChat(page);

 // Ligar os eventos libera os sete tipos na pílula.
 await page.getByRole('switch',{name:'Eventos no chat'}).click();
 await expect.poll(async()=>(await saved(page))?.value?.eventsOn,{timeout:3000}).toBe(true);
 const pills=page.locator('.al-pills');
 for(const label of ['Follow','Sub','Renovação','Sub presente','Presentes em lote','Bits','Raid']){
  await expect(pills.getByRole('button',{name:label,exact:true})).toBeVisible();
 }

 // Raid selecionada: cor própria salva, som da biblioteca e teste desenha a caixa.
 await pills.getByRole('button',{name:'Raid',exact:true}).click();
 await expect(page.getByLabel('Texto da caixa')).toHaveValue('RAID / {name} / {amount} VIEWERS');
 await page.getByLabel('Cor do evento').fill('#336699');
 await expect.poll(async()=>(await saved(page))?.value?.ev?.raid?.color,{timeout:3000}).toBe('#336699');
 await page.getByLabel('Som de Raid').selectOption('snd1');
 await expect.poll(async()=>(await saved(page))?.value?.ev?.raid?.sound,{timeout:3000}).toBe('snd1');

 await page.getByRole('button',{name:/Testar raid/}).click();
 const box=page.locator('.ch-stage .bl-chat-ev');
 await expect(box).toHaveAttribute('data-ev','raid');
 await expect(box).toContainText('RAID / NoiteNaTwitch / 12 VIEWERS');
 expect(await box.evaluate(el=>el.style.getPropertyValue('--ec'))).toBe('#336699');
 await expect(page.locator('.ch-stage .lp-ensaio')).toBeVisible();

 // Desligar só o raid some com o pontinho na pílula…
 await page.getByRole('switch',{name:'Exibir caixa de raid'}).click();
 await expect.poll(async()=>(await saved(page))?.value?.ev?.raid?.on,{timeout:3000}).toBe(false);
 await expect(pills.locator('button').filter({hasText:'Raid'})).toContainText('•');

 // …e o teste avisa em vez de mentir: com o interruptor dele desligado,
 // a live também não mostraria a caixa.
 await page.getByRole('button',{name:/Testar raid/}).click();
 await expect(page.locator('.toast')).toContainText('Ligue o evento de raid no chat');
 await expect(box).toHaveCount(1);
});

test('modo código do chat aplica HTML, CSS e JS na prévia como o OBS faria',async({page})=>{
 await setup(page);
 await enableChat(page);
 await page.getByRole('button',{name:/Comum/}).click();
 const col=page.locator('.ch-stage .bl-chat-col');
 await expect(col).toContainText('Ana');

 // CSS: seletor prefixado com o escopo da prévia e efeito imediato no palco.
 await page.getByRole('tab',{name:'CSS'}).click();
 await page.locator('.cd-panel .cd-code').fill('.bl-chat-msg{background:#001122}');
 await expect(col.locator('.bl-chat-msg').first()).toHaveCSS('background-color','rgb(0, 17, 34)');
 await expect.poll(async()=>(await page.locator('#bl-code-chat').textContent())||'')
  .toContain('.ch-stage .bl-chat-msg');

 // JS: roda com root = a camada (mesma prévia que o OBS executa).
 await page.getByRole('tab',{name:'JS'}).click();
 await page.locator('.cd-panel .cd-code').fill('root.setAttribute("data-js","ok")');
 await expect(page.locator('.ch-stage')).toHaveAttribute('data-js','ok');

 // HTML: markup extra entra no topo do escopo.
 await page.getByRole('tab',{name:'HTML'}).click();
 await page.locator('.cd-panel .cd-code').fill('<div class="miro">moldura</div>');
 await expect(page.locator('.ch-stage [data-bl-code="chat"] .miro')).toHaveText('moldura');

 // Restaurar padrão limpa só o bloco da aba (o CSS continua até a aba CSS).
 await page.getByRole('button',{name:'Restaurar padrão'}).click();
 await expect(page.locator('.ch-stage [data-bl-code="chat"]')).toHaveCount(0);

 // Data: JSON da própria tela; erro fora do formato e aplicação normalizada.
 await page.getByRole('tab',{name:'Data'}).click();
 const data=page.locator('.cd-panel .cd-code');
 await expect(data).toHaveValue(/"nameMode"/);
 await data.fill('{quebrado');
 await page.getByRole('button',{name:'Aplicar'}).click();
 await expect(page.getByRole('alert')).toContainText('JSON inválido');
 await data.fill('[]');
 await page.getByRole('button',{name:'Aplicar'}).click();
 await expect(page.getByRole('alert')).toContainText('Esperado um objeto');
 await data.fill(JSON.stringify({nameMode:'custom',max:42}));
 await page.getByRole('button',{name:'Aplicar'}).click();
 await expect.poll(async()=>(await saved(page))?.value?.nameMode,{timeout:3000}).toBe('custom');
 await expect.poll(async()=>(await saved(page))?.value?.max,{timeout:3000}).toBe(42);
 // Sucesso devolve à aba Fields com o formulário completo.
 await expect(page.getByRole('heading',{name:'Coluna',exact:true})).toBeVisible();
});
