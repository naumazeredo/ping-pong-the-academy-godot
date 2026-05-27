---
title: "bug - fix perf issues with baking the navmesh at runtime"
type: task
status: new
created: 2026-04-29T16:54:44
---

Two improvements:
1. Manually create the mesh: we can have a mapping of the tiles and manually create the inner part of the open tiles and
   then create the connections to connected
2. Make it in another thread
