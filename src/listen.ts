import {api,desktop,errorText} from './api';

/**
 * Escuta contínua do microfone.
 * É um serviço único (não um estado de tela): a captura precisa sobreviver à troca
 * de página e religar sozinha quando o app abre, senão a live começa com o bot surdo.
 * O áudio vira PCM16 de 16 kHz e sai por HTTP só para 127.0.0.1.
 */
type Listener=()=>void;
let handle:{profileId:string;stop:()=>void}|null=null;
let failure='';
const listeners:Listener[]=[];
const queue:Uint8Array[]=[];
let sending=false;

/** Nota quem acompanha a escuta (o painel de voz desenha o estado). */
export function onListenChange(fn:Listener){listeners.push(fn);return()=>{const i=listeners.indexOf(fn);if(i>=0)listeners.splice(i,1)}}
/** Perfil em escuta ou vazio. */
export function activeProfile(){return handle?handle.profileId:''}
/** Último erro de envio de áudio, vazio quando tudo vai bem. */
export function listenError(){return failure}
function announce(){for(const fn of [...listeners])fn()}
function encode(bytes:Uint8Array){let bin='';for(const b of bytes)bin+=String.fromCharCode(b);return btoa(bin)}

export function stopListen(){
 const current=handle;handle=null;failure='';queue.length=0;
 if(current)current.stop();
 announce();
}

/** Manda um lote quando o anterior já saiu; a fila evita reordenar a fala. */
function pump(profileId:string){
 if(sending||!handle||handle.profileId!==profileId||!queue.length)return;
 const batch=queue.shift();
 if(!batch||!batch.length)return;
 const payload=encode(batch);
 sending=true;
 api('voice.frame',{profileId,pcm:payload})
  .then(()=>{if(failure){failure=''}})
  .catch(e=>{failure=errorText(e)})
  .finally(()=>{sending=false;announce();pump(profileId)});
}

export async function startListen(profileId:string){
 if(!desktop)throw Error('A escuta contínua só funciona no aplicativo desktop, não na prévia do navegador.');
 if(handle&&handle.profileId===profileId)return;
 const stream=await navigator.mediaDevices.getUserMedia({audio:{channelCount:1,echoCancellation:true,noiseSuppression:true}});
 const context=new AudioContext();
 if(context.state==='suspended')await context.resume();
 const source=context.createMediaStreamSource(stream);
 const processor=context.createScriptProcessor(4096,1,1);
 const blocks:Float32Array[]=[];
 let buffered=new Float32Array(0);
 let fraction=0;
 let alive=true;
 processor.onaudioprocess=e=>{blocks.push(new Float32Array(e.inputBuffer.getChannelData(0)))};
 source.connect(processor);processor.connect(context.destination);
 // Resample exato: a fração da amostra seguinte viaja entre lotes para não perder
 // pedaço de palavra quando o relógio do navegador não cai redondo em 100 ms.
 const step=context.sampleRate/16000;
 const timer=window.setInterval(()=>{
  if(!alive)return;
  if(blocks.length){
   let size=buffered.length;for(const b of blocks)size+=b.length;
   const merged=new Float32Array(size);let at=0;
   if(buffered.length){merged.set(buffered,0);at=buffered.length}
   for(const b of blocks){merged.set(b,at);at+=b.length}
   blocks.length=0;buffered=merged;
  }
  const spoken:number[]=[];
  let position=fraction;
  while(Math.floor(position)<buffered.length){spoken.push(buffered[Math.floor(position)]);position+=step}
  const cut=Math.floor(position);
  buffered=buffered.slice(cut);fraction=position-cut;
  if(!spoken.length)return;
  const bytes=new Uint8Array(spoken.length*2);
  for(let i=0;i<spoken.length;i++){
   const value=Math.max(-1,Math.min(1,spoken[i]));
   const sample=Math.round(value<0?value*32768:value*32767);
   bytes[2*i]=sample&255;bytes[2*i+1]=(sample>>8)&255;
  }
  queue.push(bytes);
  if(queue.length>100)queue.splice(0,queue.length-100);
  pump(profileId);
 },100);
 const stop=()=>{
  alive=false;window.clearInterval(timer);
  processor.disconnect();source.disconnect();
  for(const track of stream.getTracks())track.stop();
  void context.close().catch(()=>{});
 };
 stopListen();
 handle={profileId,stop};
 announce();
}

/** religa a escuta salva no perfil quando o app abre ou o perfil muda. */
export async function restoreListen(profileId:string){
 if(!desktop||!profileId)return;
 if(handle&&handle.profileId===profileId)return;
 stopListen();
 try{
  const config=await api<Record<string,unknown>>('module.config.get',{profileId,key:'voice'});
  if(config&&config.listen===true)await startListen(profileId);
 }catch{/* a escuta é opcional: falhar aqui não pode travar o app */}
}
