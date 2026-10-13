/**
 * Overlay do BotLive servido em http://127.0.0.1:PORT/overlay e no arquivo
 * examples/goal-overlay.html. Recebe por WebSocket local a meta, o design e os
 * alertas do perfil e redesenha sem recarregar a fonte do OBS.
 *
 * Compilado por scripts/build-overlay.cjs para src-tauri/overlay.html.
 */
import {
 designVars,ensureFont,ensureOverlayStyle,goalPercent,keyTexts,normalizeDesign,overlayTree,
 type GoalModel,type OverlayDesign,type Spec,
} from './overlayDesign';
import {alertSoundUrl,isAlertKind,normalizeAlerts,queueAlert,type AlertVars,type AlertsConfig} from './alerts';
import {normalizeAudio,type AudioConfig} from './audio';
import {chatVisible,clearChat,deleteChat,deleteUserChat,normalizeChat,pushChat,type ChatConfig,type ChatMsg} from './chatOverlay';

const params=new URLSearchParams(location.hash.slice(1));
const token=params.get('token')||'';
const profile=params.get('profile')||'';
const port=Number(params.get('port')||location.port||9876)||9876;

ensureOverlayStyle();

let design:OverlayDesign=normalizeDesign(null);
let model:GoalModel={title:'',current:0,target:0};
let hidden=true;
let ws:WebSocket|null=null;
let retry=3000;
let alertsCfg:AlertsConfig=normalizeAlerts(null);
let audioCfg:AudioConfig=normalizeAudio(null);
let alertHost:HTMLElement|null=null;
let chatCfg:ChatConfig=normalizeChat(null);
let chatHost:HTMLElement|null=null;

/** Coluna do chat: criada na primeira mensagem e sempre no mesmo lugar. */
function chatBox():HTMLElement{
 if(!chatHost){
  chatHost=document.createElement('div');
  chatHost.className='bl-chat-col bl-chat-fixed';
  chatHost.setAttribute('aria-hidden','true');
  document.body.appendChild(chatHost);
 }
 return chatHost;
}

const host=document.getElementById('bl-ov-root') as HTMLDivElement;
let root:HTMLElement|null=null;

function build(n:Spec):Node{
 const el=document.createElement(n.tag||'div');
 if(n.c)el.className=n.c;
 if(n.k)el.setAttribute('data-k',n.k);
 for(const [k,v] of Object.entries(n.a||{}))el.setAttribute(k,v);
 if(n.t!==undefined)el.textContent=n.t;
 else for(const child of n.ch||[])el.appendChild(build(child));
 return el;
}

function applyVars(){
 if(!root)return;
 for(const [k,v] of Object.entries(designVars(design)))root.style.setProperty(k,v);
 root.style.setProperty('--p',String(goalPercent(model)));
}

function applyTexts(){
 if(!root)return;
 const texts=keyTexts(model);
 root.querySelectorAll('[data-k]').forEach(el=>{
  const key=el.getAttribute('data-k')||'';
  if(key in texts)el.textContent=texts[key];
 });
}

/** Reestrutura o overlay (mudou design/modelo de layout). */
function render(){
 ensureFont(design.font);
 const spec=overlayTree(design,keyTexts(model));
 host.replaceChildren(build(spec));
 root=host.firstElementChild as HTMLElement;
 applyVars();
 if(hidden)root.classList.add('is-off');
}

function flashGain(){
 if(!root||!design.pulse)return;
 root.classList.remove('is-gain');void root.offsetWidth;root.classList.add('is-gain');
 setTimeout(()=>root?.classList.remove('is-gain'),800);
}

function celebrate(){
 if(!root||!design.celebrate)return;
 root.classList.remove('is-celebrate');void root.offsetWidth;root.classList.add('is-celebrate');
 setTimeout(()=>root?.classList.remove('is-celebrate'),2400);
}

type Message={type?:string;payload?:Record<string,unknown>};

function handle(msg:Message){
 const payload=msg.payload;
 if(msg.type==='alerts'){
  if(profile&&String(payload?.profileId||'')!==profile)return;
  alertsCfg=normalizeAlerts(payload?.config);
  return;
 }
 if(msg.type==='audio.config'){
  if(profile&&String(payload?.profileId||'')!==profile)return;
  audioCfg=normalizeAudio(payload?.config);
  return;
 }
 if(msg.type==='alert'){
  if(profile&&String(payload?.profileId||'')!==profile)return;
  const kind=payload?.kind;
  if(!isAlertKind(kind))return;
  if(!alertHost){alertHost=document.createElement('div');document.body.appendChild(alertHost)}
  const ev=alertsCfg.events[kind];
  const vars=(payload?.vars&&typeof payload.vars==='object'?payload.vars:{}) as Record<string,unknown>;
  queueAlert(alertHost,{
   cfg:alertsCfg,kind,vars:vars as AlertVars,
   // O som sai da API local com a mesma chave que abriu este overlay —
   // a menos que a Central de Áudio tenha mandado só para a linha.
   audio:ev.sound&&audioCfg.alertBrowser?{src:alertSoundUrl(port,token,ev.sound),volume:alertsCfg.volume}:null,
  });
  return;
 }
 if(msg.type==='sound'){
  // Som de espectador/fluxo roteado para o navegador pela Central de Áudio.
  if(profile&&String(payload?.profileId||'')!==profile)return;
  const asset=String(payload?.asset||'');
  if(!asset)return;
  const a=new Audio(alertSoundUrl(port,token,asset));
  a.volume=Math.max(0,Math.min(1,Number(payload?.volume)||0));
  void a.play().catch(()=>{});
  return;
 }
 if(msg.type==='chat.config'){
  if(profile&&String(payload?.profileId||'')!==profile)return;
  chatCfg=normalizeChat(payload?.config);
  if(!chatCfg.enabled&&chatHost)clearChat(chatHost);
  return;
 }
 if(msg.type==='chat'){
  if(profile&&String(payload?.profileId||'')!==profile)return;
  const m=payload as unknown as ChatMsg;
  if(chatCfg.enabled&&chatVisible(m,chatCfg))pushChat(chatBox(),m,chatCfg,40);
  return;
 }
 if(msg.type==='chat.delete'){
  if(profile&&String(payload?.profileId||'')!==profile)return;
  if(chatHost)deleteChat(chatHost,String(payload?.id||''));
  return;
 }
 if(msg.type==='chat.deleteUser'){
  if(profile&&String(payload?.profileId||'')!==profile)return;
  if(chatHost)deleteUserChat(chatHost,String(payload?.userId||''));
  return;
 }
 if(msg.type==='chat.clear'){
  if(profile&&String(payload?.profileId||'')!==profile)return;
  if(chatHost)clearChat(chatHost);
  return;
 }
 if(msg.type==='design'){
  if(profile&&String(payload?.profileId||'')!==profile)return;
  design=normalizeDesign(payload?.design);
  render();
  return;
 }
 if(msg.type!=='goal')return;
 if(profile&&String(payload?.profileId||'')!==profile)return;
 if(payload?.hidden===true){hidden=true;root?.classList.add('is-off');return}
 const current=Number(payload?.current)||0;
 const previous=model.current;
 const title=String(payload?.title||'');
 hidden=false;
 model={title,current,target:Number(payload?.target)||0};
 if(!root)render();
 else{root.classList.remove('is-off');applyVars();applyTexts()}
 if(payload?.celebrate===true)celebrate();
 else if(current>previous)flashGain();
}

function connect(){
 if(!token){
  console.warn('[BotLive] Overlay sem #token=… na URL; nada será exibido.');
  hidden=true;render();
  return;
 }
 try{ws=new WebSocket(`ws://127.0.0.1:${port}`)}catch{return}
 ws.onopen=()=>{retry=3000;ws?.send(JSON.stringify({token}))};
 ws.onmessage=e=>{
  try{handle(JSON.parse(String(e.data)) as Message)}catch{/* mensagem inválida: ignora */}
 };
 ws.onclose=()=>{ws=null;setTimeout(connect,retry);retry=Math.min(Math.round(retry*1.6),15000)};
 ws.onerror=()=>ws?.close();
}

render();
connect();
