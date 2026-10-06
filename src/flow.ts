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
