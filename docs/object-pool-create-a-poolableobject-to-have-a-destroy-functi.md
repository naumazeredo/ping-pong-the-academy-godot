---
title: "Object pool - create a PoolableObject to have a destroy function"
type: idea
status: new
created: 2026-04-09T23:54:51
---

Right now we have to manually call the `return_to_pool` functions, there's no connection from the object being handled
by a pool to the pool itself, so it can't call the return to pool by itself
