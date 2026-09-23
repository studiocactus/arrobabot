import {describe,it,expect} from 'vitest';
import {insertVariable,makeVariableToken,messageParts} from './variableCatalog';
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
 it('exibe rótulos sem executar marcadores e respeita escape e texto legado',()=>{
  const p=messageParts('Olá, {{user|upper}}! $channel %userName% \\{{user}} $unknown');
  expect(p.filter(v=>v.label).map(v=>v.label)).toEqual(['Nome da pessoa','Nome do canal','Nome da pessoa']);
  expect(p.map(v=>v.text).join('')).toBe('Olá, {{user|upper}}! $channel %userName% \\{{user}} $unknown');
 });
});
