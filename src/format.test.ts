import {expect,it} from 'vitest';
import {fmtDuration,fmtTime} from './types';

it('duração legível nas tabelas: segundos curtos, minutos e horas',()=>{
 // “15 min” preservado: é o texto que os testes de timer conferem.
 expect(fmtDuration(5)).toBe('5 s');
 expect(fmtDuration(0)).toBe('0 s');
 expect(fmtDuration(60)).toBe('1 min');
 expect(fmtDuration(900)).toBe('15 min');
 expect(fmtDuration(5400)).toBe('1 h 30 min');
 expect(fmtDuration(7200)).toBe('2 h');
 // Valor quebrado não vira NaN na tela.
 expect(fmtDuration(Number.NaN)).toBe('0 s');
});

it('hora do histórico nunca mostra Invalid Date',()=>{
 expect(fmtTime('2026-10-07T10:00:00Z')).toMatch(/^\d{2}:\d{2}:\d{2}$/);
 expect(fmtTime(undefined)).toBe('—');
 expect(fmtTime('')).toBe('—');
 // Formato antigo ou corrompido vira traço, nunca Invalid Date.
 expect(fmtTime('data quebrada')).toBe('—');
});
