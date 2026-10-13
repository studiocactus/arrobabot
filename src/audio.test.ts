import {expect,it} from 'vitest';
import {DEFAULT_AUDIO,beepDataUri,normalizeAudio} from './audio';

it('padrão espelha o backend e valores inválidos voltam ao normal',()=>{
 expect(normalizeAudio(null)).toEqual(DEFAULT_AUDIO);
 expect(normalizeAudio({peopleBrowser:true,flowLine:false,flowBrowser:true,alertLine:true,device:'cable'}))
  .toEqual({peopleLine:true,peopleBrowser:true,flowLine:false,flowBrowser:true,alertLine:true,alertBrowser:true,device:'cable'});
 // Interruptor que chegou como texto volta ao padrão da origem.
 expect(normalizeAudio({peopleLine:'on',flowBrowser:'sim'}).peopleLine).toBe(true);
 expect(normalizeAudio({peopleLine:'on'}).flowBrowser).toBe(false);
 // Dispositivo gigante é cortado no limite do backend.
 expect(normalizeAudio({device:'x'.repeat(900)}).device).toHaveLength(512);
});

it('bipe da linha é um WAV data URI com cabeçalho e amostras certas',()=>{
 const uri=beepDataUri(440,0.2);
 expect(uri.startsWith('data:audio/wav;base64,')).toBe(true);
 const bytes=atob(uri.slice('data:audio/wav;base64,'.length));
 expect(bytes.slice(0,4)).toBe('RIFF');
 expect(bytes.slice(8,12)).toBe('WAVE');
 expect(bytes.length).toBe(44+1600*2); // 44 de cabeçalho + 8 kHz × 0,2 s × 2 bytes
});
