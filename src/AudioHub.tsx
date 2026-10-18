import {useCallback,useEffect,useRef,useState} from 'react';
import {Check,Copy,Play,Square} from 'lucide-react';
import {listen} from '@tauri-apps/api/event';
import {api,desktop,errorText} from './api';
import {Card} from './components';
import {stopViewerSounds} from './viewerSound';

type Sink={url:string;clients:number;claimed:boolean};
type Feedback={phase:string;detail?:string};

const FEEDBACK:Record<string,string>={
 started:'Reprodução iniciada pela fonte do OBS — o som entrou na transmissão.',
 done:'Reprodução confirmada e encerrada pela fonte do OBS.',
 stopped:'Som interrompido pela fonte do OBS.',
};

/**
 * "Áudio no OBS": a única saída dos sons do bot. Uma fonte de navegador local
 * toca todos os sons (espectador, fluxo, alertas, meta e TTS) — o fader dela
 * no mixer do OBS é o volume do bot inteiro. O aplicativo só toca localmente
 * enquanto essa fonte não está conectada; nunca os dois ao mesmo tempo.
 */
export default function AudioHub({profileId,notify}:{profileId:string;notify:(s:string)=>void}){
 const [sink,setSink]=useState<Sink>({url:'',clients:0,claimed:false});
 const [feedback,setFeedback]=useState<Feedback|null>(null);
 const [testing,setTesting]=useState(false);
 const [copied,setCopied]=useState(false);
 const testId=useRef('');
 const copyTimer=useRef(0);

 const load=useCallback(async()=>{
  if(!desktop)return;
  try{setSink(await api<Sink>('audio.get',{profileId}))}catch(e){notify(errorText(e))}
 },[profileId,desktop,notify]);
 useEffect(()=>{if(desktop)void load()},[load]);

 // Estado real da conexão, atualizado a cada mudança no backend.
 useEffect(()=>{
  if(!desktop)return;let active=true;const off:(()=>void)[]=[];
  const track=(p:Promise<()=>void>)=>{void p.then(f=>active?off.push(f):f())};
  track(listen<{profileId:string;clients:number;claimed:boolean}>('audio.state',e=>{
   if(e.payload.profileId===profileId)setSink(s=>({...s,clients:e.payload.clients,claimed:e.payload.claimed}));
  }));
  track(listen<{profileId:string;id:string;phase:string;detail?:string}>('audio.feedback',e=>{
   const p=e.payload;
   if(p.id!==testId.current)return;
   setFeedback({phase:p.phase,detail:p.detail});
  }));
  return()=>{active=false;off.forEach(f=>f())};
 },[profileId,desktop]);

 /** Teste: o mesmo caminho de qualquer som — sai pela fonte do OBS quando
  *  conectada; sem fonte, toca na saída padrão e o texto diz isso. */
 const test=async()=>{
  setTesting(true);setFeedback(null);
  try{
   const out=await api<{id:string;sink:string}>('audio.test',{profileId});
   if(out.sink!=='obs'){
    notify('A fonte do OBS ainda não está conectada — o bipe tocou na saída padrão do aplicativo. Conecte a fonte no OBS e teste de novo.');
    setTesting(false);return;
   }
   testId.current=out.id;
   setFeedback({phase:'sent'});
   window.setTimeout(()=>{
    setFeedback(f=>f&&f.phase==='sent'?{phase:'timeout'}:f);
    setTesting(false);
   },12000);
  }catch(e){notify(errorText(e));setTesting(false)}
 };

 const stop=async()=>{
  stopViewerSounds();
  try{await api('audio.stop',{profileId});notify('Todos os sons pararam.')}catch(e){notify(errorText(e))}
 };

 const copy=async()=>{
  try{await navigator.clipboard.writeText(sink.url);setCopied(true);window.clearTimeout(copyTimer.current);copyTimer.current=window.setTimeout(()=>setCopied(false),2500)}
  catch{notify('Não foi possível copiar. Selecione o endereço e copie manualmente.')}
 };

 const status=sink.clients===0
  ?{cls:'status-pill wait',text:'Aguardando o OBS'}
  :sink.clients>1
   ?{cls:'status-pill wait',text:`Conectada — ${sink.clients} fontes; só a primeira reproduz`}
   :{cls:'status-pill on',text:'Conectada'};

 return <div>
  <Card title="Áudio no OBS" action={<span className={status.cls}><span className="status-dot" aria-hidden="true"></span>{status.text}</span>}>
   <p className="help">Esta é a <strong>única saída de sons do BotLive</strong>: espectador, comandos, timers, automações, resgates, alertas, meta e texto para voz saem todos por aqui. No OBS, o fader da fonte “Áudio do BotLive” vira o volume do bot inteiro — sem VB-Cable e sem saída por módulo.</p>
   <ol className="help">
    <li>Copie o endereço abaixo.</li>
    <li>No OBS: <strong>Fontes → + → Navegador</strong>, nomeie “Áudio do BotLive” e cole o endereço. Largura 640, altura 360 (o visual é transparente).</li>
    <li>Na fonte, marque <strong>“Controlar áudio pelo OBS”</strong> — é isso que coloca o bot no mixer.</li>
    <li>Volte aqui e clique em <strong>Testar áudio</strong>.</li>
   </ol>
   <div className="row">
    <input readOnly value={sink.url} aria-label="Endereço da fonte de áudio" onFocus={e=>e.currentTarget.select()}/>
    <button onClick={()=>void copy()}>{copied?<Check size={15}/>:<Copy size={15}/>}{copied?'Copiado':'Copiar endereço'}</button>
   </div>
   <div className="row">
    <button className="primary" disabled={!desktop||testing} onClick={()=>void test()}><Play size={15}/>{testing?'Tocando…':'Testar áudio'}</button>
    <button disabled={!desktop} onClick={()=>void stop()}><Square size={15}/>Parar todos os sons</button>
   </div>
   {feedback&&feedback.phase==='sent'&&<p className="help">Som enviado — aguardando a fonte do OBS confirmar…</p>}
   {feedback&&feedback.phase==='started'&&<p className="help">{FEEDBACK.started}</p>}
   {feedback&&feedback.phase==='done'&&<p className="help">{FEEDBACK.done}</p>}
   {feedback&&feedback.phase==='stopped'&&<p className="help">{FEEDBACK.stopped}</p>}
   {feedback&&feedback.phase==='error'&&<p role="alert" className="inline-error">Falha na fonte: {feedback.detail||'a reprodução não começou'}</p>}
   {feedback&&feedback.phase==='timeout'&&<p role="alert" className="inline-error">Sem confirmação da fonte em 12 s. Confira se a fonte está ativa na cena, se “Controlar áudio pelo OBS” está marcado e se ela não foi silenciada no mixer.</p>}
   {!desktop&&<p className="help">Prévia no navegador: a saída de sons funciona no aplicativo desktop.</p>}
  </Card>

  <Card title="Como o som chega na transmissão">
   <div className="list-row"><div><strong>O fader “Áudio do BotLive” controla os sons na transmissão</strong><small>Com a fonte marcada como “Controlar áudio pelo OBS”, todos os sons do bot entram no mixer do OBS por essa fonte só. Mutar o fader silencia o bot inteiro; mexer no volume mexe em tudo.</small></div></div>
   <div className="list-row"><div><strong>Ouvir localmente depende da monitoração do OBS</strong><small>Para ouvir no seu fone, use a monitoração de áudio do próprio OBS (Configurações → Áudio → Monitoramento de áudio de saída). Não capture a monitoração como fonte de novo — isso duplicaria o som.</small></div></div>
   <div className="list-row"><div><strong>“Enviado” não é “o público ouvindo”</strong><small>O estado acima confirma conexão e reprodução iniciada pela fonte. Mesmo assim, mute, volume baixo ou transmissão parada no OBS podem impedir que o público ouça — confira o mixer durante a live.</small></div></div>
   <div className="list-row"><div><strong>Mantenha a fonte ativa nas cenas</strong><small>A fonte precisa existir nas cenas que você usa (ou numa cena-mãe sempre ativa). Se ela sumir da cena, os sons ficam de fora até voltar; ao reconectar, sons antigos não são repetidos.</small></div></div>
   <div className="list-row"><div><strong>Volumes individuais continuam onde sempre estiveram</strong><small>Som de espectador, som do fluxo, alerta e meta guardam o volume de cada tela. Aqui só se decide <em>para onde</em> o som vai.</small></div></div>
  </Card>

  <Card title="O que mudou na migração">
   <p className="help">Antes, cada som saía por um aparelho de saída escolhido no Windows (muitas vezes o <strong>VB-Cable</strong>) e o OBS capturava esse aparelho. Esse caminho foi substituído: a escolha de aparelho da tela Áudio <strong>não tem mais efeito</strong> e o VB-Cable deixou de ser necessário para os sons do bot. Arquivos, comandos, resgates, alertas e volumes continuam exatamente como você configurou.</p>
   <p className="help">Se o OBS ainda captura um cabo antigo (“Áudio da área de trabalho” apontando para CABLE Output), remova essa fonte do OBS para os sons do bot não entrarem em dois lugares. O <strong>VB-Cable em si pode continuar instalado</strong> — a Voz da live (efeitos internos e Voicemod) ainda usa o cabo como caminho próprio do microfone, separado dos sons do bot.</p>
  </Card>
 </div>;
}
