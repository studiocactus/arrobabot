import {useEffect,useRef,useState,type KeyboardEvent as ReactKeyboardEvent} from 'react';
import {createPortal} from 'react-dom';
import {Ellipsis,CircleHelp} from 'lucide-react';

type Place={left:number;top?:number;bottom?:number;width:number;maxHeight:number};

/** Popover fixo no portal do body: acompanha o gatilho, fecha no clique fora e devolve o foco ao fechar. */
function usePopover(width:number){
 const [open,setOpen]=useState(false);
 const [place,setPlace]=useState<Place|null>(null);
 const trigger=useRef<HTMLButtonElement|null>(null);
 const pop=useRef<HTMLDivElement|null>(null);
 function reposition(){
  const r=trigger.current?.getBoundingClientRect();if(!r)return;
  const w=Math.min(width,window.innerWidth-16);
  const below=window.innerHeight-r.bottom-10;
  const above=r.top-10;
  const up=below<170&&above>below;
  const maxHeight=Math.max(110,(up?above:below)-6);
  const left=Math.max(8,Math.min(r.left,window.innerWidth-w-8));
  setPlace(up?{left,bottom:Math.max(6,window.innerHeight-r.top+6),width:w,maxHeight}:{left,top:r.bottom+6,width:w,maxHeight});
 }
 function close(focus=true){setOpen(false);if(focus)trigger.current?.focus()}
 function openIt(){reposition();setOpen(true)}
 useEffect(()=>{
  if(!open)return;
  reposition();
  const outside=(e:MouseEvent)=>{const t=e.target as Node;if(pop.current?.contains(t)||trigger.current?.contains(t))return;close(false)};
  const moved=()=>reposition();
  document.addEventListener('mousedown',outside,true);
  window.addEventListener('scroll',moved,true);
  window.addEventListener('resize',moved);
  return ()=>{document.removeEventListener('mousedown',outside,true);window.removeEventListener('scroll',moved,true);window.removeEventListener('resize',moved)};
 },[open]);
 function escapeClose(e:ReactKeyboardEvent){if(e.key==='Escape'||e.key==='Tab'){e.stopPropagation();e.preventDefault();close()}}
 return {open,place,trigger,pop,openIt,close,escapeClose};
}

/** Substitui os botões Subir/Descer por um menu "•••" com os limites da cadeia já refletidos nos itens. */
export function StepMenu({canUp,canDown,onMove}:{canUp:boolean;canDown:boolean;onMove:(dir:-1|1)=>void}){
 const {open,place,trigger,pop,openIt,close,escapeClose}=usePopover(200);
 const items=[{dir:-1 as const,label:'Subir',off:!canUp},{dir:1 as const,label:'Descer',off:!canDown}];
 const first=Math.max(0,items.findIndex(i=>!i.off));
 const [active,setActive]=useState(first);
 useEffect(()=>{if(open)setActive(first)},[open,first]);
 useEffect(()=>{
  if(!open)return;
  pop.current?.querySelectorAll<HTMLButtonElement>('[role="menuitem"]')[active]?.focus();
 },[open,active]);
 function onKey(e:ReactKeyboardEvent<HTMLDivElement>){
  escapeClose(e);
  if(e.key==='ArrowDown'||e.key==='ArrowUp'){
   e.stopPropagation();e.preventDefault();
   const dir=e.key==='ArrowDown'?1:-1;
   setActive(a=>{let i=a;for(let n=0;n<items.length;n++){i=(i+dir+items.length)%items.length;if(!items[i].off)return i}return a});
  }
 }
 return <>
  <button type="button" ref={trigger} aria-label="Opções da etapa" aria-haspopup="menu" aria-expanded={open} onClick={()=>open?close(false):openIt()} onKeyDown={e=>{if(open&&(e.key==='Escape'||e.key==='Tab'))escapeClose(e)}}><Ellipsis size={16}/></button>
  {open&&place&&createPortal(
   <div className="step-menu" role="menu" aria-label="Opções da etapa" ref={pop} onKeyDown={onKey} style={{left:place.left,top:place.top,bottom:place.bottom,width:place.width,maxHeight:place.maxHeight}}>
    {items.map(it=><button key={it.label} type="button" role="menuitem" disabled={it.off} onClick={()=>{close();onMove(it.dir)}}>{it.label}</button>)}
   </div>,document.body)}
 </>;
}

/** Ajuda compacta no lugar do banner: abre por clique ou teclado, fecha com Escape sem fechar o editor. */
export function FlowHelp({note}:{note?:string}){
 const {open,place,trigger,pop,openIt,close,escapeClose}=usePopover(320);
 return <>
  <button type="button" ref={trigger} className="icon-button" aria-label="Ajuda do editor de fluxos" aria-haspopup="dialog" aria-expanded={open} onClick={()=>open?close(false):openIt()} onKeyDown={e=>{if(open&&(e.key==='Escape'||e.key==='Tab'))escapeClose(e)}}>
   <CircleHelp size={17}/>
  </button>
  {open&&place&&createPortal(
   <div className="flow-help" role="dialog" aria-label="Ajuda do editor de fluxos" ref={pop} onKeyDown={escapeClose} style={{left:place.left,top:place.top,bottom:place.bottom,width:place.width,maxHeight:place.maxHeight}}>
    <p>A ordem das etapas segue as conexões entre os blocos. A posição na tela não muda a execução.</p>
    {note&&<p>{note}</p>}
   </div>,document.body)}
 </>;
}
