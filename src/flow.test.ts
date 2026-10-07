import {describe,it,expect} from 'vitest';
import {orderedActions,moveAction,chainIds,stepBounds,testVarKeys} from './flow';
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
