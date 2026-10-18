/**
 * Reprodutor da fonte "Áudio do BotLive" do OBS.
 *
 * Uma fila serial só: os sons chegam um depois do outro (nada toca em
 * duplicidade nem atropela), no máximo 5 na fila, 15 s de teto por som.
 * Eventos antigos (mais de 10 s) são descartados e confirmados como erro —
 * uma reconexão nunca reproduz fila velha. Só o perfil dono da chave toca.
 */

export type PlayPayload = {
  profileId?: string;
  id?: string;
  kind?: string;
  src?: string;
  volume?: number;
  issuedAt?: number;
};

export type AudioMsg = { type?: string; payload?: PlayPayload };

export type Send = (msg: unknown) => void;

/** Elemento de áudio mínimo — no navegador é um HTMLAudioElement. */
export type Sound = {
  src: string;
  volume: number;
  play(): Promise<void>;
  pause(): void;
  /** Compatível com o handler do DOM: só se escreve, nunca se lê aqui. */
  onended: unknown;
  onerror: unknown;
};

export const MAX_PENDING = 5;
export const MAX_AGE_MS = 10000;
export const PLAY_TIMEOUT_MS = 15000;

type Options = {
  profile: string;
  send: Send;
  now?: () => number;
  create?: (src: string) => Sound;
  /** Chamado quando o navegador bloqueia a reprodução automática. */
  onBlocked?: () => void;
};

const ack = (id: string, phase: 'started' | 'done' | 'error' | 'stopped', detail?: string) => ({
  type: 'audio.ack',
  id,
  phase,
  ...(detail ? {detail} : {}),
});

export class SourcePlayer {
  private queue: Promise<void> = Promise.resolve();
  private pending = 0;
  private current: Sound | null = null;
  private currentId = '';
  private release: (() => void) | null = null;
  private generation = 0;
  private readonly now: () => number;
  private readonly create: (src: string) => Sound;

  constructor(private readonly opts: Options) {
    this.now = opts.now ?? Date.now;
    this.create = opts.create ?? ((src) => new Audio(src));
  }

  /** Sons esperando vez na fila (diagnóstico). */
  get queued(): number {
    return this.pending;
  }

  handle(msg: AudioMsg): void {
    if (msg?.type === 'audio.stop') {
      this.stopAll();
      return;
    }
    if (msg?.type !== 'audio.play') return;
    const p = msg.payload ?? {};
    // Só o dono: perfil do payload tem que ser o desta fonte.
    if (p.profileId !== this.opts.profile) return;
    const id = typeof p.id === 'string' && p.id.length > 0 && p.id.length <= 64 ? p.id : '';
    const src = typeof p.src === 'string' ? p.src : '';
    if (!id || !this.allowedSrc(src)) return;
    const volume = typeof p.volume === 'number' && Number.isFinite(p.volume) ? Math.min(1, Math.max(0, p.volume)) : 1;
    // Evento atrasado (página dormindo ou reconectando): descarta e avisa.
    const issuedAt = typeof p.issuedAt === 'number' ? p.issuedAt : 0;
    if (issuedAt > 0 && this.now() - issuedAt > MAX_AGE_MS) {
      this.opts.send(ack(id, 'error', 'Som antigo descartado'));
      return;
    }
    if (this.pending >= MAX_PENDING) {
      this.opts.send(ack(id, 'error', 'Fila de sons cheia'));
      return;
    }
    const gen = this.generation;
    this.pending++;
    this.queue = this.queue
      .catch(() => {})
      .then(() => this.playOne(id, src, volume, gen))
      .catch(() => {})
      .finally(() => {
        this.pending--;
      });
  }

  /** Só endereço local ou dado de áudio — nada de rede além desta máquina. */
  private allowedSrc(src: string): boolean {
    return src.startsWith('http://127.0.0.1:') || src.startsWith('http://localhost:') || src.startsWith('data:audio/');
  }

  private playOne(id: string, src: string, volume: number, gen: number): Promise<void> {
    if (gen !== this.generation) return Promise.resolve();
    return new Promise<void>((resolve) => {
      const audio = this.create(src);
      this.current = audio;
      this.currentId = id;
      audio.volume = volume;
      let settled = false;
      const finish = () => {
        if (settled) return;
        settled = true;
        clearTimeout(timer);
        try {
          audio.pause();
        } catch {}
        if (this.current === audio) this.current = null;
        if (this.release === finish) this.release = null;
        resolve();
      };
      this.release = finish;
      const timer = setTimeout(() => {
        this.opts.send(ack(id, 'error', 'Som interrompido após 15 s'));
        finish();
      }, PLAY_TIMEOUT_MS);
      audio.onended = () => {
        this.opts.send(ack(id, 'done'));
        finish();
      };
      audio.onerror = () => {
        this.opts.send(ack(id, 'error', 'Falha ao carregar o áudio'));
        finish();
      };
      void audio
        .play()
        .then(() => {
          // Confirmação de verdade: o navegador iniciou a reprodução.
          this.opts.send(ack(id, 'started'));
        })
        .catch((e: unknown) => {
          const blocked = (e as {name?: string} | null)?.name === 'NotAllowedError';
          this.opts.send(ack(id, 'error', blocked ? 'Reprodução bloqueada pelo navegador' : 'Falha ao iniciar a reprodução'));
          if (blocked) this.opts.onBlocked?.();
          finish();
        });
    });
  }

  /** Para tudo imediatamente: pausa o som atual, libera a fila na hora e
   *  descarta o que ainda não começou. */
  stopAll(): void {
    this.generation++;
    const id = this.currentId;
    const release = this.release;
    this.release = null;
    release?.();
    if (id) this.opts.send(ack(id, 'stopped'));
  }
}
