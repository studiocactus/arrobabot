import {useEffect,useRef,useState} from 'react';
import {open} from '@tauri-apps/plugin-dialog';
import {api,desktop,errorText} from './api';
import {Field} from './components';
import MessageEditor from './MessageEditor';
import {OBS_OPS,OBS_TEMP,obsCompatible,obsContent,obsInput,obsItem,obsNumeric,obsProblem} from './obsCatalog';
import type {Action,Flow} from './types';

type Targets={current:string;scenes:string[];groups:string[];inputs:{inputName:string;inputKind:string}[];sources:{sourceName:string}[]};
export default function ObsActionFields({profileId,action,onChange,issue,flow,at,operations,refreshToken=0}:{profileId:string;action:Action;onChange:(a:Action)=>void;issue?:string;flow?:Flow;at?:number;operations?:string[];refreshToken?:number}){
 const op=action.obsOp||'mute',scope=action.obsScene||'',target=action.obsTarget||'';
 const [data,setData]=useState<Targets|null>(null),[filters,setFilters]=useState<string[]>([]),[error,setError]=useState(''),[filterError,setFilterError]=useState(''),[refresh,setRefresh]=useState(0),[loading,setLoading]=useState(false),[picking,setPicking]=useState(false);
 const alive=useRef(true);useEffect(()=>{alive.current=true;return()=>{alive.current=false}},[]);
 const latest=useRef(action);latest.current=action;
 useEffect(()=>{
  let current=true;setData(null);setError('');if(!desktop)return;setLoading(true);
  api<Targets>('obs.targets',{profileId,scene:scope}).then(value=>{if(!value||!Array.isArray(value.inputs)||!Array.isArray(value.sources)||!Array.isArray(value.scenes)||!Array.isArray(value.groups))throw Error('Não foi possível carregar as fontes do OBS.');if(current)setData(value)}).catch(e=>{if(current)setError(errorText(e))}).finally(()=>{if(current)setLoading(false)});
  return()=>{current=false};
 },[profileId,scope,refresh,refreshToken]);
 useEffect(()=>{
  let current=true;setFilters([]);setFilterError('');if(!desktop||!target||!op.startsWith('filter_'))return;
  api<{filters:{name:string}[]}>('obs.filters',{profileId,target}).then(value=>{if(!value||!Array.isArray(value.filters))throw Error('Não foi possível carregar os filtros.');if(current)setFilters(value.filters.map(v=>v.name))}).catch(e=>{if(current)setFilterError(errorText(e))});return()=>{current=false};
 },[profileId,target,op,refresh]);
 const names=data?(op==='scene'?data.scenes:obsInput(op)?data.inputs.map(v=>v.inputName):op.startsWith('filter_')?[...data.inputs.map(v=>v.inputName),...data.scenes,...data.groups]:data.sources.map(v=>v.sourceName).filter(name=>!obsContent(op)||obsCompatible(op,data.inputs.find(v=>v.inputName===name)?.inputKind||''))):[];
 const choices=[...new Set(names)],scopes=[...(data?.scenes||[]),...(data?.groups||[])];
 const patch=(value:Partial<Action>)=>onChange({...action,...value});
 async function pickImage(){setPicking(true);setError('');try{const path=await open({multiple:false,directory:false,filters:[{name:'Imagens',extensions:['png','jpg','jpeg','gif','bmp','webp']}]});if(alive.current&&typeof path==='string'&&latest.current.obsTarget===target&&latest.current.obsOp===op)onChange({...latest.current,text:path})}catch(e){if(alive.current)setError(errorText(e))}finally{if(alive.current)setPicking(false)}}
 return <div className="obs-action-fields">
  <Field label={flow?'Operação no OBS':'Operação'}><select value={op} onChange={e=>patch({obsOp:e.target.value,obsTarget:'',obsFilter:'',text:obsNumeric(e.target.value)?'0':'',obsDuration:0})}>{Object.entries(OBS_OPS).filter(([key])=>!operations||operations.includes(key)).map(([key,label])=><option key={key} value={key}>{label}</option>)}</select></Field>
  {(obsItem(op)||obsContent(op))&&<Field label="Onde está a fonte?" hint="Cena atual acompanha a live. Escolha uma cena ou grupo para fixar o destino."><select value={scope} onChange={e=>patch({obsScene:e.target.value,obsTarget:'',obsFilter:''})}><option value="">Cena atual da transmissão</option>{data?.scenes.map(v=><option key={'scene:'+v} value={v}>Cena: {v}</option>)}{data?.groups.map(v=><option key={'group:'+v} value={v}>Grupo: {v}</option>)}{scope&&!scopes.includes(scope)&&<option value={scope}>{scope} (salvo)</option>}</select></Field>}
  <Field label={op==='scene'?'Cena':op.startsWith('filter_')?'Fonte ou cena':obsInput(op)?op.startsWith('media_')?'Fonte de mídia':'Entrada de áudio':'Fonte do OBS'}>
   <select value={target} onChange={e=>patch({obsTarget:e.target.value,obsFilter:''})}><option value="">{loading?'Carregando fontes…':'Escolha a fonte…'}</option>{choices.map(v=><option key={v} value={v}>{v}</option>)}{target&&!choices.includes(target)&&<option value={target}>{target} (salvo; conferir no OBS)</option>}</select>
  </Field>
  <button type="button" disabled={!desktop||loading} onClick={()=>setRefresh(v=>v+1)}>{loading?'Buscando no OBS…':'Atualizar cenas e fontes'}</button>
  {error&&<p role="alert" className="inline-error">{error} Abra o OBS, ative o WebSocket e confira a conexão em OBS Studio.</p>}
  {!desktop&&<p className="help">As listas reais aparecem no aplicativo desktop com o OBS conectado.</p>}
  {data&&!choices.length&&<p role="status" className="help">Nenhuma fonte compatível nesta seleção. No OBS, use Fontes → + → {op==='set_text'?'Texto':op==='set_image'?'Imagem':op==='set_browser_url'?'Navegador':'tipo desejado'}, ou escolha outra cena/grupo. Depois atualize a lista.</p>}
  {op.startsWith('filter_')&&<><Field label="Filtro"><select value={action.obsFilter||''} onChange={e=>patch({obsFilter:e.target.value})}><option value="">Escolha o filtro…</option>{filters.map(v=><option key={v} value={v}>{v}</option>)}{action.obsFilter&&!filters.includes(action.obsFilter)&&<option value={action.obsFilter}>{action.obsFilter} (salvo)</option>}</select></Field>{filterError&&<p role="alert" className="inline-error">{filterError}</p>}<p className="help">Crie o filtro nas propriedades da fonte no OBS. O BotLive liga, desliga ou alterna esse filtro, preservando os ajustes.</p></>}
  {op==='set_text'&&<><MessageEditor profileId={profileId} label="Texto que aparecerá no OBS" value={action.text} onChange={text=>patch({text})} flow={flow} at={at}/><p className="help">Aceita fontes Texto GDI+ e FreeType. Mantém fonte, cor e tamanho; passa a usar o texto direto, em vez de ler um arquivo.</p></>}
  {op==='set_image'&&<><Field label="Arquivo de imagem"><input value={action.text} onChange={e=>patch({text:e.target.value})} placeholder="Escolha a imagem neste computador"/></Field><button type="button" disabled={!desktop||picking} onClick={()=>void pickImage()}>{picking?'Escolhendo…':'Escolher imagem…'}</button><p className="help">Troca o arquivo de uma fonte Imagem existente. Não muda a posição da fonte nem a torna visível automaticamente.</p></>}
  {op==='set_browser_url'&&<Field label="Endereço que o OBS vai abrir" hint="URL completa com http:// ou https://. Mantém tamanho e CSS da fonte; passa do modo arquivo local para endereço."><input type="url" value={action.text} placeholder="https://exemplo.com/overlay" onChange={e=>patch({text:e.target.value})}/></Field>}
  {obsNumeric(op)&&<Field label={op==='rotate'?'Rotação (graus)':'Volume (dB)'} hint={op==='rotate'?'Ângulo absoluto entre -360 e 360.':'De -100 a 30 dB.'}><input type="number" min={op==='rotate'?-360:-100} max={op==='rotate'?360:30} step="any" value={action.text} onChange={e=>patch({text:e.target.value})}/></Field>}
  {OBS_TEMP.includes(op)&&flow&&<Field label="Duração (segundos)" hint="0 = permanente. Temporário restaura o estado anterior."><input type="number" min={0} max={3600} value={action.obsDuration??0} onChange={e=>patch({obsDuration:Math.max(0,Math.round(Number(e.target.value)||0))})}/></Field>}
  {op.startsWith('media_')&&<p className="help">A fonte precisa aceitar controle de mídia. Próxima/anterior exigem uma lista compatível, como VLC.</p>}
  {issue&&<p role="alert" className="field-warn">{issue}</p>}
  <p className="help">Mudar conteúdo ou filtro afeta esta fonte em todas as cenas que a reutilizam. Mostrar, esconder, girar e espelhar afetam o item da cena ou grupo selecionado.</p>
 </div>;
}

/** Teste explícito compartilhado: nunca salva, ativa o fluxo ou confirma no chat. */
export function ObsActionTest({profileId,action,label='Testar efeito no OBS',disabled=false,onSuccess}:{profileId:string;action:Action;label?:string;disabled?:boolean;onSuccess?:()=>void}){
 const [sample,setSample]=useState(''),[busy,setBusy]=useState(false),[result,setResult]=useState('');
 const running=useRef(false),alive=useRef(true),signature=JSON.stringify([profileId,action,sample]);const current=useRef(signature);current.current=signature;
 useEffect(()=>{alive.current=true;return()=>{alive.current=false}},[]);useEffect(()=>{setResult('')},[signature]);
 const templated=obsContent(action.obsOp||'')&&action.text.includes('{{');
 async function test(){if(running.current)return;running.current=true;setBusy(true);setResult('');const started=signature;try{
  const text=templated?sample:action.text,problem=obsProblem({...action,text});if(problem)throw Error(problem);
  const done=await api<string>('obs.execute',{profileId,op:action.obsOp||'mute',target:action.obsTarget||'',scene:action.obsScene||'',filter:action.obsFilter||'',content:text,num:obsNumeric(action.obsOp||'')?Number(text):0,secs:action.obsDuration||0});
  if(alive.current&&current.current===started){setResult(String(done));onSuccess?.()}
 }catch(e){if(alive.current&&current.current===started)setResult('Falhou: '+errorText(e))}finally{running.current=false;if(alive.current)setBusy(false)}}
 return <div className="obs-action-test">{templated&&<Field label="Conteúdo de exemplo para testar no OBS" hint="Substitui o conteúdo variável somente neste teste. Não muda a receita salva."><input value={sample} onChange={e=>setSample(e.target.value)}/></Field>}<p className="help">Este teste altera a fonte no OBS real imediatamente. Não salva a configuração nem envia confirmação no chat.</p><button type="button" disabled={!desktop||disabled||busy||(templated&&!sample.trim())} onClick={()=>void test()}>{busy?'Aguardando o OBS…':label}</button>{result&&<p role="status" className="help">{result}</p>}</div>;
}
