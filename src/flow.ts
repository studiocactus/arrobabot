import type {Action} from './types';
export type FlowNode={id:string;data:{action?:Action;[key:string]:unknown};position:{x:number;y:number}};
export type FlowEdge={source:string;target:string};
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
