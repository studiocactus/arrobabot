import {test,expect,type Page} from '@playwright/test';
import {join} from 'path';
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

/** Mock desktop da tela Voicemod: conexão, vozes reais e regras. */
async function mockVoicemod(page:Page,opts:{legacy?:boolean;rules?:unknown[]}={}){
 await page.addInitScript((p)=>{
  Object.assign(window,{isTauri:true,__TAURI_EVENT_PLUGIN_INTERNALS__:{registerListener:()=>{},unregisterListener:()=>{}},
   __TAURI_INTERNALS__:{transformCallback:()=>1,unregisterCallback:()=>{},invoke:async(cmd:string,payload:{op:string;args?:Record<string,unknown>})=>{
    if(cmd!=='rpc')return 1;
    const op=payload.op;const args=payload.args||{};
    if(op==='snapshot')return {profiles:[p.profile],flows:[],notes:{},presets:[],theme:'dark',logs:[],statuses:{},apiPort:9876,dataDir:'teste'};
    if(op==='flows')return [];
    if(op==='command.counters')return {};
    if(op==='settings.get')return {theme:'dark',apiPort:9876,apiToken:'',updateEndpoint:'',updatePublicKey:'',autoUpdate:false};
    if(op==='voicemod.get')return {status:{phase:'connected',message:'Conectado nesta máquina',port:59129,voices:[{voiceID:'robot',name:'Robô'},{voiceID:'alien',name:'Alien'}],currentVoice:'nofx',currentName:'Efeito desligado',voiceChanger:true,hearMyself:false,license:'Pro',attempts:1},test:{active:false,phase:'',voiceId:'',voiceName:'',seconds:10,remainingMs:0,interrupted:false,manual:false},outcome:{kind:'',detail:''},defaultSecs:10,minSecs:5,maxSecs:60,hasKey:true,keyLen:24};
    if(op==='voiceResgate.get')return {queue:{active:null,upcoming:[],last:'',lastKind:'',pendingRestore:false,busy:false},config:{enabled:true,queueLimit:3,bindings:p.rules,legacyMap:{},obsChecked:false}};
    if(op==='twitch.rewards')return {channel:'cactus',rewards:[{id:'r1',title:'Voz de robô',cost:100,enabled:true},{id:'r2',title:'Voz alienígena',cost:200,enabled:true}]};
    if(op==='voiceFx.get')return {phase:p.legacy?'running':'off',message:'',input:'',output:'',gate_db:-45};
    if(op==='voiceFx.selection')return p.legacy?{input:'Microfone (Realtek)',output:'CABLE Input (VB-Audio Virtual Cable)',gateDb:-45,mode:'internal'}:{gateDb:-45};
    if(op==='voiceResgate.save'){sessionStorage.setItem('rulesSaved',JSON.stringify(args.config));return null}
    return null;
   }}});
 },{profile,rules:opts.rules??[{kind:'reward',rewardId:'r1',rewardName:'Voz de robô',command:'',permission:'moderator',enabled:true,voiceId:'robot',voiceName:'Robô',durationSec:30,cooldownSec:0,concurrency:'queue'}],legacy:!!opts.legacy});
}

test('tela Voicemod tem as três etapas, um caminho só e nada de VB-Cable no caminho principal',async({page})=>{
 await mockVoicemod(page);
 await page.setViewportSize({width:1100,height:950});
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Voicemod',exact:true}).click();

 // As seções viram o acordeão do nosso padrão: todas presentes, uma aberta por vez.
 const intro=page.getByRole('button',{name:/Sua voz no Voicemod/});
 const live=page.getByRole('button',{name:/Durante a live/});
 await expect(intro).toBeVisible();
 await expect(live).toBeVisible();
 await expect(page.getByRole('button',{name:/Etapa 1 · Conectar Voicemod/})).toBeVisible();
 await expect(page.getByRole('button',{name:/Etapa 2 · Ouvir minha voz no OBS/})).toBeVisible();
 await expect(page.getByRole('button',{name:/Etapa 3 · Escolher o que troca minha voz/})).toBeVisible();

 // Abrir a apresentação mostra o caminho dito em linguagem simples.
 await intro.click();
 await expect(page.getByText(/pede ao Voicemod para trocar sua voz/)).toBeVisible();

 // Um caminho só: o seletor de motor interno não existe mais na experiência.
 await expect(page.getByLabel('Caminho da voz')).toHaveCount(0);
 // VB-Cable não aparece no caminho principal (só na migração, que fica
 // oculta quando o perfil não tem efeitos internos).
 await expect(page.getByText('VB-Cable')).toHaveCount(0);
 // Distinção honesta: conectado ≠ áudio conferido, cada um na sua seção.
 await live.click();
 await expect(page.getByText('Voicemod conectado',{exact:true}).first()).toBeVisible();
 // Botão visível para encerrar e restaurar a voz.
 await expect(page.getByRole('button',{name:/Encerrar efeitos e restaurar minha voz/})).toBeVisible();
 await page.getByRole('button',{name:/Etapa 2 · Ouvir minha voz no OBS/}).click();
 await expect(page.getByText('Áudio conferido no OBS')).toBeVisible();
 await expect(page.getByRole('switch',{name:'Ouvi minha voz no OBS'})).toBeVisible();

 const overflow=await page.evaluate(()=>document.documentElement.scrollWidth<=window.innerWidth);
 expect(overflow,'sem rolagem horizontal a 1100px').toBe(true);
 await page.screenshot({path:`${OUT}\\voicemod-1100.png`,fullPage:true});
 await page.setViewportSize({width:1600,height:1000});
 await page.screenshot({path:`${OUT}\\voicemod-largo.png`,fullPage:true});
});

test('regra de troca de voz pede acionamento, voz, duração e ativação — com Avançado recolhido',async({page})=>{
 await mockVoicemod(page);
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Voicemod',exact:true}).click();

 // As regras vivem na Etapa 3, agora em acordeão.
 await page.getByRole('button',{name:/Etapa 3 · Escolher o que troca minha voz/}).click();

 // A regra existente se apresenta pelo resumo legível.
 await expect(page.getByRole('heading',{name:'Voz de robô → Robô → 30 segundos'})).toBeVisible();
 await expect(page.getByRole('switch',{name:'Ativada'})).toBeVisible();

 // Adicionar abre a regra com os campos essenciais.
 await page.getByRole('button',{name:'Adicionar troca de voz'}).click();
 const newCard=page.locator('.voice-rules section.card').last();
 const kinds=newCard.getByLabel('Acionamento');
 await expect(kinds).toBeVisible();
 // Comandos do chat: permissão explícita em Avançado, nunca “qualquer pessoa” por padrão.
 await kinds.selectOption('command');
 await expect(newCard.getByLabel('Comando do chat')).toBeVisible();
 await newCard.locator('summary').click();
 const perm=newCard.getByLabel('Quem pode usar o comando');
 await expect(perm).toBeVisible();
 await expect(perm).toHaveValue('moderator','nada vira público sem você escolher');
 // Recompensas usam a lista real do canal, mostrada no próprio contexto.
 await kinds.selectOption('reward');
 await expect(newCard.getByLabel('Recompensa')).toBeVisible();
 // O teste de regra fica à mão e explica que não gasta pontos.
 await expect(page.getByText(/sem gastar pontos/).first()).toBeVisible();
});

test('a área de migração dos efeitos internos saiu da tela Voicemod',async({page})=>{
 await mockVoicemod(page,{legacy:true});
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Voicemod',exact:true}).click();
 await page.getByRole('button',{name:/^Avançado/}).click();
 // O cartão de migração foi retirado; mesmo com configuração antiga salva, ele
 // não volta. O Avançado continua com fila e intervalos.
 await expect(page.getByText('Efeitos internos antigos (migração)')).toHaveCount(0);
 await expect(page.getByText('Fila e intervalos')).toBeVisible();
 await page.setViewportSize({width:1100,height:860});
 await page.screenshot({path:join(OUT,'voicemod-sem-migracao.png'),fullPage:true});
 await page.setViewportSize({width:1600,height:900});
 await page.screenshot({path:join(OUT,'voicemod-sem-migracao-larga.png'),fullPage:true});
});
