---
title: "Improvement - make selector mesh follow selector better"
type: task
status: done
created: 2026-04-10T14:09:04
priority: 1
---

SelectorMesh is lerping to the current selector position, which is also a lerp.
Selector should have its own class with a `target_position`, and SelectorMesh should lerp to it
