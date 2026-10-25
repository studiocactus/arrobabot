import {describe,expect,it} from 'vitest';
import {gankFlow,readGank,isGankCustomized,GANK_TEXT_DEFAULT,GANK_PROMPT_DEFAULT} from './gankFlow';
import type {GankFields} from './gankFlow';

const campos=(p:Partial<GankFields>={}):GankFields=>({shoutout:true,ai:false,announce:false,text:GANK_TEXT_DEFAULT,prompt:GANK_PROMPT_DEFAULT,...p});

describe('anúncio de raid (shoutout e ganks)',()=>{
 it('nasce como automação de raid de verdade, com destaque e mensagem fixa',()=>{
  const f=gankFlow('perfil-1',campos());
  expect(f.trigger.kind).toBe('raid');
  expect(f.trigger.permission).toBe('everyone');
  expect(f.trigger.cooldown).toBe(0);
  expect(f.fnId).toBe('gank');
  expect(f.fnVersion).toBe(1);
  expect(f.sendType).toBe('chat');
  expect(f.actions.map(a=>a.kind)).toEqual(['twitch','chat']);
  expect(f.actions[0].twOp).toBe('shoutout');
  expect(f.actions[1].text).toBe(GANK_TEXT_DEFAULT);
 });
 it('sem destaque nativo, sobra só a mensagem',()=>{
  const f=gankFlow('perfil-1',campos({shoutout:false}));
  expect(f.actions.map(a=>a.kind)).toEqual(['chat']);
 });
 it('com IA, gera a frase na hora e mantém a reserva determinística',()=>{
  const f=gankFlow('perfil-1',campos({ai:true,text:'Reserva pronta para {{raider.name}}.'}));
  expect(f.actions.map(a=>a.kind)).toEqual(['twitch','ai.generate','chat']);
  const ia=f.actions[1];
  expect(ia.target).toBe('local.aiResponse');
  expect(ia.text).toBe(GANK_PROMPT_DEFAULT);
  expect(ia.aiFallback).toBe('Reserva pronta para {{raider.name}}.');
  expect(f.actions[2].text).toBe('{{local.aiResponse}}');
 });
 it('envia como Anúncio da Twitch quando escolhido',()=>{
  expect(gankFlow('perfil-1',campos({announce:true})).sendType).toBe('announce');
 });
 it('usa o padrão quando a mensagem fica vazia, sem publicar nada vazio',()=>{
  expect(gankFlow('perfil-1',campos({text:'   '})).actions.at(-1)?.text).toBe(GANK_TEXT_DEFAULT);
 });
 it('preserva a automação existente ao salvar de novo (mesmo id e ativação)',()=>{
  const base=gankFlow('perfil-1',campos());
  const editado=gankFlow('perfil-1',campos({announce:true}),{...base,enabled:false});
  expect(editado.id).toBe(base.id);
  expect(editado.enabled).toBe(false);
  expect(editado.sendType).toBe('announce');
 });
 it('volta para o formulário exatamente como foi salvo',()=>{
  const f=gankFlow('perfil-1',campos({shoutout:false,ai:true,announce:true,text:'Reserva.',prompt:'Instrução.'}));
  expect(readGank(f)).toEqual({shoutout:false,ai:true,announce:true,text:'Reserva.',prompt:'Instrução.'});
 });
 it('lê um anúncio fixo de volta para o formulário',()=>{
  const f=gankFlow('perfil-1',campos({text:'Valeu, {{raider.name}}!'}));
  expect(readGank(f)).toEqual(campos({text:'Valeu, {{raider.name}}!'}));
 });
 it('protege passos extras feitos no editor avançado',()=>{
  const base=gankFlow('perfil-1',campos());
  expect(isGankCustomized(base)).toBe(false);
  expect(isGankCustomized({...base,actions:[...base.actions,{...base.actions[1],kind:'tts'}]})).toBe(true);
 });
});
