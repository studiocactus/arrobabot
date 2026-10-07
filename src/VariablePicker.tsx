import {useEffect,useRef,useState,type KeyboardEvent as ReactKeyboardEvent} from 'react';
import {createPortal} from 'react-dom';
import {Search} from 'lucide-react';
import {variableLabel,type VariableChoice} from './variableCatalog';
import {searchVariables} from './variableSearch';

type Place={left:number;top?:number;bottom?:number;width:number;maxHeight:number};
type Row={head?:string;item?:VariableChoice;index:number};

/**
 * Seletor de variáveis com busca, usado na etapa Condição.
 * Abre num popover fora do painel (nada de cortar em overflow), busca por nome,
 * categoria ou identificador técnico e preserva qualquer valor salvo que não
 * exista mais no catálogo. Escape fecha só o seletor — o editor continua aberto.
 */
export default function VariablePicker({value,onChange}:{value:string;onChange:(key:string)=>void}){
 const [open,setOpen]=useState(false);
 const [query,setQuery]=useState('');
 const [active,setActive]=useState(0);
 const [place,setPlace]=useState<Place|null>(null);
 const trigger=useRef<HTMLButtonElement>(null);
 const pop=useRef<HTMLDivElement>(null);
 const input=useRef<HTMLInputElement>(null);

 const {orphan,groups}=searchVariables(query,value);
 const options:[...VariableChoice[]]=[];
 if(orphan)options.push(orphan);
 for(const g of groups)options.push(...g.items);
 const rows:Row[]=[];
 let next=0;
 if(orphan)rows.push({item:orphan,index:next++});
 for(const g of groups){rows.push({head:g.group,index:-1});for(const v of g.items)rows.push({item:v,index:next++})}
 const current=Math.min(active,Math.max(0,options.length-1));

 function reposition(){
  const r=trigger.current?.getBoundingClientRect();
  if(!r)return;
  const width=Math.min(320,window.innerWidth-16);
  const below=window.innerHeight-r.bottom-12;
  const above=r.top-12;
  const up=below<240&&above>below;
  const maxHeight=Math.max(170,(up?above:below)-8);
  const left=Math.max(8,Math.min(r.left,window.innerWidth-width-8));
  setPlace(up?{left,bottom:window.innerHeight-r.top+6,width,maxHeight}:{left,top:r.bottom+6,width,maxHeight});
 }
 function close(focusBack=true){
  setOpen(false);setQuery('');setActive(0);
  if(focusBack)trigger.current?.focus();
 }
 function choose(key:string){onChange(key);close()}

 useEffect(()=>{
  if(!open)return;
  reposition();
  input.current?.focus();
  const outside=(e:MouseEvent)=>{
   const t=e.target as Node;
   if(pop.current?.contains(t))return;
   if(trigger.current?.parentElement?.contains(t))return;
   close(false);
  };
  const moved=()=>reposition();
  document.addEventListener('mousedown',outside,true);
  window.addEventListener('scroll',moved,true);
  window.addEventListener('resize',moved);
  return ()=>{document.removeEventListener('mousedown',outside,true);window.removeEventListener('scroll',moved,true);window.removeEventListener('resize',moved)};
 },[open]);

 useEffect(()=>{
  if(!open)return;
  const el=pop.current?.querySelector('[data-active="true"]') as HTMLElement|null;
  el?.scrollIntoView({block:'nearest'});
 },[current,open]);

 function onKey(e:ReactKeyboardEvent){
  if(e.key==='Escape'){e.stopPropagation();e.preventDefault();close();return}
  if(e.key==='Tab'){e.preventDefault();close();return}
  if(e.target!==input.current)return;
  if(e.key==='ArrowDown'){e.preventDefault();setActive(i=>Math.min(i+1,options.length-1))}
  else if(e.key==='ArrowUp'){e.preventDefault();setActive(i=>Math.max(i-1,0))}
  else if(e.key==='Enter'){e.preventDefault();const item=options[current];if(item)choose(item.key)}
 }

 const display=value?variableLabel(value):'Escolha…';
 return <>
  <button type="button" ref={trigger} className={'var-picker'+(value?'':' unset')} aria-label={'Variável: '+display} aria-haspopup="listbox" aria-expanded={open} onClick={()=>{if(open)close();else{reposition();setOpen(true);setQuery('');setActive(0)}}}>
   <span className="var-picker-value">{display}</span><Search size={14}/>
  </button>
  {open&&place&&createPortal(<div className="var-pop" ref={pop} style={{left:place.left,top:place.top,bottom:place.bottom,width:place.width,maxHeight:place.maxHeight}} onKeyDown={onKey}>
   <input ref={input} type="search" value={query} aria-label="Buscar variável" placeholder="Nome, categoria ou código" onChange={e=>{setQuery(e.target.value);setActive(0)}}/>
   <div className="var-list" role="listbox" aria-label="Variáveis">
    {rows.map(r=>r.head?<p className="var-group" key={'g'+r.head}>{r.head}</p>:r.item?<button key={r.item.key} type="button" role="option" className="var-opt" data-active={r.index===current} aria-selected={r.index===current} onClick={()=>choose(r.item!.key)}>
     <strong>{r.item.label}</strong><code>{r.item.key}</code>
     {!orphan&&r.item.example&&<small>{r.item.example}</small>}
     {orphan&&r.item.key===orphan.key&&<small>Valor salvo nesta etapa · fora do catálogo</small>}
    </button>:null)}
    {!options.length&&<p className="var-empty">Nenhuma variável encontrada para “{query}”. Tente outro nome, a categoria ou o código — ex.: reward.cost.</p>}
   </div>
  </div>,document.body)}
 </>;
}
