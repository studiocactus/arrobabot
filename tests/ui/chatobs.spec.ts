import {test,expect,type Page} from '@playwright/test';
import {auditPage,fmt} from './ux-helpers';

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
    if(op==='chatExtras.get')return {config:{audioDeviceId:''},assets:[{id:'snd1',kind:'sound',name:'ping.wav'}]};
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

test('estúdio de chat cabe em janela de 1100px e ampla sem espremer texto',async({page})=>{
 await page.setViewportSize({width:1100,height:950});
 await setup(page);
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

 // Mestre desligado não mostra nada; ligando, os sete tipos aparecem.
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

 // Desligar só o raid some com o pontinho na pílula.
 await page.getByRole('switch',{name:'Exibir caixa de raid'}).click();
 await expect.poll(async()=>(await saved(page))?.value?.ev?.raid?.on,{timeout:3000}).toBe(false);
 await expect(pills.locator('button').filter({hasText:'Raid'})).toContainText('•');
});

test('modo código do chat aplica HTML, CSS e JS na prévia como o OBS faria',async({page})=>{
 await setup(page);
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
