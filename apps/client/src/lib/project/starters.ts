import type { AppKey } from '$lib/i18n/messages';

/** A ready-made raw scene of the insert menu. `body` takes the placeholder words, already translated. */
export interface Starter {
	id: string;
	label: AppKey;
	duration: number;
	words: AppKey[];
	body: (words: string[]) => string;
}

const FILL = 'position:absolute;inset:0;background:var(--bg);color:var(--fg)';

const title: Starter = {
	id: 'title',
	label: 'starter.title',
	duration: 5,
	words: ['starter.title.kicker', 'starter.title.text'],
	body: ([kicker, text]) => `<div class="st-card" data-start="0" data-duration="5" data-track-index="1" style="${FILL};display:flex;flex-direction:column;justify-content:center;padding:0 160px">
  <p class="st-kicker" style="margin:0 0 28px;font-size:34px;font-weight:600;letter-spacing:8px;text-transform:uppercase;color:var(--muted)">${kicker}</p>
  <h1 class="st-title" style="margin:0;font-size:156px;font-weight:700;line-height:1.02;letter-spacing:-4px">${text}</h1>
  <div class="st-rule" style="margin-top:48px;width:260px;height:12px;border-radius:6px;background:var(--fg);transform-origin:left center"></div>
</div>
<script>
  scene.timeline((tl) => {
    tl.from(".st-kicker", { opacity: 0, y: 24, duration: 0.6 }, 0.2);
    tl.from(".st-title", { opacity: 0, y: 70, duration: 0.9, ease: "power3.out" }, "-=0.3");
    tl.from(".st-rule", { scaleX: 0, duration: 0.7, ease: "expo.out" }, "-=0.4");
    tl.to(".st-card", { opacity: 0, duration: 0.5, ease: "power2.in" }, 4.5);
  });
</script>`
};

const lowerThird: Starter = {
	id: 'lower-third',
	label: 'starter.lower',
	duration: 5,
	words: ['starter.lower.name', 'starter.lower.role'],
	body: ([name, role]) => `<div data-start="0" data-duration="5" data-track-index="0" style="${FILL};background:var(--fill)"></div>
<div class="lt-box" data-start="0.4" data-duration="4.4" data-track-index="2" style="position:absolute;left:120px;bottom:120px;display:flex;align-items:stretch;gap:28px">
  <div class="lt-bar" style="width:14px;border-radius:7px;background:var(--fg);transform-origin:center bottom"></div>
  <div style="overflow:hidden;padding:6px 0">
    <p class="lt-name" style="margin:0;font-size:68px;font-weight:700;line-height:1.1;letter-spacing:-1px">${name}</p>
    <p class="lt-role" style="margin:8px 0 0;font-size:38px;color:var(--muted)">${role}</p>
  </div>
</div>
<script>
  scene.timeline((tl) => {
    tl.from(".lt-bar", { scaleY: 0, duration: 0.5, ease: "expo.out" }, 0.4);
    tl.from(".lt-name", { opacity: 0, x: -60, duration: 0.6, ease: "power3.out" }, "-=0.2");
    tl.from(".lt-role", { opacity: 0, x: -60, duration: 0.6, ease: "power3.out" }, "-=0.45");
    tl.to(".lt-box", { opacity: 0, y: 30, duration: 0.4, ease: "power2.in" }, 4.4);
  });
</script>`
};

const word = (text: string) => `<span class="cp-word" style="display:inline-block;margin:0 14px">${text}</span>`;

const caption: Starter = {
	id: 'caption',
	label: 'starter.caption',
	duration: 4,
	words: ['starter.caption.text'],
	body: ([text]) => `<div data-start="0" data-duration="4" data-track-index="0" style="${FILL};background:var(--fill)"></div>
<p class="cp-line" data-start="0.2" data-duration="3.6" data-track-index="3" style="position:absolute;left:160px;right:160px;bottom:110px;margin:0;padding:26px 34px;border-radius:28px;background:var(--fg);color:var(--bg);font-size:64px;font-weight:700;line-height:1.25;text-align:center">
  ${text.split(/\s+/).map(word).join('\n  ')}
</p>
<script>
  scene.timeline((tl) => {
    tl.from(".cp-line", { opacity: 0, scale: 0.92, duration: 0.35, ease: "back.out" }, 0.2);
    tl.from(".cp-word", { opacity: 0, y: 26, duration: 0.3, stagger: 0.12 }, "-=0.1");
    tl.to(".cp-line", { opacity: 0, duration: 0.3 }, 3.5);
  });
</script>`
};

const BARS = [42, 58, 51, 77, 96];

const bar = (value: number, index: number) => `<div style="flex:1;display:flex;flex-direction:column;justify-content:flex-end;gap:18px;height:100%">
      <p class="ch-value" style="margin:0;font-size:44px;font-weight:700;text-align:center">${value}</p>
      <div class="ch-bar" style="height:${value * 5}px;border-radius:20px 20px 6px 6px;background:var(--fg);opacity:${0.35 + index * 0.16};transform-origin:center bottom"></div>
    </div>`;

const chart: Starter = {
	id: 'chart',
	label: 'starter.chart',
	duration: 6,
	words: ['starter.chart.text'],
	body: ([text]) => `<div class="ch-card" data-start="0" data-duration="6" data-track-index="1" style="${FILL};display:flex;flex-direction:column;padding:110px 160px">
  <h2 class="ch-title" style="margin:0;font-size:84px;font-weight:700;letter-spacing:-2px">${text}</h2>
  <div style="flex:1;display:flex;align-items:flex-end;gap:44px;margin-top:40px">
    ${BARS.map(bar).join('\n    ')}
  </div>
  <div style="margin-top:22px;height:8px;border-radius:4px;background:var(--fill-strong)"></div>
</div>
<script>
  scene.timeline((tl) => {
    tl.from(".ch-title", { opacity: 0, y: 40, duration: 0.7 }, 0.2);
    tl.from(".ch-bar", { scaleY: 0, duration: 0.9, stagger: 0.14, ease: "back.out" }, 0.6);
    tl.from(".ch-value", { opacity: 0, y: 20, duration: 0.4, stagger: 0.14 }, 1.1);
    tl.to(".ch-card", { opacity: 0, duration: 0.5 }, 5.5);
  });
</script>`
};

const logo: Starter = {
	id: 'logo',
	label: 'starter.logo',
	duration: 4,
	words: ['starter.logo.text'],
	body: ([text]) => `<div class="lg-card" data-start="0" data-duration="4" data-track-index="1" style="${FILL};display:flex;flex-direction:column;align-items:center;justify-content:center;gap:56px">
  <div class="lg-mark" style="display:flex;align-items:center;justify-content:center;width:260px;height:260px;border-radius:72px;background:var(--fg)">
    <div class="lg-dot" style="width:96px;height:96px;border-radius:48px;background:var(--bg)"></div>
  </div>
  <p class="lg-name" style="margin:0;font-size:92px;font-weight:700;letter-spacing:2px">${text}</p>
</div>
<script>
  scene.timeline((tl) => {
    tl.from(".lg-mark", { scale: 0, rotate: -90, duration: 0.9, ease: "back.out" }, 0.2);
    tl.from(".lg-dot", { scale: 0, duration: 0.5, ease: "back.out" }, "-=0.35");
    tl.from(".lg-name", { opacity: 0, y: 30, letterSpacing: 24, duration: 0.8, ease: "power3.out" }, "-=0.2");
    tl.to(".lg-card", { opacity: 0, duration: 0.5 }, 3.5);
  });
</script>`
};

/** The motion starters of the insert menu, in menu order. */
export const STARTERS: readonly Starter[] = [title, lowerThird, caption, chart, logo];
