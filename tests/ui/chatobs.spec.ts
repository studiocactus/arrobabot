import {test,expect,type Page} from '@playwright/test';

const profile={id:'p1',name:'Canal de teste',platform:'twitch',channel:'cactus',channelId:'1',botId:'',clientId:'',blocklist:[],topics:[],editors:[],ai:{},modules:{}};

/** Mock desktop: `chat.get` devolve a config e o endereço do overlay. */
async function setup(page:Page){
 await page.addInitScript(p=>{
  Object.assign(window,{isTauri:true,
   __TAURI_EVENT_PLUGIN_INTERNALS__:{registerListener:()=>{},unregisterListener:()=>{}},
   __TAURI_INTERNALS__:{transformCallback:()=>1,unregisterCallback:()=>{},
    invoke:async(cmd:string,payload:{op:string;args?:Record<string,unknown>})=>{
     if(cmd!=='rpc')return 1;
     const op=payload.op;const args=payload.args||{};
     if(op==='snapshot')return {profiles:[p],flows:[],notes:{},presets:[],theme:'dark',logs:[],statuses:{},apiPort:9876,dataDir:'teste'};
     if(op==='flows')return [];
     if(op==='command.counters')return {};
     if(op==='settings.get')return {theme:'dark',apiPort:9876,apiToken:'',updateEndpoint:'',updatePublicKey:'',autoUpdate:false};
     if(op==='chat.get')return {config:null,overlay:{token:'tok',port:9876,url:'http://127.0.0.1:9876/overlay#token=tok&profile=p1'}};
     if(op==='module.config'){
      sessionStorage.setItem('chatSave',JSON.stringify({key:args.key,value:args.value}));
      return null;
     }
     return null;
    }}});
  sessionStorage.removeItem('chatSave');
 },profile);
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Chat no OBS',exact:true}).click();
}

const saved=async(page:Page)=>JSON.parse(await page.evaluate(()=>sessionStorage.getItem('chatSave')||'null'));

test('estúdio de chat empilha mensagens de teste pelo motor real e salva sozinho',async({page})=>{
 await setup(page);

 await expect(page.getByRole('heading',{name:'Chat no overlay'})).toBeVisible();

 // Mensagem comum entra com nome, separador e texto.
 await page.getByRole('button',{name:/Comum/}).click();
 const col=page.locator('.ch-stage .bl-chat-col');
 await expect(col).toContainText('Ana');
 await expect(col).toContainText('Boa noite, gente!');

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

 // Interruptor mestre salva a configuração inteira.
 const master=page.getByRole('switch',{name:'Ligar chat no overlay'});
 await expect(master).toHaveAttribute('aria-checked','false');
 await master.click();
 await expect.poll(async()=>(await saved(page))?.value?.enabled,{timeout:3000}).toBe(true);

 // Direção e some-apois também entram no mesmo salvamento.
 await page.getByRole('button',{name:'Novas no topo'}).click();
 await expect.poll(async()=>(await saved(page))?.value?.direction,{timeout:3000}).toBe('down');
});

test('filtros do estúdio escondem os exemplos correspondentes',async({page})=>{
 await setup(page);

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

test('estúdio de chat cabe em janela de 1100px sem espremer texto',async({page})=>{
 await page.setViewportSize({width:1100,height:950});
 await setup(page);
 await page.getByRole('button',{name:/Moderador/}).click();
 await page.getByRole('button',{name:/Em emote/}).click();
 await page.getByRole('button',{name:/Primeira vez/}).click();
 const col=page.locator('.ch-stage .bl-chat-col');
 await expect(col).toContainText('ModeradorAna');
 await col.scrollIntoViewIfNeeded();
 await page.screenshot({path:'artifacts/estudio-chat-1100.png'});
});
