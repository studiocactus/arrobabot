import {useEffect,useRef,useState} from 'react';
import {Play} from 'lucide-react';
import {api,desktop} from './api';
import {friendlyError,friendlyMessage} from './errors';
import {actions,triggers,type Flow} from './types';
import {Modal,Field} from './components';
import {testVarKeys} from './flow';
import {variableLabel} from './variableCatalog';
import {stepLabel} from './stepLabel';
import {type TestRun,isTerminalRun,runStatusLabel,stepStatusLabel,stepIcon,conditionNote,initialSamples} from './testRun';

type Err={message:string;detail:string};
const asErr=(e:unknown):Err=>friendlyError(e);
/** Devolve o foco ao botão que abriu o teste, sem tocar no editor atrás. */
export function focusTestButton(){requestAnimationFrame(()=>document.querySelector<HTMLElement>('[data-test-flow]')?.focus())}
function ErrorNote({title,error}:{title:string;error:Err}){
 return <div role="alert" className="inline-error"><strong>{title}</strong> {error.message}{error.detail&&error.detail!==error.message&&<details><summary>Detalhes técnicos</summary><code>{error.detail}</code></details>}</div>;
}

/** Testar fluxo: prepara os valores, executa uma vez, acompanha etapa a etapa e cancela.
 * Abrir não salva nem executa; começar e repetir dependem sempre de um clique. */
export default function TestFlowDialog({flow,profileId,onClose}:{flow:Flow;profileId:string;onClose:()=>void}){
 const keys=testVarKeys(flow);
 const [vals,setVals]=useState<Record<string,string>>(()=>initialSamples(keys));
 const [exec,setExec]=useState<string|null>(null);
 const [run,setRun]=useState<TestRun|null>(null);
 const [starting,setStarting]=useState(false);
 const [startError,setStartError]=useState<Err|null>(null);
 const [trackError,setTrackError]=useState<Err|null>(null);
 const [cancelState,setCancelState]=useState<'idle'|'pending'|'confirmed'>('idle');
 const [cancelError,setCancelError]=useState<Err|null>(null);
 const [cancelNote,setCancelNote]=useState('');
 const [closeChoice,setCloseChoice]=useState(false);
 const [closeIntent,setCloseIntent]=useState(false);
 /** Trava síncrona: cliques rápidos não passam duas vezes pela mesma abertura. */
 const startLock=useRef(false);
 const cancelLock=useRef(false);
 const terminal=!!run&&isTerminalRun(run.status);
 const active=starting||!!exec&&!terminal;

 async function begin(repeat=false){
  if(startLock.current)return;
  if(exec&&!repeat)return;
  startLock.current=true;
  setStarting(true);setStartError(null);
  if(repeat){setRun(null);setCancelState('idle');setCancelError(null);setCancelNote('');setTrackError(null)}
  try{
   const testVars:Record<string,string>={};keys.forEach(k=>{testVars[k]=vals[k]||''});
   const r=await api<{executionId:string}>('flow.test',{flow,testVars});
   setRun(null);setExec(r.executionId);
  }catch(e){setStartError(asErr(e))}
  finally{startLock.current=false;setStarting(false)}
 }
 /** Acompanha só a execução devolvida pelo início; falha aqui nunca começa outra. */
 useEffect(()=>{
  if(!exec||terminal)return;
  let stop=false;
  const tick=async()=>{
   try{
    const list=await api<TestRun[]>('flow.runs',{profileId});
    if(stop)return;
    const found=list.find(r=>r.id===exec);
    if(found)setRun(found);
    setTrackError(null);
   }catch(e){if(!stop)setTrackError(asErr(e))}
  };
  void tick();
  const timer=window.setInterval(()=>{void tick()},500);
  return ()=>{stop=true;window.clearInterval(timer)};
 },[exec,terminal,profileId]);
 /** Confirmação do cancelamento vem do estado final da execução, nunca do clique. */
 useEffect(()=>{
  if(cancelState!=='pending'||!terminal||!run)return;
  if(run.status==='CANCELLED'){setCancelState('confirmed')}
  else{setCancelState('idle');setCancelNote('A execução terminou como '+runStatusLabel(run.status).toLowerCase()+', antes do cancelamento.')}
  if(closeIntent){setCloseIntent(false);onClose()}
 },[cancelState,terminal,run,closeIntent,onClose]);
 async function cancel(closeAfter:boolean){
  if(!exec||cancelLock.current)return;
  cancelLock.current=true;
  setCancelError(null);setCancelNote('');
  if(closeAfter){setCloseChoice(false);setCloseIntent(true)}
  setCancelState('pending');
  try{await api('flow.cancel',{profileId,executionId:exec})}
  catch(e){
   const err=asErr(e);setCancelState('idle');setCloseIntent(false);
   if(/não encontrada|já terminada/i.test(err.message)){
    setCancelNote('A execução já tinha terminado; nada havia para cancelar.');
    setCloseChoice(false);
    if(closeAfter)onClose();
   }else{
    setCancelError(err);
    if(closeAfter)setCloseChoice(true);
   }
  }finally{cancelLock.current=false}
 }
 function requestClose(){
  if(closeChoice)return;
  if(active){setCloseChoice(true);return}
  onClose();
 }
 const trigger=triggers[flow.trigger.kind]||flow.trigger.kind;
 const summary=<div className="test-summary">
  <span className="eyebrow">FLUXO DE TESTE</span>
  <strong>{flow.name}</strong>
  <span className="help">Gatilho: {trigger}{flow.trigger.pattern?' · '+flow.trigger.pattern:''}</span>
  <span className="help">{flow.actions.length===1?'1 etapa':flow.actions.length+' etapas'}</span>
 </div>;
 const preparation=<>
  <p className="help">Abrir este painel não salva o fluxo e não executa nada. Confira os valores e escolha quando começar.</p>
  {!desktop&&<p className="help">No navegador o teste não roda: abra o aplicativo desktop para acompanhar uma execução de verdade.</p>}
  <div className="eyebrow">O QUE O TESTE FAZ</div>
  <p className="help">O teste roda a sequência neste computador, com um evento de teste e os valores abaixo. Espera, condições e variáveis locais acontecem de verdade. Não publica no chat, não altera o OBS, não atualiza o overlay, não chama webhooks, não pune e não grava memórias, pontos ou contadores: essas etapas aparecem como Puladas.</p>
  {keys.length?<><div className="eyebrow">VALORES DE EXEMPLO ({keys.length})</div>{keys.map(k=>{const label=variableLabel(k);return <Field key={k} label={label} hint={label===k?undefined:'Código: '+k}><input value={vals[k]||''} onChange={e=>setVals({...vals,[k]:e.target.value})}/></Field>})}</>:<p className="help">Este fluxo não cita variáveis; o teste roda direto.</p>}
  <footer className="form-footer"><span className="help">{desktop?'Uma execução por clique.':'Prévia sem execução'}</span><button type="button" className="primary" disabled={starting} onClick={()=>void begin()}><Play size={16}/>{starting?'Iniciando…':'Iniciar teste'}</button></footer>
 </>;
 const tracking=<>
  <p className="test-run-line"><strong>Execução {exec?.slice(0,8)}</strong> · {runStatusLabel(run?.status||'RUNNING')}</p>
  {!run&&<p className="help">Aguardando a execução aparecer no acompanhamento…</p>}
  {run&&<>
   <div className="test-steps">{run.steps.map(st=>{
    const a=flow.actions[st.index];
    const label=a?stepLabel(a):(actions[st.kind]||st.kind);
    const note=conditionNote(st.status,a?.condFalse);
    const failed=st.status==='FAILED'&&!!run.error;
    return <div className="test-step-block" key={st.index}>
     <div className="list-row test-step"><div><span>{st.index+1}. {label}</span>{note&&<small>{note}</small>}</div><strong>{stepIcon(st.status)} {stepStatusLabel(st.status)}</strong></div>
     {failed&&<ErrorNote title="Esta etapa falhou:" error={friendlyError(run.error as string)}/>}
    </div>;
   })}</div>
   {run.error&&!run.steps.some(s=>s.status==='FAILED')&&<ErrorNote title="A execução falhou:" error={friendlyError(run.error)}/>}
  </>}
  {cancelNote&&<p className="help">{cancelNote}</p>}
  {cancelState==='confirmed'&&<p className="help" role="status">Execução cancelada. Nada mais roda nesta execução.</p>}
  {!terminal&&<footer className="form-footer"><span className="help">{cancelState==='pending'?'Cancelando… O aviso de cancelado só aparece quando a execução confirmar.':'Só esta execução é cancelada; o fluxo continua salvo.'}</span><button type="button" disabled={cancelState==='pending'||!exec} onClick={()=>void cancel(false)}>{cancelState==='pending'?'Cancelando…':'Cancelar execução'}</button></footer>}
  {terminal&&<footer className="form-footer"><span className="help">Repetir só começa uma nova execução, quando você pedir.</span><button type="button" disabled={starting} onClick={()=>void begin(true)}>{starting?'Iniciando…':'Executar novamente'}</button><button type="button" className="primary" onClick={onClose}>Fechar teste</button></footer>}
 </>;
 const choice=<div className="test-choice" role="alertdialog" aria-label="Teste ainda em andamento">
  <p><strong>O teste ainda está rodando.</strong></p>
  <p className="help">Fechar agora esconde o acompanhamento desta execução. O editor atrás continua exatamente como está.</p>
  {!exec&&<p className="help">A execução ainda está sendo iniciada: cancele depois que ela aparecer.</p>}
  <footer className="form-footer"><button type="button" onClick={()=>setCloseChoice(false)}>Continuar acompanhando</button><button type="button" className="danger" disabled={!exec||cancelState==='pending'} onClick={()=>void cancel(true)}>{cancelState==='pending'?'Cancelando…':'Cancelar e fechar'}</button></footer>
 </div>;
 return <Modal title={'Testar '+flow.name} onClose={requestClose}><div className="form-pad test-flow">
  {startError&&<ErrorNote title="Não foi possível iniciar o teste." error={startError}/>}
  {startError&&<p className="help">Nenhum teste foi executado e os valores continuam aqui. Escolha Iniciar teste de novo.</p>}
  {trackError&&<ErrorNote title="Não foi possível acompanhar a execução:" error={trackError}/>}
  {cancelError&&<ErrorNote title="Não foi possível cancelar:" error={cancelError}/>}
  {summary}
  {closeChoice?choice:(exec?tracking:preparation)}
 </div></Modal>;
}
