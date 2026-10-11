import {useCallback,useEffect,useState} from 'react';
import {Play,Square,Trash2,Plus,RefreshCw} from 'lucide-react';
import {api,desktop,errorText} from './api';
import {Card,Field} from './components';

type GoalDef={title:string;target:number;font_family:string};
type Config={enabled:boolean;clientId?:string;activeGoal:string;goals:Record<string,GoalDef>;startTimestamp:number};
type Data={current:number;updated_at:number;start_timestamp:number;active_goal:string};

const FONTS=['','Segoe UI','Arial','Roboto','Inter','Montserrat'];

export default function LivePix({profileId,notify}:{profileId:string;notify:(s:string)=>void}){
 const [clientId,setClientId]=useState('');
 const [clientSecret,setClientSecret]=useState('');
 const [config,setConfig]=useState<Config|null>(null);
 const [data,setData]=useState<Data|null>(null);
 const [busy,setBusy]=useState(false);
 const [error,setError]=useState('');

 const load=useCallback(async()=>{
  if(!desktop)return;
  try{const c=await api<Config>('module.config.get',{profileId,key:'livepix'});setConfig(c);if(c?.clientId)setClientId(c.clientId)}catch{}
  try{const d=await api<Data>('module.config.get',{profileId,key:'livepix_data'});setData(d)}catch{}
 },[profileId,desktop]);
 useEffect(()=>{if(desktop)void load()},[load]);

 async function call(op:string,args:Record<string,unknown>){
  setBusy(true);setError('');
  try{await api(op,{profileId,...args});await load()}catch(e){setError(errorText(e))}
  setBusy(false);
 }

 const saveCreds=async()=>{
  setBusy(true);setError('');
  try{
   await api('livepix.save_credentials',{profileId,clientId:clientId.trim(),clientSecret});
   if(clientSecret)setClientSecret('');
   await load();
   notify('Credenciais salvas no cofre do sistema.');
  }catch(e){setError(errorText(e))}
  setBusy(false);
 };

 const enable=async()=>{
  const goals=config?.goals?Object.values(config.goals):[];
  if(!goals.length){setError('Crie ao menos uma meta antes de ativar.');return}
  await call('livepix.enable',{goals:goals.map(g=>({title:g.title,target:g.target,font_family:g.font_family}))});
 };

 const fmt=(cents:number)=>(cents/100).toLocaleString('pt-BR',{minimumFractionDigits:2,maximumFractionDigits:2});
 const active=config?.activeGoal||'';
 const activeGoal=active?config?.goals?.[active]:null;
 const pct=activeGoal&&activeGoal.target>0?Math.round(((data?.current||0)*100)/activeGoal.target):0;

 return <div>
  <Card title="LivePix — Meta de doações">
   <p className="help">Conecte sua conta LivePix para exibir uma barra de meta no OBS. O BotLive consulta a API a cada 15 segundos e envia o estado por WebSocket local (porta 9876).</p>

   <div className="form-grid">
    <Field label="Client ID" hint="UUID da aplicação criada no painel da LivePix."><input value={clientId} onChange={e=>setClientId(e.target.value)} placeholder="Ex.: 12345678-abcd-..."/></Field>
    <Field label="Client Secret" hint="Gerado no painel. Fica no cofre do sistema."><input type="password" autoComplete="new-password" value={clientSecret} onChange={e=>setClientSecret(e.target.value)} placeholder="Deixe vazio para manter o segredo atual"/></Field>
   </div>
   <button disabled={busy||(!clientId.trim()&&!clientSecret)} onClick={()=>void saveCreds()}>Salvar no cofre</button>

   {error&&<p role="alert" className="inline-error">{error}</p>}

   <div className="list-row" style={{marginTop:16}}>
    <div><strong>Estado</strong><small>{config?.enabled?'Integração ativa':'Integração desativada'}</small></div>
    {config?.enabled
     ?<button className="danger" disabled={busy} onClick={()=>void call('livepix.disable',{})}><Square size={15}/>Desativar</button>
     :<button className="primary" disabled={busy} onClick={()=>void enable()}><Play size={15}/>Ativar</button>}
   </div>

   {activeGoal&&data&&<div style={{margin:'12px 0'}}>
    <p className="help" style={{margin:0}}><strong>{activeGoal.title}</strong> — {fmt(data.current)} / {fmt(activeGoal.target)} BRL ({pct}%)</p>
   </div>}
  </Card>

  <Card title="Metas">
   <p className="help">Cada meta tem título, valor-alvo em reais e família de fonte. A meta ativa é consultada a cada 15 segundos.</p>
   {config?.goals&&Object.entries(config.goals).map(([gid,g])=>
    <div key={gid} className="list-row">
     <div><strong>{g.title}</strong><small>{fmt(g.target)} BRL — {g.font_family||'fonte padrão'}</small></div>
     <div className="row">
      {config.activeGoal===gid
       ?<span className="status-pill on"><span className="status-dot" aria-hidden="true"></span>Ativa</span>
       :<button disabled={busy||!config.enabled} onClick={()=>void call('livepix.set_active_goal',{goalId:gid})}>Ativar</button>}
      <button disabled={busy} onClick={()=>void call('livepix.remove_goal',{goalId:gid})}><Trash2 size={14}/></button>
     </div>
    </div>
   )}
   <AddGoal onAdd={(g)=>{
    const goals=config?.goals?Object.entries(config.goals).map(([id,v])=>({id,...v})):[];
    void call('livepix.enable',{goals:[...goals.map(x=>({title:x.title,target:x.target,font_family:x.font_family})),g]});
   }} disabled={busy}/>
  </Card>

  <Card title="Overlay no OBS">
   <p className="help">No OBS, adicione uma <strong>Fonte Navegador</strong> apontando para o arquivo <code>examples/goal-overlay.html</code> incluído no pacote do BotLive. Acrescente ao final do endereço <code>file:///</code> o fragmento:</p>
   <code style={{display:'block',padding:'8px 12px',background:'#111b20',borderRadius:6,fontSize:13,wordBreak:'break-all'}}>
    #token=CHAVE_DA_SESSAO&profile={profileId}&port=9876
   </code>
   <p className="help">A chave da sessão muda a cada abertura do BotLive. O fragmento não é enviado a servidores externos.</p>
  </Card>
 </div>
}

function AddGoal({onAdd,disabled}:{onAdd:(g:{title:string;target:number;font_family:string})=>void;disabled:boolean}){
 const [title,setTitle]=useState('');
 const [target,setTarget]=useState('');
 const [font,setFont]=useState('');
 const ok=title.trim()&&Number(target)>0;
 return <details style={{marginTop:12}}><summary><Plus size={14} style={{verticalAlign:'middle'}}/> Nova meta</summary>
  <div className="form-grid" style={{marginTop:8}}>
   <Field label="Título"><input value={title} onChange={e=>setTitle(e.target.value)} placeholder="Ex.: Meta do mês"/></Field>
   <Field label="Valor-alvo (R$)"><input type="number" min="1" step="0.01" value={target} onChange={e=>setTarget(e.target.value)} placeholder="Ex.: 500"/></Field>
   <Field label="Fonte" hint="Deixe vazio para a fonte padrão do sistema.">
    <select value={font} onChange={e=>setFont(e.target.value)}>{FONTS.map(f=><option key={f} value={f}>{f||'Padrão do sistema'}</option>)}</select>
   </Field>
  </div>
  <button disabled={disabled||!ok} onClick={()=>{onAdd({title:title.trim(),target:Math.round(Number(target)*100),font_family:font});setTitle('');setTarget('')}}>Adicionar meta</button>
 </details>
}
