import {describe,it,expect} from 'vitest';
import {navigationSections,navigationMatches,setupJourneys} from './navigation';
describe('organização e descoberta',()=>{
 it('preserva os 23 destinos sem duplicação',()=>{
  const pages=navigationSections.flatMap(section=>section.pages);
  expect(new Set(pages).size).toBe(23);
  expect(pages).toHaveLength(23);
  for(const journey of setupJourneys)for(const step of journey.steps)expect(pages).toContain(step.page);
 });
 it('encontra tarefas sem exigir acento ou nome técnico',()=>{
  expect(navigationMatches('vozfx','Voicemod','resgate')).toBe(true);
  expect(navigationMatches('audio','Áudio no OBS','audio')).toBe(true);
  expect(navigationMatches('settings','Configurações','BACKUP')).toBe(true);
  expect(navigationMatches('history','Histórico','erro')).toBe(true);
  expect(navigationMatches('flows','Automações','evento resgate')).toBe(true);
  expect(navigationMatches('obs','OBS Studio','filtro')).toBe(true);
  expect(navigationMatches('obs','OBS Studio','imagem')).toBe(true);
  expect(navigationMatches('flows','Automações','receita obs')).toBe(true);
  expect(navigationMatches('commands','Comandos','spotify')).toBe(false);
 });
});
