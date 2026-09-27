import {describe,it,expect} from 'vitest';
import {friendlyError,friendlyMessage} from './errors';

describe('friendlyError',()=>{
 it('mantém mensagens que já estão em português',()=>{
  const raw='Conecte todos os blocos ao gatilho antes de salvar.';
  expect(friendlyError(Error(raw))).toEqual({message:raw,detail:''});
 });
 it('traduz falha de rede e guarda o texto original',()=>{
  const result=friendlyError(Error('Failed to fetch'));
  expect(result.message).toContain('Não consegui falar com o aplicativo');
  expect(result.detail).toBe('Failed to fetch');
 });
 it('traduz erro de leitura de dados',()=>{
  const result=friendlyError('invalid type: string "x", expected u32');
  expect(result.message).toContain('não conseguiu ler os dados');
  expect(result.detail).toBe('invalid type: string "x", expected u32');
 });
 it('traduz falta de permissão no sistema',()=>{
  expect(friendlyMessage(Error('EACCES: permission denied, open \'C:\\app\\config.json\''))).toContain('permissão');
 });
 it('não vaza texto técnico em erro vazio ou objeto genérico',()=>{
  for(const bad of ['', '[object Object]', undefined, null]){
   const result=friendlyError(bad);
   expect(result.message).toContain('Não foi possível concluir');
   expect(result.message.includes('[object')).toBe(false);
  }
 });
 it('oferece mensagem amigável e detalhe para erro técnico desconhecido',()=>{
  const result=friendlyError(new Error('zyxwv: something broke in module 7'));
  expect(result.message).toContain('Não foi possível concluir');
  expect(result.detail).toBe('zyxwv: something broke in module 7');
 });
 it('reconhece erros em português sem acento vindos do núcleo',()=>{
  const raw='Conecte uma acao por vez; use a condicao de cada acao para execucao condicional.';
  expect(friendlyError(raw).message).toBe(raw);
 });
});
