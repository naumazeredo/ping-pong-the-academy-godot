---
title: "Design - systems architecture"
type: reference
status: new
created: 2026-05-29T13:49:55
---

# Systems

- Grid System
- Selection System
- Building System
- Player System
- Gym System

# Responsibilities

## Grid System

- Holds data of the ground plane and the grid
- Is the standard way of getting mouse projection and grid cell position

## Selection System

- Holds data of which entity is selected
- Calls appropriate systems

Currently this is being handled by the Building System

## Building System

- Holds data of all placed objects
- Handles placing, moving and deleting objects

## Player System

- Holds data of all players
- Holds data of the active, in gym, players
- Handles player movement
- Handles player progression

## Gym System

- Holds data of the gym management: money, schedules, memberships

# Interactions

## Player-Table


