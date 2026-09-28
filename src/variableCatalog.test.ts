import {describe,it,expect} from 'vitest';
import {insertVariable,makeVariableToken,messageParts,missingLocals,variableLabel} from './variableCatalog';
describe('edição de mensagens com variáveis',()=>{
 it('substitui só a seleção e preserva o restante da mensagem',()=>{
  expect(insertVariable('Olá, convidado!','{{user}}',5,14)).toEqual({text:'Olá, {{user}}!',caret:13});
  expect(insertVariable('🎉 Olá!','{{user}}',2,2)).toEqual({text:'🎉{{user}} Olá!',caret:10});
 });
 it('traduz as opções visuais sem permitir injetar outros filtros',()=>{
  expect(makeVariableToken('arg0','amigo','upper')).toBe('{{arg0|default:amigo|upper}}');
  expect(makeVariableToken('global.meta','0','number:2')).toBe('{{global.meta|default:0|number:2}}');
  expect(()=>makeVariableToken('global.x}}','', '')).toThrow();
  expect(()=>makeVariableToken('arg0','x|upper','')).toThrow();
 });
 it('exibe rótulos sem executar marcadores, respeita escape e deixa o cifrão legado como texto',()=>{
  const p=messageParts('Olá, {{user|upper}}! $channel %userName% \\{{user}} $unknown');
  expect(p.filter(v=>v.label).map(v=>v.label)).toEqual(['Nome da pessoa','Nome da pessoa']);
  expect(p.some(v=>v.text.includes('$')&&v.label)).toBe(false);
  expect(p.map(v=>v.text).join('')).toBe('Olá, {{user|upper}}! $channel %userName% \\{{user}} $unknown');
 });
 it('cataloga o nome sorteado para mensagens sem pessoa no evento',()=>{
  const p=messageParts('Boa live, {{randomViewer}}!');
  expect(p.filter(v=>v.label).map(v=>v.label)).toEqual(['Nome sorteado no chat']);
 });
 it('cataloga as falas da escuta contínua para as mensagens usarem',()=>{
  expect(variableLabel('lastSpeech')).toBe('Última fala no microfone');
  expect(variableLabel('liveSpeech')).toBe('Falas da escuta nesta sessão');
  const p=messageParts('Você disse {{lastSpeech}} e antes {{liveSpeech}}');
  expect(p.filter(v=>v.label).map(v=>v.label)).toEqual(['Última fala no microfone','Falas da escuta nesta sessão']);
  expect(p.map(v=>v.text).join('')).toBe('Você disse {{lastSpeech}} e antes {{liveSpeech}}');
 });
 it('aceita a faixa do número sorteado e rotula ela como número',()=>{
  expect(makeVariableToken('random:1,50','','')).toBe('{{random:1,50}}');
  expect(makeVariableToken('random:1,50.00','','')).toBe('{{random:1,50.00}}');
  expect(variableLabel('random:1,50.00')).toBe('Número sorteado');
  expect(messageParts('doou R$ {{random:1,50.00}}!').find(v=>v.label)?.label).toBe('Número sorteado');
  expect(()=>makeVariableToken('random:1','','')).toThrow('variável válida');
  expect(()=>makeVariableToken('random:x,y','','')).toThrow();
 });
});
describe('aviso de variável local sem origem no fluxo',()=>{
 const chat={kind:'chat',text:'',target:''};
 const gerada={kind:'ai.generate',target:'local.aiResponse'};
 const ia={kind:'ai',target:''};
 it('aponta o local que nenhuma ação do fluxo define',()=>{
  expect(missingLocals('A resposta é {{local.aiResponse}}',[chat])).toEqual(['local.aiResponse']);
  expect(missingLocals('{{local.a}} e depois {{local.b}}',[chat])).toEqual(['local.a','local.b']);
 });
 it('some quando alguma ação gera o valor',()=>{
  expect(missingLocals('A resposta é {{local.aiResponse}}',[chat,gerada])).toEqual([]);
  expect(missingLocals('{{local.aiResponse}} / {{local.aiSuccess}}',[ia])).toEqual([]);
  expect(missingLocals('{{local.pontos}}',[chat,{kind:'variable.set',target:'local.pontos'}])).toEqual([]);
 });
 it('respeita o texto alternativo e ignora o que não é local',()=>{
  expect(missingLocals('{{local.aiResponse|default:0}}',[chat])).toEqual([]);
  expect(missingLocals('{{local.aiResponse|default:s/n}}',[chat])).toEqual([]);
  expect(missingLocals('{{user}} com {{commandCount}} e {{local.x}}',[chat])).toEqual(['local.x']);
  expect(missingLocals('{{global.x}} e {{local.y}}',[chat])).toEqual(['local.y']);
 });
 it('apagar ou somar o valor não é origem',()=>{
  expect(missingLocals('{{local.aiResponse}}',[{kind:'variable.delete',target:'local.aiResponse'}])).toEqual(['local.aiResponse']);
  expect(missingLocals('{{local.x}}',[{kind:'variable.increment',target:'local.x'}])).toEqual([]);
 });
 it('ação Twitch define jogo, título e alvo',()=>{
  const jogo={kind:'twitch',target:'',twOp:'game'};
  expect(missingLocals('{{local.twitchGame}}',[chat,jogo])).toEqual([]);
  expect(missingLocals('{{local.twitchTitle}}',[chat,jogo])).toEqual(['local.twitchTitle']);
  expect(missingLocals('{{local.twitchTarget}}',[chat,{kind:'twitch',target:'',twOp:'ban'}])).toEqual([]);
 });
});
