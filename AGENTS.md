# AGENTS.md

## Overview

A Cargo workspace containing foundational crates for application development. Primarily for internal use within the `alternate` organization.

## Layout

```
crates/
├── authorization/  – authorization abstraction (policy evaluation, access requests, roles, permissions, relations)
├── codec/          – data serialization abstraction (binary, JSON, MessagePack)
├── crypto/         – crypto utilities (csprng, digest, encoding)
├── domain/         – DDD abstractions (entities, aggregate roots, events, repositories, use cases)
└── logic/          – logic abstractions (truth tables, predicate evaluation, logic combinators)
```
