/**
 * Explosão de emotes no overlay: imagens da CDN da Twitch voando do centro para
 * fora com rotação e queda, como um confete de emotes. Usada pelas camadas de
 * alertas (sub, renovação, presente), meta (meta completada) e, quando ligado,
 * pelo chat — o backend escolhe os ids e aqui só desenhamos.
 */
const CDN = (id: string): string => `https://static-cdn.jtvnw.net/emoticons/v2/${encodeURIComponent(id)}/default/dark/3.0`;

const STYLE = `
.bl-eb{position:fixed;inset:0;pointer-events:none;z-index:9999;overflow:hidden}
.bl-eb img{position:absolute;left:50%;top:55%;image-rendering:auto;animation:bl-eb-fly 2.6s cubic-bezier(.15,.6,.35,1) forwards}
@keyframes bl-eb-fly{
 0%{transform:translate(-50%,-50%) scale(.3) rotate(0deg);opacity:0}
 10%{opacity:1}
 35%{transform:translate(calc(-50% + var(--dx)*.6),calc(-50% + var(--dy)*.6)) scale(1.15) rotate(calc(var(--rot)*.5))}
 100%{transform:translate(calc(-50% + var(--dx)),calc(-50% + var(--dy) + 180px)) scale(.85) rotate(var(--rot));opacity:0}
}`;

/** Injeta o CSS uma vez por documento (a fonte do OBS não tem bundler). */
function ensureStyle(doc: Document): void {
 if (doc.getElementById('bl-eb-style')) return;
 const s = doc.createElement('style');
 s.id = 'bl-eb-style';
 s.textContent = STYLE;
 doc.head.appendChild(s);
}

/**
 * Explode `count` emotes do centro do ecrã. Os ids vêm do backend (todos, mais
 * usados ou aleatórios) — aqui só montamos as imagens com atrasos espalhados.
 * Emote que não carregar some sozinho, sem deixar buraco no efeito.
 */
export function explode(host: HTMLElement, ids: string[], size: number): void {
 if (!ids.length) return;
 ensureStyle(host.ownerDocument);
 const fx = host.ownerDocument.createElement('div');
 fx.className = 'bl-eb';
 fx.setAttribute('aria-hidden', 'true');
 const px = Math.max(24, Math.min(120, size));
 const reach = Math.max(160, Math.min(window.innerWidth, window.innerHeight) / 2.2);
 ids.forEach((id, i) => {
  const img = host.ownerDocument.createElement('img');
  img.src = CDN(id);
  img.alt = '';
  img.width = px;
  img.height = px;
  img.onerror = () => img.remove();
  const ang = Math.random() * Math.PI * 2;
  const dist = reach * (0.35 + Math.random() * 0.65);
  img.style.setProperty('--dx', `${Math.round(Math.cos(ang) * dist)}px`);
  img.style.setProperty('--dy', `${Math.round(Math.sin(ang) * dist) - 40}px`);
  img.style.setProperty('--rot', `${Math.round(Math.random() * 540 - 270)}deg`);
  img.style.animationDelay = `${(i % 14) * 45}ms`;
  fx.appendChild(img);
 });
 host.appendChild(fx);
 setTimeout(() => fx.remove(), 3400);
}
