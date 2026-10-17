import {test,expect} from '@playwright/test';
import {OUT} from './ux-helpers';

test.use({permissions:['microphone'],launchOptions:{args:['--use-fake-device-for-media-stream','--use-fake-ui-for-media-stream']}});

const profile={id:'p1',name:'Canal de teste',platform:'twitch',channel:'cactus',channelId:'1',botId:'',clientId:'',blocklist:[],topics:[],editors:[],ai:{},modules:{voice:true}};

test('painel de voz expõe a escuta contínua e avisa que a prévia não captura o microfone',async({page})=>{
 await page.addInitScript(p=>{
  localStorage.setItem('botlive-preview',JSON.stringify({profiles:[p],flows:[],notes:{},presets:[],theme:'dark'}));
 },profile);
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Comunidade',exact:true}).click();
 await page.getByRole('button',{name:'Abrir Controle por voz',exact:true}).click();
 await expect(page.getByLabel('Servidor de transcrição (RealtimeSTT)')).toHaveValue('http://127.0.0.1:8010/transcribe-pcm16');
 await expect(page.getByLabel('Idioma falado')).toHaveValue('pt');
 await expect(page.getByLabel('Palavras de ativação')).toHaveValue('');
 await expect(page.getByLabel('Duração do “quicar” (segundos)')).toHaveValue('600');
 await expect(page.getByLabel('Microfone físico')).toBeVisible();
 await expect(page.getByText('Abra o aplicativo desktop para ligar a escuta contínua; a prévia do navegador não captura o microfone.')).toBeVisible();
 await expect(page.getByText('Nenhum áudio sai desta máquina.')).toBeVisible();
 await expect(page.getByRole('switch',{name:'Legenda no overlay'})).toHaveAttribute('aria-checked','true');
 const toggle=page.getByRole('switch',{name:'Escuta contínua',exact:true});
 await expect(toggle).toHaveAttribute('aria-checked','false');
 await toggle.click();
 await expect(toggle).toHaveAttribute('aria-checked','false');
});

test('escuta contínua abre o microfone, manda lotes de áudio e desliga quando pedida',async({page})=>{
 await page.addInitScript(p=>{
  sessionStorage.setItem('voiceFrames','0');
  const status={enabled:true,listening:true,frames:0,heard:0,last:'',error:'',endpoint:'http://127.0.0.1:8010/transcribe-pcm16',language:'pt',activation:'arroba',captions:true};
  Object.assign(window,{isTauri:true,__TAURI_INTERNALS__:{transformCallback:()=>1,unregisterCallback:()=>{},invoke:async(cmd:string,payload:{op:string;args?:Record<string,unknown>})=>{
   if(cmd!=='rpc')return 1;
   const op=payload.op;const args=payload.args||{};
   if(op==='snapshot')return {profiles:[p],flows:[],notes:{},presets:[],theme:'dark',logs:[],statuses:{},apiPort:9876,dataDir:'teste'};
   if(op==='flows')return [];
   if(op==='module.config.get')return {};
   if(op==='module.config'){sessionStorage.setItem('voiceConfig',JSON.stringify(args.value));return null}
   if(op==='voice.frame'){
    const done=Number(sessionStorage.getItem('voiceFrames')||0)+1;
    sessionStorage.setItem('voiceFrames',String(done));
    sessionStorage.setItem('voicePcm',String(args.pcm));
    return {...status,frames:done};
   }
   if(op==='voice.status')return status;
   if(op==='voice.listenStop'){sessionStorage.setItem('voiceStopped','1');return null}
   if(op==='command.counters')return {};
   if(op==='settings.get')return {updateEndpoint:'',updatePublicKey:'',autoUpdate:false};
   return null;
  }}});
 },profile);
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Comunidade',exact:true}).click();
 await page.getByRole('button',{name:'Abrir Controle por voz',exact:true}).click();

 const toggle=page.getByRole('switch',{name:'Escuta contínua',exact:true});
 await page.getByLabel('Servidor de transcrição (RealtimeSTT)').fill('http://127.0.0.1:9010/transcribe-pcm16');
 await toggle.click();
 await expect(toggle).toHaveAttribute('aria-checked','true');
 await expect(page.getByText(/^Ouvindo o microfone ·/)).toBeVisible();
 await expect.poll(async()=>Number(await page.evaluate(()=>sessionStorage.getItem('voiceFrames')||0))).toBeGreaterThan(2);
 const pcm=await page.evaluate(()=>sessionStorage.getItem('voicePcm')||'');
 expect(pcm.length).toBeGreaterThan(100,'cada lote leva amostras de verdade');
 expect(/^[A-Za-z0-9+/=]+$/.test(pcm),'o lote sai em base64').toBe(true);
 const saved=JSON.parse(await page.evaluate(()=>sessionStorage.getItem('voiceConfig')||'{}'));
 expect(saved.listen).toBe(true);
 expect(saved.listenEndpoint).toBe('http://127.0.0.1:9010/transcribe-pcm16');

 await toggle.click();
 await expect(toggle).toHaveAttribute('aria-checked','false');
 await expect(page.getByText(/^Ouvindo o microfone ·/)).toHaveCount(0);
 // Parar também descarta transcrições em curso no backend (guarda de sessão).
 await expect.poll(async()=>await page.evaluate(()=>sessionStorage.getItem('voiceStopped'))).toBe('1');
 const stopped=Number(await page.evaluate(()=>sessionStorage.getItem('voiceFrames')||0));
 await page.waitForTimeout(700);
 expect(Number(await page.evaluate(()=>sessionStorage.getItem('voiceFrames')||0))).toBe(stopped,'o microfone para de enviar lotes');
});

test('hub da voz reúne os três caminhos com diagnóstico honesto',async({page})=>{
 await page.addInitScript(p=>{
  localStorage.setItem('botlive-preview',JSON.stringify({profiles:[p],flows:[],notes:{},presets:[],theme:'dark'}));
 },profile);
 await page.setViewportSize({width:1100,height:900});
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Voz da live',exact:true}).click();
 // Três seções, na ordem prometida.
 const cards=page.locator('.card-heading h3');
 await expect(cards.nth(0)).toHaveText('Minha voz na live');
 await expect(page.getByRole('heading',{name:'Comandos do Arroba',exact:true})).toBeVisible();
 await expect(page.getByRole('heading',{name:'Resgates de voz (Voicemod)',exact:true})).toBeVisible();
 // Caminho da voz: duas opções exclusivas, nunca as duas capturando juntas.
 const caminho=page.getByLabel('Caminho da voz');
 await expect(caminho).toBeVisible();
 await expect(caminho.locator('option')).toHaveCount(2);
 // Diagnóstico mostra o estado real, sem promessas.
 await expect(page.getByText('Microfone (captura interna)')).toBeVisible();
 await expect(page.getByText('não conectado — necessário só no caminho Voicemod')).toBeVisible();
 await expect(page.getByText('parada — ligue em “Comandos do Arroba”')).toBeVisible();
 // Fila de resgates: a prévia sem núcleo diz a verdade em vez de fingir.
 await expect(page.getByText('Sem estado ainda — abra o aplicativo desktop para ver a fila viva.')).toBeVisible();
 // Comandos do Arroba embarcados no hub, com os mesmos campos do módulo.
 await expect(page.getByLabel('Palavras de ativação')).toBeVisible();
 await expect(page.getByText('Nenhum áudio sai desta máquina.')).toBeVisible();
 await page.screenshot({path:`${OUT}\\voz-hub-1100.png`,fullPage:true});
 await page.setViewportSize({width:1600,height:1000});
 await page.screenshot({path:`${OUT}\\voz-hub-largo.png`,fullPage:true});
});
