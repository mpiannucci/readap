
# Plan: High-Level JavaScript OPeNDAP Client (`readap-js`)

This document outlines the plan to add a new high-level JavaScript/TypeScript package to the `readap` workspace.

## 1. Goal

Create a new npm package, `readap-js`, that provides a simple, developer-friendly API for downloading and using OPeNDAP data in JavaScript/TypeScript environments (Node.js and browser). This package will use the existing `readap-wasm` package for low-level parsing.

## 2. Repository and Workspace Changes

1.  **NPM Workspace:** Introduce a root `package.json` to manage `readap-wasm` and the new `readap-js` as a monorepo.
2.  **New Directory:** Create a new top-level directory `readap-js/` for the new package.

The new structure will look like this:
```
/
├── package.json             (New: for npm workspace)
├── readap/                  (Existing)
├── readap-wasm/             (Existing)
└── readap-js/               (New: High-level TS/JS package)
    ├── package.json
    ├── tsconfig.json
    └── src/
        └── index.ts
```

## 3. `readap-js` API and Functionality

The core of the new package will be a `DAPClient` class.

### Proposed API

```typescript
import { DAPClient } from 'readap-js';

// Usage:
const client = new DAPClient('https://example.com/opendap/data-source');
const data = await client.fetchData('temperature', { time: [0, 10] });
```

### How It Works (Interaction with Wasm)

1.  **Dependency:** `readap-js` will depend on `readap-wasm` via an npm workspace link.
2.  **Abstraction:** `readap-js` will handle the initialization of the Wasm module from `readap-wasm`, making it seamless for the end-user.
3.  **Data Flow:**
    *   `readap-js` makes the `fetch` request to an OPeNDAP server to get binary data.
    *   The binary data (`ArrayBuffer`) is passed to `readap-wasm` for high-performance parsing.
    *   `readap-wasm` returns a structured JavaScript object.
    *   `readap-js` formats this object and returns it to the user.
