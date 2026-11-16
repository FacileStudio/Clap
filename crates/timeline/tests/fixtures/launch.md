---
title: Launch
size: 1280x720
layout: raw
duration: 6
---

<style>
  .clip { position: absolute; inset: 0; }
</style>
<!-- keep me -->
<video id="shot" src="assets/shot.mp4" data-start="0" data-duration="4" data-track-index="0" data-in="1"></video>
<div class="clip" data-start="0" data-duration="6" data-track-index="1">
  <h1 id="t">Launch day</h1>
</div>
<p id="sub" data-start="2" data-duration="3" data-track-index="2">Today, everywhere.</p>
<script>
  scene.timeline((tl) => tl.from("#t", { opacity: 0, duration: 1 }));
</script>

<!-- Voice-over: it is launch day. -->

---
layout: raw
duration: 4
---

<audio data-clip="music" src="assets/music.mp3" data-start="0" data-duration="4" data-volume="0.5"></audio>

---

## Thanks

Bye.
