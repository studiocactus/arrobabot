import {describe,it,expect} from 'vitest';
import {orderedActions,moveAction,chainIds,stepBounds,testVarKeys,appendStepPlan,insertStepPlan,fullChain,type FlowNode} from './flow';
import {newAction} from './types';
const nodes=['trigger','a','b','c'].map(id=>({id,position:{x:0,y:0},data:id==='trigger'?{}:{action:{...newAction(),text:id}}}));
describe('editor de automação',()=>{
 it('serializa pela conexão, não pela posição ou ordem da lista',()=>{expect(orderedActions([nodes[3],nodes[0],nodes[2],nodes[1]],[{source:'trigger',target:'b'},{source:'b',target:'a'},{source:'a',target:'c'}]).map(a=>a.text)).toEqual(['b','a','c'])});
 it('recusa blocos desconectados',()=>expect(()=>orderedActions(nodes,[])).toThrow('Conecte todos'));
 it('recusa ramificação ambígua',()=>expect(()=>orderedActions(nodes,[{source:'trigger',target:'a'},{source:'trigger',target:'b'}])).toThrow('uma ação'));
 it('recusa ciclos',()=>expect(()=>orderedActions(nodes,[{source:'trigger',target:'a'},{source:'a',target:'b'},{source:'b',target:'a'}])).toThrow('ciclo'));
 it('move ações para cima e para baixo refazendo as conexões',()=>{
  const edges=[{source:'trigger',target:'a'},{source:'a',target:'b'},{source:'b',target:'c'}];
  const up=moveAction(nodes,edges,'b',-1);
  expect(up.edges).toEqual([{source:'trigger',target:'b'},{source:'b',target:'a'},{source:'a',target:'c'}]);
  expect(orderedActions(up.nodes,up.edges).map(a=>a.text)).toEqual(['b','a','c']);
  const down=moveAction(nodes,edges,'a',1);
  expect(orderedActions(down.nodes,down.edges).map(a=>a.text)).toEqual(['b','a','c']);
  expect(()=>moveAction(nodes,edges,'a',-1)).toThrow('Nada para mover');
  expect(()=>moveAction(nodes,edges,'c',1)).toThrow('Nada para mover');
  expect(chainIds(nodes,edges)).toEqual(['trigger','a','b','c']);
 });
 it('coleta variáveis do teste sem duplicar',()=>{
  const flow={actions:[{kind:'condition',text:'',condVar:'reward.cost'},{kind:'chat',text:'{{redeemer}} pagou {{reward.cost|number:0}} e {{redeemer}}!'}]};
  expect(testVarKeys(flow)).toEqual(['reward.cost','redeemer']);
  expect(testVarKeys({actions:[{kind:'chat',text:'sem marcador'}]})).toEqual([]);
 });
 it('limita o menu da etapa: sem Subir na primeira nem Descer na última',()=>{
  expect(stepBounds(['a'],'a')).toEqual({up:false,down:false});
  expect(stepBounds(['a','b'],'a')).toEqual({up:false,down:true});
  expect(stepBounds(['a','b'],'b')).toEqual({up:true,down:false});
  expect(stepBounds(['a','b','c'],'b')).toEqual({up:true,down:true});
  expect(stepBounds(['a','b'],'fora-da-cadeia')).toEqual({up:false,down:false});
  expect(stepBounds([],'action-0')).toEqual({up:false,down:false});
 });
});
const sizes={trigger:{w:185,h:70},a:{w:185,h:70},b:{w:185,h:70},c:{w:185,h:70}};
const layout=(pos:Record<string,{x:number;y:number}>):FlowNode[]=>['trigger','a','b','c'].map(id=>({id,position:pos[id],data:id==='trigger'?{}:{action:{...newAction(),text:id}}}));
const vertical=layout({trigger:{x:60,y:140},a:{x:60,y:230},b:{x:60,y:350},c:{x:60,y:470}});
const chainEdges=[{source:'trigger',target:'a'},{source:'a',target:'b'},{source:'b',target:'c'}];
const box=(p:{x:number;y:number},s:{w:number;h:number})=>({x1:p.x,y1:p.y,x2:p.x+s.w,y2:p.y+s.h});
const hits=(A:{x1:number;y1:number;x2:number;y2:number},B:{x1:number;y1:number;x2:number;y2:number})=>A.x1<B.x2&&B.x1<A.x2&&A.y1<B.y2&&B.y1<A.y2;
describe('adição e inserção de etapas na cadeia',()=>{
 it('adiciona ao fim ligando exatamente à última etapa, sem duplicar conexão',()=>{
  const plan=appendStepPlan(vertical,chainEdges,'n',{...sizes,n:{w:185,h:70}});
  expect(plan.edges).toEqual([...chainEdges,{source:'c',target:'n'}]);
  expect(plan.positions).toEqual({n:{x:60,y:590}});
  expect(plan.edges.filter(e=>e.source==='b'&&e.target==='c')).toHaveLength(1);
 });
 it('adiciona ao gatilho quando o fluxo ainda não tem etapa nenhuma',()=>{
  const plan=appendStepPlan([vertical[0]],[],'n');
  expect(plan.edges).toEqual([{source:'trigger',target:'n'}]);
  expect(plan.positions.n.y).toBeGreaterThan(140);
 });
 it('posiciona sem sobrepor a última etapa e segue o eixo do desenho',()=>{
  const novo={w:185,h:70};
  const fim=appendStepPlan(vertical,chainEdges,'n',{...sizes,n:novo});
  expect(hits(box(vertical[3].position,novo),box(fim.positions.n,novo))).toBe(false);
  expect(fim.positions.n.y-box(vertical[3].position,novo).y2).toBeGreaterThanOrEqual(16);
  // fila horizontal: o avanço continua para a direita, na mesma altura
  const horizontal=layout({trigger:{x:60,y:200},a:{x:340,y:200},b:{x:620,y:200},c:{x:900,y:200}});
  const dir=appendStepPlan(horizontal,chainEdges,'n',{...sizes,n:novo});
  expect(dir.positions.n).toEqual({x:1180,y:200});
 });
 it('fullChain aceita a cadeia inteira e recusa bloco solto',()=>{
  expect(fullChain(vertical,chainEdges)).toEqual(['trigger','a','b','c']);
  expect(()=>fullChain(vertical,[chainEdges[0]])).toThrow('Conecte todos os blocos');
 });
 it('recusa fluxo desconectado, em ciclo ou com ramificação, sem alterar nada',()=>{
  expect(()=>appendStepPlan(vertical,[chainEdges[0],chainEdges[1]],'n')).toThrow('Conecte todos os blocos');
  expect(()=>insertStepPlan(vertical,[chainEdges[0],chainEdges[1]],'b','n')).toThrow('Conecte todos os blocos');
  expect(()=>appendStepPlan(vertical,[chainEdges[0],chainEdges[1],{source:'b',target:'a'}],'n')).toThrow('ciclo');
  expect(()=>appendStepPlan(vertical,[...chainEdges,{source:'trigger',target:'b'}],'n')).toThrow('uma ação');
 });
 it('insere entre A e B trocando só a conexão necessária e abrindo espaço no eixo',()=>{
  const plan=insertStepPlan(vertical,chainEdges,'a','n',{...sizes,n:{w:185,h:70}});
  expect(plan.edges).toEqual([{source:'trigger',target:'a'},{source:'b',target:'c'},{source:'a',target:'n'},{source:'n',target:'b'}]);
  // gatilho e A ficam onde estavam; só o trecho seguinte desloca para abrir espaço
  expect(plan.positions).toEqual({b:{x:60,y:402},c:{x:60,y:522},n:{x:60,y:316}});
  expect(hits(box(vertical[1].position,sizes.a),box(plan.positions.n,sizes.a))).toBe(false);
  expect(hits(box(plan.positions.b,sizes.b),box(plan.positions.n,sizes.a))).toBe(false);
  expect(plan.positions.n.y-vertical[1].position.y-sizes.a.h).toBeGreaterThanOrEqual(16);
 });
 it('insere depois da última etapa como se fosse uma adição',()=>{
  const plan=insertStepPlan(vertical,chainEdges,'c','n',{...sizes,n:{w:185,h:70}});
  expect(plan.edges).toEqual([...chainEdges,{source:'c',target:'n'}]);
  expect(plan.positions).toEqual({n:{x:60,y:590}});
 });
 it('insere numa fila horizontal antiga sem tirar ninguém da linha',()=>{
  const horizontal=layout({trigger:{x:60,y:200},a:{x:340,y:200},b:{x:620,y:200},c:{x:900,y:200}});
  const plan=insertStepPlan(horizontal,chainEdges,'a','n',{...sizes,n:{w:220,h:70}});
  expect(plan.positions).toEqual({b:{x:777,y:200},c:{x:1057,y:200},n:{x:541,y:200}});
  expect(hits(box({x:340,y:200},{w:185,h:70}),box(plan.positions.n,{w:220,h:70}))).toBe(false);
  expect(hits(box(plan.positions.b,{w:185,h:70}),box(plan.positions.n,{w:220,h:70}))).toBe(false);
 });
 it('uma condição com "Pular a próxima etapa" mantém a configuração ao inserir depois dela',()=>{
  const nodes2=vertical.map(n=>n.id==='b'?{...n,data:{action:{...newAction(),kind:'condition',condVar:'reward.cost',condOp:'greater_or_equal',condValue:'5000',condFalse:'skip'}}}:n);
  const plan=insertStepPlan(nodes2,chainEdges,'b','n',{...sizes,n:{w:185,h:70}});
  const cond=nodes2.find(n=>n.id==='b')!.data.action!;
  expect(cond.condFalse).toBe('skip');
  expect(cond.condVar).toBe('reward.cost');
  expect(plan.edges).toEqual([{source:'trigger',target:'a'},{source:'a',target:'b'},{source:'b',target:'n'},{source:'n',target:'c'}]);
  // a serialização passa a ser a → b → nova → c: o "próximo" da condição é a etapa inserida
  const comNova=[...nodes2,{id:'n',position:plan.positions.n,data:{action:newAction()}}];
  expect(orderedActions(comNova,plan.edges).map(a=>a.kind)).toEqual(['chat','condition','chat','chat']);
 });
});
