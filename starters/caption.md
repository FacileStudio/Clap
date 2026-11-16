---
layout: raw
duration: 6
---
<style>
  .st-caption { position: absolute; left: 0; right: 0; bottom: 96px; display: grid; justify-items: center; }
  .st-caption p { grid-area: 1 / 1; margin: 0; padding: 20px 40px; border-radius: 20px; font-size: 56px; background: var(--fill-strong); }
</style>
<div class="st-caption" data-track-index="9">
  <p id="st-caption-1" data-start="0" data-duration="2">Captions are clips.</p>
  <p id="st-caption-2" data-start="2" data-duration="2">Each one has a start and a duration.</p>
  <p id="st-caption-3" data-start="4" data-duration="2">Drag them on the timeline.</p>
</div>
<script>
  scene.timeline((tl) => {
    tl.from("#st-caption-1", { opacity: 0, y: 24, duration: 0.25 }, 0);
    tl.from("#st-caption-2", { opacity: 0, y: 24, duration: 0.25 }, 2);
    tl.from("#st-caption-3", { opacity: 0, y: 24, duration: 0.25 }, 4);
  });
</script>
