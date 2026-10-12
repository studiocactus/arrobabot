import {useEffect,useState,useRef} from 'react';
import {Mic,Square,Save,Volume2,Plus,Wand2} from 'lucide-react';
import {api,errorText,desktop} from './api';
import {Field,Toggle} from './components';
import {activeProfile,listenError,onListenChange,startListen,stopListen,listenFrame} from './listen';
import {saveFlow} from './FlowTools';
import {newFlow,newAction,voiceEffects,type Flow,type Action} from './types';
/** Prévia do reconhecimento, medidor de nível ao vivo e atalho para criar um comando de voz. */
function VoiceExtra({profileId,config,notify,listening,voice}:{profileId:string;config:Record<string,unknown>;notify:(s:string)=>void;listening:boolean;voice:Record<string,unknown>|null}){
 const [level,setLevel]=useState(0);
 const [creating,setCreating]=useState(false);
 const [phrase,setPhrase]=useState('');
 const [doWhat,setDoWhat]=useState('chat');
 const [content,setContent]=useState('');
 const [fxEffect,setFxEffect]=useState('fina');
 const [fxSeconds,setFxSeconds]=useState(15);
 const [busy,setBusy]=useState(false);
 useEffect(()=>{
  if(!listening)return;
  let alive=true;
  const tick=()=>{if(alive)setLevel(Number(listenFrame()?.level)||0)};
  tick();
  const timer=window.setInterval(tick,120);
  return()=>{alive=false;window.clearInterval(timer)};
 },[listening]);
 // Escala 0..0.3 cobre o que o microfone costuma entregar; acima disso satura.
 const pct=(v:number)=>Math.max(0,Math.min(100,(v/0.3)*100));
 const threshold=Number(config.threshold??0.02);
 const matched=voice?.matched===true;
 const last=String(voice?.last||'');
 async function createCommand(){
  const words=phrase.trim();
  if(!words){notify('Escreva a frase que aciona o comando.');return}
  setBusy(true);
  try{
   const action:Action=doWhat==='voiceFx'?{...newAction('voiceFx'),target:fxEffect,value:fxSeconds}:{...newAction(doWhat),text:content};
   const flow:Flow={...newFlow(profileId),name:'Comando de voz: '+words,trigger:{kind:'voice',pattern:words,permission:'everyone',cooldown:0,userCooldown:0},actions:[action]};
   await saveFlow(flow);
   notify(`Comando criado: fale "${words}" e o bot ${doWhat==='voiceFx'?'muda sua voz':doWhat==='tts'?'lê em voz alta':doWhat==='overlay'?'atualiza o overlay':'responde no chat'}.`);
   setPhrase('');setContent('');setCreating(false);
  }catch(e){notify(errorText(e))}finally{setBusy(false)}
 }
 return <>
  <h4>Prévia do reconhecimento</h4>
  <p className="help">Fale no microfone e veja o que o bot entendeu e se usou a palavra de ativação. Ajuste a “Sensibilidade” até a barra reagir só quando você fala.</p>
  <div className="voice-meter" role="meter" aria-label="Nível do microfone" aria-valuenow={Math.round(pct(level))} aria-valuemin={0} aria-valuemax={100}>
   <div className="voice-meter-fill" style={{width:pct(level)+'%'}}/>
   <div className="voice-meter-mark" style={{left:pct(threshold)+'%'}} title="limite de sensibilidade"/>
  </div>
  <p className="help">{listening?`Nível ${level.toFixed(3)} · limite ${threshold.toFixed(3)}`:'Ligue a escuta contínua para ver o nível do microfone.'}</p>
  {last?<>
   <div className="voice-preview-line"><span className={'voice-badge '+(matched?'on':'off')}>{matched?'ativou o bot':'só registro'}</span><span className="voice-preview-text">{last}</span></div>
   <p className="help">{Number(voice?.heard)||0} falas reconhecidas</p>
  </>:<p className="help">Nenhuma fala reconhecida ainda.</p>}
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
    <Field label="Voz"><select value={fxEffect} onChange={e=>setFxEffect(e.target.value)}>{Object.entries(voiceEffects).filter(([k])=>k!=='normal').map(([k,n])=><option key={k} value={k}>{n}</option>)}</select></Field>
    <Field label="Duração (segundos)" hint="0 = manter até a próxima troca."><input type="number" min={0} max={3600} value={fxSeconds} onChange={e=>setFxSeconds(Math.max(0,Math.round(Number(e.target.value)||0)))}/></Field>
   </>:<Field label={doWhat==='tts'?'Texto que o bot lê':doWhat==='overlay'?'Texto no overlay':'Mensagem do bot'}>
    <input value={content} onChange={e=>setContent(e.target.value)} placeholder="Pronto, {{user}}!"/>
   </Field>}
   <div className="row">
    <button className="primary" disabled={busy||!phrase.trim()} onClick={()=>void createCommand()}><Plus size={15}/>Criar comando</button>
    <button onClick={()=>setCreating(false)}>Cancelar</button>
   </div>
  </>:<div className="row"><button onClick={()=>setCreating(true)}><Wand2 size={15}/>Criar comando por voz</button></div>}
 </>
}
export default function ModuleSettings({profileId,kind,notify}:{profileId:string;kind:string;notify:(s:string)=>void}){
 const [config,setConfig]=useState<Record<string,unknown>>({});const [recording,setRecording]=useState(false);const [busy,setBusy]=useState(false);const stop=useRef<(()=>void)|null>(null);const cancelled=useRef(false);
 const [voices,setVoices]=useState<SpeechSynthesisVoice[]>([]);
 const [listening,setListening]=useState(false);const [voice,setVoice]=useState<Record<string,unknown>|null>(null);const [starting,setStarting]=useState(false);
 useEffect(()=>{api<Record<string,unknown>>('module.config.get',{profileId,key:kind}).then(v=>setConfig(v||{})).catch(e=>notify(errorText(e)));if(kind==='tts'){const load=()=>setVoices(speechSynthesis.getVoices());load();speechSynthesis.addEventListener('voiceschanged',load);return()=>speechSynthesis.removeEventListener('voiceschanged',load)}},[profileId,kind]);
 useEffect(()=>()=>{cancelled.current=true;stop.current?.()},[]);
 useEffect(()=>{
  if(kind!=='voice')return;
  const sync=()=>setListening(activeProfile()===profileId);
  sync();
  return onListenChange(sync);
 },[kind,profileId]);
 useEffect(()=>{
  if(kind!=='voice'||!listening)return;
  let alive=true;
  const tick=async()=>{try{const next=await api<Record<string,unknown>>('voice.status',{profileId});if(alive)setVoice(next)}catch(e){if(alive)setVoice({error:errorText(e)})}};
  void tick();
  const timer=window.setInterval(()=>void tick(),2000);
  return()=>{alive=false;window.clearInterval(timer)};
 },[kind,listening,profileId]);
 async function save(){await api('module.config',{profileId,key:kind,value:config})}
 async function toggleListen(next:boolean){
  const value={...config,listen:next};
  if(next)setStarting(true);
  try{
   await api('module.config',{profileId,key:kind,value});
   setConfig(value);
   if(next)await startListen(profileId);else stopListen();
  }catch(e){notify(errorText(e))}
  finally{setStarting(false)}
 }
 const problema=activeProfile()===profileId?listenError():'';
 const estado=problema?`Escuta interrompida: ${problema}`
  :voice&&voice.error?`Escuta com erro: ${String(voice.error)}`
  :voice&&voice.listening?`Ouvindo o microfone · ${Number(voice.heard)||0} falas reconhecidas${voice.last?` · última: ${String(voice.last)}`:''}`
  :voice?'Aguardando fala no microfone…':'Ouvindo o microfone…';
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
 return <div className="module-settings">{kind==='moderation'&&<><p className="help">O bot ignora mensagens com expressões proibidas, links e repetições idênticas em 15 segundos. Streamer e moderadores ficam isentos.</p><Field label="O que fazer ao detectar?" hint="Avisos, timeout e banimento usam a conta do canal Twitch autorizada."><select value={String(config.action||'ignore')} onChange={e=>setConfig({...config,action:e.target.value})}><option value="ignore">Só ignorar e registrar</option><option value="warn">Enviar aviso nativo (Twitch)</option><option value="timeout">Aplicar timeout (Twitch)</option><option value="ban">Banir (Twitch)</option></select></Field>{config.action==='timeout'&&<Field label="Duração em segundos"><input type="number" min="1" max="1209600" value={Number(config.seconds)||60} onChange={e=>setConfig({...config,seconds:+e.target.value})}/></Field>}</>}{kind==='tts'&&<><p className="help">Adicione a ação “Ler em voz alta” ao fluxo. O texto passa pelo filtro do perfil antes da leitura.</p><Field label="Voz do sistema"><select value={String(config.voice||'')} onChange={e=>setConfig({...config,voice:e.target.value})}><option value="">Padrão em português</option>{voices.map(v=><option key={v.voiceURI} value={v.voiceURI}>{v.name} · {v.lang}</option>)}</select></Field><Field label="Velocidade"><input type="range" min=".5" max="2" step=".1" value={Number(config.rate)||1} onChange={e=>setConfig({...config,rate:+e.target.value})}/></Field><Field label="Limite de caracteres"><input type="number" min="1" max="500" value={Number(config.limit)||300} onChange={e=>setConfig({...config,limit:+e.target.value})}/></Field><button onClick={()=>{const u=new SpeechSynthesisUtterance('Olá, comunidade! O BotLive está pronto para a próxima live.');u.lang='pt-BR';u.rate=Number(config.rate)||1;u.voice=voices.find(v=>v.voiceURI===config.voice)||null;speechSynthesis.speak(u)}}><Volume2 size={15}/>Testar voz local</button></>}{kind==='voice'&&<><p className="help">Use um whisper-server local com endpoint /inference. Clique para gravar até 8 segundos. O áudio fica neste computador; a fala reconhecida dispara os fluxos “Comando de voz”.</p><Field label="Servidor local de reconhecimento"><input value={String(config.endpoint||'http://127.0.0.1:8080/inference')} onChange={e=>setConfig({...config,endpoint:e.target.value})}/></Field><button className="primary" disabled={busy} onClick={()=>recording?stop.current?.():void record()}>{recording?<Square size={15}/>:<Mic size={15}/>} {recording?'Parar e reconhecer':busy?'Reconhecendo…':'Gravar comando'}</button><h4>Escuta contínua</h4><p className="help">Com a escuta ligada o bot acompanha o que você fala na live: cada fala entra no Histórico, alimenta a IA e as variáveis {'{{lastSpeech}}'} e {'{{liveSpeech}}'}, e dispara as automações quando você usa uma palavra de ativação. O áudio vai só para o RealtimeSTT local, por HTTP na própria máquina.</p><Field label="Servidor de transcrição (RealtimeSTT)" hint="Endereço local com o endpoint /transcribe-pcm16, rodando na mesma máquina."><input value={String(config.listenEndpoint||'http://127.0.0.1:8010/transcribe-pcm16')} onChange={e=>setConfig({...config,listenEndpoint:e.target.value})}/></Field><Field label="Idioma falado"><select value={String(config.language||'pt')} onChange={e=>setConfig({...config,language:e.target.value})}><option value="pt">Português</option><option value="en">Inglês</option><option value="es">Espanhol</option><option value="fr">Francês</option><option value="de">Alemão</option><option value="it">Italiano</option><option value="auto">Detectar sozinho</option></select></Field><Field label="Palavras de ativação" hint="Separe por vírgula. Só uma fala que contenha uma delas dispara as automações; vazio dispara toda fala."><input value={String(config.activation||'')} onChange={e=>setConfig({...config,activation:e.target.value})} placeholder="arroba, botlive"/></Field><p className="help">No chat, streamer e moderadores também escrevem por extenso — “arroba, mude a categoria para Minecraft ou AION2” — e o bot troca a categoria ou o título com as mesmas palavras, sem a escuta ligada.</p><div className="switch-row"><span>Legenda no overlay</span><Toggle label="Legenda no overlay" checked={config.captions!==false} onChange={captions=>setConfig({...config,captions})}/></div><details><summary>Ajuste fino do microfone</summary><Field label="Silêncio que encerra a frase (ms)" hint="De 200 a 3000. Aumente se a frase é cortada no meio."><input type="number" min={200} max={3000} step={100} value={String(config.silenceMs??600)} onChange={e=>setConfig({...config,silenceMs:Number(e.target.value)})}/></Field><Field label="Sensibilidade do microfone" hint="De 0,002 a 0,5. Aumente se não escuta; diminua se considera ruído de fundo como fala."><input type="number" min={0.002} max={0.5} step={0.002} value={String(config.threshold??0.02)} onChange={e=>setConfig({...config,threshold:Number(e.target.value)})}/></Field></details><div className="switch-row"><span>Escuta contínua</span><Toggle label="Escuta contínua" checked={listening} onChange={next=>void toggleListen(next)}/></div>{starting&&<p className="help">Pedindo o microfone…</p>}{listening&&<p className="help">{estado}</p>}{kind==='voice'&&<VoiceExtra profileId={profileId} config={config} notify={notify} listening={listening} voice={voice}/>}{!desktop&&<p className="help">Abra o aplicativo desktop para ligar a escuta contínua; a prévia do navegador não captura o microfone.</p>}</>}<footer className="form-footer"><span className="help">Configuração exclusiva deste perfil</span><button onClick={()=>save().then(()=>notify('Configuração salva.')).catch(e=>notify(errorText(e)))}><Save size={15}/>Salvar configuração</button></footer></div>
}
