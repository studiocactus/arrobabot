import {useEffect,useRef,useState} from 'react';
import {api,errorText} from './api';
import {Field,Modal} from './components';
import {newAction,type Action} from './types';
import {stepLabel} from './stepLabel';
type Entry={id:string;name:string;action:Action};
/** O original é editado pelo mesmo inspetor, como cópia explícita antes de atualizar. */
export default function ActionLibrary({profileId,action,onChange}:{profileId:string;action:Action;onChange:(a:Action)=>void}){
 const [entries,setEntries]=useState<Entry[]|null>(null),[error,setError]=useState(''),[open,setOpen]=useState(false);
 const [deleting,setDeleting]=useState(false);
 const [name,setName]=useState(''),[replace,setReplace]=useState(''),[busy,setBusy]=useState(false);const lock=useRef(false);
 async function load(){setError('');try{const result=await api<Entry[]>('actionLibrary.list',{profileId});if(!Array.isArray(result))throw Error('Não foi possível ler a biblioteca');setEntries(result)}catch(e){setError(errorText(e))}}
 useEffect(()=>{if(action.kind==='action.ref')void load()},[profileId,action.kind]);
 const chosen=entries?.find(e=>e.id===action.target);
 function link(entry:Entry){onChange({...newAction('action.ref'),target:entry.id,text:entry.name})}
 async function save(){if(lock.current)return;lock.current=true;setBusy(true);setError('');try{
  const entry=await api<Entry>('actionLibrary.save',{profileId,entry:{id:replace||crypto.randomUUID(),name:name.trim(),action}});
  link(entry);setOpen(false);
 }catch(e){setError(errorText(e))}finally{lock.current=false;setBusy(false)}}
 return <>{action.kind==='action.ref'?<>
  <Field label="Etapa da biblioteca" hint="Usa a configuração original deste perfil no próximo disparo. Alterações não mudam execuções já iniciadas."><select aria-label="Etapa da biblioteca" value={action.target} onChange={e=>{const item=entries?.find(x=>x.id===e.target.value);if(item)link(item)}}><option value="">Escolha…</option>{entries?.map(e=><option key={e.id} value={e.id}>{e.name}</option>)}{action.target&&!chosen&&<option value={action.target}>Referência indisponível — atualize a biblioteca</option>}</select></Field>
  {chosen&&<><p className="help">{stepLabel(chosen.action)}</p><button type="button" onClick={()=>onChange(structuredClone(chosen.action))}>Editar uma cópia no inspetor</button><button type="button" className="danger" onClick={()=>setDeleting(true)}>Excluir original da biblioteca</button></>}
  <button type="button" onClick={()=>void load()}>Atualizar biblioteca</button>
  <p className="help">Para criar: configure uma etapa normal e use Guardar etapa reutilizável. Referências não podem chamar outras referências; variáveis e condições usam o contexto do fluxo atual.</p>
 </>:<button type="button" onClick={()=>{setName('');setReplace('');setOpen(true);void load()}}>Guardar etapa reutilizável</button>}
 {error&&!open&&!deleting&&<p role="alert" className="inline-error">{error}</p>}
 {deleting&&<Modal title="Excluir etapa reutilizável?" onClose={()=>{if(!busy)setDeleting(false)}}><div className="form-pad"><p>Excluir {chosen?.name}? A exclusão é recusada se algum fluxo salvo ainda usar esta etapa.</p>{error&&<p role="alert" className="inline-error">{error}</p>}<footer className="form-footer"><button disabled={busy} onClick={()=>setDeleting(false)}>Cancelar</button><button disabled={busy} onClick={async()=>{if(lock.current)return;lock.current=true;setBusy(true);setError('');try{await api('actionLibrary.delete',{profileId,id:action.target});setDeleting(false);await load()}catch(e){setError(errorText(e))}finally{lock.current=false;setBusy(false)}}}>Confirmar exclusão</button></footer></div></Modal>}
 {open&&<Modal title="Guardar etapa reutilizável" onClose={()=>{if(!busy)setOpen(false)}}><div className="form-pad">
  <p>A configuração desta etapa será guardada e o bloco atual ficará vinculado ao original. O fluxo ainda precisa ser salvo.</p>
  <Field label="Guardar como"><select aria-label="Guardar como" value={replace} disabled={busy||!entries} onChange={e=>{setReplace(e.target.value);setName(entries?.find(x=>x.id===e.target.value)?.name||'')}}><option value="">Nova etapa reutilizável</option>{entries?.map(e=><option value={e.id} key={e.id}>Atualizar: {e.name}</option>)}</select></Field>
  <Field label="Nome da etapa reutilizável"><input value={name} maxLength={80} disabled={busy} onChange={e=>setName(e.target.value)}/></Field>
  {replace&&<p className="notice">Atualizar altera todas as referências deste perfil nos próximos disparos.</p>}
  {error&&<p role="alert" className="inline-error">{error}</p>}
  <footer className="form-footer"><button disabled={busy} onClick={()=>setOpen(false)}>Cancelar</button><button className="primary" disabled={busy||!entries||!name.trim()} onClick={()=>void save()}>{busy?'Guardando…':'Salvar e vincular etapa'}</button></footer>
 </div></Modal>}
 </>;
}
