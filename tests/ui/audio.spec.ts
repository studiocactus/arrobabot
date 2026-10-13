import {test,expect,type Page} from '@playwright/test';

const profile={id:'p1',name:'Canal de teste',platform:'twitch',channel:'cactus',channelId:'1',botId:'',clientId:'',blocklist:[],topics:[],editors:[],ai:{},modules:{}};

/** Mock desktop: `audio.get` devolve a config da Central e os aparelhos da voz. */
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
     if(op==='audio.get')return {config:null,voiceFx:{selection:{input:'Microfone (Realtek)',output:'CABLE Input (VB-Audio Virtual Cable)'},devices:{inputs:[{name:'Microfone (Realtek)',default:true}],outputs:[{name:'CABLE Input (VB-Audio Virtual Cable)',default:true}]}},status:{phase:'off',message:'Prévia no navegador: a voz ao vivo só existe no aplicativo desktop.',input:'',output:''}};
     if(op==='voiceFx.saveSelection'){sessionStorage.setItem('voiceSave',JSON.stringify(args));return null}
     if(op==='module.config'){sessionStorage.setItem('audioSave',JSON.stringify({key:args.key,value:args.value}));return null}
     return null;
    }}});
  sessionStorage.removeItem('audioSave');
 },profile);
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Áudio',exact:true}).click();
}

const saved=async(page:Page)=>JSON.parse(await page.evaluate(()=>sessionStorage.getItem('audioSave')||'null'));

test('central de áudio roteia cada origem por linha e navegador e salva sozinha',async({page})=>{
 await setup(page);

 await expect(page.getByRole('heading',{name:'Para onde vai cada som'})).toBeVisible();

 // Padrão do backend: sons na linha, alertas no navegador.
 await expect(page.getByRole('switch',{name:'Sons de espectador na linha de áudio'})).toHaveAttribute('aria-checked','true');
 await expect(page.getByRole('switch',{name:'Sons de espectador no navegador'})).toHaveAttribute('aria-checked','false');
 await expect(page.getByRole('switch',{name:'Alertas da live no navegador'})).toHaveAttribute('aria-checked','true');
 await expect(page.getByRole('switch',{name:'Alertas da live na linha de áudio'})).toHaveAttribute('aria-checked','false');

 // Ligar a linha dos alertas salva a configuração inteira com a chave certa.
 await page.getByRole('switch',{name:'Alertas da live na linha de áudio'}).click();
 await expect.poll(async()=>(await saved(page))?.key,{timeout:3000}).toBe('audio');
 await expect.poll(async()=>(await saved(page))?.value?.alertLine,{timeout:3000}).toBe(true);

 // Som do espectador para o navegador, fora da linha.
 await page.getByRole('switch',{name:'Sons de espectador no navegador'}).click();
 await page.getByRole('switch',{name:'Sons de espectador na linha de áudio'}).click();
 await expect.poll(async()=>{const v=(await saved(page))?.value;return [v?.peopleLine,v?.peopleBrowser]},{timeout:3000}).toEqual([false,true]);

 // Origens independentes: o fluxo não acompanha o espectador.
 await expect(page.getByRole('switch',{name:'Sons de fluxo na linha de áudio'})).toHaveAttribute('aria-checked','true');
});

test('aparelhos da linha e da voz da live ficam na mesma tela com o TTS declarado',async({page})=>{
 await setup(page);

 // Voz da live: aparelhos vindos do voiceFx, salvos ao trocar.
 const mic=page.getByRole('combobox',{name:'Microfone de entrada'});
 await expect(mic).toHaveValue('Microfone (Realtek)');
 await expect(page.getByRole('combobox',{name:'Saída da linha (VB-Cable)'})).toHaveValue('CABLE Input (VB-Audio Virtual Cable)');
 await mic.selectOption('');
 await expect.poll(async()=>page.evaluate(()=>sessionStorage.getItem('voiceSave')),{timeout:3000}).toContain('"input":""');

 // O TTS aparece com a limitação declarada: saída padrão do Windows.
 await expect(page.getByText('saída padrão do Windows').first()).toBeVisible();
});

test('central de áudio cabe em 1100px sem espremer texto',async({page})=>{
 await page.setViewportSize({width:1100,height:950});
 await setup(page);
 await expect(page.getByRole('heading',{name:'Voz da live — linha do microfone'})).toBeVisible();
 await page.screenshot({path:'artifacts/central-audio-1100.png'});
});
