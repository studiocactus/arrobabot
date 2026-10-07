import type {Action} from './types';
export type FlowNode={id:string;data:{action?:Action;[key:string]:unknown};position:{x:number;y:number}};
export type FlowEdge={source:string;target:string};
export function chainIds(nodes:FlowNode[],edges:FlowEdge[]):string[]{
 const ids=new Set(nodes.map(n=>n.id));if(!ids.has('trigger'))throw Error('O fluxo precisa de um gatilho.');
 const seen=new Set<string>();const order:string[]=[];let current='trigger';
 while(true){
 if(seen.has(current))throw Error('Remova o ciclo do fluxo.');seen.add(current);order.push(current);
 const outgoing=edges.filter(e=>e.source===current);if(outgoing.length>1)throw Error('Conecte uma ação por vez; use a condição de cada ação para execução condicional.');
 if(!outgoing.length)break;current=outgoing[0].target;
 }
 return order;
}
/** Limites do menu da etapa: sem Subir na primeira posição nem Descer na última (o gatilho fica fora da cadeia). */
export function stepBounds(chain:string[],id:string):{up:boolean;down:boolean}{const at=chain.indexOf(id);return {up:at>0,down:at>=0&&at<chain.length-1}}
export type StepPlan={positions:Record<string,{x:number;y:number}>;edges:FlowEdge[]};
type Box={w:number;h:number};type Sizes=Record<string,Box>;
/** Folga mínima entre blocos ao adicionar ou inserir, em unidades do canvas. */
const MARGIN=16;
/** Tamanho estimado da etapa nova, que ainda não existe no canvas para ser medida. */
const ADDED:Box={w:220,h:80};
const boxAt=(s:Sizes,id:string)=>s[id]||{w:185,h:70};
/** Cadeia completa exigida para mexer na estrutura: gatilho presente, sem ciclo, sem ramificação e nenhum bloco solto. */
export function fullChain(nodes:FlowNode[],edges:FlowEdge[]):string[]{
 const order=chainIds(nodes,edges);
 if(nodes.some(n=>n.data.action&&!order.includes(n.id)))throw Error('Conecte todos os blocos em uma única sequência começando no gatilho antes de adicionar ou inserir etapas.');
 return order;
}
function nodePos(nodes:FlowNode[],id:string){const n=nodes.find(x=>x.id===id);if(!n)throw Error('Bloco ausente do fluxo.');return n.position}
function axisOf(dx:number,dy:number){const vertical=Math.abs(dy)>=Math.abs(dx);return vertical?{vertical:true,sign:dy>=0?1:-1,mag:Math.abs(dy)}:{vertical:false,sign:dx>=0?1:-1,mag:Math.abs(dx)}}
/** Avanço depois de `fromId`: segue a direção da conexão anterior (ou empilha para baixo no início) com espaço para não cobrir o vizinho. */
function advancePos(nodes:FlowNode[],order:string[],fromId:string,sizes:Sizes):{x:number;y:number}{
 const at=order.indexOf(fromId);
 const prev=at>0?nodes.find(n=>n.id===order[at-1]):undefined;
 const anchor=nodePos(nodes,fromId);
 let dx=prev?anchor.x-prev.position.x:0;
 let dy=prev?anchor.y-prev.position.y:120;
 if(prev&&dx===0&&dy===0){dx=0;dy=120}
 const axis=axisOf(dx,dy);
 const size=boxAt(sizes,fromId);
 const mag=Math.max(axis.mag,(axis.vertical?size.h:size.w)+MARGIN);
 return axis.vertical?{x:anchor.x,y:anchor.y+axis.sign*mag}:{x:anchor.x+axis.sign*mag,y:anchor.y};
}
/** Adiciona uma etapa ao fim da cadeia válida e a liga à última etapa (ou ao gatilho, quando ainda não há etapa nenhuma). */
export function appendStepPlan(nodes:FlowNode[],edges:FlowEdge[],newId:string,sizes:Sizes={}):StepPlan{
 const order=fullChain(nodes,edges);
 const last=order[order.length-1];
 return {positions:{[newId]:advancePos(nodes,order,last,sizes)},edges:[...edges,{source:last,target:newId}]};
}
/** Insere uma etapa depois de `afterId` (A → nova → B): troca só a conexão necessária e abre espaço apenas no eixo do desenho. */
export function insertStepPlan(nodes:FlowNode[],edges:FlowEdge[],afterId:string,newId:string,sizes:Sizes={}):StepPlan{
 const order=fullChain(nodes,edges);
 const at=order.indexOf(afterId);
 if(at<0)throw Error('Esta etapa não está na cadeia do fluxo.');
 const nextId=order[at+1];
 const dropped=nextId?edges.filter(e=>e.source===afterId&&e.target===nextId):[];
 const kept=edges.filter(e=>!dropped.includes(e));
 const made:FlowEdge[]=[{source:afterId,target:newId},...(nextId?[{source:newId,target:nextId}]:[])];
 if(!nextId)return {positions:{[newId]:advancePos(nodes,order,afterId,sizes)},edges:[...kept,...made]};
 const anchor=nodePos(nodes,afterId);
 const next=nodePos(nodes,nextId);
 const axis=axisOf(next.x-anchor.x,next.y-anchor.y);
 const sa=boxAt(sizes,afterId);
 const sn=boxAt(sizes,newId);
 const span=(axis.vertical?sa.h+sn.h:sa.w+sn.w)+2*MARGIN;
 const gap=axis.vertical?Math.abs(next.y-anchor.y):Math.abs(next.x-anchor.x);
 const shift=Math.max(0,span-gap);
 const ux=axis.vertical?0:axis.sign;
 const uy=axis.vertical?axis.sign:0;
 const positions:StepPlan['positions']={};
 // só o trecho seguinte desloca, para preservar o resto do desenho (inclusive filas horizontais antigas)
 if(shift>0)for(const id of order.slice(at+1)){const p=nodePos(nodes,id);positions[id]={x:p.x+ux*shift,y:p.y+uy*shift}}
 const end=(axis.vertical?next.y:next.x)+(axis.vertical?uy:ux)*shift;
 const start=axis.vertical?anchor.y:anchor.x;
 const lo=start+(axis.vertical?sa.h:sa.w)+MARGIN;
 const hi=end-(axis.vertical?sn.h:sn.w)-MARGIN;
 const mid=(lo+hi)/2;
 positions[newId]=axis.vertical?{x:(anchor.x+next.x)/2,y:mid}:{x:mid,y:(anchor.y+next.y)/2};
 return {positions,edges:[...kept,...made]};
}
export function moveAction(nodes:FlowNode[],edges:FlowEdge[],id:string,dir:-1|1):{nodes:FlowNode[];edges:FlowEdge[]}{
 const order=chainIds(nodes,edges).filter(x=>x!=='trigger');
 const at=order.indexOf(id);const to=at+dir;
 if(at<0||to<0||to>=order.length)throw Error('Nada para mover.');
 const next=[...order];const [moved]=next.splice(at,1);next.splice(to,0,moved);
 const other=order[to];
 const nodes2=nodes.map(n=>{
  if(n.id===moved){const o=nodes.find(x=>x.id===other);return o?{...n,position:{...o.position}}:n}
  if(n.id===other){const o=nodes.find(x=>x.id===moved);return o?{...n,position:{...o.position}}:n}
  return n;
 });
 const inChain=(e:FlowEdge)=>(e.source==='trigger'||order.includes(e.source))&&order.includes(e.target);
 const others=edges.filter(e=>!inChain(e));
 const rebuilt:FlowEdge[]=[];let prev='trigger';
 for(const nid of next){rebuilt.push({source:prev,target:nid});prev=nid;}
 return {nodes:nodes2,edges:others.concat(rebuilt)};
}
export function testVarKeys(flow:{actions:{kind:string;text:string;condVar?:string}[]}):string[]{
 const out:string[]=[];
 const push=(k:string)=>{const key=k.split('|')[0].trim();if(key&&!out.includes(key))out.push(key)};
 for(const a of flow.actions){
  if(a.condVar)push(a.condVar);
  for(const m of (a.text||'').matchAll(/\{\{([^{}]+)\}\}/g))push(m[1]);
 }
 return out.slice(0,10);
}
export function orderedActions(nodes:FlowNode[],edges:FlowEdge[]):Action[]{
 const ids=new Set(nodes.map(n=>n.id));if(!ids.has('trigger'))throw Error('O fluxo precisa de um gatilho.');
 for(const e of edges)if(!ids.has(e.source)||!ids.has(e.target))throw Error('Conexão aponta para um bloco ausente.');
 if(edges.some(e=>e.target==='trigger'))throw Error('O gatilho deve ser o primeiro bloco.');
 const output:Action[]=[];const seen=new Set<string>();let current='trigger';
 while(true){
 if(seen.has(current))throw Error('Remova o ciclo do fluxo.');seen.add(current);
 const outgoing=edges.filter(e=>e.source===current);if(outgoing.length>1)throw Error('Conecte uma ação por vez; use a condição de cada ação para execução condicional.');
 if(current!=='trigger'){const node=nodes.find(n=>n.id===current);if(!node?.data.action)throw Error('Ação incompleta.');output.push(node.data.action)}
 if(!outgoing.length)break;current=outgoing[0].target;
 }
 if(seen.size!==nodes.length)throw Error('Conecte todos os blocos ao gatilho antes de salvar.');
 if(!output.length)throw Error('Adicione pelo menos uma ação.');
 return output;
}
