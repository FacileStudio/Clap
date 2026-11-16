---
layout: raw
duration: 5
---
<style>
  .st-third { position: absolute; left: 120px; bottom: 120px; padding: 28px 48px; border-radius: 24px; background: var(--fill-strong); }
  .st-third strong { display: block; font-size: 64px; line-height: 1.1; }
  .st-third span { display: block; margin-top: 8px; font-size: 40px; color: var(--muted); }
</style>
<div class="st-third" id="st-third" data-start="0.3" data-duration="4.4" data-track-index="5">
  <strong id="st-third-name">Ada Lovelace</strong>
  <span id="st-third-role">Analyst, Analytical Engine</span>
</div>
<script>
  scene.timeline((tl) => {
    tl.from("#st-third", { x: -900, opacity: 0, duration: 0.7, ease: "power3.out" }, 0.3);
    tl.from("#st-third-name", { opacity: 0, x: -40, duration: 0.4 }, 0.6);
    tl.from("#st-third-role", { opacity: 0, x: -40, duration: 0.4 }, 0.75);
    tl.to("#st-third", { x: -900, opacity: 0, duration: 0.5, ease: "power2.in" }, 4.2);
  });
</script>
