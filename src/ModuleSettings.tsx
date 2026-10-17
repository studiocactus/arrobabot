import {useEffect,useState} from 'react';
import {Save,Volume2} from 'lucide-react';
import {api,errorText} from './api';
import {Field} from './components';
import VoiceCommands from './VoiceCommands';

/** Configurações de módulo por perfil. Comandos de voz vive em VoiceCommands. */
export default function ModuleSettings({profileId,kind,notify}:{profileId:string;kind:string;notify:(s:string)=>void}){
 const [config,setConfig]=useState<Record<string,unknown>>({});
 const [voices,setVoices]=useState<SpeechSynthesisVoice[]>([]);
 useEffect(()=>{
  api<Record<string,unknown>>('module.config.get',{profileId,key:kind}).then(v=>setConfig(v||{})).catch(e=>notify(errorText(e)));
  if(kind==='tts'){const load=()=>setVoices(speechSynthesis.getVoices());load();speechSynthesis.addEventListener('voiceschanged',load);return()=>speechSynthesis.removeEventListener('voiceschanged',load)}
 },[profileId,kind]);
 async function save(){await api('module.config',{profileId,key:kind,value:config})}
 // Comandos de voz é um painel próprio: escuta, prévia e gravação vivem juntos
 // ali, e também na tela "Voz da live" — um só componente, um só comportamento.
 if(kind==='voice')return <div className="module-settings"><VoiceCommands profileId={profileId} notify={notify}/></div>;
 return <div className="module-settings">
  {kind==='moderation'&&<><p className="help">O bot ignora mensagens com expressões proibidas, links e repetições idênticas em 15 segundos. Streamer e moderadores ficam isentos.</p><Field label="O que fazer ao detectar?" hint="Avisos, timeout e banimento usam a conta do canal Twitch autorizada."><select value={String(config.action||'ignore')} onChange={e=>setConfig({...config,action:e.target.value})}><option value="ignore">Só ignorar e registrar</option><option value="warn">Enviar aviso nativo (Twitch)</option><option value="timeout">Aplicar timeout (Twitch)</option><option value="ban">Banir (Twitch)</option></select></Field>{config.action==='timeout'&&<Field label="Duração em segundos"><input type="number" min="1" max="1209600" value={Number(config.seconds)||60} onChange={e=>setConfig({...config,seconds:+e.target.value})}/></Field>}</>}
  {kind==='tts'&&<><p className="help">Adicione a ação “Ler em voz alta” ao fluxo. O texto passa pelo filtro do perfil antes da leitura.</p><Field label="Voz do sistema"><select value={String(config.voice||'')} onChange={e=>setConfig({...config,voice:e.target.value})}><option value="">Padrão em português</option>{voices.map(v=><option key={v.voiceURI} value={v.voiceURI}>{v.name} · {v.lang}</option>)}</select></Field><Field label="Velocidade"><input type="range" min=".5" max="2" step=".1" value={Number(config.rate)||1} onChange={e=>setConfig({...config,rate:+e.target.value})}/></Field><Field label="Limite de caracteres"><input type="number" min="1" max="500" value={Number(config.limit)||300} onChange={e=>setConfig({...config,limit:+e.target.value})}/></Field><button onClick={()=>{const u=new SpeechSynthesisUtterance('Olá, comunidade! O BotLive está pronto para a próxima live.');u.lang='pt-BR';u.rate=Number(config.rate)||1;u.voice=voices.find(v=>v.voiceURI===config.voice)||null;speechSynthesis.speak(u)}}><Volume2 size={15}/>Testar voz local</button></>}
  <footer className="form-footer"><span className="help">Configuração exclusiva deste perfil</span><button onClick={()=>save().then(()=>notify('Configuração salva.')).catch(e=>notify(errorText(e)))}><Save size={15}/>Salvar configuração</button></footer>
 </div>;
}
