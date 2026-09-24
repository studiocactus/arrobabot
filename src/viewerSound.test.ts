import {afterEach,expect,it,vi} from 'vitest';
vi.mock('./api',()=>({api:vi.fn(async()=> 'data:audio/wav;base64,AAAA')}));
import {playViewerSound} from './viewerSound';
const sounds:FakeAudio[]=[];
class FakeAudio {
 setSinkId=vi.fn(async(_id:string)=>{});
 volume=0;onended:(()=>void)|null=null;onerror:(()=>void)|null=null;pause=vi.fn();load=vi.fn();removeAttribute=vi.fn();play=vi.fn(async()=>{});
 constructor(public src:string){sounds.push(this)}
}
afterEach(()=>{vi.useRealTimers();vi.unstubAllGlobals();sounds.length=0});
it('serializa sons, aplica volume e limita reprodução a 15 segundos',async()=>{
 vi.stubGlobal('Audio',FakeAudio);vi.useFakeTimers();
 const a=playViewerSound({profileId:'p',asset:'1',volume:0.4});const b=playViewerSound({profileId:'p',asset:'2',volume:0.8});
 await vi.advanceTimersByTimeAsync(0);expect(sounds).toHaveLength(1);expect(sounds[0].volume).toBe(0.4);
 sounds[0].onended?.();await a;await vi.advanceTimersByTimeAsync(0);expect(sounds).toHaveLength(2);
 await vi.advanceTimersByTimeAsync(15000);await b;expect(sounds[1].pause).toHaveBeenCalled();expect(sounds[1].removeAttribute).toHaveBeenCalledWith('src');
});
it('seleciona a saída antes de reproduzir e informa dispositivo removido',async()=>{
 vi.stubGlobal('Audio',FakeAudio);vi.useFakeTimers();
 const a=playViewerSound({profileId:'p',asset:'1',volume:1,deviceId:'speakers'});
 await vi.advanceTimersByTimeAsync(0);expect(sounds[0].setSinkId).toHaveBeenCalledWith('speakers');expect(sounds[0].setSinkId.mock.invocationCallOrder[0]).toBeLessThan(sounds[0].play.mock.invocationCallOrder[0]);sounds[0].onended?.();await a;
 class MissingDevice extends FakeAudio{setSinkId=vi.fn(async()=>{throw Error('missing')})}
 vi.stubGlobal('Audio',MissingDevice);const b=playViewerSound({profileId:'p',asset:'2',volume:1,deviceId:'removed'});await expect(b).rejects.toThrow('Saída de áudio indisponível');expect(sounds[1].play).not.toHaveBeenCalled();
});
it('informa falha e libera a fila para o próximo som',async()=>{
 vi.stubGlobal('Audio',FakeAudio);vi.useFakeTimers();const a=playViewerSound({profileId:'p',asset:'1',volume:1});const rejected=expect(a).rejects.toThrow('reproduzir');
 await vi.advanceTimersByTimeAsync(0);sounds[0].onerror?.();await rejected;
 const b=playViewerSound({profileId:'p',asset:'2',volume:1});await vi.advanceTimersByTimeAsync(0);sounds[1].onended?.();await b;
});
