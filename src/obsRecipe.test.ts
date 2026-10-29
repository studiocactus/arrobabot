import {describe,it,expect} from 'vitest';
import {recipeDraft,recipeExisting} from './recipes';
import {obsRecipeMatches} from './obsRecipe';
import {obsCompatible,obsProblem} from './obsCatalog';
import {newAction} from './types';
import {triggerForSave} from './flow';
describe('receitas e controles OBS preservam o fluxo existente',()=>{
 it('apresenta a sequência inteira no formulário simples e preserva personalizações',()=>{
   for(const id of ['obsText','obsImage','obsFilter'] as const){const f=recipeDraft(id,'p1')!;expect(f.enabled).toBe(false);expect(obsRecipeMatches(f)).toBe(true);expect(obsRecipeMatches({...f,layout:{nodes:[],edges:[]}})).toBe(false);expect(obsRecipeMatches({...f,actions:[...f.actions,newAction('delay')]})).toBe(false);expect(obsRecipeMatches({...f,audio:'custom'})).toBe(false);expect(recipeExisting(f,[{...f,name:'Meu modelo'}])?.name).toBe('Meu modelo');}
  const f=recipeDraft('obsText','p1')!;expect(f.actions[0]).toMatchObject({kind:'condition',condVar:'rawInput',condOp:'is_not_empty',condFalse:'stop'});expect(f.actions.at(-1)?.kind).toBe('chat');
 });
 it('preserva os limites de batimentos ao salvar pelo editor',()=>{const trigger={kind:'heartRate',pattern:'above:120',permission:'everyone',cooldown:10,userCooldown:0};expect(triggerForSave(trigger)).toEqual(trigger);});
 it('filtra os tipos compatíveis e valida o essencial antes do teste',()=>{
  expect(obsCompatible('set_text','text_gdiplus_v2')).toBe(true);expect(obsCompatible('set_text','text_ft2_source_v2')).toBe(true);expect(obsCompatible('set_image','browser_source')).toBe(false);
  const a={...newAction('obs'),obsTarget:'Browser',obsOp:'set_browser_url',text:'javascript:alert(1)'};expect(obsProblem(a)).toBeTruthy();expect(obsProblem({...a,text:'http://127.0.0.1:9876/overlay'})).toBeNull();expect(obsProblem({...a,text:'https://name:password@example.com'})).toBeTruthy();expect(obsProblem({...a,obsOp:'filter_enable',obsFilter:''})).toMatch(/filtro/);expect(obsProblem({...a,obsOp:'rotate',text:'361'})).toMatch(/360/);
 });
});
