import {describe,it,expect} from 'vitest';
import {type Action} from './types';
import {stepLabel} from './stepLabel';
import {variableLabel} from './variableCatalog';
import {testVarKeys} from './flow';
import {isTerminalRun,runStatusLabel,stepStatusLabel,stepIcon,conditionNote,initialSamples,TERMINAL_RUNS} from './testRun';

const action=(a:Partial<Action>):Action=>({kind:'chat',text:'',target:'',value:0,condition:'',aiAnchor:'',aiKnowledge:'',aiLength:'',aiStyle:'',aiNoRepeat:null,...a});

describe('estados do acompanhamento',()=>{
 it('para o polling em todo estado final, inclusive parada por condição',()=>{
  expect(TERMINAL_RUNS).toEqual(['COMPLETED','FAILED','CANCELLED','STOPPED_BY_CONDITION']);
  for(const s of TERMINAL_RUNS)expect(isTerminalRun(s),s).toBe(true);
  for(const s of ['RUNNING','WAITING',''])expect(isTerminalRun(s),s).toBe(false);
 });
 it('traduz estados de execução e de etapa para português',()=>{
  expect(runStatusLabel('WAITING')).toBe('Em espera');
  expect(runStatusLabel('COMPLETED')).toBe('Concluída');
  expect(runStatusLabel('CANCELLED')).toBe('Cancelada');
  expect(runStatusLabel('STOPPED_BY_CONDITION')).toBe('Parada por condição');
  expect(runStatusLabel('ALGO_NOVO'),'estado desconhecido passa adentro').toBe('ALGO_NOVO');
  expect(stepStatusLabel('PENDING')).toBe('Aguardando execução');
  expect(stepStatusLabel('SKIPPED')).toBe('Pulada');
  expect(stepStatusLabel('FAILED')).toBe('Falhou');
  expect(stepStatusLabel('CANCELLED')).toBe('Cancelada');
  expect(stepStatusLabel('FALSE')).toBe('Condição falsa');
  expect(stepIcon('SKIPPED')).toBe('–');
  expect(stepIcon('NADA')).toBe('?');
 });
});

describe('condição falsa não é falha técnica',()=>{
 it('diz se o fluxo parou ou pulou a próxima etapa',()=>{
  expect(conditionNote('FALSE','stop')).toContain('parou aqui');
  expect(conditionNote('FALSE','skip')).toContain('próxima etapa foi pulada');
  expect(conditionNote('FALSE','stop')).toContain('Não é uma falha');
  expect(conditionNote('TRUE')).toContain('continuou');
  expect(conditionNote('SUCCESS')).toBe('');
  expect(conditionNote('FAILED')).toBe('');
 });
});

describe('rótulos compartilhados com o editor',()=>{
 it('usa o mesmo texto do bloco no canvas',()=>{
  expect(stepLabel(action({kind:'condition',condVar:'reward.cost',condOp:'greater_than',condValue:'5000'}))).toBe('◆ Custo da recompensa (ponto) > 5000');
  expect(stepLabel(action({kind:'condition',condVar:'chave.desconhecida'}))).toContain('chave.desconhecida');
  expect(stepLabel(action({kind:'wait',value:4500}))).toBe('◷ 4.5 s');
  expect(stepLabel(action({kind:'wait',value:180000}))).toBe('◷ 3 min');
  expect(stepLabel(action({kind:'punish',punish:'timeout'}))).toBe('🔨 Silenciar por um tempo');
  expect(stepLabel(action({kind:'obs',obsOp:'scene',obsTarget:'BRB'}))).toBe('◉ Trocar cena · BRB');
  expect(stepLabel(action({kind:'fluxo.futuro'}))).toBe('fluxo.futuro');
 });
 it('preserva o código da variável quando não há nome no catálogo',()=>{
  expect(variableLabel('qualquer.chave')).toBe('qualquer.chave');
  expect(variableLabel('reward.cost')).not.toBe('reward.cost');
 });
});

describe('valores de exemplo',()=>{
 it('só traz o que o fluxo cita e começa com exemplo conhecido ou vazio',()=>{
  const flow={actions:[action({kind:'chat',text:'Oi {{viewer.name}}'}),action({kind:'condition',condVar:'qualquer.chave'})]};
  const keys=testVarKeys(flow);
  expect(keys).toEqual(['viewer.name','qualquer.chave']);
  expect(initialSamples(keys)).toEqual({'viewer.name':'TesteViewer','qualquer.chave':''});
  expect(initialSamples([])).toEqual({});
 });
});
