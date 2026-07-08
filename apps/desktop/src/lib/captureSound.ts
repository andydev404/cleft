// A synthesized blip, not a bundled audio file — a single short sine tone
// with a fast decay envelope, matching the Settings copy ("Subtle click
// when a clip is saved"). One shared AudioContext, created lazily on first
// use since browsers block autoplay/audio-context creation before any user
// gesture has happened in the page.
let ctx: AudioContext | null = null;

export function playCaptureSound() {
  ctx ??= new AudioContext();
  const oscillator = ctx.createOscillator();
  const gain = ctx.createGain();
  oscillator.type = "sine";
  oscillator.frequency.value = 880;
  gain.gain.setValueAtTime(0.15, ctx.currentTime);
  gain.gain.exponentialRampToValueAtTime(0.0001, ctx.currentTime + 0.12);
  oscillator.connect(gain).connect(ctx.destination);
  oscillator.start();
  oscillator.stop(ctx.currentTime + 0.12);
}
