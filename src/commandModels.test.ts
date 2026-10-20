import {describe,it,expect} from 'vitest';
import {DONATION_PROMPT,clippersCommandIssue,clippersFlow,donationCommandIssue,donationFlow,donationLinkIssue} from './commandModels';

describe('comando pronto de doação',()=>{
 it('monta o !pix com resposta de IA variada e o link no fim',()=>{
  const f=donationFlow('perfil-1',{command:'!pix',link:'livepix.gg/thenees'});
  expect(f.trigger.kind).toBe('command');
  expect(f.trigger.pattern).toBe('!pix');
  expect(f.trigger.permission).toBe('everyone');
  expect(f.actions).toHaveLength(2);
  expect(f.actions[0].kind).toBe('ai.generate');
  expect(f.actions[0].target).toBe('local.aiResponse');
  expect(f.actions[0].text).toBe(DONATION_PROMPT);
  expect(f.actions[0].text).toContain('DIFERENTE a cada resposta');
  expect(f.actions[1].kind).toBe('chat');
  expect(f.actions[1].text).toBe('{{local.aiResponse}} 💜 livepix.gg/thenees');
  expect(f.enabled).toBe(true);
 });
 it('aceita variações do comando e nomeia o fluxo pela primeira',()=>{
  const f=donationFlow('perfil-1',{command:'!pix, !doar',link:'livepix.gg/thenees'});
  expect(f.trigger.pattern).toBe('!pix, !doar');
  expect(f.name).toBe('Doação (!pix)');
  const outro=donationFlow('perfil-1',{command:'!apoie',link:'livepix.gg/thenees'});
  expect(outro.name).toBe('Doação (!apoie)');
 });
 it('explica o que falta preencher, sem salvar às cegas',()=>{
  expect(donationCommandIssue('   ')).toContain('comando');
  expect(donationLinkIssue('')).toContain('link');
  expect(donationLinkIssue('live pix.gg/x')).toContain('espaços');
  expect(donationLinkIssue('semendereco')).toContain('livepix.gg');
  expect(donationCommandIssue('!pix')).toBe('');
  expect(donationLinkIssue('livepix.gg/thenees')).toBe('');
  expect(donationLinkIssue('https://livepix.gg/thenees')).toBe('');
 });
});

describe('comando pronto de clipadores',()=>{
 it('cria o clipe, conta por pessoa e anuncia a contagem',()=>{
  const f=clippersFlow('p1',{command:'!clip, !clipe'});
  expect(f.trigger.kind).toBe('command');
  expect(f.trigger.pattern).toBe('!clip, !clipe');
  expect(f.trigger.permission).toBe('everyone');
  expect(f.actions.map(a=>a.kind)).toEqual(['twitch','variable.increment','chat']);
  expect(f.actions[0].twOp).toBe('clip');
  expect(f.actions[1].target).toBe('user.clips');
  expect(f.actions[1].text).toBe('1');
  expect(f.actions[2].text).toBe('{{user}}, esse é o seu {{user.clips}}º clipe da live! {{local.clipUrl}}');
  expect(f.name).toBe('Clipadores (!clip)');
  expect(f.enabled).toBe(true);
 });
 it('explica o que falta no comando',()=>{
  expect(clippersCommandIssue('   ')).toContain('comando');
  expect(clippersCommandIssue('!clip')).toBe('');
  const outro=clippersFlow('p1',{command:'!cortar'});
  expect(outro.name).toBe('Clipadores (!cortar)');
  expect(outro.trigger.pattern).toBe('!cortar');
 });
});
