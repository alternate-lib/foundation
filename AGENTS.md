# AGENTS.md

## Overview

A Cargo workspace containing foundational crates for application development. Primarily for internal use within the `alternate` organization.

## Layout

```
crates/
├── authorization/  – authorization abstraction (policy evaluation, access requests, roles, permissions, relations)
├── codec/          – data serialization abstraction (binary, JSON, MessagePack)
├── crypto/         – crypto utilities (csprng, digest, encoding)
├── domain/         – DDD abstractions (entities, value objects, aggregate roots, events, repositories, use cases)
├── domain-derive/  – Derive macros for DDD abstractions (entities, value objects)
├── logic/          – logic abstraction (truth tables, predicate evaluation, logic combinators)
└── validation/     – validation abstraction (strings, slices, URLs, email addresses)
```
