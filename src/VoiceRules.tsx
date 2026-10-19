import {useCallback,useEffect,useState} from 'react';
import {Plus,Trash2,Play,RefreshCw} from 'lucide-react';
import {api,desktop,errorText} from './api';
import {Field,Toggle} from './components';

/**
 * Etapa 3 da tela Voicemod: o que troca a sua voz. Cada regra liga um
 * acionamento (recompensa da Twitch ou comando do chat) a uma voz real do
 * Voicemod por um tempo. Tudo passa pelo mesmo serviço de troca — fila,
 * duração e restauração — e o teste de regra não gasta pontos nem publica
 * nada. Ajustes finos (permissão, intervalo, política de fila) ficam em
 * “Avançado”, recolhidos.
 */

export type VmVoice={voiceID:string;name:string;enabled?:boolean};

export type Rule={
 kind:'reward'|'command';
 rewardId:string;
 rewardName:string;
 command:string;
 permission:'everyone'|'moderator'|'broadcaster';
 enabled:boolean;
 voiceId:string;
 voiceName:string;
 durationSec:number;
 cooldownSec:number;
 concurrency:'queue'|'ignore'|'replace';
};

export type RulesConfig={
 enabled:boolean;
 queueLimit:number;
 bindings:Rule[];
 legacyMap:Record<string,string>;
 obsChecked:boolean;
};

type Reward={id:string;title:string;cost:number};

const NEW_RULE=(voices:VmVoice[]):Rule=>({
 kind:'reward',rewardId:'',rewardName:'',command:'',permission:'moderator',enabled:true,
 voiceId:voices[0]?.voiceID||'',voiceName:voices[0]?.name||'',durationSec:30,cooldownSec:0,concurrency:'queue',
});

const POLICIES:{[k:string]:string}={queue:'Fila (um depois do outro)',ignore:'Ignorar enquanto houver efeito',replace:'Substituir o efeito atual'};
const PERMISSIONS:{[k:string]:string}={everyone:'Qualquer pessoa do chat',moderator:'Moderadores e o broadcaster',broadcaster:'Só o broadcaster'};

/** Resumo legível da regra: “Comando !robô → Robô → 30 segundos”. */
export function ruleLabel(r:Rule):string{
 const trigger=r.kind==='command'?`Comando ${r.command||'?'}`:(r.rewardName||r.rewardId||'Recompensa ?');
 const secs=r.durationSec===0?'até cancelar':`${r.durationSec} segundos`;
 return `${trigger} → ${r.voiceName||r.voiceId} → ${secs}`;
}

export default function VoiceRules({profileId,notify,voices}:{profileId:string;notify:(s:string)=>void;voices:VmVoice[]}){
 const [cfg,setCfg]=useState<RulesConfig>({enabled:false,queueLimit:3,bindings:[],legacyMap:{},obsChecked:false});
 const [rewards,setRewards]=useState<Reward[]>([]);
 const [twitchOn,setTwitchOn]=useState(true);
 const [busy,setBusy]=useState(false);
 const [error,setError]=useState('');
 const [device,setDevice]=useState<{device_code:string;user_code:string;verification_uri:string}|null>(null);

 const load=useCallback(()=>{
  if(!desktop)return;
  api<{config:RulesConfig}>('voiceResgate.get',{profileId}).then(r=>{
   if(r?.config)setCfg({enabled:!!r.config.enabled,queueLimit:r.config.queueLimit||3,bindings:r.config.bindings||[],legacyMap:r.config.legacyMap||{},obsChecked:!!r.config.obsChecked});
  }).catch(()=>{});
  api<{rewards:Reward[]}>('twitch.rewards',{profileId}).then(r=>{
   setRewards(r?.rewards||[]);setTwitchOn(true);
  }).catch(()=>{setTwitchOn(false);setRewards([])});
 },[profileId]);
 useEffect(()=>{void load()},[load]);

 const save=async(next:RulesConfig)=>{
  setBusy(true);setError('');
  try{
   const out=await api<{config?:RulesConfig}>('voiceResgate.save',{profileId,config:next});
   if(out?.config)setCfg({enabled:!!out.config.enabled,queueLimit:out.config.queueLimit||3,bindings:out.config.bindings||[],legacyMap:out.config.legacyMap||{},obsChecked:!!out.config.obsChecked});
   notify('Trocas de voz salvas.');
  }catch(e){setError(errorText(e))}
  finally{setBusy(false)}
 };

 const patch=(i:number,p:Partial<Rule>)=>{
  const bindings=cfg.bindings.map((b,idx)=>{
   if(idx!==i)return b;
   const n={...b,...p};
   // Nome de recompensa acompanha a escolha; nome de voz acompanha o ID.
   if(p.rewardId!==undefined){const r=rewards.find(x=>x.id===p.rewardId);n.rewardName=r?.title||n.rewardName}
   if(p.voiceId!==undefined){const v=voices.find(x=>x.voiceID===p.voiceId);n.voiceName=v?.name||n.voiceName}
   return n;
  });
  const next={...cfg,bindings};
  setCfg(next);
  return next;
 };

 const testRule=async(r:Rule)=>{
  if(!r.voiceId){setError('Escolha uma voz para esta regra antes de testar.');return}
  setBusy(true);setError('');
  try{
   await api('voiceResgate.testApply',{profileId,voiceId:r.voiceId,seconds:Math.min(Math.max(r.durationSec||10,5),60)});
   notify('Teste no caminho da regra: a voz de antes volta sozinha. Não gasta pontos nem publica nada.');
  }catch(e){setError(errorText(e))}
  finally{setBusy(false)}
 };

 const connectTwitch=async()=>{
  setBusy(true);setError('');
  try{
   const d=await api<{device_code:string;user_code:string;verification_uri:string}>('oauth.start',{profileId,account:'channel'});
   setDevice(d);
  }catch(e){setError(errorText(e))}
  finally{setBusy(false)}
 };
 const finishTwitch=async()=>{
  if(!device)return;
  setBusy(true);setError('');
  try{
   await api('oauth.finish',{profileId,account:'channel',code:device.device_code});
   setDevice(null);notify('Conta do canal vinculada.');void load();
  }catch(e){setError(errorText(e))}
  finally{setBusy(false)}
 };

 return <div className="voice-rules">
  <div className="list-row">
   <div><strong>Trocas de voz ligadas</strong><small>{cfg.enabled?'Regras ativas disparam durante a live.':'Tudo pausado: nenhuma regra dispara.'}</small></div>
   <Toggle label="Ligar trocas de voz" checked={cfg.enabled} onChange={v=>void save({...cfg,enabled:v})}/>
  </div>

  {!cfg.bindings.length&&<p className="help">Nenhuma troca de voz ainda. Use <strong>Adicionar troca de voz</strong> para ligar uma recompensa da Twitch ou um comando do chat a uma voz.</p>}

  {cfg.bindings.map((r,i)=>(
   <section className="card" key={i}>
    <div className="card-heading">
     <h3>{ruleLabel(r)}</h3>
     <Toggle label={r.enabled?'Ativada':'Desativada'} checked={r.enabled} onChange={v=>{const next=patch(i,{enabled:v});void save(next)}}/>
    </div>
    <div className="form-grid">
     <Field label="Acionamento" hint="O que dispara a troca de voz.">
      <select aria-label="Acionamento da troca de voz" value={r.kind} disabled={busy} onChange={e=>{const next=patch(i,{kind:e.target.value as Rule['kind']});void save(next)}}>
       <option value="reward">Recompensa da Twitch</option>
       <option value="command">Comando do chat</option>
      </select>
     </Field>
     {r.kind==='reward'?(
      <Field label="Recompensa" hint="Lista real do seu canal (a conta do canal precisa estar autorizada).">
       <select aria-label="Recompensa da Twitch" value={r.rewardId} disabled={busy} onChange={e=>{const next=patch(i,{rewardId:e.target.value});void save(next)}}>
        <option value="">Escolha a recompensa…</option>
        {rewards.map(x=><option key={x.id} value={x.id}>{x.title}</option>)}
        {!rewards.some(x=>x.id===r.rewardId)&&r.rewardId&&<option value={r.rewardId}>{r.rewardName||'Recompensa guardada fora desta lista'}</option>}
       </select>
      </Field>
     ):(
      <Field label="Comando do chat" hint="Uma palavra, tipo !robô. Quem pode usar fica em Avançado.">
       <input aria-label="Comando do chat" value={r.command} placeholder="!robo" disabled={busy} onChange={e=>{const next=patch(i,{command:e.target.value});void save(next)}}/>
      </Field>
     )}
     <Field label="Voz do Voicemod" hint="Só vozes de verdade da sua licença — “Efeito desligado” não serve.">
      <select aria-label="Voz do Voicemod" value={r.voiceId} disabled={busy} onChange={e=>{const next=patch(i,{voiceId:e.target.value});void save(next)}}>
       {voices.length?<option value="">Escolha a voz…</option>:<option value="" disabled>Nenhuma voz carregada — atualize as vozes na tela Voicemod</option>}
       {voices.map(v=><option key={v.voiceID} value={v.voiceID}>{v.name}</option>)}
      </select>
     </Field>
     <Field label="Duração (segundos)" hint="0 = até a próxima troca ou você encerrar.">
      <input aria-label="Duração do efeito em segundos" type="number" min={0} max={3600} value={r.durationSec} disabled={busy} onChange={e=>{const next=patch(i,{durationSec:Math.max(0,Math.min(3600,+e.target.value||0))});void save(next)}}/>
     </Field>
    </div>
    {r.kind==='command'&&<p className="help">Quem pode usar: <strong>{PERMISSIONS[r.permission]||r.permission}</strong>. Trocar em <strong>Avançado</strong>.</p>}
    <div className="form-footer">
     <span className="help">Testar exercita o caminho da regra sem gastar pontos, sem resgates e sem publicar mensagens.</span>
     <div className="row">
      <button className="secondary" disabled={busy} onClick={()=>void testRule(r)}><Play size={14}/> Testar esta voz</button>
      <button className="icon-button" aria-label="Remover troca de voz" disabled={busy} onClick={()=>{const next={...cfg,bindings:cfg.bindings.filter((_,idx)=>idx!==i)};setCfg(next);void save(next)}}><Trash2 size={16}/></button>
     </div>
    </div>
    <details>
     <summary>Avançado</summary>
     <div className="form-grid">
      {r.kind==='command'&&<Field label="Quem pode usar o comando" hint="Permissão explícita: nada vira público sem você escolher.">
       <select aria-label="Quem pode usar o comando" value={r.permission} disabled={busy} onChange={e=>{const next=patch(i,{permission:e.target.value as Rule['permission']});void save(next)}}>
        {Object.entries(PERMISSIONS).map(([k,n])=><option key={k} value={k}>{n}</option>)}
       </select>
      </Field>}
      <Field label="Intervalo entre usos (segundos)" hint="Cooldown da regra: 0 = sem intervalo.">
       <input aria-label="Intervalo entre usos em segundos" type="number" min={0} max={86400} value={r.cooldownSec} disabled={busy} onChange={e=>{const next=patch(i,{cooldownSec:Math.max(0,Math.min(86400,+e.target.value||0))});void save(next)}}/>
      </Field>
      <Field label="Se outro efeito já estiver no ar" hint="Política da fila: por padrão, um efeito por vez.">
       <select aria-label="Política quando outro efeito está no ar" value={r.concurrency} disabled={busy} onChange={e=>{const next=patch(i,{concurrency:e.target.value as Rule['concurrency']});void save(next)}}>
        {Object.entries(POLICIES).map(([k,n])=><option key={k} value={k}>{n}</option>)}
       </select>
      </Field>
     </div>
    </details>
   </section>
  ))}

  <div className="form-footer">
   <span className="help">{!twitchOn?'Recompensas exigem a conta do canal da Twitch autorizada; comandos do chat funcionam sem isso.':'Regras ficam salvas neste perfil.'}</span>
   <button className="primary" disabled={busy} onClick={()=>{const next={...cfg,bindings:[...cfg.bindings,NEW_RULE(voices)]};setCfg(next)}}><Plus size={16}/> Adicionar troca de voz</button>
  </div>

  {!twitchOn&&!device&&<div className="notice">
   <strong>Sem recompensas da Twitch por aqui.</strong>
   <p className="help">Para acionar por recompensa, conecte a conta do canal neste ponto (mesma autorização do perfil). Comandos do chat não precisam de Twitch.</p>
   <button className="secondary" disabled={busy} onClick={()=>void connectTwitch()}><RefreshCw size={14}/> Conectar conta do canal</button>
  </div>}
  {device&&<div className="notice">
   <strong>Seu código: {device.user_code}</strong>
   <p className="help">Abra a autorização, entre na conta do canal e permita o acesso. Depois volte aqui.</p>
   <div className="row">
    <button className="secondary" onClick={()=>api('url.open',{url:device.verification_uri}).catch(e=>setError(errorText(e)))}>Abrir autorização</button>
    <button className="primary" disabled={busy} onClick={()=>void finishTwitch()}>Já autorizei</button>
   </div>
  </div>}

  {error&&<p role="alert" className="inline-error">{error}</p>}
 </div>;
}
