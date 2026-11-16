---
layout: raw
duration: 6
---
<style>
  .st-race { position: absolute; inset: 120px 160px; }
  .st-race h2 { margin: 0 0 48px; font-size: 72px; }
  .st-race div { position: absolute; left: 0; height: 110px; border-radius: 20px; background: var(--fill-strong); }
  .st-race span { position: absolute; left: 32px; top: 24px; font-size: 48px; white-space: nowrap; }
</style>
<div class="st-race" data-start="0" data-duration="6">
  <h2 id="st-race-title">Sessions by channel</h2>
  <div id="st-race-a" style="top: 160px"><span>Search</span></div>
  <div id="st-race-b" style="top: 300px"><span>Direct</span></div>
  <div id="st-race-c" style="top: 440px"><span>Social</span></div>
  <div id="st-race-d" style="top: 580px"><span>Mail</span></div>
</div>
<script>
  scene.timeline((tl) => {
    tl.from("#st-race-title", { opacity: 0, y: 30, duration: 0.5 }, 0);
    tl.fromTo("#st-race-a", { width: 0 }, { width: 900, duration: 1.2, ease: "power2.out" }, 0.4);
    tl.fromTo("#st-race-b", { width: 0 }, { width: 700, duration: 1.2, ease: "power2.out" }, 0.5);
    tl.fromTo("#st-race-c", { width: 0 }, { width: 520, duration: 1.2, ease: "power2.out" }, 0.6);
    tl.fromTo("#st-race-d", { width: 0 }, { width: 380, duration: 1.2, ease: "power2.out" }, 0.7);
    tl.to("#st-race-c", { width: 1280, y: -280, duration: 1.4, ease: "power2.inOut" }, 2.6);
    tl.to("#st-race-a", { width: 1040, y: 140, duration: 1.4, ease: "power2.inOut" }, 2.6);
    tl.to("#st-race-b", { width: 760, y: 140, duration: 1.4, ease: "power2.inOut" }, 2.6);
    tl.to("#st-race-d", { width: 460, duration: 1.4, ease: "power2.inOut" }, 2.6);
  });
</script>
