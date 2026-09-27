import {describe,expect,it} from 'vitest';
import {newAction,newProfile} from './types';
import {knowledgeGroups,resolveAnchor,resolveKnowledge,resolveLength,resolveNoRepeat,toggleKnowledgeFile} from './aiOptions';

const ai={...newProfile().ai,anchor:'fixed',answerLength:'short',knowledge:true,noRepeat:true};

describe('controles por ação de IA',()=>{
 it('ação que não escolhe nada herda o padrão do perfil',()=>{
  const action=newAction('ai');
  expect(resolveAnchor(action,ai)).toBe('fixed');
  expect(resolveLength(action,ai)).toBe('short');
  expect(resolveKnowledge(action,ai)).toBe(true);
  expect(resolveNoRepeat(action,ai)).toBe(true);
 });
 it('o que a ação escolhe sobrescreve o perfil',()=>{
  const action={...newAction('ai'),aiAnchor:'message',aiLength:'free',aiKnowledge:'off',aiNoRepeat:false,aiStyle:'gírias do chat'};
  expect(resolveAnchor(action,ai)).toBe('message');
  expect(resolveLength(action,ai)).toBe('free');
  expect(resolveKnowledge(action,ai)).toBe(false);
  expect(resolveNoRepeat(action,ai)).toBe(false);
  expect(action.aiStyle).toBe('gírias do chat');
 });
 it('a ação pode forçar a base mesmo com o perfil desligado',()=>{
  const off={...ai,knowledge:false};
  expect(resolveKnowledge(newAction('ai'),off)).toBe(false);
  expect(resolveKnowledge({...newAction('ai'),aiKnowledge:'on'},off)).toBe(true);
 });
 it('a ação nova já nasce com os controles prontos',()=>{
  const action=newAction('ai.generate');
  expect(action).toMatchObject({aiAnchor:'',aiKnowledge:'',aiLength:'',aiStyle:'',aiNoRepeat:null});
 });
});

describe('base de conhecimento',()=>{
 const items=[
  {path:'girias/gerais.md',category:'girias',file:'gerais',title:'Gírias',kind:'',size:120},
  {path:'nichos/fps.md',category:'nichos',file:'fps',title:'FPS',kind:'',size:40},
  {path:'tom-e-comportamento/anti.md',category:'tom-e-comportamento',file:'anti',title:'Anti',kind:'',size:80},
 ];
 it('agrupa por pasta e marca o que ficou de fora',()=>{
  const off=toggleKnowledgeFile(newProfile().ai,'girias/gerais.md');
  const groups=knowledgeGroups(items,off);
  expect(groups.map(g=>g.category)).toEqual(['girias','nichos','tom-e-comportamento']);
  expect(groups.find(g=>g.category==='girias')!.files[0].included).toBe(false);
  expect(groups.find(g=>g.category==='nichos')!.files[0].included).toBe(true);
 });
 it('ligar e desligar um arquivo é reversível',()=>{
  const base=newProfile().ai;
  const off=toggleKnowledgeFile(base,'nichos/fps.md');
  expect(off.knowledgeOff).toEqual(['nichos/fps.md']);
  expect(base.knowledgeOff).toEqual([]);
  expect(toggleKnowledgeFile(off,'nichos/fps.md').knowledgeOff).toEqual([]);
 });
});
