import {describe,it,expect} from 'vitest';
import {orderedActions,moveAction,chainIds} from './flow';
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
});
