import {describe,expect,it} from 'vitest';
import {scopeCss} from './codeMode';

describe('scopeCss',()=>{
 it('prefixa seletores simples e múltiplos com o escopo',()=>{
  expect(scopeCss('.bl-chat-name{color:red}','body')).toBe('body .bl-chat-name{color:red}');
  expect(scopeCss('a, .b ,  *{margin:0}',' .ch-stage'.trim())).toBe('.ch-stage a,.ch-stage .b,.ch-stage *{margin:0}');
 });
 it(':root, html e body viram o próprio escopo (a janela do widget)',()=>{
  expect(scopeCss(':root{--x:1}','body')).toBe('body{--x:1}');
  expect(scopeCss('body{background:#000}','.ch-stage')).toBe('.ch-stage{background:#000}');
  expect(scopeCss('html body .a{}','.ch-stage')).toBe('.ch-stage .a{}');
 });
 it('regras dentro de @media também são prefixadas; keyframes ficam intactos',()=>{
  expect(scopeCss('@media (min-width:600px){.a{display:none}}','body'))
   .toBe('@media (min-width:600px){body .a{display:none}}');
  expect(scopeCss('@keyframes pulse{from{opacity:0}to{opacity:1}}','body'))
   .toBe('@keyframes pulse{from{opacity:0}to{opacity:1}}');
 });
 it('remove comentários e mantém o resto íntegro',()=>{
  expect(scopeCss('/* nota */.a{color:red}','body')).toBe('body .a{color:red}');
  expect(scopeCss('.a{content:"{}"}','body')).toBe('body .a{content:"{}"}');
 });
 it('vazio devolve vazio',()=>{
  expect(scopeCss('','body')).toBe('');
 });
});
