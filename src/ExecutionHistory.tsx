import {useEffect,useRef,useState} from 'react';
import {api,desktop,errorText} from './api';
import {Card,Empty,Modal} from './components';
import {actions} from './types';
import {type TestRun,runStatusLabel,stepStatusLabel,isTerminalRun,runDuration,runModeLabel,eventKindLabel} from './testRun';
import type {ReactNode} from 'react';

/** Usa o acompanhamento do motor; consultar nunca inicia ou repete um efeito. */
export default function ExecutionHistory({profileId,events}:{profileId:string;events:ReactNode}){
 const [tab,setTab]=useState<'runs'|'events'>('runs');
 const [runs,setRuns]=useState<TestRun[]>([]),[error,setError]=useState(''),[loaded,setLoaded]=useState(false);
 const [query,setQuery]=useState(''),[filter,setFilter]=useState('all'),[selected,setSelected]=useState('');
 const [retry,setRetry]=useState(0);
 const [cancelChoice,setCancelChoice]=useState<TestRun|null>(null),[cancelError,setCancelError]=useState('');
 const [requested,setRequested]=useState<string[]>([]),[sending,setSending]=useState(false);
 const cancelLock=useRef(false);
 async function cancel(){
  if(!cancelChoice||cancelLock.current)return;
  cancelLock.current=true;setSending(true);setCancelError('');
  const id=cancelChoice.id;
  try{await api('flow.cancel',{profileId,executionId:id});setRequested(ids=>[...ids,id]);setCancelChoice(null);setRetry(n=>n+1)}
  catch(e){setCancelError(errorText(e));setRetry(n=>n+1)}
  finally{cancelLock.current=false;setSending(false)}
 }

 useEffect(()=>{
  if(!desktop||!profileId)return;
  let stop=false;let timer:ReturnType<typeof setTimeout>;
  const tick=async()=>{try{
   const result=await api<TestRun[]>('flow.runs',{profileId});
   if(!Array.isArray(result))throw Error('Resposta de execuções inválida.');
   if(!stop){setRuns(result);setLoaded(true);setError('')}
  }catch(e){if(!stop)setError(errorText(e))}
  finally{if(!stop)timer=setTimeout(tick,2000)}};
  void tick();return()=>{stop=true;clearTimeout(timer)};
 },[profileId,retry]);
 const visible=runs.filter(r=>(filter==='all'||filter==='failed'&&['FAILED','INTERRUPTED'].includes(r.status)||filter==='active'&&['RUNNING','WAITING'].includes(r.status)||filter==='test'&&r.test)&&`${r.flow} ${r.id} ${runStatusLabel(r.status)} ${r.context?.eventRef||''} ${r.context?eventKindLabel(r.context.eventKind):''} ${runModeLabel(r)}`.toLocaleLowerCase().includes(query.toLocaleLowerCase()));
 const run=runs.find(r=>r.id===selected);
 return <>
  <div className="section-tools"><div className="row"><button aria-pressed={tab==='runs'} onClick={()=>setTab('runs')}>Execuções</button><button aria-pressed={tab==='events'} onClick={()=>setTab('events')}>Eventos gerais</button></div></div>
  {tab==='events'?events:<>
   <p className="help">Até 30 execuções finalizadas por perfil, além das que estão em andamento. O histórico é preservado ao reiniciar. Execuções sem resultado final confirmado são marcadas como interrompidas e não são repetidas.</p>
   {!desktop||!profileId?<Card><Empty title={profileId?'Acompanhamento disponível no aplicativo desktop':'Selecione um perfil'}>Consultar esta tela não inicia nem repete automações.</Empty></Card>:<>
    <div className="section-tools execution-filters"><label><span>Buscar execução</span><input value={query} onChange={e=>setQuery(e.target.value)} placeholder="Nome, estado ou identificador"/></label><label><span>Mostrar</span><select aria-label="Mostrar" value={filter} onChange={e=>setFilter(e.target.value)}><option value="all">Todas as execuções</option><option value="failed">Falhas e interrupções</option><option value="active">Em andamento</option><option value="test">Testes, ensaios e simulações</option></select></label></div>
    {error&&<div className="inline-error" role="alert">Não foi possível atualizar as execuções. {error} {loaded&&'Os dados abaixo podem estar desatualizados.'}<button onClick={()=>setRetry(n=>n+1)}>Tentar novamente</button></div>}
    {!loaded&&!error&&<p role="status">Carregando execuções…</p>}
    {loaded&&<div className="execution-layout"><Card title="Execuções recentes">{visible.length?<div className="execution-list">{visible.map(r=><button key={r.id} aria-pressed={r.id===selected} onClick={()=>setSelected(r.id)}><strong>{r.flow}</strong><span>{runStatusLabel(r.status)} · {runModeLabel(r)}</span><small>{r.id}</small></button>)}</div>:<Empty title={runs.length?'Nenhuma execução corresponde ao filtro':'Nenhuma execução registrada'}>As execuções de fluxos aparecerão aqui quando o motor as registrar.</Empty>}</Card>
     <Card title="Detalhes da execução">{run?<div className="form-pad"><h3>{run.flow}</h3><p>{runStatusLabel(run.status)} · {runModeLabel(run)}</p>{run.context?<div className="execution-context"><strong>Origem da execução</strong><p>{eventKindLabel(run.context.eventKind)} · {runModeLabel(run)}</p><p className="execution-id">Referência do evento: {run.context.eventRef||'Não disponível'}</p><small>Mensagens, pessoas e payloads não são incluídos neste contexto. A referência permite relacionar execuções do mesmo evento neste perfil.</small></div>:<p className="help">Origem não registrada para esta execução.</p>}<p>Início: {run.startedAt?new Date(run.startedAt).toLocaleString('pt-BR'):'Não registrado'} · Tempo total: {runDuration(run.durationMs)}</p><p className="help execution-id">Identificador: {run.id}</p>{run.interruptionNote&&<p role="status" className="inline-error">{run.interruptionNote} Os tempos exibidos correspondem ao último registro salvo, sem incluir o período em que o aplicativo esteve fechado.</p>}{run.error&&<div role="alert" className="inline-error">{run.error}</div>}<ol className="execution-steps">{run.steps.map(s=><li key={s.index}><strong>{actions[s.kind]||s.kind}</strong><span>{stepStatusLabel(s.status)} · {runDuration(s.durationMs)}</span></li>)}</ol><p className="help">A tela mostra o estado confirmado pelo motor. O tempo de uma etapa inclui suas esperas. O tempo não mede a latência acústica dos dispositivos.</p>{!isTerminalRun(run.status)&&<>{run.cancelRequested||requested.includes(run.id)?<p role="status">Cancelamento solicitado. Aguardando o motor encerrar; a ação atual pode continuar até concluir ou atingir seu limite.</p>:<button disabled={!!error} onClick={()=>{setCancelError('');setCancelChoice(run)}}>Cancelar execução</button>}</>}</div>:<Empty title={selected?'Execução fora da retenção':'Escolha uma execução'}>Selecione um registro para conferir cada etapa e o erro informado pelo motor.</Empty>}</Card>
    </div>}
   </>}
  </>}
  {cancelChoice&&<Modal title="Cancelar esta execução?" onClose={()=>{if(!sending)setCancelChoice(null)}}><div className="form-pad"><p>Solicitar parada de <strong>{cancelChoice.flow}</strong>?</p><p>Ações já concluídas não são desfeitas. A ação atual pode continuar até terminar ou atingir o limite; o motor impede as próximas etapas quando recebe o pedido.</p>{cancelError&&<p role="alert" className="inline-error">{cancelError}</p>}<footer className="form-footer"><button disabled={sending} onClick={()=>setCancelChoice(null)}>Voltar</button><button disabled={sending} onClick={()=>void cancel()}>{sending?'Solicitando…':'Confirmar cancelamento'}</button></footer></div></Modal>}
 </>;
}
