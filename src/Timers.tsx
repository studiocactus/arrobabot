import {useState} from 'react';
import {Plus,Search} from 'lucide-react';
import {newTimer,secondsToMinutes,timerCategories,timerPriorities,type Flow,type Profile} from './types';
import {Card,Empty,Tag} from './components';
import {FlowDialog,FlowEnableToggle,FlowRowActions,RemoveFlowDialog,useFlows} from './FlowTools';

export default function Timers({profile,notify,onChanged}:{profile:Profile;notify:(s:string)=>void;onChanged:()=>void}){
 const {flows,refresh}=useFlows(profile.id,notify);
 const [query,setQuery]=useState('');
 const [editing,setEditing]=useState<Flow|null>(null);
 const [remove,setRemove]=useState<Flow|null>(null);
 const afterChange=async()=>{refresh();onChanged()};
 const visible=flows.filter(f=>f.trigger.kind==='timer'&&(f.name+' '+f.trigger.pattern).toLowerCase().includes(query.toLowerCase()));
 const intervalCount:Record<number,number>={};visible.forEach(f=>{intervalCount[f.timerSeconds||300]=(intervalCount[f.timerSeconds||300]||0)+1});
 return <>
 <p className="help">Mensagens periódicas enquanto o perfil está conectado. Ao pausar, desconectar ou editar, o intervalo começa novamente. Não há envios acumulados ao reconectar. Evite usar o mesmo intervalo em vários timers para reduzir disparos simultâneos.</p>
 <div className="section-tools"><div className="search"><Search size={17}/><input aria-label="Buscar timers" value={query} onChange={e=>setQuery(e.target.value)} placeholder="Buscar timers…"/></div><div className="row"><button className="primary" onClick={()=>setEditing(newTimer(profile.id))}><Plus size={16}/>Novo timer</button></div></div>
 {visible.length?<Card><div className="table-scroll"><table><thead><tr><th>Timer</th><th>Categoria</th><th>Resposta</th><th>Intervalo</th><th>Prioridade</th><th>Ativo</th><th></th></tr></thead><tbody>{visible.map(f=>{const variants=f.timerVariants||[];const dup=(intervalCount[f.timerSeconds||300]||0)>1;return <tr key={f.id}><td><strong>{f.name}</strong></td><td><Tag color="blue">{timerCategories[f.timerCategory||'personalizado']||'Personalizado'}</Tag></td><td className="response-cell">{variants[0]||f.actions[0]?.text}{variants.length>1&&<Tag color="purple">+{variants.length-1}</Tag>}</td><td>{'A cada '+secondsToMinutes(f.timerSeconds||300)+' min'}{dup&&<Tag color="amber">mesmo intervalo</Tag>}</td><td><Tag color={f.timerPriority==='alta'?'green':f.timerPriority==='baixa'?'gray':'purple'}>{timerPriorities[f.timerPriority||'normal']||'Normal'}</Tag></td><td><FlowEnableToggle flow={f} notify={notify} onSaved={afterChange}/></td><td><FlowRowActions flow={f} profile={profile} notify={notify} onEdit={()=>setEditing(f)} onRemove={()=>setRemove(f)}/></td></tr>})}</tbody></table></div></Card>:<Empty title={query?'Nenhum timer encontrado':'Nenhum timer por aqui'} action={<button className="primary" onClick={()=>setEditing(newTimer(profile.id))}><Plus size={16}/>Criar timer</button>}>{query?'Tente outro nome.':'Crie lembretes de água, de pausas ou de novidades no seu canal.'}</Empty>}
 {editing&&<FlowDialog key={editing.id} flow={editing} mode="timer" profile={profile} notify={notify} onSaved={afterChange} onClose={()=>setEditing(null)}/>}
 {remove&&<RemoveFlowDialog flow={remove} mode="timer" notify={notify} onClose={()=>setRemove(null)} onRemoved={afterChange}/>}
 </>
}
