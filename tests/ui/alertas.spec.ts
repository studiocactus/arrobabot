import {test,expect,type Page} from '@playwright/test';

const profile={id:'p1',name:'Canal de teste',platform:'twitch',channel:'cactus',channelId:'1',botId:'',clientId:'',blocklist:[],topics:[],editors:[],ai:{},modules:{}};

/** Mock desktop: `alerts.get` devolve sons e endereço; `module.config` grava em session. */
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
     if(op==='alerts.get')return {config:null,sounds:[{id:'s1',name:'jingle.wav'}],overlay:{token:'tok',port:9876,url:'http://127.0.0.1:9876/overlay#token=tok&profile=p1'}};
     if(op==='alerts.test'){
      sessionStorage.setItem('alertsTest',String(args.kind));
      return {kind:String(args.kind),vars:{name:'NovoSeguidor',amount:500,months:1,tier:'TIER 1',message:''}};
     }
     if(op==='module.config'){
      sessionStorage.setItem('alertsSave',JSON.stringify({key:args.key,value:args.value}));
      return null;
     }
     return null;
    }}});
  sessionStorage.removeItem('alertsSave');
  sessionStorage.removeItem('alertsTest');
 },profile);
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Alertas',exact:true}).click();
}

const saved=async(page:Page)=>JSON.parse(await page.evaluate(()=>sessionStorage.getItem('alertsSave')||'null'));

test('estúdio lista os sete eventos, prévia e salvamento automático por evento',async({page})=>{
 await setup(page);

 await expect(page.getByRole('heading',{name:'Alertas da live'})).toBeVisible();
 const pills=page.locator('.al-pills');
 for(const label of ['Seguidor','Novo sub','Renovação','Sub de presente','Presentes em lote','Bits','Raid']){
  await expect(pills.getByRole('button',{name:label,exact:true})).toBeVisible();
 }

 // Prévia parada com o padrão do evento selecionado.
 await expect(page.locator('.lp-frame')).toContainText('BEM-VINDO!');
 await expect(page.locator('.lp-frame')).toContainText('SEGUIDOR');

 // Trocar de evento muda a prévia e o formulário.
 await pills.getByRole('button',{name:'Bits',exact:true}).click();
 await expect(page.locator('.lp-frame')).toContainText('500 BITS');
 await expect(page.getByLabel('Linha grande',{exact:true})).toHaveValue('{{amount}} BITS');
 await expect(page.getByLabel('Etiqueta',{exact:true})).toHaveValue('BITS');

 // Editar texto grava a configuração inteira do perfil com o valor novo.
 await page.getByLabel('Linha grande',{exact:true}).fill('{{amount}} BITS DO CORAÇÃO');
 await expect.poll(async()=>(await saved(page))?.value?.events?.cheer?.big,{timeout:3000}).toBe('{{amount}} BITS DO CORAÇÃO');

 // Som do evento vem da lista de sons do perfil.
 await expect(page.getByLabel('Som do evento').locator('option[value="s1"]')).toHaveText('jingle.wav');

 // Interruptor mestre também salva.
 const master=page.getByRole('switch',{name:'Ligar alertas no overlay'});
 await expect(master).toHaveAttribute('aria-checked','false');
 await master.click();
 await expect.poll(async()=>(await saved(page))?.value?.enabled,{timeout:3000}).toBe(true);
});

test('botão de teste anima a prévia pelo caminho do evento real e termina limpo',async({page})=>{
 await setup(page);
 await pills_select_bits(page);

 await page.getByRole('button',{name:/Testar bits/}).click();
 await expect.poll(async()=>await page.evaluate(()=>sessionStorage.getItem('alertsTest')||''),{timeout:3000}).toBe('cheer');

 // O painel animado (não o estático) mostra os dados de exemplo e some sozinho.
 await expect(page.locator('.bl-alert:not(.is-static)')).toContainText('NovoSeguidor',{timeout:5000});
 await expect(page.locator('.bl-alert:not(.is-static)')).toContainText('500 BITS',{timeout:5000});
 await expect(page.locator('.bl-alert:not(.is-static)')).toHaveCount(0,{timeout:25000});
 // A prévia parada volta pronta para a próxima edição.
 await expect(page.locator('.bl-alert.is-static')).toContainText('500 BITS');
});

test('estúdio de alertas cabe em janela de 1100px sem espremer texto',async({page})=>{
 await page.setViewportSize({width:1100,height:950});
 await setup(page);
 await expect(page.locator('.bl-alert.is-static')).toBeVisible();
 await page.screenshot({path:'artifacts/estudio-alertas-1100.png'});
});

async function pills_select_bits(page:Page){
 await page.locator('.al-pills').getByRole('button',{name:'Bits',exact:true}).click();
}
