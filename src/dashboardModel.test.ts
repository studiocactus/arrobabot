import {describe,it,expect} from 'vitest';
import {fresh,showBanner,uptime,bitrate,chartPaths} from './dashboardModel';
describe('contratos visuais do dashboard',()=>{
 it('banner distingue conexão e live confirmada, sem inferir offline em falha',()=>{expect(showBanner('offline',null,100)).toBe(true);expect(showBanner('online',{state:'fresh',lastSuccessAt:90,data:{live:false}},100)).toBe(true);expect(showBanner('online',{state:'stale',lastSuccessAt:90,data:{live:false}},100)).toBe(false);expect(showBanner('online',{state:'fresh',lastSuccessAt:90,data:{live:true}},100)).toBe(false)});
 it('leitura velha perde validade sem converter dado para zero',()=>{expect(fresh({state:'fresh',lastSuccessAt:10,data:{live:true,viewers:0}},20)).toBe(true);expect(fresh({state:'fresh',lastSuccessAt:10,data:{live:true,viewers:0}},110)).toBe(false)});
 it('duração usa início da plataforma e não conexão do bot',()=>{expect(uptime('2026-10-29T20:00:00Z',Date.parse('2026-10-29T21:42:16Z')/1000)).toBe('01:42:16');expect(uptime(undefined,1)).toBe('—')});
 it('bitrate rejeita reset e leituras repetidas',()=>{expect(bitrate({bytes:1000,at:1},{bytes:2000,at:2})).toBe(8);expect(bitrate({bytes:2000,at:1},{bytes:100,at:2})).toBe(null);expect(bitrate({bytes:1,at:1},{bytes:2,at:1})).toBe(null)});
 it('gráfico não liga pontos durante lacunas ou falhas',()=>{expect(chartPaths([[1,0],[31,10],[61,null],[91,20],[200,30]])).toHaveLength(3)});
});
