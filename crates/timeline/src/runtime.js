(function () {
  "use strict";
  const DRIFT = 0.25;
  const CLIP_SECONDS = 0.5;
  const SCENE_SECONDS = 0.6;
  const BEFORE = 1e-4;
  const FITS = ["cover", "contain", "fill"];
  const HOLDING = ["fade", "slide-left", "slide-right", "wipe-left", "wipe-right", "zoom"];
  const OWNED = ["position", "inset", "left", "top", "width", "height", "margin", "maxWidth", "maxHeight",
    "transform", "transformOrigin", "objectFit", "opacity", "filter", "clipPath", "visibility", "background"];
  const state = { playing: false, ghost: null, time: 0, ids: new Map(), touched: new Set() };

  const clamp = (value, low, high) => Math.min(Math.max(value, low), high);
  const ease = (p) => (p < 0.5 ? 4 * p * p * p : 1 - Math.pow(-2 * p + 2, 3) / 2);
  const overshoot = (p) => 1 + 2.70158 * Math.pow(p - 1, 3) + 1.70158 * Math.pow(p - 1, 2);
  const finite = (value) => typeof value === "number" && Number.isFinite(value);
  const number = (text, fallback) => (text === undefined || text === "" || !Number.isFinite(Number(text)) ? fallback : Number(text));

  function slide(dx, dy) {
    return (p, entering, box) => {
      const a = ease(p);
      const sign = entering ? -1 : 1;
      return { x: sign * dx * (1 - a) * box.w, y: sign * dy * (1 - a) * box.h, opacity: Math.min(1, 2 * a) };
    };
  }

  const PRESETS = {
    fade: (p) => ({ opacity: ease(p) }),
    "slide-left": slide(-1, 0),
    "slide-right": slide(1, 0),
    "slide-up": slide(0, -1),
    "slide-down": slide(0, 1),
    "zoom-in": (p, entering) => ({ opacity: ease(p), scale: entering ? 0.6 + 0.4 * ease(p) : 1 + 0.4 * (1 - ease(p)) }),
    "zoom-out": (p, entering) => ({ opacity: ease(p), scale: entering ? 1.4 - 0.4 * ease(p) : 1 - 0.4 * (1 - ease(p)) }),
    "wipe-left": (p, entering) => ({ clip: entering ? [0, 0, 0, 1 - ease(p)] : [0, 1 - ease(p), 0, 0] }),
    "wipe-right": (p, entering) => ({ clip: entering ? [0, 1 - ease(p), 0, 0] : [0, 0, 0, 1 - ease(p)] }),
    blur: (p) => ({ opacity: ease(p), blur: 24 * (1 - ease(p)) }),
    pop: (p, entering) => ({ opacity: Math.min(1, 3 * p), scale: entering ? 0.5 + 0.5 * overshoot(p) : 0.6 + 0.4 * ease(p) })
  };

  function preset(name) {
    return Object.prototype.hasOwnProperty.call(PRESETS, name) ? PRESETS[name] : null;
  }

  function stage() {
    const el = document.querySelector(".stage");
    return { el, w: number(el && el.dataset.width, 1280), h: number(el && el.dataset.height, 720) };
  }

  function sections() {
    return Array.from(document.querySelectorAll(".stage > .scene"));
  }

  function info(section) {
    const list = (window.__scene && window.__scene.scenes) || [];
    const known = list[sections().indexOf(section)];
    return known || { start: 0, duration: number(section && section.dataset.duration, 0) };
  }

  function timing(el) {
    const scene = info(el.closest(".scene"));
    const start = number(el.dataset.start, 0);
    const length = el.dataset.duration === undefined ? scene.duration - start : number(el.dataset.duration, 0);
    return { scene, start, end: Math.min(start + Math.max(0, length), scene.duration) };
  }

  function fades(el) {
    const { start, end } = timing(el);
    const half = Math.max(0, end - start) / 2;
    const pick = (name, seconds) => (preset(el.dataset[name]) ? clamp(number(el.dataset[seconds], CLIP_SECONDS), 0, half) : 0);
    return { enter: pick("enter", "enterDuration"), exit: pick("exit", "exitDuration"), start, end };
  }

  function sceneTransition(section) {
    const found = section ? /(?:^|\s)clap-transition-([a-z]+(?:-[a-z]+)?)(?:\s|$)/.exec(section.className) : null;
    if (!found || !HOLDING.concat("dip").includes(found[1])) return null;
    const ms = /(?:^|\s)clap-transition-ms-(\d+)(?:\s|$)/.exec(section.className);
    const seconds = ms ? Number(ms[1]) / 1000 : SCENE_SECONDS;
    return { kind: found[1], seconds: clamp(seconds, 0, info(section).duration) };
  }

  function hold(el) {
    const section = el.closest(".scene");
    const next = section ? section.nextElementSibling : null;
    const motion = next && next.classList.contains("scene") ? sceneTransition(next) : null;
    const { scene, end } = timing(el);
    return motion && HOLDING.includes(motion.kind) && end >= scene.duration - 1e-6 ? motion.seconds : 0;
  }

  function blank() {
    return { opacity: 1, x: 0, y: 0, scale: 1, blur: 0, clip: [0, 0, 0, 0] };
  }

  function mix(into, part) {
    if (part.opacity !== undefined) into.opacity *= part.opacity;
    into.x += part.x || 0;
    into.y += part.y || 0;
    if (part.scale !== undefined) into.scale *= part.scale;
    into.blur += part.blur || 0;
    if (part.clip) into.clip = into.clip.map((side, i) => Math.max(side, part.clip[i]));
    return into;
  }

  function motion(el, local, box) {
    const fx = blank();
    const { enter, exit, start, end } = fades(el);
    if (enter > 0 && local < start + enter) mix(fx, preset(el.dataset.enter)(clamp((local - start) / enter, 0, 1), true, box));
    if (exit > 0 && local > end - exit) mix(fx, preset(el.dataset.exit)(clamp((end - local) / exit, 0, 1), false, box));
    fx.opacity *= clamp(number(el.dataset.opacity, 1), 0, 1);
    return fx;
  }

  function geometry(el, size) {
    const ghost = state.ghost && state.ghost.id === state.ids.get(el) ? state.ghost : null;
    if (ghost) return ghost;
    const keys = ["x", "y", "w", "h", "rot"];
    if (!keys.some((key) => el.dataset[key] !== undefined)) return null;
    const side = (key, fallback) => number(el.dataset[key], fallback);
    const w = side("w", size.w) > 0 ? side("w", size.w) : size.w;
    const h = side("h", size.h) > 0 ? side("h", size.h) : size.h;
    return { x: side("x", 0), y: side("y", 0), w, h, rot: side("rot", 0) };
  }

  function owned(el) {
    const d = el.dataset;
    return ["x", "y", "w", "h", "rot", "fit", "opacity", "enter", "exit"].some((key) => d[key] !== undefined) ||
      (state.ghost && state.ghost.id === state.ids.get(el));
  }

  function reset(el) {
    for (const prop of OWNED) el.style[prop] = "";
  }

  function paint(el, box, fx, size) {
    const st = el.style;
    const pct = (part, whole) => `${(100 * part) / whole}%`;
    if (box) {
      Object.assign(st, { position: "absolute", inset: "auto", left: pct(box.x, size.w), top: pct(box.y, size.h),
        width: pct(box.w, size.w), height: pct(box.h, size.h), margin: "0", maxWidth: "none", maxHeight: "none" });
      if (el.matches("img, video")) st.objectFit = FITS.includes(el.dataset.fit) ? el.dataset.fit : "cover";
    }
    const turn = box ? box.rot : 0;
    const moved = fx.x || fx.y || turn || fx.scale !== 1;
    st.transformOrigin = moved ? "50% 50%" : "";
    st.transform = moved ? `translate(${fx.x}px, ${fx.y}px) rotate(${turn}deg) scale(${fx.scale})` : "";
    st.opacity = fx.opacity < 1 ? String(fx.opacity) : "";
    st.filter = fx.blur > 0 ? `blur(${fx.blur}px)` : "";
    const [top, right, bottom, left] = fx.clip.map((side) => `${100 * side}%`);
    st.clipPath = fx.clip.some((side) => side > 0) ? `inset(${top} ${right} ${bottom} ${left})` : "";
  }

  function layout(section, local, size) {
    for (const el of section.querySelectorAll("[data-start]")) {
      if (state.touched.has(el)) reset(el);
      if (!owned(el)) {
        state.touched.delete(el);
        continue;
      }
      state.touched.add(el);
      const box = geometry(el, size);
      paint(el, box, motion(el, local, box || size), size);
    }
  }

  function ids() {
    const seen = new Set();
    for (const section of sections()) {
      const scene = number(section.dataset.index, 0);
      section.querySelectorAll("[data-start]").forEach((el, nth) => {
        const named = [el.dataset.clip, el.id].map((id) => (id || "").trim()).find((id) => id && !seen.has(id));
        let id = named;
        for (let derived = `clip-${scene}-${nth}`; !id; derived += "x") if (!seen.has(derived)) id = derived;
        seen.add(id);
        state.ids.set(el, id);
      });
    }
  }

  function active(time) {
    const list = sections();
    const scenes = list.map(info);
    const at = scenes.findIndex((scene) => time < scene.start + scene.duration);
    const index = at < 0 ? list.length - 1 : at;
    return { list, scenes, index, local: time - (scenes[index] ? scenes[index].start : 0) };
  }

  function sceneMotion(time) {
    const now = active(time);
    const incoming = sceneTransition(now.list[now.index]);
    const next = sceneTransition(now.list[now.index + 1]);
    const scene = now.scenes[now.index];
    if (incoming && now.local < incoming.seconds) {
      const dip = incoming.kind === "dip";
      const p = clamp(now.local / (dip ? incoming.seconds / 2 : incoming.seconds), 0, 1);
      const held = !dip && now.index > 0 ? now.index - 1 : -1;
      if (!dip || p < 1) return { ...now, kind: incoming.kind, p, held, side: "in" };
    }
    if (next && next.kind === "dip" && scene && now.local > scene.duration - next.seconds / 2) {
      const p = clamp((scene.duration - now.local) / (next.seconds / 2), 0, 1);
      return { ...now, kind: "dip", p, held: -1, side: "out" };
    }
    return { ...now, kind: null, p: 1, held: -1, side: "in" };
  }

  function paintScenes(m, stageEl) {
    for (const section of m.list) {
      for (const prop of ["opacity", "transform", "clipPath", "visibility", "background"]) section.style[prop] = "";
    }
    if (stageEl) stageEl.style.background = m.kind ? "#000" : "";
    const el = m.list[m.index];
    if (!m.kind || !el) return;
    const a = ease(m.p);
    const held = m.list[m.held];
    el.style.background = "var(--bg)";
    if (held) Object.assign(held.style, { visibility: "visible", background: "var(--bg)" });
    const looks = {
      fade: () => (el.style.opacity = String(a)),
      dip: () => (el.style.opacity = String(a)),
      zoom: () => Object.assign(el.style, { opacity: String(a), transform: `scale(${1.2 - 0.2 * a})` }),
      "wipe-left": () => (el.style.clipPath = `inset(0 0 0 ${100 * (1 - a)}%)`),
      "wipe-right": () => (el.style.clipPath = `inset(0 ${100 * (1 - a)}% 0 0)`),
      "slide-left": () => push(el, held, 1, a),
      "slide-right": () => push(el, held, -1, a)
    };
    looks[m.kind]();
  }

  function push(el, held, sign, a) {
    el.style.transform = `translateX(${sign * 100 * (1 - a)}%)`;
    if (held) held.style.transform = `translateX(${-sign * 100 * a}%)`;
  }

  function draw(m) {
    const size = stage();
    paintScenes(m, size.el);
    m.list.forEach((section, i) => {
      if (i === m.index) layout(section, m.local, size);
      if (i === m.held) layout(section, m.scenes[i].duration - BEFORE, size);
    });
  }

  function sound(el) {
    const { enter, exit } = fades(el);
    return { fade_in: el.dataset.enter === "fade" ? enter : 0, fade_out: el.dataset.exit === "fade" ? exit : 0 };
  }

  function ramp(el, local, length) {
    const { fade_in: rise, fade_out: fall } = sound(el);
    const up = rise > 0 ? clamp(local / rise, 0, 1) : 1;
    return up * (fall > 0 ? clamp((length - local) / fall, 0, 1) : 1);
  }

  function sync(time, m) {
    for (const el of document.querySelectorAll("video, audio")) {
      const { scene, start, end } = timing(el);
      const index = m.list.indexOf(el.closest(".scene"));
      const local = time - scene.start - start;
      const length = end - start;
      const wanted = number(el.dataset.in, 0) + clamp(index === m.held ? scene.duration - BEFORE - start : local, 0, length);
      const shown = (index === m.index || index < 0) && local >= 0 && local < length && !el.closest(".is-off");
      el.volume = clamp(number(el.dataset.volume, 1), 0, 1) * (shown ? ramp(el, local, length) : 1);
      if (!shown) el.pause();
      if (index === m.held && Math.abs(el.currentTime - wanted) > 0.04) el.currentTime = wanted;
      if (!shown) continue;
      if (!state.playing || Math.abs(el.currentTime - wanted) > DRIFT) el.currentTime = wanted;
      if (state.playing && el.paused) el.play().catch(() => {});
      if (!state.playing) el.pause();
    }
  }

  function kind(el) {
    if (el.matches("video")) return "video";
    if (el.matches("img")) return "image";
    return el.children.length === 0 && el.textContent.trim() ? "text" : "html";
  }

  function boxes(time, m) {
    const size = stage();
    const section = m.list[m.index];
    if (!section || !size.el || window.parent === window) return;
    const frame = size.el.getBoundingClientRect();
    const scale = frame.width / size.w || 1;
    const clips = [];
    for (const el of section.querySelectorAll("[data-start]")) {
      if (el.matches("audio") || el.closest(".is-off")) continue;
      const rect = el.getBoundingClientRect();
      const measured = { x: (rect.left - frame.left) / scale, y: (rect.top - frame.top) / scale,
        w: rect.width / scale, h: rect.height / scale, rot: 0 };
      const box = geometry(el, size) || measured;
      clips.push({ id: state.ids.get(el), kind: kind(el), x: box.x, y: box.y, w: box.w, h: box.h, rot: box.rot });
    }
    window.parent.postMessage({ type: "clap:boxes", t: time, clips }, "*");
  }

  function frame(t, seeked) {
    const api = window.__scene;
    const time = clamp(number(t, 0), 0, api.duration || 0);
    const m = sceneMotion(time);
    if (m.held >= 0) {
      const before = m.scenes[m.held];
      state.seek(before.start + before.duration - BEFORE);
      state.seek(time);
    } else if (!seeked) state.seek(time);
    state.time = time;
    draw(m);
    if (window.__clapRender) return;
    sync(time, m);
    boxes(time, m);
  }

  function ghost(data) {
    const sides = ["x", "y", "w", "h", "rot"].every((key) => finite(data[key])) && data.w > 0 && data.h > 0;
    state.ghost = sides && typeof data.id === "string" ? { id: data.id, x: data.x, y: data.y, w: data.w, h: data.h, rot: data.rot } : null;
    const m = sceneMotion(state.time);
    draw(m);
    boxes(state.time, m);
  }

  function message(event) {
    const data = event.data || {};
    if (data.type === "clap:playing") state.playing = !!data.playing;
    if (data.type === "scene:seek") frame(data.t, true);
    if (data.type === "scene:step") frame(window.__scene.state().time, true);
    if (data.type === "clap:ghost" && !window.__clapRender) ghost(data);
  }

  function wrap() {
    const api = window.__scene;
    if (!api || api.__clap) return;
    api.__clap = true;
    ids();
    state.seek = api.seek;
    api.seek = (t) => frame(t, false);
    window.addEventListener("message", message);
    frame(api.state ? api.state().time : 0, true);
  }

  window.__clapLayout = { sound, hold };
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", wrap);
  else wrap();
})();
