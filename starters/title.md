---
layout: raw
duration: 4
---
<style>
  .st-title { position: absolute; inset: 0; display: grid; align-content: center; padding: 0 160px; }
  .st-title h1 { margin: 0; font-size: 160px; line-height: 1; }
  .st-title p { margin: 32px 0 0; font-size: 56px; color: var(--muted); }
  .st-title i { display: block; width: 240px; height: 16px; margin-bottom: 48px; background: var(--fg); transform-origin: left; }
</style>
<div class="st-title" data-start="0" data-duration="4">
  <i id="st-title-bar"></i>
  <h1 id="st-title-head">Your title</h1>
  <p id="st-title-sub">A line that says what this is about</p>
</div>
<script>
  scene.timeline((tl) => {
    tl.from("#st-title-bar", { scaleX: 0, duration: 0.6, ease: "power3.out" }, 0.1);
    tl.from("#st-title-head", { opacity: 0, y: 60, duration: 0.8 }, 0.3);
    tl.from("#st-title-sub", { opacity: 0, y: 30, duration: 0.6 }, 0.8);
    tl.to(".st-title", { opacity: 0, duration: 0.4, ease: "power2.in" }, 3.6);
  });
</script>
