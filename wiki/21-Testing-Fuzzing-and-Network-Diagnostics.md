# 21 — Testing, Fuzzing & Network Diagnostics

> **Corresponding Specifications:** [`sys-arch/10-fuzzing-protocol-test-suite-architecture.md`](../sys-arch/10-fuzzing-protocol-test-suite-architecture.md), [`sys-arch/18-network-diagnostics-path-visualization-architecture.md`](../sys-arch/18-network-diagnostics-path-visualization-architecture.md), [`sys-arch/ui-ux-20-diagnostics-network-paths-advanced-developer-architecture.md`](../sys-arch/ui-ux-20-diagnostics-network-paths-advanced-developer-architecture.md)  
> **Key Crates & Directories:** [`crates/siar-testkit`](../crates/siar-testkit), [`fuzz`](../fuzz)

---

## 1. Simulated Multi-Hop Mesh Environment (`siar-testkit`)

Testing distributed mesh networks with physical hardware is slow and non-deterministic. The [`siar-testkit`](../crates/siar-testkit) crate provides a fully simulated, in-memory virtual radio mesh:

```rust
#[tokio::test]
async fn test_5_hop_mesh_spray_delivery() {
    let mut network = VirtualMeshNetwork::new();
    
    // Spawn 5 virtual nodes with simulated physical coordinates
    let node1 = network.spawn_node(GeoPoint::new(0.0, 0.0)).await;
    let node2 = network.spawn_node(GeoPoint::new(0.0, 1.0)).await; // In range of 1 & 3
    let node3 = network.spawn_node(GeoPoint::new(0.0, 2.0)).await;
    let node4 = network.spawn_node(GeoPoint::new(0.0, 3.0)).await;
    let node5 = network.spawn_node(GeoPoint::new(0.0, 4.0)).await; // 4 km away from Node 1
    
    // Inject packet loss and random latency
    network.set_packet_loss_rate(0.15); // 15% packet drop
    network.set_link_jitter_ms(20, 100);
    
    // Node 1 sends message to Node 5
    let msg_id = node1.send_message(node5.account_id(), "Disaster check").await.unwrap();
    
    // Wait for multi-hop DTN spray propagation
    network.run_simulation_steps(50).await;
    
    // Verify Node 5 received and decrypted the payload
    assert!(node5.inbox_contains(msg_id));
}
```

---

## 2. End-to-End Multi-Node Integration Testing (`siar-messaging/tests/end_to_end.rs`)

While `siar-testkit` simulates virtual physical layers, SIAR features a dedicated full-stack integration test harness in [`crates/siar-messaging/tests/end_to_end.rs`](../crates/siar-messaging/tests/end_to_end.rs). This test suite exercises the complete live stack between distinct nodes:

- **Full Node Instantiation**: Automatically initializes sovereign cryptographic keypairs, Stoolap SQL storage engines, transport managers, and `MessageService` instances.
- **Out-of-Band Ticket Exchange**: Simulates real-world `PeerTicket` export, Base64 transmission, and contact resolution.
- **Bi-Directional Messaging & ACKs**: Transmits encrypted messages across loopback sockets, verifying end-to-end delivery state transitions (`Pending` -> `Sending` -> `Delivered`).
- **Connection Recycling & Stream Multiplexing**: Validates that pooled transport connections sustain continuous, multi-message traffic without connection stalls or stream drops.

Run the end-to-end test suite:
```bash
cargo test -p siar-messaging --test end_to_end
```

---

## 3. Continuous Fuzzing Pipeline (`fuzz/`)

All untrusted wire deserializers, frame decoders, and manifest parsers undergo continuous mutation fuzzing via `cargo-fuzz` (LLVM libFuzzer):

```
[Random Byte Generator / Mutator]
                 |
                 v
+---------------------------------------------------------------+
|                      Fuzzing Target Harness                   |
|  - fuzz_protocol_decode: Tests siar-protocol envelope parsing |
|  - fuzz_blob_manifest: Tests BLAKE3 Merkle tree verification  |
|  - fuzz_dtn_bundle: Tests malformed bundle header handling    |
+-------------------------------+-------------------------------+
                                |
               +----------------+----------------+
               | (Valid Input)                   | (Malformed Input)
               v                                 v
   [Normal Execution Path]              [Graceful Parse Error]
                                        (ZERO Panics / ZERO Memory Faults)
```

---

## 4. Network Diagnostics & Path Visualizer UI

The diagnostics view provides real-time visibility into local and mesh networking health:

```
+-------------------------------------------------------------------------------+
|                         SIAR Mesh Network Visualizer                          |
+-------------------------------------------------------------------------------+
| Topology: 8 Nodes Online • Active Route to Basecamp: 3 Hops                   |
|                                                                               |
| [You: Node-A] ===(BLE: -64dBm)===> [Node-B: Phone] ===(Wi-Fi Direct)===> ...  |
|      |                                                                        |
|      +========(Wi-Fi Aware)========> [Node-C: Repeater 1] ===(LoRa)===> ...   |
|                                                                               |
| Real-time Telemetry:                                                          |
|   • Average RTT: 42 ms              • Loss Rate: 1.2%                         |
|   • Outbox Queue: 0 Pending         • Battery Drain: 14 mW (Eco Mode)         |
+-------------------------------------------------------------------------------+
```
