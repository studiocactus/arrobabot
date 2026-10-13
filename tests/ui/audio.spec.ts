import {test,expect,type Page} from '@playwright/test';

const profile={id:'p1',name:'Canal de teste',platform:'twitch',channel:'cactus',channelId:'1',botId:'',clientId:'',blocklist:[],topics:[],editors:[],ai:{},modules:{}};

/** Mock desktop: `audio.get` devolve a config da tela Áudio e os aparelhos da voz. */
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
     if(op==='audio.get')return {config:{device:''},voiceFx:{selection:{input:'Microfone (Realtek)',output:'CABLE Input (VB-Audio Virtual Cable)'},devices:{inputs:[{name:'Microfone (Realtek)',default:true}],outputs:[{name:'CABLE Input (VB-Audio Virtual Cable)',default:true}]}},status:{phase:'off',message:'Prévia no navegador: a voz ao vivo só existe no aplicativo desktop.',input:'',output:''}};
     if(op==='voiceFx.saveSelection'){sessionStorage.setItem('voiceSave',JSON.stringify(args));return null}
     if(op==='module.config'){sessionStorage.setItem('audioSave',JSON.stringify({key:args.key,value:args.value}));return null}
     return null;
    }}});
  // Uma saída só no seletor: o CABLE que o OBS captura como fonte única.
  Object.defineProperty(navigator,'mediaDevices',{configurable:true,value:{
   enumerateDevices:async()=>[{kind:'audiooutput',deviceId:'cable-1',label:'CABLE Input (VB-Audio)',groupId:''}],
   addEventListener:()=>{},removeEventListener:()=>{}
  }});
  sessionStorage.removeItem('audioSave');
 },profile);
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Áudio',exact:true}).click();
}

const saved=async(page:Page)=>JSON.parse(await page.evaluate(()=>sessionStorage.getItem('audioSave')||'null'));

test('tela Áudio leva todos os sons para uma saída só e salva o aparelho',async({page})=>{
 await setup(page);

 await expect(page.getByRole('heading',{name:'Saída dos sons do bot'})).toBeVisible();

 // O modelo novo dito na própria tela: uma fonte de áudio no OBS para o bot inteiro.
 await expect(page.getByText('uma única Fonte de áudio')).toBeVisible();

 // Escolher o aparelho grava a config {device} sozinha, com a chave certa.
 const device=page.getByRole('combobox',{name:'Dispositivo de saída dos sons'});
 await expect(device).toHaveValue('');
 await device.selectOption('cable-1');
 await expect.poll(async()=>(await saved(page))?.key,{timeout:3000}).toBe('audio');
 await expect.poll(async()=>(await saved(page))?.value?.device,{timeout:3000}).toBe('cable-1');

 // Os interruptores por origem não existem mais: só a saída e a voz.
 await expect(page.getByRole('switch',{name:'Sons de espectador na linha de áudio'})).toHaveCount(0);
 await expect(page.getByRole('heading',{name:'Voz da live — linha do microfone'})).toBeVisible();

 // O TTS declara a limitação: saída padrão do Windows.
 await expect(page.getByText('saída padrão do Windows').first()).toBeVisible();
});

test('aparelhos da voz da live ficam na mesma tela e salvam ao trocar',async({page})=>{
 await setup(page);

 const mic=page.getByRole('combobox',{name:'Microfone de entrada'});
 await expect(mic).toHaveValue('Microfone (Realtek)');
 await expect(page.getByRole('combobox',{name:'Saída da linha (VB-Cable)'})).toHaveValue('CABLE Input (VB-Audio Virtual Cable)');
 await mic.selectOption('');
 await expect.poll(async()=>page.evaluate(()=>sessionStorage.getItem('voiceSave')),{timeout:3000}).toContain('"input":""');
});

test('tela de áudio cabe em 1100px sem espremer texto',async({page})=>{
 await page.setViewportSize({width:1100,height:950});
 await setup(page);
 await expect(page.getByRole('heading',{name:'Saída dos sons do bot'})).toBeVisible();
 await page.screenshot({path:'artifacts/saida-audio-1100.png'});
});
