import {useCallback,useEffect,useRef,useState} from 'react';
import {Mic,Square,Save,Plus,Wand2} from 'lucide-react';
import {api,errorText,desktop} from './api';
import {Field,Toggle} from './components';
import {activeProfile,listenError,onListenChange,startListen,stopListen,listenFrame,micDevices} from './listen';
import {saveFlow} from './FlowTools';
import {newFlow,newAction,type Flow,type Action} from './types';

/**
 * Comandos por voz: escuta contínua, prévia do reconhecimento, gravação de
 * comando e criação rápida. Caminho único por padrão: RealtimeSTT local — o
 * whisper-server continua opcional (abaixo, em "Servidor legado").
 * Usado pelo módulo "Comandos de voz" e pela tela própria "Comandos por voz".
 */
export default function VoiceCommands({profileId,notify}:{profileId:string;notify:(s:string)=>void}){
 const [config,setConfig]=useState<Record<string,unknown>>({});
 const [recording,setRecording]=useState(false);
 const [busy,setBusy]=useState(false);
 const [listening,setListening]=useState(false);
 const [starting,setStarting]=useState(false);
 const [voice,setVoice]=useState<Record<string,unknown>|null>(null);
 const [level,setLevel]=useState(0);
 const [mics,setMics]=useState<{id:string;label:string;default:boolean}[]>([]);
 const [creating,setCreating]=useState(false);
 const [phrase,setPhrase]=useState('');
 const [doWhat,setDoWhat]=useState('chat');
 const [content,setContent]=useState('');
 const [fxVoice,setFxVoice]=useState('');
 const [fxVoiceName,setFxVoiceName]=useState('');
 const [fxSeconds,setFxSeconds]=useState(15);
 const [vmVoices,setVmVoices]=useState<{voiceID:string;name:string}[]>([]);
 const stop=useRef<(()=>void)|null>(null);
 const cancelled=useRef(false);

 useEffect(()=>{
  api<Record<string,unknown>>('module.config.get',{profileId,key:'voice'}).then(v=>setConfig(v||{})).catch(e=>notify(errorText(e)));
  micDevices().then(setMics).catch(()=>setMics([]));
  if(desktop)api<{status?:{voices?:{voiceID:string;name:string;enabled?:boolean}[]}}>('voicemod.get',{profileId}).then(v=>setVmVoices((v?.status?.voices||[]).filter(x=>x.enabled!==false))).catch(()=>setVmVoices([]));
 },[profileId,notify]);
 // A escuta é um serviço que sobrevive à troca de página: sair da tela não
 // para o microfone. Só o flag de gravação é cancelado aqui.
 useEffect(()=>{
  const sync=()=>setListening(activeProfile()===profileId);
  sync();
  return onListenChange(sync);
 },[profileId]);
 useEffect(()=>{
  if(!listening)return;
  let alive=true;
  const tick=()=>{if(alive)setLevel(Number(listenFrame()?.level)||0)};
  tick();
  const timer=window.setInterval(tick,120);
  return()=>{alive=false;window.clearInterval(timer)};
 },[listening]);
 useEffect(()=>{
  if(!listening)return;
  let alive=true;
  const tick=async()=>{try{const next=await api<Record<string,unknown>>('voice.status',{profileId});if(alive)setVoice(next)}catch(e){if(alive)setVoice({error:errorText(e)})}};
  void tick();
  const timer=window.setInterval(()=>void tick(),2000);
  return()=>{alive=false;window.clearInterval(timer)};
 },[listening,profileId]);
 useEffect(()=>()=>{cancelled.current=true;stop.current?.()},[]);

 const save=useCallback(async()=>{await api('module.config',{profileId,key:'voice',value:config})},[profileId,config]);
 // Escala 0..0.3 cobre o que o microfone costuma entregar; acima disso satura.
 const pct=(v:number)=>Math.max(0,Math.min(100,(v/0.3)*100));
 const threshold=Number(config.threshold??0.02);
 const matched=voice?.matched===true;
 const last=String(voice?.last||'');

 async function toggleListen(next:boolean){
  const value={...config,listen:next};
  if(next)setStarting(true);
  try{
   await api('module.config',{profileId,key:'voice',value});
   setConfig(value);
   if(next)await startListen(profileId,String(config.micDevice||''));else stopListen();
  }catch(e){notify(errorText(e))}
  finally{setStarting(false)}
 }
 async function changeMic(id:string){
  const value={...config,micDevice:id};
  setConfig(value);
  try{
   await api('module.config',{profileId,key:'voice',value});
   if(listening){stopListen();await startListen(profileId,id)}
  }catch(e){notify(errorText(e))}
 }
 async function record(){
  try{
   cancelled.current=false;await save();const stream=await navigator.mediaDevices.getUserMedia({audio:true});const ctx=new AudioContext();const source=ctx.createMediaStreamSource(stream);const processor=ctx.createScriptProcessor(4096,1,1);const chunks:Float32Array[]=[];processor.onaudioprocess=e=>chunks.push(new Float32Array(e.inputBuffer.getChannelData(0)));source.connect(processor);processor.connect(ctx.destination);setRecording(true);
   let finished=false;let timer:ReturnType<typeof setTimeout>;
   const end=async()=>{if(finished)return;finished=true;clearTimeout(timer);processor.disconnect();source.disconnect();stream.getTracks().forEach(t=>t.stop());const rate=ctx.sampleRate;await ctx.close();setRecording(false);stop.current=null;if(cancelled.current)return;
   const length=chunks.reduce((n,c)=>n+c.length,0);const pcm=new Float32Array(length);let offset=0;for(const chunk of chunks){pcm.set(chunk,offset);offset+=chunk.length}
   const count=Math.floor(length*16000/rate);const buffer=new ArrayBuffer(44+count*2);const view=new DataView(buffer);const text=(at:number,s:string)=>{for(let i=0;i<s.length;i++)view.setUint8(at+i,s.charCodeAt(i))};
   text(0,'RIFF');view.setUint32(4,36+count*2,true);text(8,'WAVE');text(12,'fmt ');view.setUint32(16,16,true);view.setUint16(20,1,true);view.setUint16(22,1,true);view.setUint32(24,16000,true);view.setUint32(28,32000,true);view.setUint16(32,2,true);view.setUint16(34,16,true);text(36,'data');view.setUint32(40,count*2,true);
   for(let i=0;i<count;i++){const sample=Math.max(-1,Math.min(1,pcm[Math.floor(i*rate/16000)]));view.setInt16(44+i*2,sample<0?sample*32768:sample*32767,true)}
   let binary='';for(const b of new Uint8Array(buffer))binary+=String.fromCharCode(b);
   setBusy(true);try{const result=await api<{text:string}>('voice.transcribe',{profileId,audio:btoa(binary)});notify('Comando reconhecido: '+result.text)}catch(e){notify(errorText(e))}finally{setBusy(false)}
   };
   stop.current=()=>{void end()};timer=setTimeout(()=>void end(),8000);
  }catch(e){notify(errorText(e));setRecording(false)}
 }
 async function createCommand(){
  const words=phrase.trim();
  if(!words){notify('Escreva a frase que aciona o comando.');return}
  setBusy(true);
  try{
   const action:Action=doWhat==='voiceFx'?{...newAction('voiceFx'),target:fxVoice,text:fxVoiceName,value:fxSeconds}:{...newAction(doWhat),text:content};
   const flow:Flow={...newFlow(profileId),name:'Comando de voz: '+words,trigger:{kind:'voice',pattern:words,permission:'everyone',cooldown:0,userCooldown:0},actions:[action]};
   await saveFlow(flow);
   notify(`Comando criado: fale "${words}" e o bot ${doWhat==='voiceFx'?'muda sua voz':doWhat==='tts'?'lê em voz alta':doWhat==='overlay'?'atualiza o overlay':'responde no chat'}.`);
   setPhrase('');setContent('');setCreating(false);
  }catch(e){notify(errorText(e))}finally{setBusy(false)}
 }

 const problema=activeProfile()===profileId?listenError():'';
 const estado=problema?`Escuta interrompida: ${problema}`
  :voice&&voice.error?`Escuta com erro: ${String(voice.error)}`
  :voice&&voice.listening?`Ouvindo o microfone · ${Number(voice.heard)||0} falas reconhecidas${voice.last?` · última: ${String(voice.last)}`:''}`
  :voice?'Aguardando fala no microfone…':'Ouvindo o microfone…';

 return <>
  <p className="help">Caminho único: o RealtimeSTT local transcreve o microfone físico — antes de qualquer efeito — e a fala vira comando. Nenhum áudio sai desta máquina. No chat, streamer e moderadores também escrevem por extenso — “arroba, mude a categoria para Minecraft” — e o bot troca a categoria ou o título com as mesmas palavras, sem a escuta ligada.</p>
  <Field label="Servidor de transcrição (RealtimeSTT)" hint="Endereço local com o endpoint /transcribe-pcm16, rodando na mesma máquina.">
   <input value={String(config.listenEndpoint||'http://127.0.0.1:8010/transcribe-pcm16')} onChange={e=>setConfig({...config,listenEndpoint:e.target.value})}/>
  </Field>
  <Field label="Idioma falado">
   <select value={String(config.language||'pt')} onChange={e=>setConfig({...config,language:e.target.value})}><option value="pt">Português</option><option value="en">Inglês</option><option value="es">Espanhol</option><option value="fr">Francês</option><option value="de">Alemão</option><option value="it">Italiano</option><option value="auto">Detectar sozinho</option></select>
  </Field>
  <Field label="Palavras de ativação" hint="Separe por vírgula. Só uma fala que contenha uma delas dispara as automações; vazio dispara toda fala.">
   <input value={String(config.activation||'')} onChange={e=>setConfig({...config,activation:e.target.value})} placeholder="arroba, botlive"/>
  </Field>
  <div className="switch-row"><span>Arroba responde por voz e no chat</span><Toggle label="Respostas por voz do Arroba" checked={config.spokenReplies!==false} onChange={spokenReplies=>setConfig({...config,spokenReplies})}/></div>
  <p className="help">Pedidos de categoria, título e moderação recebem confirmação no chat e em voz alta, inclusive quando falta informação ou ocorre uma falha. Escolha a voz em Comunidade → Texto para voz. No OBS, adicione a fonte indicada em Áudio do BotLive. Use o microfone físico na escuta e fones de ouvido para a voz do bot não voltar como comando. Desligar esta opção mantém as respostas no chat. Salve a configuração para aplicar.</p>
  <Field label="Duração do “quicar” (segundos)" hint="Quando você fala “quicar”, o alvo leva timeout de tanto tempo. Padrão 600 (10 min); nunca vira banimento.">
   <input type="number" min={1} max={1209600} value={String(config.quicarSec??600)} onChange={e=>setConfig({...config,quicarSec:Math.max(1,Math.min(1209600,Math.round(Number(e.target.value)||600)))})}/>
  </Field>
  <Field label="Microfone físico" hint="A escuta captura o microfone de verdade — antes do Voicemod e do OBS. Não escolha o CABLE nem o virtual do Voicemod aqui.">
   <select value={String(config.micDevice||'')} onChange={e=>void changeMic(e.target.value)}>
    <option value="">Microfone padrão do sistema</option>
    {mics.map(m=><option key={m.id} value={m.id}>{m.label}{m.default?' (padrão)':''}</option>)}
   </select>
  </Field>
  <div className="switch-row"><span>Escuta contínua</span><Toggle label="Escuta contínua" checked={listening} onChange={next=>void toggleListen(next)}/></div>
  {starting&&<p className="help">Pedindo o microfone…</p>}
  {listening&&<p className="help">{estado}</p>}
  {listening&&<>
   <div className="voice-meter" role="meter" aria-label="Nível do microfone" aria-valuenow={Math.round(pct(level))} aria-valuemin={0} aria-valuemax={100}>
    <div className="voice-meter-fill" style={{width:pct(level)+'%'}}/>
    <div className="voice-meter-mark" style={{left:pct(threshold)+'%'}} title="limite de sensibilidade"/>
   </div>
   <p className="help">{`Nível ${level.toFixed(3)} · limite ${threshold.toFixed(3)}`}</p>
  </>}
  {last?<div className="voice-preview-line"><span className={'voice-badge '+(matched?'on':'off')}>{matched?'ativou o bot':'só registro'}</span><span className="voice-preview-text">{last}</span></div>
   :<p className="help">{listening?'Nenhuma fala reconhecida ainda.':'Ligue a escuta contínua para ver o que o bot entende.'}</p>}
  <div className="switch-row"><span>Legenda no overlay</span><Toggle label="Legenda no overlay" checked={config.captions!==false} onChange={captions=>setConfig({...config,captions})}/></div>
  <button className="primary" disabled={busy} onClick={()=>recording?stop.current?.():void record()}>{recording?<Square size={15}/>:<Mic size={15}/>} {recording?'Parar e reconhecer':busy?'Reconhecendo…':'Gravar comando'}</button>
  <h4>Criar comando por voz em segundos</h4>
  {creating?<>
   <Field label="Frase que aciona o comando" hint="O que você fala. Ex.: troca o jogo, manda um alô, muda minha voz.">
    <input value={phrase} onChange={e=>setPhrase(e.target.value)} placeholder="troca o jogo"/>
   </Field>
   <Field label="O que o bot faz">
    <select value={doWhat} onChange={e=>setDoWhat(e.target.value)}>
     <option value="chat">Responder no chat</option>
     <option value="tts">Ler em voz alta</option>
     <option value="overlay">Atualizar overlay</option>
     <option value="voiceFx">Mudar minha voz</option>
    </select>
   </Field>
   {doWhat==='voiceFx'?<>
    <Field label="Voz do Voicemod" hint={vmVoices.length?'Voz real do Voicemod: o comando troca pela mesma fila das recompensas.':'Conecte o Voicemod (tela Voicemod) para escolher vozes reais.'}>
     <select value={fxVoice} onChange={e=>{const v=vmVoices.find(x=>x.voiceID===e.target.value);setFxVoice(e.target.value);setFxVoiceName(v?.name||'')}}>
      <option value="">Escolha a voz…</option>
      {vmVoices.map(v=><option key={v.voiceID} value={v.voiceID}>{v.name}</option>)}
     </select>
    </Field>
    <Field label="Duração (segundos)" hint="0 = manter até a próxima troca."><input type="number" min={0} max={3600} value={fxSeconds} onChange={e=>setFxSeconds(Math.max(0,Math.round(Number(e.target.value)||0)))}/></Field>
   </>:<Field label={doWhat==='tts'?'Texto que o bot lê':doWhat==='overlay'?'Texto no overlay':'Mensagem do bot'}>
    <input value={content} onChange={e=>setContent(e.target.value)} placeholder="Pronto, {{user}}!"/>
   </Field>}
   <div className="row">
    <button className="primary" disabled={busy||!phrase.trim()} onClick={()=>void createCommand()}><Plus size={15}/>Criar comando</button>
    <button onClick={()=>setCreating(false)}>Cancelar</button>
   </div>
  </>:<div className="row"><button onClick={()=>setCreating(true)}><Wand2 size={15}/>Criar comando por voz</button></div>}
  <details>
   <summary>Ajuste fino do microfone</summary>
   <Field label="Silêncio que encerra a frase (ms)" hint="De 200 a 3000. Aumente se a frase é cortada no meio.">
    <input type="number" min={200} max={3000} step={100} value={String(config.silenceMs??600)} onChange={e=>setConfig({...config,silenceMs:Number(e.target.value)})}/>
   </Field>
   <Field label="Sensibilidade do microfone" hint="De 0,002 a 0,5. Diminua se o bot não escuta você; aumente se ruído de fundo vira fala.">
    <input type="number" min={0.002} max={0.5} step={0.002} value={String(config.threshold??0.02)} onChange={e=>setConfig({...config,threshold:Number(e.target.value)})}/>
   </Field>
  </details>
  <details>
   <summary>Servidor legado de reconhecimento (whisper-server)</summary>
   <p className="help">Opcional. Um whisper-server local com endpoint /inference atende a gravação de comandos de até 8 segundos. Deixe vazio para usar o RealtimeSTT (padrão). O áudio continua no seu computador.</p>
   <Field label="Endpoint do whisper-server"><input value={String(config.endpoint||'')} onChange={e=>setConfig({...config,endpoint:e.target.value})} placeholder="http://127.0.0.1:8080/inference"/></Field>
  </details>
  {!desktop&&<p className="help">Abra o aplicativo desktop para ligar a escuta contínua; a prévia do navegador não captura o microfone.</p>}
  <div className="form-footer"><span className="help">Configuração exclusiva deste perfil</span><button onClick={()=>save().then(()=>notify('Configuração salva.')).catch(e=>notify(errorText(e)))}><Save size={15}/>Salvar configuração</button></div>
 </>;
}
