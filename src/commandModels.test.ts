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
import {describe,it,expect} from 'vitest';
import {FOLLOW_ASK_DEFAULT,commandConflict,followAgeFlow,followersCountFlow,nowPlayingFlow} from './commandModels';

describe('modelo Tempo de follow',()=>{
 it('sem IA: consulta real e respostas determinísticas nos dois desfechos',()=>{
  const f=followAgeFlow('p1',{command:'!followers',alias:'!followage',mention:true,ai:false,ok:''});
  expect(f.trigger.kind).toBe('command');
  expect(f.trigger.pattern).toBe('!followers, !followage');
  expect(f.fnId).toBe('tempo-de-follow');
  expect(f.actions.map(a=>a.kind)).toEqual(['followAge','condition','chat','condition','chat']);
  expect(f.actions[0].target).toBe('mention');
  expect(f.actions[1].condOp).toBe('equals');
  expect(f.actions[2].text).toBe('{{user}} segue {{channel}} há {{local.followAge}}, desde {{local.followSince}}.');
  expect(f.actions[3].condOp).toBe('not_equals');
  expect(f.actions[4].text).toBe('{{user}}, {{local.followNote}}');
  expect(f.actions.every(a=>a.kind!=='ai.generate')).toBe(true);
 });
 it('com IA: a frase vem da IA e a reserva continua determinística',()=>{
  const f=followAgeFlow('p1',{command:'!followers',alias:'',mention:true,ai:true,ok:'Fale com humor sobre {{local.followSince}}.'});
  expect(f.actions.map(a=>a.kind)).toEqual(['followAge','ai.generate','chat']);
  expect(f.actions[1].text).toBe('Fale com humor sobre {{local.followSince}}.');
  expect(f.actions[1].aiFallback).toContain('{{local.followAge}}');
  expect(f.actions[2].text).toBe('{{local.aiResponse}}');
 });
 it('sem @nome consulta quem enviou e aceita texto próprio',()=>{
  const f=followAgeFlow('p1',{command:'!followage',alias:'',mention:false,ai:false,ok:'',sayOk:'Oi {{user}}! {{local.followAge}}.'});
  expect(f.trigger.pattern).toBe('!followage');
  expect(f.actions[0].target).toBe('sender');
  expect(f.actions[2].text).toBe('Oi {{user}}! {{local.followAge}}.');
 });
});

describe('modelos simples de dados reais',()=>{
 it('seguidores usa a contagem do canal e música usa a integração',()=>{
  const s=followersCountFlow('p1',{command:'!seguidores',text:''});
  expect(s.actions).toHaveLength(1);
  expect(s.actions[0].text).toContain('{{followerCount}}');
  const m=nowPlayingFlow('p1',{command:'!musica',text:''});
  expect(m.actions[0].text).toContain('{{nowplaying.musica');
 });
});

describe('conflitos de nomes',()=>{
 const flows=[{name:'Meu !followers antigo',trigger:{kind:'command',pattern:'!followers, !fa'}},{name:'Outro',trigger:{kind:'command',pattern:'!oi'}}];
 it('encontra o comando que já existe, sem diferenciar maiúsculas',()=>{
  expect(commandConflict(flows,'!Followers')).toBe('Meu !followers antigo');
  expect(commandConflict(flows,'!seguidores, !fa')).toBe('Meu !followers antigo');
  expect(commandConflict(flows,'!seguidores')).toBeNull();
  expect(commandConflict(flows,'')).toBeNull();
 });
});
