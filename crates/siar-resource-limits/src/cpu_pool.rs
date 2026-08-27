//! §33 "CPU Budgeting", §34 "CPU Work Classes", §35 "Blocking Work
//! Pool", §36 "Hashing Concurrency", §37 "AV1 Software Encoding".
//!
//! §33: "Precise CPU quotas are difficult in-process. Use practical
//! controls: max concurrent CPU-heavy jobs, worker semaphore..." — a
//! worker semaphore is exactly what this crate's internal bounded
//! atomic counter (built last pass for `permits::BufferPool`/
//! `permits::StreamLimiter`) already is, so [`CpuWorkPool`] reuses it
//! directly rather than hand-rolling a second bounded-counting
//! primitive under a new name. §34's "use separate semaphores/pools when needed" is why [`CpuWorkPools`]
//! gives each [`CpuWorkClass`] its own independent [`CpuWorkPool`] —
//! the same structural-isolation reasoning `queue::BoundedPriorityQueue`
//! already established for priority tiers, applied here to CPU
//! concurrency instead of queue slots.
//!
//! §36 (hashing many large files) and §37 (AV1 software encoding) are
//! not built as two more named types — the spec gives neither any
//! field or behavior beyond "acquire a permit before doing this work,
//! degrade/queue if unavailable," which is exactly what
//! [`CpuWorkPools::acquire`] already provides. A caller doing bulk file
//! hashing sizes a `Bulk`-class pool small (§36's "do not hash 20
//! multi-gig files concurrently"); a caller doing AV1 encode sizes an
//! `Interactive`- or `Critical`-class pool and falls back to a lower
//! quality preset on [`AdmissionResult::Dropped`]/[`AdmissionResult::Deferred`]
//! (§37's "degrade quality if unavailable") — both are usage patterns
//! of the one primitive here, not separate mechanisms.

use crate::admission::{AdmissionResult, DeferredReason, DropReason};
use crate::permits::BoundedCounter;

/// §34, verbatim variant list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CpuWorkClass {
    Critical,
    Interactive,
    Bulk,
    Background,
}

/// A worker semaphore for one [`CpuWorkClass`] (§33/§35: bound how
/// many CPU-heavy/blocking jobs of this class may run at once, so they
/// never flood the async executor's core threads or starve the
/// device). Built on the same internal bounded atomic counter
/// `permits::BufferPool` uses — see this module's own doc comment for
/// why that's reuse, not a coincidence.
#[derive(Debug, Clone)]
pub struct CpuWorkPool {
    class: CpuWorkClass,
    counter: BoundedCounter,
}

impl CpuWorkPool {
    pub fn new(class: CpuWorkClass, max_concurrent: u64) -> Self {
        Self {
            class,
            counter: BoundedCounter::new(max_concurrent),
        }
    }

    pub fn available_slots(&self) -> u64 {
        self.counter.available()
    }

    /// §35: admit one job onto this pool's semaphore, or signal
    /// backpressure — `durable` work (a queued hash job that can just
    /// wait its turn) gets [`AdmissionResult::Deferred`]; non-durable
    /// work (§37's realtime-ish "degrade quality if unavailable" case)
    /// gets [`AdmissionResult::Dropped`], the caller's cue to fall back
    /// to a cheaper encode/skip the work rather than block.
    pub fn acquire(&self, durable: bool) -> Result<CpuJobPermit, AdmissionResult> {
        if self.counter.try_reserve(1) {
            Ok(CpuJobPermit {
                class: self.class,
                counter: self.counter.clone(),
            })
        } else if durable {
            Err(AdmissionResult::Deferred(DeferredReason::AwaitingBudget))
        } else {
            Err(AdmissionResult::Dropped(DropReason::Stale))
        }
    }
}

/// RAII job slot — releases back to its pool on [`Drop`], the same
/// "no manual release()" pattern `permits::MemoryPermit`/
/// `permits::StreamPermit` already established (§39's reasoning
/// applies here too, even though §33-37 don't restate it explicitly:
/// a CPU job that panics mid-hash must not leak its semaphore slot
/// forever).
#[derive(Debug)]
pub struct CpuJobPermit {
    class: CpuWorkClass,
    counter: BoundedCounter,
}

impl CpuJobPermit {
    pub fn class(&self) -> CpuWorkClass {
        self.class
    }
}

impl Drop for CpuJobPermit {
    fn drop(&mut self) {
        self.counter.release(1);
    }
}

/// Per-class concurrency caps (§34's "separate semaphores/pools").
/// No concrete numbers are given anywhere in §33-37's text — every
/// field here is a caller-supplied choice.
/// [`CpuWorkCapacities::conservative_default`] is this module's own
/// reasoned starting point (documented inline), not a spec default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuWorkCapacities {
    pub critical: u64,
    pub interactive: u64,
    pub bulk: u64,
    pub background: u64,
}

impl CpuWorkCapacities {
    /// A deliberately small, conservative default: real-time-adjacent
    /// work (`critical`/`interactive`, e.g. a single AV1 encode for
    /// the call currently in progress) gets just enough room to run
    /// without contending against itself, while `bulk` (§36's file
    /// hashing) and `background` stay tight specifically so they can
    /// never crowd out the device's own responsiveness — not a
    /// measured number, this crate's own reasoned floor for a caller
    /// that hasn't profiled its own workload yet.
    pub fn conservative_default() -> Self {
        Self {
            critical: 2,
            interactive: 2,
            bulk: 1,
            background: 1,
        }
    }
}

/// One independent [`CpuWorkPool`] per [`CpuWorkClass`] — §34's
/// "separate semaphores/pools when needed" as a structural guarantee:
/// a flood of `Bulk` hashing jobs can never starve a `Critical` AV1
/// encode's semaphore, because they are not the same semaphore.
#[derive(Debug, Clone)]
pub struct CpuWorkPools {
    critical: CpuWorkPool,
    interactive: CpuWorkPool,
    bulk: CpuWorkPool,
    background: CpuWorkPool,
}

impl CpuWorkPools {
    pub fn new(capacities: CpuWorkCapacities) -> Self {
        Self {
            critical: CpuWorkPool::new(CpuWorkClass::Critical, capacities.critical),
            interactive: CpuWorkPool::new(CpuWorkClass::Interactive, capacities.interactive),
            bulk: CpuWorkPool::new(CpuWorkClass::Bulk, capacities.bulk),
            background: CpuWorkPool::new(CpuWorkClass::Background, capacities.background),
        }
    }

    fn pool(&self, class: CpuWorkClass) -> &CpuWorkPool {
        match class {
            CpuWorkClass::Critical => &self.critical,
            CpuWorkClass::Interactive => &self.interactive,
            CpuWorkClass::Bulk => &self.bulk,
            CpuWorkClass::Background => &self.background,
        }
    }

    pub fn available_slots(&self, class: CpuWorkClass) -> u64 {
        self.pool(class).available_slots()
    }

    pub fn acquire(
        &self,
        class: CpuWorkClass,
        durable: bool,
    ) -> Result<CpuJobPermit, AdmissionResult> {
        self.pool(class).acquire(durable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquire_within_capacity_succeeds() {
        let pool = CpuWorkPool::new(CpuWorkClass::Bulk, 2);
        let permit = pool.acquire(true).unwrap();
        assert_eq!(permit.class(), CpuWorkClass::Bulk);
        assert_eq!(pool.available_slots(), 1);
    }

    #[test]
    fn dropping_a_job_permit_releases_its_slot() {
        let pool = CpuWorkPool::new(CpuWorkClass::Bulk, 1);
        {
            let _permit = pool.acquire(true).unwrap();
            assert_eq!(pool.available_slots(), 0);
        }
        assert_eq!(pool.available_slots(), 1);
    }

    #[test]
    fn durable_job_past_capacity_defers() {
        let pool = CpuWorkPool::new(CpuWorkClass::Bulk, 1);
        let _held = pool.acquire(true).unwrap();
        assert_eq!(
            pool.acquire(true).unwrap_err(),
            AdmissionResult::Deferred(DeferredReason::AwaitingBudget)
        );
    }

    #[test]
    fn non_durable_job_past_capacity_drops_matching_37s_degrade_case() {
        // §37: "degrade quality if unavailable" — a realtime AV1
        // encode attempt that can't get a slot should be told to
        // degrade/skip, not queue indefinitely.
        let pool = CpuWorkPool::new(CpuWorkClass::Critical, 1);
        let _held = pool.acquire(true).unwrap();
        assert_eq!(
            pool.acquire(false).unwrap_err(),
            AdmissionResult::Dropped(DropReason::Stale)
        );
    }

    #[test]
    fn each_work_class_has_an_independent_semaphore_bulk_full_does_not_block_critical() {
        // §34's whole point.
        let pools = CpuWorkPools::new(CpuWorkCapacities {
            critical: 1,
            interactive: 1,
            bulk: 1,
            background: 1,
        });
        let _bulk_job = pools.acquire(CpuWorkClass::Bulk, true).unwrap();
        assert_eq!(
            pools.acquire(CpuWorkClass::Bulk, true).unwrap_err(),
            AdmissionResult::Deferred(DeferredReason::AwaitingBudget)
        );
        // Critical's own slot is untouched by Bulk being full.
        assert!(pools.acquire(CpuWorkClass::Critical, true).is_ok());
    }

    #[test]
    fn hashing_many_large_files_is_bounded_by_a_small_bulk_pool() {
        // §36's own example: "do not hash 20 multi-gig files
        // concurrently." A pool sized for 3 concurrent hashes must
        // reject the 4th until one finishes.
        let pool = CpuWorkPool::new(CpuWorkClass::Bulk, 3);
        let mut permits = Vec::new();
        for _ in 0..3 {
            permits.push(pool.acquire(true).unwrap());
        }
        assert_eq!(
            pool.acquire(true).unwrap_err(),
            AdmissionResult::Deferred(DeferredReason::AwaitingBudget)
        );

        permits.pop(); // one hash finishes
        assert!(pool.acquire(true).is_ok());
    }
}
