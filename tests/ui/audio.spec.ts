import {test,expect,type Page} from '@playwright/test';

const profile={id:'p1',name:'Canal de teste',platform:'twitch',channel:'cactus',channelId:'1',botId:'',clientId:'',blocklist:[],topics:[],editors:[],ai:{},modules:{}};
const URL='http://127.0.0.1:9876/audio#token=tok123&profile=p1';

/**
 * Mock desktop: `audio.get` devolve o endereço da fonte e quantas fontes estão
 * conectadas; `audio.test`/`audio.stop` registram os argumentos em
 * sessionStorage para o teste conferir.
 */
async function setup(page:Page,sink:{url:string;clients:number;claimed:boolean},testResult:{id:string;sink:string}={id:'t1',sink:'local'}){
 await page.addInitScript(p=>{
  Object.assign(window,{isTauri:true,
   __TAURI_EVENT_PLUGIN_INTERNALS__:{registerListener:()=>{},unregisterListener:()=>{}},
   __TAURI_INTERNALS__:{transformCallback:()=>1,unregisterCallback:()=>{},
    invoke:async(cmd:string,payload:{op:string;args?:Record<string,unknown>})=>{
     if(cmd!=='rpc')return 1;
     const op=payload.op;const args=payload.args||{};
     if(op==='snapshot')return {profiles:[p.profile],flows:[],notes:{},presets:[],theme:'dark',logs:[],statuses:{},apiPort:9876,dataDir:'teste'};
     if(op==='flows')return [];
     if(op==='command.counters')return {};
     if(op==='settings.get')return {theme:'dark',apiPort:9876,apiToken:'',updateEndpoint:'',updatePublicKey:'',autoUpdate:false};
     if(op==='audio.get')return p.sink;
     if(op==='audio.test'){sessionStorage.setItem('audioTest',JSON.stringify(args));return p.testResult}
     if(op==='audio.stop'){sessionStorage.setItem('audioStop',JSON.stringify(args));return {stopped:p.sink.claimed}}
     return null;
    }}});
  sessionStorage.removeItem('audioTest');sessionStorage.removeItem('audioStop');
 },{sink,testResult,profile});
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Áudio no OBS',exact:true}).click();
}

test('tela Áudio no OBS dita o caminho único, copia o endereço e testa pelo mesmo caminho',async({page})=>{
 await page.context().grantPermissions(['clipboard-read','clipboard-write']);
 await setup(page,{url:URL,clients:0,claimed:false});

 await expect(page.getByRole('heading',{name:'Áudio no OBS'}).first()).toBeVisible();

 // O modelo dito na própria tela: uma fonte de navegador no OBS, um fader.
 await expect(page.getByText('única saída de sons do BotLive')).toBeVisible();
 await expect(page.getByText('“Controlar áudio pelo OBS”', {exact:true})).toBeVisible();

 // Estado real: sem fonte conectada, o painel espera o OBS.
 await expect(page.getByText('Aguardando o OBS')).toBeVisible();

 // Endereço visível, selecionável e copiável.
 const url=page.getByLabel('Endereço da fonte de áudio');
 await expect(url).toHaveValue(URL);
 await page.getByRole('button',{name:'Copiar endereço'}).click();
 await expect(page.getByRole('button',{name:'Copiado'})).toBeVisible();
 await expect.poll(()=>page.evaluate(()=>navigator.clipboard.readText())).toBe(URL);

 // Testar sem fonte conectada: o backend toca local e a tela avisa sem fingir sucesso.
 await page.getByRole('button',{name:'Testar áudio'}).click();
 await expect(page.getByRole('status')).toContainText('A fonte do OBS ainda não está conectada');
 await expect.poll(async()=>page.evaluate(()=>sessionStorage.getItem('audioTest'))).toContain('"profileId":"p1"');

 // Parar todos os sons limpa a fila local e manda para a fonte.
 await page.getByRole('button',{name:'Parar todos os sons'}).click();
 await expect(page.getByRole('status')).toContainText('Todos os sons pararam');
 await expect.poll(async()=>page.evaluate(()=>sessionStorage.getItem('audioStop'))).toContain('"profileId":"p1"');

 // A seleção de aparelho de saída não existe mais nesta tela (o único seletor
 // da página de áudio seria de dispositivo — o global é "Perfil ativo", do
 // cabeçalho, e continua de pé).
 await expect(page.getByRole('combobox',{name:/aparelho|dispositivo|saída/i})).toHaveCount(0);
 await expect(page.getByText('VB-Cable deixou de ser necessário')).toBeVisible();
});

test('com a fonte conectada o teste segue o mesmo caminho, sem aviso de bloqueio',async({page})=>{
 await setup(page,{url:URL,clients:1,claimed:true},{id:'t-obs',sink:'obs'});

 await expect(page.getByText('Conectada',{})).toBeVisible();

 await page.getByRole('button',{name:'Testar áudio'}).click();
 // "Enviado" antes da confirmação: enviado não é "o público ouvindo".
 await expect(page.getByText('Som enviado — aguardando a fonte do OBS confirmar…')).toBeVisible();
 // Sem erro de bloqueio: com a fonte conectada o som foi para ela.
 await expect(page.getByRole('alert')).toHaveCount(0);
});

test('duas fontes conectadas aparecem como aviso, sem fingir que tudo bem',async({page})=>{
 await setup(page,{url:URL,clients:2,claimed:true});
 await expect(page.getByText('Conectada — 2 fontes; só a primeira reproduz')).toBeVisible();
});

test('tela de áudio cabe em 1100px e em janela larga sem espremer texto',async({page})=>{
 await page.setViewportSize({width:1100,height:950});
 await setup(page,{url:URL,clients:0,claimed:false});
 await expect(page.getByRole('heading',{name:'Áudio no OBS'}).first()).toBeVisible();
 const noScroll=await page.evaluate(()=>document.documentElement.scrollWidth<=window.innerWidth);
 expect(noScroll,'sem rolagem horizontal a 1100px').toBe(true);
 await page.screenshot({path:'artifacts/saida-audio-1100.png'});
 await page.setViewportSize({width:1680,height:1000});
 await page.screenshot({path:'artifacts/saida-audio-larga.png'});
});
