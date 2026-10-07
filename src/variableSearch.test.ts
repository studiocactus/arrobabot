import {describe,it,expect} from 'vitest';
import {variableCatalog} from './variableCatalog';
import {normalizeQuery,filterVariables,groupVariables,searchVariables} from './variableSearch';

describe('busca do seletor de variáveis',()=>{
 it('encontra "Custo da recompensa" ao buscar "custo"',()=>{
  const hits=filterVariables('custo');
  expect(hits.some(v=>v.key==='reward.cost'&&v.label.includes('Custo da recompensa'))).toBe(true);
 });
 it('encontra a mesma variável pelo identificador reward.cost',()=>{
  expect(filterVariables('reward.cost').map(v=>v.key)).toEqual(['reward.cost']);
 });
 it('encontra pela categoria do catálogo',()=>{
  const hits=filterVariables('Eventos');
  expect(hits.length).toBeGreaterThan(1);
  expect(hits.every(v=>v.group==='Eventos')).toBe(true);
 });
 it('ignora maiúscula e acento',()=>{
  expect(normalizeQuery('Custo da Recompensa')).toBe('custo da recompensa');
  expect(filterVariables('inscricao').map(v=>v.key)).toContain('subscription.message');
 });
 it('busca também pelo exemplo quando o catálogo traz um',()=>{
  expect(filterVariables('VALORANT').map(v=>v.key)).toContain('local.twitchGame');
 });
 it('sem resultados devolve lista vazia',()=>{
  expect(filterVariables('zzznenhuma')).toEqual([]);
 });
 it('mantém as categorias na ordem do catálogo e nada se perde',()=>{
  const groups=groupVariables(filterVariables(''));
  const ordem=[...new Set(variableCatalog.map(v=>v.group))];
  expect(groups.map(g=>g.group)).toEqual(ordem);
  expect(groups.reduce((n,g)=>n+g.items.length,0)).toBe(variableCatalog.length);
 });
 it('valor antigo fora do catálogo continua visível',()=>{
  expect(searchVariables('','local.legado').orphan?.key).toBe('local.legado');
  expect(searchVariables('legado','local.legado').orphan?.key).toBe('local.legado');
  expect(searchVariables('custo','local.legado').orphan).toBeNull();
  expect(searchVariables('','reward.cost').orphan).toBeNull();
 });
});
