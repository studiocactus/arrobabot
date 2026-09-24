import {useEffect,useState} from 'react';
import {listen} from '@tauri-apps/api/event';
import {Search} from 'lucide-react';
import {api,errorText,desktop} from './api';
import {type Flow,type Profile} from './types';
import {Card,Empty,Jump,Tag} from './components';
import {CounterAdjustDialog,useFlows} from './FlowTools';

export default function Counters({profile,notify,onGoCommands}:{profile:Profile;notify:(s:string)=>void;onGoCommands:()=>void}){
 const {flows,refresh}=useFlows(profile.id,notify);
 const [counts,setCounts]=useState<Record<string,number>>({});
 const [query,setQuery]=useState('');
 const [editing,setEditing]=useState<Flow|null>(null);
 const refreshCounts=()=>api<Record<string,number>>('command.counters',{profileId:profile.id}).then(setCounts).catch(e=>notify(errorText(e)));
 useEffect(()=>{void refreshCounts()},[profile.id]);
 useEffect(()=>{if(!desktop)return;let cancelled=false;let unlisten:(()=>void)|undefined;void listen<{profileId:string;id:string;value:number}>('command-counter',({payload})=>{if(payload.profileId===profile.id)setCounts(c=>({...c,[payload.id]:payload.value}))}).then(fn=>{if(cancelled)fn();else unlisten=fn});return()=>{cancelled=true;unlisten?.()}},[profile.id]);
 const counters=flows.filter(f=>f.counter);
 const visible=counters.filter(f=>(f.name+' '+f.trigger.pattern).toLowerCase().includes(query.toLowerCase()));
 const total=counters.reduce((sum,f)=>sum+(counts[f.id]||0),0);
 return <>
 <p className="help">Cada comando com contador ativo soma 1 a cada uso aceito, antes das ações. Simulações não alteram o total e os valores são reiniciados apenas quando você zera ou apaga o comando. Use {'{{commandCount}}'} na resposta para mostrar a contagem.</p>
 {counters.length>0&&<div className="section-tools"><div className="search"><Search size={17}/><input aria-label="Buscar contadores" value={query} onChange={e=>setQuery(e.target.value)} placeholder="Buscar contadores…"/></div><Tag color="purple">{total} usos registrados</Tag></div>}
 {visible.length?<Card title="Contador por comando"><div className="table-scroll"><table><thead><tr><th>Comando</th><th>Resposta</th><th>Contador</th><th></th></tr></thead><tbody>{visible.map(f=><tr key={f.id}><td><strong>{f.trigger.pattern}</strong><small>{f.name}</small></td><td className="response-cell">{f.actions[0]?.text}</td><td>{desktop?<button onClick={()=>setEditing(f)} aria-label={'Ajustar contador de '+f.name}>{counts[f.id]||0} · Ajustar</button>:counts[f.id]||0}</td><td><Jump onClick={onGoCommands}>Ver comando</Jump></td></tr>)}</tbody></table></div></Card>:<Empty title={counters.length?'Nenhum resultado':'Nenhum contador por aqui'} action={<button className="primary" onClick={onGoCommands}>Abrir Comandos</button>}>{counters.length?'Tente outro nome ou comando.':'Abra um comando, ative “Contar usos deste comando” e a contagem aparece aqui. A variável {{commandCount}} mostra o total na resposta.'}</Empty>}
 {editing&&<CounterAdjustDialog key={editing.id} profile={profile} flow={editing} value={editing.id in counts?counts[editing.id]:0} notify={notify} onClose={()=>setEditing(null)} onSaved={()=>{void refreshCounts();refresh()}}/>}
 </>
}
