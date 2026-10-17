import {useCallback,useEffect,useState} from 'react';
import {Plus,Trash2,Save,Play,X,LifeBuoy} from 'lucide-react';
import {api,errorText,desktop} from './api';
import {Card,Field,Toggle} from './components';
import {voiceEffects} from './types';

/**
 * Resgates de voz: cada recompensa do canal ligada a uma voz real do Voicemod,
 * com duração, cooldown e política de concorrência explícitas. A fila mostra o
 * efeito atual, o tempo que resta e o que vem depois — e cumprir o resgate só
 * acontece depois que o Voicemod confirma a troca.
 */
type Reward={id:string;title:string;cost:number;enabled?:boolean;paused?:boolean};
type VmVoice={voiceID:string;name:string;enabled?:boolean};
type Binding={rewardId:string;rewardName:string;voiceId:string;voiceName:string;durationSec:number;cooldownSec:number;concurrency:string};
type Config={enabled:boolean;queueLimit:number;bindings:Binding[];legacyMap:Record<string,string>};
type Active={rewardId:string;rewardName:string;voiceId:string;voiceName:string;user:string;durationSecs:number;remainingMs:number;forever:boolean};
type Upcoming={rewardId:string;rewardName:string;voiceId:string;voiceName:string;user:string;durationSecs:number};
type Queue={active:Active|null;upcoming:Upcoming[];last:string;lastKind:string;pendingRestore:boolean;busy:boolean};
type View={queue:Queue;config:Config};

const POLICIES={queue:'Fila (um depois do outro)',ignore:'Ignorar enquanto houver efeito',replace:'Substituir o efeito atual'};
const EMPTY:Config={enabled:false,queueLimit:3,bindings:[],legacyMap:{}};

function secsLabel(n:number){return n===0?'até cancelar':n%3600===0?`${n/3600}h`:n%60===0?`${n/60}min`:`${n}s`}

export default function VoiceResgates({profileId,notify}:{profileId:string;notify:(s:string)=>void}){
 const [view,setView]=useState<View|null>(null);
 const [config,setConfig]=useState<Config>(EMPTY);
 const [rewards,setRewards]=useState<Reward[]>([]);
 const [voices,setVoices]=useState<VmVoice[]>([]);
 const [busy,setBusy]=useState(false);
 const [error,setError]=useState('');
 const [testSecs,setTestSecs]=useState(15);

 const load=useCallback(async()=>{
  if(!desktop){setView(null);return}
  try{
   const v=await api<View>('voiceResgate.get',{profileId});
   setView(v);setConfig(v.config||EMPTY);
  }catch(e){setError(errorText(e))}
  try{const r=await api<{rewards:Reward[]}>('twitch.rewards',{profileId});setRewards(r.rewards||[])}catch{/* sem Twitch autorizada a tela ainda mostra a fila */}
  try{const vm=await api<{status:{voices:VmVoice[]}}>('voicemod.get',{profileId});setVoices((vm.status?.voices||[]).filter(v=>v.enabled!==false))}catch{/* Voicemod fechado: lista vazia com aviso próprio */}
 },[profileId]);
 useEffect(()=>{void load()},[load]);
 // Fila muda sozinha (relógio do efeito): acompanhar enquanto houver movimento.
 useEffect(()=>{
  if(!view?.queue.busy)return;
  const t=window.setInterval(()=>{api<View>('voiceResgate.get',{profileId}).then(v=>setView(v)).catch(()=>{})},2000);
  return()=>window.clearInterval(t);
 },[view?.queue.busy,profileId]);

 async function run(fn:()=>Promise<void>){
  setBusy(true);setError('');
  try{await fn()}catch(e){setError(errorText(e))}
  setBusy(false);
 }
 const save=()=>run(async()=>{
  const v=await api<View>('voiceResgate.save',{profileId,config});
  setView(v);setConfig(v.config||EMPTY);
  notify('Vínculos de resgate de voz salvos.');
 });
 const cancel=()=>run(async()=>{
  const v=await api<View>('voiceResgate.cancel',{profileId});
  setView(v);
  notify('Efeitos cancelados; a voz de antes foi restaurada.');
 });
 const recover=()=>run(async()=>{
  const v=await api<View>('voiceResgate.recover',{profileId});
  setView(v);
  notify('Recuperação pedida ao Voicemod.');
 });
 const testVoice=()=>run(async()=>{
  const id=voices[0]?.voiceID||'';
  if(!id){setError('Conecte o Voicemod para testar.');return}
  await api('voiceResgate.testApply',{profileId,voiceId:id,seconds:Math.max(5,testSecs)});
  notify('Voz aplicada pela tela; a fila assume o mesmo caminho de um resgate de verdade.');
  void load();
 });
 const setRow=(i:number,patch:Partial<Binding>)=>setConfig(c=>({...c,bindings:c.bindings.map((b,j)=>j===i?{...b,...patch}:b)}));
 const addRow=()=>setConfig(c=>{
  const used=new Set(c.bindings.map(b=>b.rewardId));
  const reward=rewards.find(r=>!used.has(r.id));
  return {...c,bindings:[...c.bindings,{rewardId:reward?.id||'',rewardName:reward?.title||'',voiceId:'',voiceName:'',durationSec:30,cooldownSec:0,concurrency:'queue'}]};
 });
 const delRow=(i:number)=>setConfig(c=>({...c,bindings:c.bindings.filter((_,j)=>j!==i)}));

 const q=view?.queue;
 const active=q?.active;
 const vmConnected=voices.length>0;

 return <Card title="Resgates de voz (Voicemod)">
  <p className="help">Cada recompensa do canal pode trocar a sua voz no Voicemod por um tempo combinado. O resgate é marcado como cumprido <strong>só depois que o Voicemod confirma a troca</strong> — sem promessa de reembolso se a voz falhar: a Twitch decide o estorno. Trocas manuais que você fizer no Voicemod durante o efeito são respeitadas.</p>
  {error&&<p role="alert" className="inline-error">{error}</p>}
  <h4>Fila agora</h4>
  {!q&&<p className="help">Sem estado ainda — abra o aplicativo desktop para ver a fila viva.</p>}
  {q&&<>
   {active?<div className="list-row"><div><strong>{active.rewardName||active.rewardId}</strong><small>por {active.user} · voz {active.voiceName||active.voiceId} · {active.forever?'sem fim (até cancelar)':`${secsLabel(Math.ceil(active.remainingMs/1000))} restantes`}</small></div><span className="status-pill on"><span className="status-dot" aria-hidden="true"></span><span>No ar</span></span></div>
    :<p className="help">Nenhum efeito no ar.</p>}
   {q.upcoming.length>0&&<>
    <p className="help">Na fila:</p>
    {q.upcoming.map((u,i)=><div className="list-row" key={u.rewardId+i}><div><strong>{i+1}. {u.rewardName||u.rewardId}</strong><small>por {u.user} · voz {u.voiceName||u.voiceId} · {u.durationSecs===0?'até cancelar':secsLabel(u.durationSecs)}</small></div></div>)}
   </>}
   {q.pendingRestore&&<p role="alert" className="inline-error">A conexão caiu no meio do efeito: a restauração da voz de antes está pendente. Reconecte o Voicemod e use “Recuperar” — nada é reaplicado sozinho.</p>}
   {q.last&&<p className="help">{q.last}</p>}
   <div className="row">
    <button className="danger" disabled={busy||!q.busy} onClick={()=>void cancel()}><X size={15}/>Cancelar efeitos e voltar à voz normal</button>
    <button disabled={busy||!q.pendingRestore} onClick={()=>void recover()}><LifeBuoy size={15}/>Recuperar restauração pendente</button>
   </div>
  </>}
  <h4>Configurar vínculos</h4>
  <div className="switch-row"><span>Ligar resgates de voz</span><Toggle label="Ligar resgates de voz" checked={config.enabled} onChange={enabled=>setConfig({...config,enabled})}/></div>
  <Field label="Tamanho da fila" hint="Quantos resgates esperam enquanto um efeito está no ar. Passou disso, o novo resgate é recusado com aviso — a live não vira fila de espera.">
   <input type="number" min={1} max={10} value={String(config.queueLimit)} onChange={e=>setConfig({...config,queueLimit:Math.max(1,Math.min(10,Math.round(Number(e.target.value)||3)))})}/>
  </Field>
  {config.bindings.map((b,i)=><div key={i} className="resgate-row">
   <Field label="Recompensa do canal" hint="Lista real das recompensas autorizadas da Twitch.">
    <select value={b.rewardId} onChange={e=>{const r=rewards.find(x=>x.id===e.target.value);setRow(i,{rewardId:e.target.value,rewardName:r?.title||b.rewardName})}}>
     <option value="">Escolha a recompensa…</option>
     {rewards.map(r=><option key={r.id} value={r.id}>{r.title} · {r.cost} pts{r.enabled===false?' (desativada)':''}{r.paused?' (pausada)':''}</option>)}
     {b.rewardId&&!rewards.some(r=>r.id===b.rewardId)&&<option value={b.rewardId}>{b.rewardName||b.rewardId} — guardada fora desta lista</option>}
    </select>
   </Field>
   <Field label="Voz do Voicemod" hint="Lista real vinda do aplicativo. “Efeito desligado” não serve: o vínculo precisa trocar para uma voz de verdade.">
    <select value={b.voiceId} onChange={e=>{const v=voices.find(x=>x.voiceID===e.target.value);setRow(i,{voiceId:e.target.value,voiceName:v?.name||b.voiceName})}}>
     <option value="">Escolha a voz…</option>
     {voices.map(v=><option key={v.voiceID} value={v.voiceID}>{v.name}</option>)}
     {b.voiceId&&!voices.some(v=>v.voiceID===b.voiceId)&&<option value={b.voiceId}>{b.voiceName||b.voiceId} — guardada fora desta lista</option>}
    </select>
   </Field>
   <div className="form-grid">
    <Field label="Duração (segundos)" hint="0 = até cancelar ou outro resgate.">
     <input type="number" min={0} max={86400} value={String(b.durationSec)} onChange={e=>setRow(i,{durationSec:Math.max(0,Math.min(86400,Math.round(Number(e.target.value)||0)))})}/>
    </Field>
    <Field label="Cooldown do canal (segundos)" hint="0 = sem limite entre resgates do mesmo reward.">
     <input type="number" min={0} max={86400} value={String(b.cooldownSec)} onChange={e=>setRow(i,{cooldownSec:Math.max(0,Math.min(86400,Math.round(Number(e.target.value)||0)))})}/>
    </Field>
    <Field label="Se outro efeito já estiver no ar">
     <select value={b.concurrency} onChange={e=>setRow(i,{concurrency:e.target.value})}>{Object.entries(POLICIES).map(([k,n])=><option key={k} value={k}>{n}</option>)}</select>
    </Field>
   </div>
   <div className="row">
    <button disabled={busy||!b.voiceId} onClick={()=>void run(async()=>{await api('voiceResgate.testApply',{profileId,voiceId:b.voiceId,seconds:Math.max(5,testSecs)});notify('Voz testada; a restauração segue o mesmo caminho dos resgates.');void load()})}><Play size={15}/>Testar esta voz</button>
    <button className="danger" onClick={()=>delRow(i)}><Trash2 size={15}/>Remover vínculo</button>
   </div>
  </div>)}
  <div className="row"><button onClick={addRow} disabled={!rewards.length}><Plus size={15}/>Adicionar vínculo</button></div>
  {!rewards.length&&<p className="help">Sem recompensas na lista: autorize a conta do canal em Meta → Twitch (permissão de gerenciar recompensas) e atualize. Restrições de preço e criação continuam sendo da Twitch.</p>}
  {!vmConnected&&<p className="help">Sem vozes do Voicemod na lista: conecte-o acima (seção “Minha voz na live”) para escolher vozes reais.</p>}
  <h4>Teste de voz pela tela</h4>
  <Field label="Duração (segundos)" hint="Aplica a primeira voz conectada e devolve a de antes ao terminar — o mesmo caminho dos resgates.">
   <input type="number" min={5} max={600} value={String(testSecs)} onChange={e=>setTestSecs(Math.max(5,Math.min(600,Math.round(Number(e.target.value)||15))))}/>
  </Field>
  <button disabled={busy||!vmConnected} onClick={()=>void testVoice()}><Play size={15}/>Aplicar primeira voz conectada</button>
  <details>
   <summary>Efeitos internos antigos (migração para o Voicemod)</summary>
   <p className="help">Fluxos e comandos que ainda pedem um efeito interno (fina, robô…) podem ser ligados a uma voz do Voicemod aqui. O mapeamento é escolhido por você — nenhum nome vira ID por suposição. Com o caminho da voz em “Voicemod”, esses efeitos passam a usar estas vozes; sem vínculo, o pedido é recusado com aviso.</p>
   {Object.entries(voiceEffects).filter(([k])=>k!=='normal').map(([k,n])=><Field key={k} label={n}>
    <select value={String(config.legacyMap[k]||'')} onChange={e=>setConfig({...config,legacyMap:{...config.legacyMap,[k]:e.target.value}})}>
     <option value="">Sem equivalência (mantém recusa)</option>
     {voices.map(v=><option key={v.voiceID} value={v.voiceID}>{v.name}</option>)}
    </select>
   </Field>)}
  </details>
  <div className="form-footer"><span className="help">Configuração exclusiva deste perfil{!desktop?' · a fila viva só aparece no aplicativo desktop':''}</span><button disabled={busy} onClick={()=>void save()}><Save size={15}/>Salvar vínculos</button></div>
 </Card>;
}
