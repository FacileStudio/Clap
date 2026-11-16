---
lang: {{lang}}
size: 1920x1080
layout: raw
duration: 4
---

<h1 id="title" class="clap-title" data-start="0" data-duration="4" data-track-index="1">{{title}}</h1>
<p id="hint" class="clap-caption" data-start="0.8" data-duration="3.2" data-track-index="2">{{hint}}</p>
<script>
  scene.timeline((tl) => {
    tl.from("#title", { opacity: 0, y: 48, scale: 0.96, duration: 0.9, ease: "power3.out" }, 0.1);
    tl.from("#hint", { opacity: 0, y: 16, duration: 0.6 }, 0.8);
    tl.to("#title", { scale: 1.04, duration: 3, ease: "none" }, 1);
  });
</script>
