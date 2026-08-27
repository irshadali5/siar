//! A deterministic, in-memory mesh simulation over real
//! `siar_dtn::store::BundleStore`/`dedup::SeenBundles` instances — next.md
//! §113 ("your test framework should simulate A-B-C-D... ensure a
//! message → D eventually arrives"), §114 (partition/rejoin), §115
//! (mobility: links appearing and disappearing between ticks).
//!
//! `tick()` deliberately implements the simplest possible forwarding
//! rule — "flood whatever a node has that its neighbor hasn't seen,
//! respecting hop_limit" — not next.md §36's Bloom-filter inventory
//! reconciliation (nothing in this workspace implements that yet) and
//! not §38's replication-budget consumption (a bundle's
//! `replication_budget` field is carried through untouched by this
//! harness; nothing here decrements it). Both are real forwarding-
//! policy refinements a more realistic simulation would need — this
//! one is scoped to proving the *loop-prevention and eventual-delivery*
//! guarantees (§30's hop limit, §31's dedup, §116's "no duplicate
//! logical messages... eventual delivery when a route eventually
//! exists"), which don't depend on either refinement to be meaningful.

use std::collections::{HashMap, HashSet};

use siar_domain::{DeviceId, MessageId};
use siar_dtn::bundle::MeshBundle;
use siar_dtn::dedup::SeenBundles;
use siar_dtn::store::BundleStore;

pub struct SimNode {
    pub store: BundleStore,
    pub seen: SeenBundles,
}

pub struct MeshSimulation {
    nodes: HashMap<DeviceId, SimNode>,
    /// Directed pairs that can currently exchange — `connect` inserts
    /// both `(a, b)` and `(b, a)` so callers get symmetric links
    /// without this crate needing `DeviceId: Ord` to normalize a pair
    /// ordering (it isn't `Ord`).
    links: HashSet<(DeviceId, DeviceId)>,
    now: u64,
}

impl MeshSimulation {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            links: HashSet::new(),
            now: 0,
        }
    }

    pub fn add_node(&mut self, id: DeviceId, quota_bytes: u64, seen_capacity: usize) {
        self.nodes.insert(
            id,
            SimNode {
                store: BundleStore::new(quota_bytes),
                seen: SeenBundles::new(seen_capacity),
            },
        );
    }

    /// Symmetric — both directions can exchange once connected. Safe to
    /// call again on an already-connected pair (idempotent, `HashSet`
    /// insert).
    pub fn connect(&mut self, a: DeviceId, b: DeviceId) {
        self.links.insert((a, b));
        self.links.insert((b, a));
    }

    /// next.md §115's "peer disappears" / §114's partition.
    pub fn disconnect(&mut self, a: DeviceId, b: DeviceId) {
        self.links.remove(&(a, b));
        self.links.remove(&(b, a));
    }

    /// Injects `bundle` directly into `at`'s store, as if created there
    /// locally (next.md §32: "Alice creates... phone stores it"). Also
    /// marks it seen on that node, so `tick` doesn't treat it as a
    /// fresh arrival to re-forward back to itself.
    pub fn originate(&mut self, at: DeviceId, bundle: MeshBundle) {
        if let Some(node) = self.nodes.get_mut(&at) {
            node.seen.check_and_record(bundle.id);
            node.store.insert(bundle, self.now);
        }
    }

    /// One round: every currently-connected pair exchanges whatever the
    /// sender has that the receiver hasn't seen yet, respecting
    /// `hop_limit` (next.md §30 — a bundle at zero hops is simply not
    /// forwarded, matching `MeshBundle::forwarded`'s own "drop" return).
    pub fn tick(&mut self) {
        self.now += 1;

        // Immutable pass: decide what should move without holding any
        // mutable borrow yet — two different entries of the same
        // `HashMap` can't be borrowed mutably at the same time, so
        // collecting first sidesteps that rather than fighting the
        // borrow checker over it.
        let mut deliveries: Vec<(DeviceId, MeshBundle)> = Vec::new();
        for &(from, to) in &self.links {
            let (Some(from_node), Some(to_node)) = (self.nodes.get(&from), self.nodes.get(&to))
            else {
                continue;
            };
            for bundle in from_node.store.iter() {
                if to_node.seen.contains(bundle.id) {
                    continue;
                }
                if let Some(forwarded) = bundle.clone().forwarded() {
                    deliveries.push((to, forwarded));
                }
            }
        }

        // Mutable pass: apply what the immutable pass decided.
        for (to, bundle) in deliveries {
            if let Some(node) = self.nodes.get_mut(&to) {
                if node.seen.check_and_record(bundle.id) {
                    // Arrived via another link earlier in this same
                    // tick already (e.g. two neighbors both had it) —
                    // next.md §31's dedup doing exactly its job.
                    continue;
                }
                node.store.insert(bundle, self.now);
            }
        }
    }

    /// Runs `tick` `rounds` times — a convenience for "let the mesh
    /// settle" in a test, not a different forwarding rule.
    pub fn run(&mut self, rounds: u32) {
        for _ in 0..rounds {
            self.tick();
        }
    }

    pub fn has_bundle(&self, node: DeviceId, id: MessageId) -> bool {
        self.nodes
            .get(&node)
            .map(|n| n.store.contains(id))
            .unwrap_or(false)
    }

    pub fn now(&self) -> u64 {
        self.now
    }
}

impl Default for MeshSimulation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use siar_dtn::bundle::MessagePriority;

    fn bundle(hop_limit: u8, expires_at: u64) -> MeshBundle {
        MeshBundle {
            id: MessageId::new(),
            destination: DeviceId::new(),
            payload_hash: [0u8; 32],
            ciphertext: vec![1, 2, 3],
            priority: MessagePriority::Normal,
            hop_limit,
            replication_budget: 4,
            created_at: 0,
            expires_at,
        }
    }

    /// next.md §113: A-B-C-D chain, A can't see C/D directly, but a
    /// message from A should eventually reach D via B and C.
    #[test]
    fn message_crosses_a_multi_hop_chain() {
        let mut sim = MeshSimulation::new();
        let (a, b, c, d) = (
            DeviceId::new(),
            DeviceId::new(),
            DeviceId::new(),
            DeviceId::new(),
        );
        for node in [a, b, c, d] {
            sim.add_node(node, 1_000_000, 100);
        }
        sim.connect(a, b);
        sim.connect(b, c);
        sim.connect(c, d);

        let msg = bundle(8, 1000); // hop_limit comfortably more than 3 hops needed
        let id = msg.id;
        sim.originate(a, msg);

        // One tick per hop needed, plus a little slack.
        sim.run(4);

        assert!(
            sim.has_bundle(d, id),
            "message should have crossed A -> B -> C -> D"
        );
    }

    /// next.md §30: hop_limit exhausting before reaching the destination
    /// means the message is dropped, not delivered anyway.
    #[test]
    fn hop_limit_too_low_for_the_chain_means_no_delivery() {
        let mut sim = MeshSimulation::new();
        let (a, b, c, d) = (
            DeviceId::new(),
            DeviceId::new(),
            DeviceId::new(),
            DeviceId::new(),
        );
        for node in [a, b, c, d] {
            sim.add_node(node, 1_000_000, 100);
        }
        sim.connect(a, b);
        sim.connect(b, c);
        sim.connect(c, d);

        let msg = bundle(2, 1000); // only 2 hops — A->B->C, not far enough for D
        let id = msg.id;
        sim.originate(a, msg);
        sim.run(5);

        assert!(
            !sim.has_bundle(d, id),
            "hop_limit=2 should not have been enough to reach a 3-hop-away destination"
        );
        assert!(
            sim.has_bundle(c, id),
            "it should have gotten as far as C, though"
        );
    }

    /// next.md §114: a partitioned network, later reconciled once a
    /// bridging connection appears.
    #[test]
    fn partitioned_networks_reconcile_once_a_bridge_connects() {
        let mut sim = MeshSimulation::new();
        let (a, b, c, d, e, f) = (
            DeviceId::new(),
            DeviceId::new(),
            DeviceId::new(),
            DeviceId::new(),
            DeviceId::new(),
            DeviceId::new(),
        );
        for node in [a, b, c, d, e, f] {
            sim.add_node(node, 1_000_000, 100);
        }
        // Network 1: A-B-C. Network 2: D-E-F. No link between them yet.
        sim.connect(a, b);
        sim.connect(b, c);
        sim.connect(d, e);
        sim.connect(e, f);

        let msg = bundle(8, 1000);
        let id = msg.id;
        sim.originate(a, msg);
        sim.run(3);

        assert!(sim.has_bundle(c, id));
        assert!(
            !sim.has_bundle(f, id),
            "no bridge exists yet — the two networks must not have reconciled"
        );

        // C meets D — the bridge next.md §114 describes.
        sim.connect(c, d);
        sim.run(3);

        assert!(
            sim.has_bundle(f, id),
            "once C-D bridges the two networks, the pending bundle should reach network 2"
        );
    }

    /// next.md §31: a bundle arriving at the same node via two different
    /// paths must not be double-counted/re-inserted.
    #[test]
    fn a_node_reachable_by_two_paths_does_not_receive_duplicates() {
        let mut sim = MeshSimulation::new();
        let (a, b, c, d) = (
            DeviceId::new(),
            DeviceId::new(),
            DeviceId::new(),
            DeviceId::new(),
        );
        for node in [a, b, c, d] {
            sim.add_node(node, 1_000_000, 100);
        }
        // Diamond: A -> B -> D and A -> C -> D, two paths to D.
        sim.connect(a, b);
        sim.connect(a, c);
        sim.connect(b, d);
        sim.connect(c, d);

        let msg = bundle(8, 1000);
        let id = msg.id;
        sim.originate(a, msg);
        sim.run(3);

        // `BundleStore` doesn't store two copies of the same id in the
        // first place (dedup happens before `insert` is ever called),
        // so this is really asserting `tick` didn't panic/misbehave
        // trying to double-deliver — `contains` can't distinguish "one
        // copy" from "rejected duplicate" on its own, `len` can.
        assert!(sim.has_bundle(d, id));
        assert_eq!(sim.nodes.get(&d).expect("d exists").store.len(), 1);
    }
}
