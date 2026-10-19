/**
 * Fila de reprodução local do aplicativo — usada só quando a fonte
 * "Áudio do BotLive" do OBS **não** está conectada. O backend decide o
 * destino (sink) e envia o mesmo evento para os dois lados; o aplicativo
 * nunca toca junto com a fonte do OBS.
 */
export type ViewerSound = {profileId: string; src: string; volume: number; onStart?: () => void};

let queue = Promise.resolve();
let pending = 0;
let current: HTMLAudioElement | null = null;
let release: (() => void) | null = null;
let generation = 0;

export function playViewerSound(sound: ViewerSound): Promise<void> {
  if (pending >= 5) return Promise.reject(Error('Fila de sons cheia. Aguarde o áudio atual.'));
  const gen = generation;
  pending++;
  const next = queue.catch(() => {}).then(
    () =>
      new Promise<void>((resolve, reject) => {
        if (gen !== generation) return resolve();
        const audio = new Audio(sound.src);
        current = audio;
        audio.volume = Math.max(0, Math.min(1, sound.volume));
        let done = false;
        const finish = (err?: Error) => {
          if (done) return;
          done = true;
          clearTimeout(timer);
          try {
            audio.pause();
          } catch {}
          audio.removeAttribute('src');
          audio.load();
          if (current === audio) current = null;
          if (release === finish) release = null;
          err ? reject(err) : resolve();
        };
        release = finish;
        const timer = setTimeout(() => finish(), 15000);
        audio.onended = () => finish();
        audio.onerror = () => finish(Error('Não foi possível reproduzir este áudio. Tente WAV ou MP3.'));
        void audio
          .play()
          .then(() => {
            // Começou mesmo: confirma e abre o teto para falas longas de TTS.
            sound.onStart?.();
            clearTimeout(timer);
            setTimeout(() => finish(), 60000);
          })
          .catch(() => finish(Error('O navegador bloqueou a reprodução. Clique em Testar áudio de novo.')));
      }),
  );
  queue = next.catch(() => {});
  void next.catch(() => {}).finally(() => {
    pending--;
  });
  return next;
}

/** Para imediatamente o som atual e descarta a fila local. */
export function stopViewerSounds(): void {
  generation++;
  const releaseNow = release;
  release = null;
  releaseNow?.();
  current = null;
}
