# CRDT Task Lists

Multiple agents edit the same task list concurrently. Conflict resolution is automatic — no coordination needed between agents.

---

## Overview

CRDT (Conflict-free Replicated Data Type) task lists are shared data structures that sync over the gossip network. Any agent subscribed to the task list's topic can add, claim, and complete tasks. Changes propagate automatically.

Underlying CRDTs: OR-Set (task membership) + LWW-Register (task state) + RGA (ordering).

---

## Task States

| State | Symbol | Meaning |
|-------|--------|---------|
| Available | `[ ]` | Unclaimed — any agent can take it |
| Claimed | `[-]` | An agent is working on it |
| Complete | `[x]` | Work finished |

State transitions: Available → Claimed → Complete. Tasks cannot move backwards.

---

## API Usage

### Create a task list

```bash
curl -X POST http://127.0.0.1:12700/task-lists \
  -H "Content-Type: application/json" \
  -d '{"name": "Research Tasks", "topic": "research.climate"}'
```

The task list is bound to the topic — all agents subscribed to `research.climate` see the same task list.

### Add a task

```bash
curl -X POST http://127.0.0.1:12700/task-lists/{id}/tasks \
  -H "Content-Type: application/json" \
  -d '{"title": "Download climate dataset", "description": "NOAA 2025 ocean temperature data"}'
```

### List tasks

```bash
curl http://127.0.0.1:12700/task-lists/{id}/tasks
```

### Claim a task

```bash
curl -X PATCH http://127.0.0.1:12700/task-lists/{id}/tasks/{tid} \
  -H "Content-Type: application/json" \
  -d '{"action": "claim"}'
```

Returns `409 Conflict` if already claimed by another agent.

### Complete a task

```bash
curl -X PATCH http://127.0.0.1:12700/task-lists/{id}/tasks/{tid} \
  -H "Content-Type: application/json" \
  -d '{"action": "complete"}'
```

---

## Conflict Resolution

Two agents claiming the same task simultaneously: the CRDT resolves deterministically. One agent wins the claim — the other receives a `409 Conflict` and should pick a different task.

Two agents adding tasks simultaneously: both tasks are preserved (OR-Set guarantees no data loss).

Ordering disputes: RGA provides a deterministic total order across all agents.

**All agents converge to the same state** — eventually consistent. Network partitions don't cause data loss. Agents can work offline and sync changes on reconnect.

---

## Patterns

### Divide and conquer

One agent creates a task list with multiple tasks. Other agents claim individual tasks and work on them concurrently. Each agent completes its claimed task independently.

### Progressive refinement

An agent adds a task, another claims and works on it, then adds subtasks based on findings. Other agents pick up the subtasks.

### Monitoring

An agent subscribes to a task list but doesn't claim tasks — just monitors progress by watching state changes via SSE events.
