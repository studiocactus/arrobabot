import {createElement,useEffect,useRef,useState,type CSSProperties,type ReactNode} from 'react';
import {
 designVars,ensureFont,ensureOverlayStyle,goalPercent,keyTexts,overlayTree,
 type GoalModel,type OverlayDesign,type Spec,
} from './overlayDesign';

export type StageBg='checker'|'dark'|'light';

/** Converte a árvore do overlay em elementos React (mesma estrutura do overlay real). */
function node(n:Spec,index:number):ReactNode{
 const attrs:Record<string,unknown>={key:(n.c||'n')+index};
 if(n.c)attrs.className=n.c;
 if(n.k)attrs['data-k']=n.k;
 if(n.a)Object.assign(attrs,n.a);
 const tag=(n.tag||'div') as unknown;
 if(n.t!==undefined)return createElement(tag as 'div',attrs as never,n.t);
 return createElement(tag as 'div',attrs as never,(n.ch||[]).map(node));
}

type Props={
 design:OverlayDesign;
 model:GoalModel;
 bg:StageBg;
 /** Cada incremento pulsa a barra (simulação de doação na prévia). */
 gainKey?:number;
 /** Cada incremento dispara a celebração de 100%. */
 celebrateKey?:number;
};

/**
 * Prévia em miniatura do quadro 1920×1080 do OBS: o mesmo CSS e a mesma árvore
 * do overlay servido na porta local, escalado para caber na tela.
 */
export default function OverlayPreview({design,model,bg,gainKey=0,celebrateKey=0}:Props){
 const frame=useRef<HTMLDivElement>(null);
 const [scale,setScale]=useState(0.3);
 const [gain,setGain]=useState(false);
 const [celebrate,setCelebrate]=useState(false);

 useEffect(()=>{
  ensureOverlayStyle();ensureFont(design.font);
  const el=frame.current;if(!el)return;
  const fit=()=>setScale(Math.max(0.05,el.clientWidth/1920));
  fit();
  const ro=new ResizeObserver(fit);ro.observe(el);
  return()=>ro.disconnect();
 },[design.font]);

 useEffect(()=>{
  if(gainKey<=0||!design.pulse)return;
  setGain(true);const t=setTimeout(()=>setGain(false),800);
  return()=>clearTimeout(t);
 },[gainKey,design.pulse]);

 useEffect(()=>{
  if(celebrateKey<=0||!design.celebrate)return;
  setCelebrate(true);const t=setTimeout(()=>setCelebrate(false),2400);
  return()=>clearTimeout(t);
 },[celebrateKey,design.celebrate]);

 const spec=overlayTree(design,keyTexts(model));
 const cls=(spec.c||'bl-ov')+(model.hidden?' is-off':'')+(gain?' is-gain':'')+(celebrate?' is-celebrate':'');
 const style={...designVars(design),'--p':String(goalPercent(model))} as CSSProperties;

 return <div className="lp-frame" data-bg={bg} ref={frame}>
  <div className="lp-canvas" style={{transform:`scale(${scale})`}}>
   <div className={cls} style={style}>{(spec.ch||[]).map(node)}</div>
  </div>
 </div>;
}
