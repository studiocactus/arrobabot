import {useCallback,useEffect,useRef,useState} from 'react';
import {KeyRound,Play,Square,RefreshCw,StopCircle,Mic} from 'lucide-react';
import {listen} from '@tauri-apps/api/event';
import {api,desktop,errorText} from './api';
import {AccordionItem,Card,Field,ItemStack,Toggle} from './components';
import VoiceRules,{type VmVoice} from './VoiceRules';

/**
 * Tela Voicemod: configura uma vez e deixa a troca de voz no automático.
 *
 * O caminho é um só e dito em linguagem simples: o BotLive recebe a recompensa
 * ou o comando e pede ao Voicemod para trocar a sua voz; o OBS recebe a sua
 * voz pelo microfone virtual do Voicemod. Sons do bot, alertas e TTS não
 * passam por aqui — eles têm a fonte própria “Áudio do BotLive”.
 */

type VmStatus={phase:string;message:string;port:number;voices:VmVoice[];currentVoice:string;currentName:string;voiceChanger:boolean|null;hearMyself:boolean|null;license:string;attempts:number};
type VmTest={active:boolean;phase:string;voiceId:string;voiceName:string;seconds:number;remainingMs:number;interrupted:boolean;manual:boolean};
type VmSnapshot={status:VmStatus;test:VmTest;outcome:{kind:string;detail:string};defaultSecs:number;minSecs:number;maxSecs:number;hasKey:boolean;keyLen:number};
type Active={voiceName:string;user:string;durationSecs:number;remainingMs:number;forever:boolean};
type QueueView={active:Active|null;upcoming:{voiceName:string;durationSecs:number}[];last:string;lastKind:string;pendingRestore:boolean;busy:boolean};
type RulesConfig={enabled:boolean;queueLimit:number;bindings:{kind:string;enabled:boolean}[];legacyMap:Record<string,string>;obsChecked:boolean;queueNotice?:string};
type FxStatus={phase:string;message:string;input:string;output:string;effect?:string};
type FxSelection={input?:string;output?:string;gateDb?:number;mode?:string};

const PHASE:{[k:string]:{cls:string;text:string}}={
 off:{cls:'status-pill off',text:'Desconectado'},
 searching:{cls:'status-pill wait',text:'Procurando o Voicemod…'},
 authorizing:{cls:'status-pill wait',text:'Autorizando…'},
 connected:{cls:'status-pill on',text:'Voicemod conectado'},
 failed:{cls:'status-pill off',text:'Falha na conexão'},
};

const secsLabel=(ms:number)=>`${Math.max(0,Math.round(ms/1000))}s`;

export default function Voicemod({profileId,notify,initialSection}:{profileId:string;notify:(s:string)=>void;initialSection?:string}){
 const [snap,setSnap]=useState<VmSnapshot|null>(null);
 const [queue,setQueue]=useState<QueueView|null>(null);
 const [cfg,setCfg]=useState<RulesConfig|null>(null);
 const [fx,setFx]=useState<{status:FxStatus;selection:FxSelection}|null>(null);
 const [key,setKey]=useState('');
 const [busy,setBusy]=useState(false);
 // Acordeão das etapas: uma seção aberta por vez — a tela cresce sem virar
 // parede de texto. Começa na Etapa 1.
 const [openSection,setOpenSection]=useState<string>(()=>initialSection||'etapa1');
 const [error,setError]=useState('');
 const [testVoice,setTestVoice]=useState('');
 const [testSecs,setTestSecs]=useState(10);
 const [now,setNow]=useState(Date.now());
 const timer=useRef<number|null>(null);
 const gotAt=useRef(Date.now());
 const testGotAt=useRef(Date.now());
 useEffect(()=>{testGotAt.current=Date.now()},[snap?.test]);
 // O backend confirma começo e fim; aqui só o relógio visível anda, para o
 // tempo restante acompanhar sem prometer nada por conta própria.
 useEffect(()=>{gotAt.current=Date.now()},[queue]);
 useEffect(()=>{
  if(!queue?.busy){setNow(Date.now());return}
  const id=window.setInterval(()=>setNow(Date.now()),500);
  return()=>window.clearInterval(id);
 },[queue?.busy]);

 const load=useCallback(async()=>{
  if(!desktop)return;
  try{const s=await api<VmSnapshot>('voicemod.get',{profileId});setSnap(s);if(s?.defaultSecs)setTestSecs(s.defaultSecs)}catch{}
  try{const q=await api<{queue:QueueView;config:RulesConfig}>('voiceResgate.get',{profileId});setQueue(q?.queue||null);if(q?.config){setCfg(q.config)}}catch{}
  try{const status=await api<FxStatus>('voiceFx.get',{profileId});const selection=await api<FxSelection>('voiceFx.selection',{profileId});setFx({status,selection})}catch{}
 },[profileId]);
 useEffect(()=>{if(desktop)void load()},[load]);

 useEffect(()=>{
  if(!desktop)return;
  const unsubs=[
   listen<{status?:VmStatus;test?:VmTest;outcome?:VmSnapshot['outcome']}>('voicemod',e=>setSnap(s=>s?{...s,...e.payload}:{status:e.payload.status as VmStatus,test:e.payload.test as VmTest,outcome:{kind:'',detail:''},defaultSecs:10,minSecs:5,maxSecs:60,hasKey:false,keyLen:0})),
   listen<QueueView>('voiceResgate',e=>setQueue(e.payload)),
  ];
  return()=>{void Promise.all(unsubs).then(list=>list.forEach(u=>u()))};
 },[]);

 // Contador local do teste: o backend confirma o começo e o fim; aqui só o
 // relógio visível, sem prometer nada por conta própria.
 useEffect(()=>{
  if(!snap?.test?.active){if(timer.current)window.clearInterval(timer.current);timer.current=null;return}
  timer.current=window.setInterval(()=>setNow(Date.now()),500);
  return()=>{if(timer.current)window.clearInterval(timer.current)};
 },[snap?.test?.active]);

 // Fila viva: refresca enquanto há efeito, para o tempo restante acompanhar.
 useEffect(()=>{
  if(!queue?.busy)return;
  const id=window.setInterval(()=>{
   api<{queue:QueueView}>('voiceResgate.get',{profileId}).then(q=>setQueue(q?.queue||null)).catch(()=>{});
  },2000);
  return()=>window.clearInterval(id);
 },[queue?.busy,profileId]);

 const run=async(fn:()=>Promise<void>)=>{setBusy(true);setError('');try{await fn()}catch(e){setError(errorText(e))}finally{setBusy(false)}};

 const saveKey=()=>run(async()=>{
  if(!key.trim())throw new Error('Cole a chave da Control API antes de salvar.');
  await api('voicemod.key',{profileId,value:key.trim()});
  setKey('');notify('Chave do Voicemod salva no cofre.');
 });
 const connect=()=>run(async()=>{
  await api('voicemod.connect',{profileId});
  await load();
 });
 const disconnect=()=>run(async()=>{
  const out=await api<{restored?:boolean}>('voicemod.disconnect',{profileId});
  notify(out?.restored?'Desconectado: a voz de antes foi restaurada.':'Desconectado.');
  await load();
 });
 const startTest=()=>run(async()=>{
  if(!testVoice)throw new Error('Escolha uma voz para o teste.');
  const test=await api<VmTest>('voicemod.testStart',{profileId,voiceId:testVoice,seconds:testSecs});
  setSnap(s=>s?{...s,test,outcome:{kind:'',detail:''}}:s);
  notify('Teste no ar: fale agora e confira sua voz no OBS. A voz de antes volta sozinha.');
 });
 const stopTest=()=>run(async()=>{await api('voicemod.testStop',{profileId});await load()});
 const cancelAll=()=>run(async()=>{
  await api('voiceResgate.cancel',{profileId});
  notify('Efeitos encerrados; a voz de antes foi restaurada.');
  await load();
 });
 const recover=()=>run(async()=>{
  await api(testPending?'voicemod.recover':'voiceResgate.recover',{profileId});
  notify('Recuperação pedida ao Voicemod.');
  await load();
 });
 const markObsChecked=(v:boolean)=>run(async()=>{
  try{
   const q=await api<{config:RulesConfig}>('voiceResgate.get',{profileId});
   const config={...(q?.config||{enabled:false,queueLimit:3,bindings:[],legacyMap:{},obsChecked:false}),obsChecked:v};
   await api('voiceResgate.save',{profileId,config});
   setCfg(config);
   notify(v?'Marcado: você conferiu sua voz no OBS.':'Marca removida: confira de novo quando puder.');
  }catch(e){setError(errorText(e))}
 });

 const phase=snap?.status?.phase||'off';
 const pill=PHASE[phase]||PHASE.off;
 const connected=phase==='connected';
 const changer=snap?.status?.voiceChanger??null;
 const voices=(snap?.status?.voices||[]).filter(v=>v.enabled!==false);
 const qActive=queue?.active||null;
 const testPending=snap?.outcome?.kind==='unconfirmed';
 const pending=!!queue?.pendingRestore||testPending;
 const rulesOn=cfg?cfg.bindings.filter(b=>b.enabled).length:0;
 const rulesOff=cfg?cfg.bindings.filter(b=>!b.enabled).length:0;
 const fxRunning=fx?.status?.phase==='running';
 // Migração contextual: só aparece para perfis afetados (motor interno em uso,
 // configuração antiga salva ou mapeamento existente).

 return <div>
  <ItemStack>
   <AccordionItem id="intro" title="Sua voz no Voicemod" meta={[]} open={openSection==='intro'} onToggle={()=>setOpenSection(openSection==='intro'?'':'intro')}>
   <p className="help">Configure uma vez e deixe no automático: <strong>o BotLive recebe a recompensa ou o comando e pede ao Voicemod para trocar sua voz; o OBS recebe sua voz pelo microfone virtual do Voicemod.</strong> A troca dura o tempo que você escolher e a voz de antes volta sozinha, confirmada pelo próprio Voicemod.</p>
   <p className="help">Sons do bot, alertas e texto para voz <strong>não passam por aqui</strong> — eles saem pela fonte “Áudio do BotLive” (tela <strong>Áudio no OBS</strong>). São duas fontes com funções distintas no OBS: “Áudio do BotLive” para os sons e “Voicemod Microphone” para a sua voz.</p>
  </AccordionItem>

   <AccordionItem id="live" title="Durante a live" meta={[connected?'CONECTADO':'SEM CONEXÃO']} open={openSection==='live'} onToggle={()=>setOpenSection(openSection==='live'?'':'live')}>
   <div className="list-row">
    <div><strong>Conexão</strong><small>{pill.text}{snap?.status?.message?` · ${snap.status.message}`:''}</small></div>
    <span className={pill.cls}><span className="status-dot" aria-hidden="true"></span><span>{pill.text}</span></span>
   </div>
   <div className="list-row">
    <div><strong>Trocas de voz</strong><small>{cfg?`${rulesOn} ativada${rulesOn===1?'':'s'}${rulesOff?` · ${rulesOff} pausada${rulesOff===1?'':'s'}`:''}`:'Carregando…'}</small></div>
    <span className={'status-pill '+(cfg?.enabled?'on':'off')}><span className="status-dot" aria-hidden="true"></span><span>{cfg?.enabled?'Ativas':'Pausadas'}</span></span>
   </div>
   {qActive&&<div className="list-row">
    <div><strong>No ar agora</strong><small>{qActive.voiceName}{qActive.user?` · pedido de ${qActive.user}`:''}{qActive.forever?' · até você encerrar':` · faltam ${secsLabel(Math.max(0,qActive.remainingMs-(now-gotAt.current)))}`}</small></div>
    <span className="status-pill on"><span className="status-dot" aria-hidden="true"></span><span>Tocando</span></span>
   </div>}
   {!!queue?.upcoming?.length&&<div className="list-row">
    <div><strong>A seguir</strong><small>{queue.upcoming.map(u=>u.voiceName).join(' · ')}</small></div>
    <span className="status-pill wait"><span className="status-dot" aria-hidden="true"></span><span>Na fila</span></span>
   </div>}
   {queue?.last&&<p className="help">Último resultado: {queue.last}</p>}
   {pending&&<div className="notice">
    <strong>Restauração pendente.</strong>
    <p className="help">A conexão caiu com um efeito no ar. Reconecte o Voicemod e use <strong>Recuperar</strong>: nada é reaplicado sozinho e nenhum efeito antigo volta por conta própria.</p>
    <button className="secondary" disabled={busy||!connected} onClick={()=>void recover()}><RefreshCw size={14}/> Recuperar restauração</button>
   </div>}
   <div className="form-footer">
    <span className="help">Encerrar devolve a voz que estava antes — se você tinha um efeito escolhido, é ele que volta.</span>
    <button className="secondary" disabled={busy} onClick={()=>void cancelAll()}><StopCircle size={15}/> Encerrar efeitos e restaurar minha voz</button>
   </div>
  </AccordionItem>

   <AccordionItem id="etapa1" title="Etapa 1 · Conectar Voicemod" meta={[connected?'PRONTO':'PENDENTE']} open={openSection==='etapa1'} onToggle={()=>setOpenSection(openSection==='etapa1'?'':'etapa1')}>
   {!connected&&<p className="help">Abra o aplicativo <strong>Voicemod</strong> nesta máquina e deixe-o aberto. A conexão é local (Control API): nada sai do computador e a chave fica no cofre, nunca no chat nem no Histórico.</p>}
   {connected
    ?<div className="list-row">
     <div><strong>Conectado</strong><small>Porta {snap?.status?.port||'—'} · {snap?.status?.voices?.length||0} vozes · voz atual: {snap?.status?.currentName||'—'}</small></div>
     <span className="status-pill on"><span className="status-dot" aria-hidden="true"></span><span>Pronto</span></span>
    </div>
    :<div className="form-grid">
     {(!snap?.hasKey||key)&&<Field label="Chave da Control API" error={error&&error.includes('chave')?error:undefined} hint={`Peça em control-api.voicemod.net (menu Control API). ${snap?.hasKey?`Já guardamos uma chave (${snap.keyLen} caracteres) — só preencha para trocar.`:'Ela vai direto para o cofre desta máquina.'}`}>
      <input type="password" value={key} autoComplete="off" onChange={e=>setKey(e.target.value)} placeholder="Cole a chave aqui"/>
     </Field>}
    </div>}
   <div className="form-footer">
    <span className="help">{phase==='failed'?'Se a conexão falhar: confira se o Voicemod está aberto e logado, se a chave veio do menu Control API e se a sua licença libera a Control API.':'Depois de conectar, esta etapa recolhe sozinha.'}</span>
    <div className="row">
     {!connected&&key.trim()&&<button className="secondary" disabled={busy} onClick={()=>void saveKey()}><KeyRound size={14}/> Salvar chave</button>}
     {!connected
      ?<button className="primary" disabled={busy} onClick={()=>void connect()}><RefreshCw size={14}/> Conectar</button>
      :<button className="secondary" disabled={busy} onClick={()=>void disconnect()}>Desconectar</button>}
    </div>
   </div>
   <details>
    <summary>Alterar conexão e diagnóstico</summary>
    <div className="form-grid">
     <Field label="Chave da Control API" hint="Trocar a chave não apaga suas regras. A chave nunca aparece aqui nem nos logs.">
      <input type="password" value={key} autoComplete="off" onChange={e=>setKey(e.target.value)} placeholder={snap?.hasKey?'Chave guardada no cofre':'Cole a chave aqui'}/>
     </Field>
    </div>
    <div className="row">
     <button className="secondary" disabled={busy} onClick={()=>void saveKey()}><KeyRound size={14}/> Salvar chave</button>
     <button className="secondary" disabled={busy} onClick={()=>void connect()}><RefreshCw size={14}/> Conectar de novo</button>
     <button className="secondary" disabled={busy||!connected} onClick={()=>void run(async()=>{await api('voicemod.refresh',{profileId});await load()})}><RefreshCw size={14}/> Atualizar vozes</button>
    </div>
    <p className="help">Diagnóstico: {snap?.status?.message||'sem detalhes'} · modificador de voz: {changer==null?'desconhecido':changer?'ligado':'desligado (algumas vozes não soam)'} · licença: {snap?.status?.license||'—'} · tentativas: {snap?.status?.attempts??0}.</p>
   </details>
  </AccordionItem>

   <AccordionItem id="etapa2" title="Etapa 2 · Ouvir minha voz no OBS" meta={[cfg?.obsChecked?'CONFERIDO NO OBS':'A CONFERIR']} open={openSection==='etapa2'} onToggle={()=>setOpenSection(openSection==='etapa2'?'':'etapa2')}>
   <p className="help">No OBS, edite a fonte de áudio da <strong>sua voz</strong> e escolha <strong>Voicemod Microphone</strong> como dispositivo de entrada. Evite captura duplicada: se uma fonte já pega o microfone físico, não pegue o mesmo áudio em outra — a voz entraria em dobro. Nada aqui muda seus dispositivos do Windows nem mexe nas suas cenas.</p>
   <details>
    <summary>Preparar áudio limpo antes da live</summary>
    <p className="help">A conexão controla a troca de voz; ela não mede a qualidade do áudio. Faça estes ajustes e compare uma gravação sem efeito com outra usando cada voz dos resgates.</p>
    <p className="help"><strong>Voz robótica, estalos ou atraso?</strong> Compare primeiro sem efeito. Se o defeito já existir, confira frequência, cabo e carga de CPU. Se aparecer só com uma voz, compare esse efeito isolado e reduza processamentos simultâneos. Ajuste um item por vez e grave novamente.</p>
    <ol>
     <li><strong>Uma entrada, um caminho.</strong> No Voicemod, selecione o microfone físico. No OBS, capture apenas o microfone virtual do Voicemod para sua voz. Evite capturá-lo também em Mic/Aux ou monitorar de volta para a entrada.</li>
     <li><strong>Mesma frequência.</strong> Confira 48.000 Hz no microfone, no dispositivo virtual e no OBS, quando os aparelhos suportarem. Prefira microfone USB ou com fio se o Bluetooth perder qualidade ao ligar a captura.</li>
     <li><strong>Sem filtros empilhados.</strong> Compare com supressão de ruído, gate e melhorias do Windows desligados; reative um por vez se precisar. Filtros fortes podem cortar sílabas e partes da voz alterada.</li>
     <li><strong>Volume com folga.</strong> Como ponto de partida, fale com picos entre −12 e −6 dBFS no OBS. Se distorcer, reduza o ganho antes do Voicemod: baixar só o fader não recupera uma entrada já saturada.</li>
     <li><strong>Proteção na saída.</strong> Um limiter no fim dos filtros do OBS (por exemplo, −1 dB) pode conter picos finais. Ajuste cada efeito ouvindo a gravação; ele não conserta distorção que já veio do microfone.</li>
    </ol>
    <p className="help">Grave voz normal, efeito, dois resgates em sequência e retorno à voz anterior. Inclua fala baixa, forte e alguns segundos de silêncio. Só marque “Ouvi minha voz no OBS” depois de ouvir esse resultado. Repita após trocar microfone, filtros ou vozes.</p>
   </details>
   <div className="form-grid">
    <Field label="Voz para o teste" hint="Lista real do Voicemod, com as vozes da sua licença.">
     <select value={testVoice} disabled={!connected||busy} onChange={e=>setTestVoice(e.target.value)}>
      {voices.length?<option value="">Escolha a voz…</option>:<option value="" disabled>Nenhuma voz carregada — atualize as vozes em “Alterar conexão”</option>}
      {voices.map(v=><option key={v.voiceID} value={v.voiceID}>{v.name}</option>)}
     </select>
    </Field>
    <Field label="Duração do teste (segundos)" hint="Depois o BotLive devolve a voz de antes, com confirmação.">
     <input type="number" min={snap?.minSecs??5} max={snap?.maxSecs??60} value={testSecs} disabled={!connected||busy} onChange={e=>setTestSecs(+e.target.value||10)}/>
    </Field>
   </div>
   <div className="form-footer">
    <span className="help">{snap?.test?.active?`Teste no ar: fale agora (${secsLabel(Math.max(0,(snap.test.remainingMs||0)-(now-testGotAt.current)))}). A voz de antes volta sozinha ao terminar.`:'Um único teste cobre o caminho todo: aplicar temporariamente e restaurar. Fale durante o teste — é a sua voz que o OBS precisa ouvir.'}</span>
    <div className="row">
     {!snap?.test?.active
      ?<button className="primary" disabled={!connected||busy||!testVoice} onClick={()=>void startTest()}><Play size={14}/> Testar voz</button>
      :<button className="secondary" disabled={busy} onClick={()=>void stopTest()}><Square size={14}/> Parar teste ({secsLabel(Math.max(0,(snap?.test?.remainingMs||0)-(now-testGotAt.current)))})</button>}
    </div>
   </div>
   {(snap?.test?.interrupted||testPending)&&<p role="alert" className="inline-error">O teste terminou sem confirmação da restauração. Conecte de novo e use <strong>Recuperar restauração</strong> na seção “Durante a live”.</p>}
   <div className="list-row">
    <div><strong>Voicemod conectado</strong><small>{connected?'A API respondeu e autorizou.':'Ainda não.'}</small></div>
    <span className={connected?'status-pill on':'status-pill off'}><span className="status-dot" aria-hidden="true"></span><span>{connected?'Sim':'Não'}</span></span>
   </div>
   <div className="list-row">
    <div><strong>Áudio conferido no OBS</strong><small>{cfg?.obsChecked?'Você ouviu sua voz na gravação.':'Só você pode conferir: a conexão não comprova áudio na cena.'}</small></div>
    <Toggle label="Ouvi minha voz no OBS" checked={!!cfg?.obsChecked} onChange={v=>void markObsChecked(v)}/>
   </div>
   {!cfg?.obsChecked&&<p className="help">Para conferir de verdade: grave um trecho no OBS (Arquivo → Gravar), fale durante o teste e ouça a gravação. Se não ouvir, confira o dispositivo de entrada da fonte e o fader do Voicemod.</p>}
  </AccordionItem>

   <AccordionItem id="etapa3" title="Etapa 3 · Escolher o que troca minha voz" meta={[(cfg?.bindings?.length||0)===1?'1 regra':(cfg?.bindings?.length||0)+' regras']} open={openSection==='etapa3'} onToggle={()=>setOpenSection(openSection==='etapa3'?'':'etapa3')}>
   <p className="help">Regras com recompensa da Twitch ou comando do chat. Exemplo: <strong>“Voz de robô → Robô → 30 segundos”</strong>. Tudo passa pelo mesmo caminho: um efeito por vez, próximos esperam na fila e a voz de antes volta ao final. Resgates são marcados como cumpridos <strong>só depois que o Voicemod confirma a troca</strong>; se algo falhar, o motivo fica no Histórico e o estorno é com a Twitch — o BotLive não devolve pontos sozinho.</p>
   <VoiceRules profileId={profileId} notify={notify} voices={voices}/>
  </AccordionItem>

  <AccordionItem id="avancado" title="Avançado" meta={[]} open={openSection==='avancado'} onToggle={()=>setOpenSection(openSection==='avancado'?'':'avancado')}>

   <Card title="Fila e intervalos">
    <div className="form-grid">
     <Field label="Tamanho da fila" hint="Passou disso, a troca nova é recusada com aviso — a live não vira fila de espera.">
      <input type="number" min={1} max={10} value={cfg?.queueLimit??3} disabled={busy} onChange={e=>setCfg(c=>c?{...c,queueLimit:Math.max(1,Math.min(10,+e.target.value||1))}:c)} onBlur={()=>{if(cfg)void run(async()=>{const q=await api<{config:RulesConfig}>('voiceResgate.get',{profileId});const config={...(q?.config||cfg),queueLimit:cfg.queueLimit};await api('voiceResgate.save',{profileId,config});setCfg(config)})}}/>
     </Field>
    </div>
    <Field label="Mensagem quando o resgate entra na fila" hint="Avisa quem resgatou que o pedido começa em seguida. Use {{user}}, {{pos}} e {{voz}}; vazio desliga o aviso.">
     <input value={cfg?.queueNotice??''} disabled={busy} placeholder="{{user}}, seu resgate de voz está na fila ({{pos}}º) — começa em seguida!" onChange={e=>setCfg(c=>c?{...c,queueNotice:e.target.value}:c)} onBlur={()=>{if(cfg)void run(async()=>{const q=await api<{config:RulesConfig}>('voiceResgate.get',{profileId});const config={...(q?.config||cfg),queueNotice:cfg.queueNotice};await api('voiceResgate.save',{profileId,config});setCfg(config)})}}/>
    </Field>
   </Card>
  </AccordionItem>
  </ItemStack>

  {!desktop&&<div className="notice"><strong>Prévia no navegador.</strong><p className="help">O Voicemod é um aplicativo desta máquina: conexão, vozes e testes só existem no aplicativo desktop. Nada aqui finge conexão real.</p></div>}
  {snap?.outcome?.detail&&<p role={testPending?'alert':'status'} className={testPending?'inline-error':'help'}>{snap.outcome.detail}</p>}
  {error&&<p role="alert" className="inline-error">{error}</p>}
 </div>;
}
