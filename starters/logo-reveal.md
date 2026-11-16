---
layout: raw
duration: 3
---
<style>
  .st-logo { position: absolute; inset: 0; display: grid; place-content: center; justify-items: center; gap: 56px; }
  .st-logo i { display: block; width: 280px; height: 280px; border-radius: 72px; background: var(--fg); }
  .st-logo b { font-size: 96px; letter-spacing: 0; }
</style>
<div class="st-logo" data-start="0" data-duration="3">
  <i id="st-logo-mark"></i>
  <b id="st-logo-name">Your brand</b>
</div>
<script>
  scene.timeline((tl) => {
    tl.fromTo("#st-logo-mark", { scale: 0, rotate: -90 }, { scale: 1, rotate: 0, duration: 0.9, ease: "back.out" }, 0.1);
    tl.from("#st-logo-name", { opacity: 0, letterSpacing: 40, duration: 0.8, ease: "power3.out" }, 0.7);
    tl.to(".st-logo", { opacity: 0, duration: 0.3, ease: "power2.in" }, 2.7);
  });
</script>
