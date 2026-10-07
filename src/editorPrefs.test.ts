import {describe,it,expect} from 'vitest';
import {parsePrefs,readPrefs,writePrefs} from './editorPrefs';
describe('preferências locais do editor',()=>{
 it('parsePrefs lê objeto válido e descarta JSON inválido ou que não é objeto',()=>{
  expect(parsePrefs(null)).toEqual({});
  expect(parsePrefs('')).toEqual({});
  expect(parsePrefs('não é json')).toEqual({});
  expect(parsePrefs('[1,2,3]')).toEqual({});
  expect(parsePrefs('"texto"')).toEqual({});
  expect(parsePrefs('{"minimap":true}')).toEqual({minimap:true});
  expect(parsePrefs('{"minimap":false,"outro":1}')).toEqual({minimap:false,outro:1});
 });
 it('readPrefs não explode sem armazenamento e writePrefs mescla o patch',()=>{
  expect(readPrefs()).toEqual({});
  expect(writePrefs({minimap:true})).toEqual({minimap:true});
  expect(writePrefs({minimap:false})).toEqual({minimap:false});
 });
});
