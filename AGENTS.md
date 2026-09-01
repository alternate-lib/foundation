# AGENTS.md

## Overview

A Cargo workspace containing foundational crates for application development. Primarily for internal use within the `alternate` organization.

## Layout

```
crates/
├── authorization/  – policy evaluation, access requests, roles, permissions, relations
├── codec/          – data serialization abstraction (binary, JSON, MessagePack)
└── logic/          – truth tables, predicate evaluation, logic combinators
```
