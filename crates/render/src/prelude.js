(() => {
  "use strict";
  window.__clapRender = true;
  const media = [];
  const DROPPED = ["src", "poster", "controls", "autoplay", "loop", "muted", "playsinline", "preload", "crossorigin"];

  function source(el) {
    const child = el.querySelector("source[src]");
    return el.getAttribute("src") ? el.src : child ? child.src : "";
  }

  function swap(video) {
    const img = document.createElement("img");
    for (const attr of Array.from(video.attributes)) {
      if (!DROPPED.includes(attr.name)) img.setAttribute(attr.name, attr.value);
    }
    img.alt = "";
    img.decoding = "sync";
    media.push({ el: img, video: true, url: source(video), muted: video.hasAttribute("muted") });
    video.replaceWith(img);
  }

  function silence(audio) {
    media.push({ el: audio, video: false, url: source(audio), muted: audio.hasAttribute("muted") });
    audio.removeAttribute("src");
    audio.replaceChildren();
  }

  function timing(el) {
    const sections = Array.from(document.querySelectorAll(".stage > .scene"));
    const known = (window.__scene.scenes || [])[sections.indexOf(el.closest(".scene"))];
    const scene = known || { start: 0, duration: window.__scene.duration };
    const start = Number(el.dataset.start) || 0;
    const left = Math.max(0, scene.duration - start);
    const duration = el.dataset.duration === undefined ? left : Math.min(Number(el.dataset.duration) || 0, left);
    const volume = el.dataset.volume === undefined ? 1 : Number(el.dataset.volume);
    const layout = window.__clapLayout;
    const sound = layout ? layout.sound(el) : { fade_in: 0, fade_out: 0 };
    return {
      ...sound,
      hold: layout ? layout.hold(el) : 0,
      at: scene.start + start,
      duration,
      offset: Math.max(0, Number(el.dataset.in) || 0),
      volume: Number.isFinite(volume) ? Math.max(0, volume) : 1
    };
  }

  function show([index, url]) {
    const img = media[index].el;
    if (img.getAttribute("src") === url) return Promise.resolve();
    return new Promise((done, fail) => {
      img.onload = () => done();
      img.onerror = () => fail(new Error(`video frame ${url} did not load`));
      img.src = url;
    });
  }

  document.addEventListener("DOMContentLoaded", () => {
    document.querySelectorAll("video").forEach(swap);
    document.querySelectorAll("audio").forEach(silence);
  });

  window.__clap = {
    scan: () => media.map(({ el, video, url, muted }) => ({ video, url, muted, ...timing(el) })),
    show: (frames) => Promise.all(frames.map(show)).then(() => true)
  };
})();
