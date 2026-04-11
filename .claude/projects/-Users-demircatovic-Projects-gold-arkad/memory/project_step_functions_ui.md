---
name: Step Functions-style execution dashboard
description: Future plan to build a real-time web UI showing state machine execution progress per CIK, inspired by AWS Step Functions — graph visualization with live green/red status per node
type: project
---

Build a real-time dashboard (like AWS Step Functions UI) that shows state machine execution progress per CIK.

**Why:** Visual feedback on which extraction phases succeeded/failed across many concurrent CIKs. Useful for monitoring, debugging, and demos.

**Architecture:**
- Separate `monitor` or `dashboard` binary alongside `extract`
- Backend: axum/actix web server exposing `GET /graph` (static graph from `StateMachineGraph`) + WebSocket `/ws` (live phase completion events from `IntoStateMachineStream`)
- Frontend: renders the DAG, listens on WebSocket, colors nodes green (success) / red (failure) per CIK as events arrive
- All building blocks exist: `StateMachineGraph` for structure, `IntoStateMachineStream` for events, state name constants for mapping events to nodes

**How to apply:** When starting this, create a new binary crate that imports the sec library. Wrap the extraction stream with an event broadcaster that pushes to WebSocket clients. Frontend can use D3.js, cytoscape.js, or similar for graph rendering.
