import {afterEach,expect,it,vi} from 'vitest';
import {playViewerSound,stopViewerSounds} from './viewerSound';
const sounds:FakeAudio[]=[];
class FakeAudio {
 volume=0;onended:(()=>void)|null=null;onerror:(()=>void)|null=null;pause=vi.fn();load=vi.fn();removeAttribute=vi.fn();play=vi.fn(async()=>{});
 constructor(public src:string){sounds.push(this)}
}
afterEach(()=>{vi.useRealTimers();vi.unstubAllGlobals();sounds.length=0;stopViewerSounds()});
it('serializa sons, aplica volume e limita reprodução a 15 segundos',async()=>{
 vi.stubGlobal('Audio',FakeAudio);vi.useFakeTimers();
 const a=playViewerSound({profileId:'p',src:'http://127.0.0.1:9876/asset?token=t&id=1',volume:0.4});
 const b=playViewerSound({profileId:'p',src:'http://127.0.0.1:9876/asset?token=t&id=2',volume:0.8});
 await vi.advanceTimersByTimeAsync(0);expect(sounds).toHaveLength(1);expect(sounds[0].volume).toBe(0.4);
 sounds[0].onended?.();await a;await vi.advanceTimersByTimeAsync(0);expect(sounds).toHaveLength(2);
 await vi.advanceTimersByTimeAsync(15000);await b;expect(sounds[1].pause).toHaveBeenCalled();expect(sounds[1].removeAttribute).toHaveBeenCalledWith('src');
});
it('informa falha e libera a fila para o próximo som',async()=>{
 vi.stubGlobal('Audio',FakeAudio);vi.useFakeTimers();const a=playViewerSound({profileId:'p',src:'x.wav',volume:1});const rejected=expect(a).rejects.toThrow('reproduzir');
 await vi.advanceTimersByTimeAsync(0);sounds[0].onerror?.();await rejected;
 const b=playViewerSound({profileId:'p',src:'y.wav',volume:1});await vi.advanceTimersByTimeAsync(0);sounds[1].onended?.();await b;
});
it('recusa a fila cheia em vez de acumular sons',async()=>{
 vi.stubGlobal('Audio',FakeAudio);vi.useFakeTimers();
 const all=[1,2,3,4,5].map(i=>playViewerSound({profileId:'p',src:`s${i}.wav`,volume:1}));
 await expect(playViewerSound({profileId:'p',src:'s6.wav',volume:1})).rejects.toThrow('Fila de sons cheia');
 await vi.advanceTimersByTimeAsync(0);sounds[0].onended?.();await all[0];
 for(let i=1;i<5;i++){await vi.advanceTimersByTimeAsync(0);sounds[i].onended?.();}
 await Promise.all(all);
});
it('parar tudo pausa o som atual e descarta a fila',async()=>{
 vi.stubGlobal('Audio',FakeAudio);vi.useFakeTimers();
 const a=playViewerSound({profileId:'p',src:'a.wav',volume:1});
 const b=playViewerSound({profileId:'p',src:'b.wav',volume:1});
 await vi.advanceTimersByTimeAsync(0);expect(sounds).toHaveLength(1);
 stopViewerSounds();expect(sounds[0].pause).toHaveBeenCalled();
 await vi.advanceTimersByTimeAsync(0);await a;await b;
 expect(sounds,'o som da fila nunca chega a tocar').toHaveLength(1);
});
