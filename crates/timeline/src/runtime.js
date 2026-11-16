(function () {
  "use strict";
  const DRIFT = 0.25;

  function offset(el) {
    const section = el.closest(".scene");
    const sections = Array.from(document.querySelectorAll(".stage > .scene"));
    const scene = (window.__scene.scenes || [])[sections.indexOf(section)];
    return (scene ? scene.start : 0) + (Number(el.dataset.start) || 0);
  }

  function sync(time, playing) {
    for (const el of document.querySelectorAll("video, audio")) {
      const local = time - offset(el);
      const length = el.dataset.duration === undefined ? Infinity : Number(el.dataset.duration);
      const shown = local >= 0 && local < length && !el.closest(".is-off") && !!el.closest(".scene.is-active, body");
      const volume = Number(el.dataset.volume);
      el.volume = Number.isFinite(volume) ? Math.min(Math.max(volume, 0), 1) : 1;
      if (!shown) {
        el.pause();
        continue;
      }
      const wanted = (Number(el.dataset.in) || 0) + local;
      if (!playing || Math.abs(el.currentTime - wanted) > DRIFT) el.currentTime = wanted;
      if (playing && el.paused) el.play().catch(() => {});
      if (!playing) el.pause();
    }
  }

  function wrap() {
    const scene = window.__scene;
    if (!scene || window.__clapRender || scene.__clap) return;
    const seek = scene.seek;
    scene.__clap = { playing: false };
    scene.seek = (t) => {
      const drawn = seek(t);
      sync(scene.state ? scene.state().time : Number(t) || 0, scene.__clap.playing);
      return drawn;
    };
    window.addEventListener("message", (event) => {
      const data = event.data || {};
      if (data.type === "clap:playing") scene.__clap.playing = !!data.playing;
      if (data.type === "scene:seek") sync(Number(data.t) || 0, scene.__clap.playing);
    });
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", wrap);
  else wrap();
})();
