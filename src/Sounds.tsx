import {useCallback,useEffect,useState} from 'react';
import {open} from '@tauri-apps/plugin-dialog';
import {Play,Plus,Search,Trash2,Upload} from 'lucide-react';
import {api,desktop,errorText} from './api';
import {AccordionItem,Card,Empty,Field,ItemStack,ListTools,MissingOption,Toggle} from './components';
import {fmtDuration,minutesToSeconds,secondsToMinutes} from './types';
type Asset={id:string;kind:string;name:string};
type Person={trigger:string;id:string;enabled:boolean;name:string;nickname:string;userId:string;asset:string;mode:string;cooldown:number;volume:number};
/** A config salva junta respostas e sons; esta tela mexe só nos sons por espectador. */
type Config={soundsEnabled:boolean;presenceEnabled:boolean;repliesEnabled:boolean;replies:unknown[];people:Person[]};
const TRIGGER:Record<string,string>={message:'Ao falar',join:'Ao entrar',either:'Entrar ou falar'};
const MODE:Record<string,string>={first:'Uma vez por sessão',interval:'Com intervalo'};
const blank:Config={soundsEnabled:false,presenceEnabled:false,repliesEnabled:false,replies:[],people:[]};
export default function Sounds({profileId,notify}:{profileId:string;notify:(s:string)=>void}){
 const [assets,setAssets]=useState<Asset[]>([]);
 const [config,setState]=useState<Config>(blank);
 const [query,setQuery]=useState('');const [qPeople,setQPeople]=useState('');
 const [busy,setBusy]=useState(false);
 const [error,setError]=useState('');
 const [message,setMessage]=useState('');
 const [playing,setPlaying]=useState('');
 const [openIds,setOpenIds]=useState<Record<string,boolean>>({});
 const [dirty,setDirty]=useState(false);
 const load=useCallback(async()=>{
  try{
   const v=await api<{assets:Asset[];config:Config}>('chatExtras.get',{profileId});
   setAssets((v.assets||[]).filter(a=>a.kind==='sound'));
   const c={...blank,...v.config,people:((v.config?.people||[]) as Person[]).map(p=>({...p,trigger:p.trigger||'message'}))};
   setState(c);
  }catch(e){setError(errorText(e))}
 },[profileId]);
 useEffect(()=>{if(desktop)void load()},[load]);
 async function run(fn:()=>Promise<void>){setBusy(true);setError('');setMessage('');try{await fn()}catch(e){setError(errorText(e))}finally{setBusy(false)}}
 async function upload(){await run(async()=>{
  const path=await open({multiple:false,filters:[{name:'Áudio',extensions:['wav','mp3','ogg']}]});if(typeof path!=='string')return;
  const a=await api<Asset>('chatExtras.import',{profileId,kind:'sound',path});setAssets(x=>x.concat(a));setMessage('Som enviado para a biblioteca.');
 })}
 async function remove(a:Asset){
  if(!window.confirm(`Apagar o som “${a.name}”? Onde ele estiver em uso (comandos, timers, resgates, alertas) fica sem áudio até você escolher outro.`))return;
  await run(async()=>{
   await api('chatExtras.remove',{profileId,asset:a.id});setAssets(x=>x.filter(v=>v.id!==a.id));setMessage('Som apagado.');
  });
 }
 async function play(a:Asset){
  setPlaying(a.id);setError('');
  try{
   // Mesmo caminho da live: sai pela fonte do OBS quando conectada; sem
   // fonte, o aplicativo toca na saída padrão do Windows.
   await api('audio.preview',{profileId,asset:a.id,volume:1});
   setMessage('Som enviado pela saída única do BotLive: toca na fonte do OBS quando ela está conectada.');
  }
  catch(e){setError(errorText(e))}
  finally{setPlaying('')}
 }
 const person=(id:string,patch:Partial<Person>)=>{setDirty(true);setState(c=>({...c,people:c.people.map(p=>p.id===id?{...p,...patch}:p)}))};
 const assetName=(id:string)=>assets.find(a=>a.id===id)?.name||'';
 const addPerson=()=>{const id=crypto.randomUUID();setDirty(true);setOpenIds(o=>({...o,[id]:true}));setState(c=>({...c,people:c.people.concat({id,enabled:true,trigger:'message',name:'',nickname:'',userId:'',asset:'',mode:'first',cooldown:60,volume:0.7})}))};
 const personRows=config.people.map((p,i)=>({p,i}));
 const personVisible=personRows.filter(({p,i})=>!qPeople||`${p.nickname} ${p.name} ${assetName(p.asset)} pessoa ${i+1}`.toLowerCase().includes(qPeople.toLowerCase()));
 const allPersonIds=personVisible.map(({p})=>p.id);
 const visible=assets.filter(a=>!query||a.name.toLowerCase().includes(query.toLowerCase()));
 return <div>
  <Card title="Biblioteca de sons" action={assets.length?<span className="list-count">{assets.length} {assets.length===1?'som':'sons'}</span>:undefined}>
   <p className="help">Sons importados valem para tudo o que o bot toca: comandos, timers, automações, resgates de pontos, alertas, respostas e sons por espectador. Um som apagado aqui fica sem áudio onde estiver em uso. A saída de todos eles é a mesma: a fonte “Áudio do BotLive” do OBS (tela <strong>Áudio no OBS</strong>).</p>
   <div className="row">
    <span className="grow" style={{position:'relative'}}>
     <Search size={15} aria-hidden="true" style={{position:'absolute',left:11,top:'50%',transform:'translateY(-50%)',color:'var(--muted)'}}/>
     <input type="search" value={query} placeholder="Filtrar sons" aria-label="Filtrar sons" style={{paddingLeft:34}} onChange={e=>setQuery(e.target.value)}/>
    </span>
    <button className="primary" disabled={!desktop||busy} onClick={()=>void upload()}><Upload size={15}/>Adicionar som</button>
   </div>
   {visible.length===0
    ?<Empty title={query?'Nenhum som encontrado':'Nenhum som ainda'}>{query?'Ajuste o filtro acima.':'Envie um WAV, MP3 ou OGG para o bot poder tocar em comandos, alertas e mais.'}</Empty>
    :<table className="asset-table">
     <thead><tr><th>Nome</th><th className="actions-right">Ações</th></tr></thead>
     <tbody>{visible.map(a=><tr key={a.id}>
      <td>{a.name}</td>
      <td className="row actions-right">
       <button disabled={!desktop||playing===a.id||busy} onClick={()=>void play(a)}><Play size={14}/>{playing===a.id?'Tocando…':'Ouvir'}</button>
       <button className="danger" disabled={busy} onClick={()=>void remove(a)}><Trash2 size={14}/>Apagar</button>
      </td>
     </tr>)}</tbody>
    </table>}
   <p className="help">Clique em Ouvir para conferir pelo mesmo caminho que o bot usa na live — a fonte do OBS quando ela está conectada; senão, a saída padrão do aplicativo.</p>
  </Card>

  <Card title="Sons por espectador">
   <p className="help">Cada pessoa pode ter seu próprio som: escolha o nome de quem fala no chat, o áudio e quando ele toca — ao falar, ao entrar no chat ou nos dois casos. Na Twitch, a entrada silenciosa exige ativar o monitor abaixo e autorizar novamente a conta do bot. Informe o nome exato do chat ou o ID da plataforma; o apelido serve para sua organização. Os sons saem pela saída única do BotLive (tela <strong>Áudio no OBS</strong>), com o Testar som de cada pessoa.</p>
   <div className="list-row"><div><strong>Sons por espectador</strong><small>{config.soundsEnabled?'Quem fala no chat dispara o som da pessoa':'Desligado: nada toca até você ativar'}</small></div><Toggle label="Ativar sons por espectador" checked={config.soundsEnabled} onChange={soundsEnabled=>setState({...config,soundsEnabled})}/></div>
    <div className="list-row"><div><strong>Monitorar entradas silenciosas na Twitch</strong><small>Detecta conexões ao chat, não quem está assistindo ao vídeo. A Twitch pode atrasar ou omitir entradas. Pessoas já presentes ao conectar o bot não disparam som. No YouTube, use mensagens; outras plataformas precisam de uma ponte de entrada.</small></div><Toggle label="Monitorar entradas silenciosas na Twitch" checked={config.presenceEnabled} onChange={presenceEnabled=>setState({...config,presenceEnabled})}/></div>
    {config.people.length===0
     ?<p className="help">Nenhuma pessoa cadastrada ainda. Clique em Adicionar pessoa para escolher o áudio e definir quando ele toca.</p>
     :<>
     <ListTools count={personRows.length+(personRows.length===1?' pessoa':' pessoas')} total={personRows.length} shown={personVisible.length} search={qPeople} onSearch={setQPeople} searchLabel="Filtrar pessoas" onExpandAll={()=>setOpenIds(o=>{const n={...o};allPersonIds.forEach(id=>n[id]=true);return n})} onCollapseAll={()=>setOpenIds(o=>{const n={...o};allPersonIds.forEach(id=>n[id]=false);return n})}/>
     <ItemStack>{personVisible.map(({p,i})=>{
      const label='Pessoa '+(i+1);
      return <AccordionItem key={p.id} id={p.id} open={!!openIds[p.id]} enabled={p.enabled} enabledLabel={'Ativar som da pessoa '+(i+1)} title={p.nickname.trim()||p.name.trim()||label} meta={[TRIGGER[p.trigger]||'',MODE[p.mode]||'',assetName(p.asset)||'sem som',fmtDuration(p.cooldown)]} onToggle={()=>setOpenIds(o=>({...o,[p.id]:!o[p.id]}))} onEnabled={enabled=>person(p.id,{enabled})}>
       <div className="form-grid"><Field label="Nome no chat" hint="Exatamente como a pessoa aparece no chat (sem @)."><input value={p.name} maxLength={100} onChange={e=>person(p.id,{name:e.target.value})}/></Field><Field label="Apelido" hint="Só para sua organização na lista."><input value={p.nickname} maxLength={100} onChange={e=>person(p.id,{nickname:e.target.value})}/></Field></div>
       <Field label="ID da pessoa (opcional)" hint="Quando preenchido, o ID tem prioridade sobre o nome; recomendado para evitar nomes duplicados."><input value={p.userId} maxLength={200} onChange={e=>person(p.id,{userId:e.target.value})}/></Field>
       <Field label="Som da pessoa" hint="Um áudio da biblioteca; toca na saída dos sons do bot."><div className="row"><select aria-label={'Som da pessoa '+(i+1)} value={p.asset} onChange={e=>person(p.id,{asset:e.target.value})}><option value="">Escolha um som</option>{assets.map(a=><option key={a.id} value={a.id}>{a.name}</option>)}{!assets.some(a=>a.id===p.asset)&&<MissingOption value={p.asset}/>}</select><button disabled={!desktop||!p.asset} onClick={()=>void play(assets.find(a=>a.id===p.asset)||{id:p.asset,kind:'sound',name:''})}>Testar som</button></div></Field>
       <div className="form-grid"><Field label="Disparar som" hint="Quando a pessoa precisa estar presente para o som tocar."><select value={p.trigger} onChange={e=>person(p.id,{trigger:e.target.value})}><option value="message">Quando enviar mensagem</option><option value="join">Quando entrar, mesmo sem falar</option><option value="either">Ao entrar ou enviar mensagem</option></select></Field><Field label="Quando tocar" hint="Uma vez evita repetição na mesma live; com intervalo permite tocar de novo."><select value={p.mode} onChange={e=>person(p.id,{mode:e.target.value})}><option value="first">Uma vez por sessão</option><option value="interval">Repetir com intervalo</option></select></Field><Field label="Intervalo mínimo (minutos)" hint="Com intervalo mínimo de 5 minutos."><input type="number" min="0.5" max="1440" step="any" value={secondsToMinutes(p.cooldown)} onChange={e=>person(p.id,{cooldown:Math.max(5,minutesToSeconds(Number(e.target.value)))})}/></Field></div>
       <Field label={'Volume: '+Math.round(p.volume*100)+'%'}><input type="range" min="0" max="1" step="0.05" value={p.volume} onChange={e=>person(p.id,{volume:+e.target.value})}/></Field>
       <div className="item-actions"><button className="danger" onClick={()=>setState({...config,people:config.people.filter(x=>x.id!==p.id)})}>Remover pessoa</button></div>
      </AccordionItem>})}
     </ItemStack>
     {personVisible.length===0&&<p className="help">Nenhuma pessoa corresponde a “{qPeople}”.</p>}
     </>}
    <div className="row">
     <button onClick={addPerson}><Plus size={15}/>Adicionar pessoa</button>
     <button disabled={!desktop} onClick={()=>void run(async()=>{await api('chatExtras.reset',{profileId});setMessage('Sessão de sons reiniciada. A próxima participação poderá tocar novamente.')})}>Reiniciar sessão de sons</button>
    </div>
    <p className="help">Há intervalo geral de 5 segundos entre sons. A sessão reinicia ao fechar o app ou pelo botão acima. Mantenha o aplicativo aberto e desbloqueado. Testar som reproduz mesmo com a função desativada.</p>
    <footer className="form-footer"><span className={'save-state '+(dirty?'dirty':'saved')} role="status">{dirty?'Alterações não salvas':'Tudo salvo'}</span><button className="primary" disabled={!desktop} onClick={()=>void run(async()=>{await api('chatExtras.save',{profileId,config});setDirty(false);setMessage('Sons por espectador salvos.')})}>Salvar sons por espectador</button></footer>
    {error&&<p role="alert" className="inline-error">{error}</p>}{message&&<p role="status">{message}</p>}
   </Card>
  </div>;
}
