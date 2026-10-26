import {describe,it,expect} from 'vitest';
import {recipes,recipeDraft,recipeExisting,recipeConfiguration} from './recipes';
import {newAction,newFlow} from './types';
describe('receitas sem duplicar configurações',()=>{
 it('gera rascunhos novos desativados e deixa raid/voz/resenha nos seus editores',()=>{
  for(const item of recipes){
   const draft=recipeDraft(item.id,'p1');
   if(['raid','voice','conversation'].includes(item.id)){expect(draft).toBeNull();continue}
   expect(draft?.enabled).toBe(false);expect(draft?.fnId).toBeTruthy();expect(draft?.profileId).toBe('p1');
  }
 });
 it('preserva o objeto existente inclusive passos personalizados e outro nome de comando',()=>{
  const draft=recipeDraft('followAge','p1')!;
  const customized={...draft,trigger:{...draft.trigger,pattern:'!meufollow'},enabled:true,actions:[...draft.actions,newAction('delay')]};
  expect(recipeExisting(draft,[customized])).toBe(customized);
  expect(recipeExisting(draft,[{...customized,profileId:'p2'}])).toBeUndefined();
 });
 it('reconhece aliases e conserva timer/redemption em seus tipos',()=>{
  const draft=recipeDraft('followAge','p1')!;
  const existing={...newFlow('p1'),trigger:{...newFlow('p1').trigger,pattern:'!FOLLOWAGE'}};
  expect(recipeExisting(draft,[existing])).toBe(existing);
  expect(recipeDraft('timer','p1')?.timerSeconds).toBe(900);
  expect(recipeDraft('sound','p1')?.trigger).toMatchObject({kind:'redemption',pattern:''});
  expect(recipeDraft('counter','p1')?.trigger.permission).toBe('moderator');
  const timer=recipeDraft('timer','p1')!;
  const renamed={...timer,name:'Minha pausa',timerSeconds:1200};
  expect(recipeExisting(timer,[renamed])).toBe(renamed);
  const conversation={...newFlow('p1'),fnId:'recipe:conversation',name:'Minha conversa personalizada'};
  expect(recipeConfiguration('conversation','p1',[conversation])).toBe(conversation);
  expect(recipeConfiguration('conversation','p2',[conversation])).toBeUndefined();
 });
});
