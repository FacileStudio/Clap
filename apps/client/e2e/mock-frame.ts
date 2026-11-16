import { outline } from '../src/lib/timeline/outline';
import { blocks } from '../src/lib/timeline/scenes';

const STYLE = `html,body{margin:0;height:100%;overflow:hidden;background:#0a0a0a}
.stage{position:absolute;left:50%;top:50%;overflow:hidden;font:40px/1.3 system-ui,sans-serif;background:var(--bg);color:var(--fg);
--bg:#fafafa;--fg:#0a0a0a;--muted:#737373;--fill:#ececec;--fill-strong:#dcdcdc}
.scene{position:absolute;inset:0;visibility:hidden;padding:80px;box-sizing:border-box}
.scene.layout-raw{padding:0}.scene.is-active{visibility:visible}.is-off{visibility:hidden!important}
video,img{position:absolute;inset:0;width:100%;height:100%;object-fit:cover}`;

const RUNTIME = `(() => {
  const chain = new Proxy({}, { get: () => () => chain });
  window.scene = { timeline: (build) => (build(chain), chain) };
  let scenes = [];
  const tell = (type, detail) => parent !== window && parent.postMessage({ type, ...detail }, "*");
  const fit = () => {
    const el = document.querySelector(".stage");
    const scale = Math.min(innerWidth / el.dataset.width, innerHeight / el.dataset.height);
    el.style.transform = "translate(-50%, -50%) scale(" + scale + ")";
  };
  const seek = (t) => {
    const last = scenes[scenes.length - 1];
    const time = Math.min(Math.max(Number(t) || 0, 0), last.start + last.duration);
    const shown = scenes.find((one) => time < one.start + one.duration) || last;
    const local = time - shown.start;
    for (const one of scenes) one.el.classList.toggle("is-active", one === shown);
    for (const el of shown.el.querySelectorAll("[data-start]")) {
      const from = Number(el.dataset.start) || 0;
      const to = el.dataset.duration === undefined ? Infinity : from + Number(el.dataset.duration);
      el.classList.toggle("is-off", !(local >= from && (local < to || to >= shown.duration)));
      if (el.tagName === "VIDEO") el.currentTime = Math.max(0, local - from + (Number(el.dataset.in) || 0));
    }
    tell("scene:state", { time, step: scenes.indexOf(shown), scene: scenes.indexOf(shown) });
  };
  addEventListener("DOMContentLoaded", () => {
    let start = 0;
    scenes = [...document.querySelectorAll(".stage > .scene")].map((el) => {
      const one = { el, start, duration: Number(el.dataset.duration) || 1, steps: 0, first: 0 };
      start += one.duration;
      return one;
    });
    for (const el of document.querySelectorAll("[data-track-index]")) el.style.zIndex = el.dataset.trackIndex;
    fit();
    seek(0);
    tell("scene:ready", { duration: start, steps: 0, scenes: scenes.map(({ el, ...rest }) => rest) });
  });
  addEventListener("resize", fit);
  addEventListener("message", (event) => event.data && event.data.type === "scene:seek" && seek(event.data.t));
})();`;

/**
 * A stand-in for the scene compiler: the scenes of `source` as one seekable HTML document that speaks the
 * `scene:seek`, `scene:ready` and `scene:state` messages. It shows and hides clips; it runs no timeline.
 */
function compileFrame(source: string, assetBase: string, only: number | null): string {
	const plan = outline(source);
	const lines = source.split('\n');
	const sections = blocks(lines).map((block, index) => {
		const body = lines.slice(block.body, block.to).join('\n').replaceAll('"assets/', `"${assetBase}/`);
		const layout = plan.scenes[index]?.layout ?? 'default';
		return `<section class="scene layout-${layout}" data-index="${index}" data-duration="${plan.scenes[index]?.duration ?? 1}">${body}</section>`;
	});
	const shown = only === null ? sections : sections.slice(only, only + 1);
	const size = `data-width="${plan.width}" data-height="${plan.height}" style="width:${plan.width}px;height:${plan.height}px"`;
	return `<!doctype html><meta charset="utf-8"><style>${STYLE}</style><script>${RUNTIME}</script><main class="stage" ${size}>${shown.join('')}</main>`;
}

/** `/projects/{id}/frame?scene=n`: the stand-in document, sandboxed as the real route sandboxes it. */
export function frameResponse(project: { id: string; source: string }, scene: string | null): Response {
	const html = compileFrame(project.source, `/api/projects/${project.id}/assets`, scene === null ? null : Number(scene));
	const headers = { 'content-type': 'text/html; charset=utf-8', 'content-security-policy': 'sandbox allow-scripts' };
	return new Response(html, { headers });
}
