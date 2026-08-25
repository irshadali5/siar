# 18 — Protocol Extensions & WASM Plugins

> **Corresponding Specifications:** [`sys-arch/01-protocol-extension-system-architecture.md`](../sys-arch/01-protocol-extension-system-architecture.md), [`sys-arch/21-third-party-protocol-extensions-architecture.md`](../sys-arch/21-third-party-protocol-extensions-architecture.md), [`sys-arch/22-wasm-compatible-components-architecture.md`](../sys-arch/22-wasm-compatible-components-architecture.md), [`sys-arch/24-plugin-module-ecosystem-architecture.md`](../sys-arch/24-plugin-module-ecosystem-architecture.md), [`sys-arch/ui-ux-19-plugin-module-ecosystem-architecture.md`](../sys-arch/ui-ux-19-plugin-module-ecosystem-architecture.md)  
> **Key Crates:** [`crates/siar-protocol-ext`](../crates/siar-protocol-ext), [`crates/siar-protocol`](../crates/siar-protocol)

---

## 1. Dynamic Protocol Extension Architecture

To allow third-party developers, NGOs, and specialized defense/rescue teams to extend SIAR without forking the codebase, the protocol provides an extensible envelope architecture:

```rust
pub struct ExtensionDescriptor {
    pub extension_id: ExtensionId,        // e.g. "org.siar.gis.tactical-map"
    pub version: ExtensionVersion,        // SemVer 1.2.0
    pub author_signature: Signature,      // Ed25519 publisher signature
    pub required_capabilities: CapabilityBitmask,
    pub permissions: Vec<PluginPermission>, // StorageAccess, LocationAccess, NetworkAccess
}
```

```
+-------------------------------------------------------------------------------+
|                         Standard SIAR Wire Frame                              |
|  - Magic Header + Frame Length + Version + Ephemeral Routing Token            |
+-------------------------------------------------------------------------------+
|                       Extension Envelope (Dynamic Tag)                        |
|  - Extension ID: 0x474953 ("GIS")  - Extension Version: 0x010200              |
|  - Typed Binary Payload (Protobuf / Postcard / Raw Bytes)                     |
+-------------------------------------------------------------------------------+
```

---

## 2. WebAssembly (WASM) Sandbox Runtime

Third-party plugin code runs in a sandboxed WebAssembly execution environment with strict hardware boundaries:

```mermaid
graph TD
    HostCore[SIAR Rust Host Core] -->|Enforces Limits| Sandbox[Wasmtime Execution Sandbox]
    Sandbox --> WasmModule[Third-Party Plugin Wasm Module]
    
    WasmModule -.->|Requests Location| PermCheck{Permission Granted?}
    PermCheck -->|Yes| HostCore
    PermCheck -->|No| Reject[Permission Denied Error]
```

### Sandbox Security Invariants
1. **Instruction / Fuel Limiting**: Each plugin execution is granted a finite fuel budget (e.g. 5,000,000 instructions) to prevent infinite loops and denial-of-service hangs.
2. **Strict Memory Quotas**: Linear memory is capped (maximum 32 MB per plugin instance).
3. **No Direct OS Syscalls**: Plugins have zero direct access to filesystem, raw network sockets, or hardware devices; all I/O occurs via audited host function bindings.

---

## 3. Plugin Lifecycle & Marketplace Distribution

- **Self-Contained Bundles (`.siarplugin`)**: A signed ZIP archive containing the compiled `plugin.wasm`, manifest, localized strings, and UI vector icons.
- **Offline Sideloading**: Plugins can be shared peer-to-peer over BLE or Wi-Fi Direct like any regular file attachment.
- **Revocation List**: Compromised or malicious plugin IDs are gossiped via the standard cryptographic tombstone network.
