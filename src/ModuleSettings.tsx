import {useEffect,useState} from 'react';
import {Save,Volume2} from 'lucide-react';
import {api,desktop,errorText} from './api';
import {Field} from './components';
import VoiceCommands from './VoiceCommands';

/** Voz instalada neste Windows (SAPI) — a mesma que gera o áudio da live. */
type TtsVoice={name:string;culture:string;gender:string};

/** Configurações de módulo por perfil. Comandos de voz vive em VoiceCommands. */
export default function ModuleSettings({profileId,kind,notify}:{profileId:string;kind:string;notify:(s:string)=>void}){
 const [config,setConfig]=useState<Record<string,unknown>>({});
 const [voices,setVoices]=useState<TtsVoice[]>([]);
 const [testing,setTesting]=useState(false);
 useEffect(()=>{
  api<Record<string,unknown>>('module.config.get',{profileId,key:kind}).then(v=>setConfig(v||{})).catch(e=>notify(errorText(e)));
  if(kind==='tts'&&desktop){api<TtsVoice[]>('tts.voices',{profileId}).then(list=>setVoices(Array.isArray(list)?list:[])).catch(e=>notify(errorText(e)))}
 },[profileId,kind,desktop,notify]);
 async function save(){await api('module.config',{profileId,key:kind,value:config})}
 // Comandos de voz é um painel próprio: escuta, prévia e gravação vivem juntos
 // ali, e também na tela própria "Comandos por voz" — um só componente, um só comportamento.
 if(kind==='voice')return <div className="module-settings"><VoiceCommands profileId={profileId} notify={notify}/></div>;
 const savedVoice=String(config.voice||'');
 const voiceMissing=Boolean(savedVoice)&&!voices.some(v=>v.name===savedVoice);
 const testVoice=async()=>{
  setTesting(true);
  try{await api('tts.test',{profileId});notify('Voz enviada pela saída única do BotLive — com a fonte do OBS conectada, ela fala na transmissão.')}
  catch(e){notify(errorText(e))}
  finally{setTesting(false)}
 };
 return <div className="module-settings">
  {kind==='moderation'&&<><p className="help">O bot ignora mensagens com expressões proibidas, links e repetições idênticas em 15 segundos. Streamer e moderadores ficam isentos.</p><Field label="O que fazer ao detectar?" hint="Avisos, timeout e banimento usam a conta do canal Twitch autorizada."><select value={String(config.action||'ignore')} onChange={e=>setConfig({...config,action:e.target.value})}><option value="ignore">Só ignorar e registrar</option><option value="warn">Enviar aviso nativo (Twitch)</option><option value="timeout">Aplicar timeout (Twitch)</option><option value="ban">Banir (Twitch)</option></select></Field>{config.action==='timeout'&&<Field label="Duração em segundos"><input type="number" min="1" max="1209600" value={Number(config.seconds)||60} onChange={e=>setConfig({...config,seconds:+e.target.value})}/></Field>}</>}
  {kind==='tts'&&<>
   <p className="help">Adicione a ação “Ler em voz alta” ao fluxo. O texto passa pelo filtro do perfil e é falado pela <strong>mesma saída dos sons do bot</strong> — a fonte “Áudio do BotLive” do OBS. A fala é gerada neste computador pelo Windows; nada é enviado à nuvem.</p>
   <Field label="Voz do Windows" hint={voiceMissing?'A voz salva não existe mais neste Windows — escolha uma nova abaixo.':'Escolhida entre as vozes instaladas neste computador.'}>
    <select value={voices.some(v=>v.name===savedVoice)?savedVoice:''} onChange={e=>setConfig({...config,voice:e.target.value})} disabled={!desktop}>
     <option value="">Voz padrão em português</option>
     {voices.map(v=><option key={v.name} value={v.name}>{v.name} · {v.culture}</option>)}
    </select>
   </Field>
   <Field label={'Velocidade: '+String(config.rate||1)} hint="1 é a velocidade normal da voz."><input type="range" min=".5" max="2" step=".1" value={Number(config.rate)||1} onChange={e=>setConfig({...config,rate:+e.target.value})}/></Field>
   <Field label="Limite de caracteres"><input type="number" min="1" max="500" value={Number(config.limit)||300} onChange={e=>setConfig({...config,limit:+e.target.value})}/></Field>
   <div className="row">
    <button disabled={!desktop||testing} onClick={()=>void testVoice()}><Volume2 size={15}/>{testing?'Falando…':'Testar voz'}</button>
   </div>
   {!desktop&&<p className="help">Prévia no navegador: listar vozes e testar exigem o aplicativo desktop.</p>}
  </>}
  <footer className="form-footer"><span className="help">Configuração exclusiva deste perfil</span><button onClick={()=>save().then(()=>notify('Configuração salva.')).catch(e=>notify(errorText(e)))}><Save size={15}/>Salvar configuração</button></footer>
 </div>;
}
