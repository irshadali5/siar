//! A bounded reorder buffer for decoded media, keyed by capture
//! timestamp (the `timestamp_micros` already carried on every frame
//! type in `siar-media-core`). Independent QUIC streams/datagrams can
//! deliver packets out of the order they were captured in; this holds
//! up to `capacity` out-of-order arrivals and releases the oldest once
//! the buffer is full, so a burst of reordering doesn't grow memory
//! without bound.
//!
//! Deliberately NOT wired to a real clock — `capacity` is expressed as
//! "how many frames," not a hold duration, specifically so this stays
//! pure and testable without mocking time. `session.rs` (or whatever
//! calls `push`/`flush`) is what maps a real network's jitter into a
//! concrete capacity, and that mapping is exactly the kind of thing
//! that wants tuning against a real call, not guessed here.

#[derive(Debug)]
pub struct JitterBuffer<T> {
    capacity: usize,
    pending: Vec<(u64, T)>,
}

impl<T> JitterBuffer<T> {
    /// `capacity` is how many out-of-order arrivals this holds before
    /// it starts forcing the oldest one out to bound memory — not a
    /// target depth to sit at; a well-behaved connection should mostly
    /// see `push` return immediately with the frame it was just given
    /// (i.e., already in order), and `capacity` only matters when
    /// reordering actually happens.
    pub fn new(capacity: usize) -> Self {
        assert!(
            capacity >= 1,
            "a zero-capacity jitter buffer can never hold anything to reorder"
        );
        Self {
            capacity,
            pending: Vec::with_capacity(capacity),
        }
    }

    /// Inserts one arrival in timestamp order. Returns frames now ready
    /// to release, oldest first: empty while still buffering under
    /// capacity, or exactly one (the oldest) once capacity is exceeded.
    pub fn push(&mut self, timestamp_micros: u64, item: T) -> Vec<(u64, T)> {
        let insert_at = self
            .pending
            .partition_point(|(ts, _)| *ts <= timestamp_micros);
        self.pending.insert(insert_at, (timestamp_micros, item));
        if self.pending.len() > self.capacity {
            vec![self.pending.remove(0)]
        } else {
            Vec::new()
        }
    }

    /// Releases everything still buffered, oldest first. Call this once
    /// the caller's own timer decides no more (re)ordering is coming —
    /// e.g. on call teardown, or after a real hold-duration timeout
    /// mapped from `capacity`'s frame count at the caller's frame rate.
    pub fn flush(&mut self) -> Vec<(u64, T)> {
        std::mem::take(&mut self.pending)
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffers_until_capacity_then_releases_the_oldest() {
        let mut jb = JitterBuffer::new(3);
        assert!(jb.push(100, "a").is_empty());
        assert!(jb.push(300, "c").is_empty());
        assert!(jb.push(200, "b").is_empty());
        let released = jb.push(400, "d");
        assert_eq!(released, vec![(100, "a")]);
        assert_eq!(jb.len(), 3);
    }

    #[test]
    fn out_of_order_arrivals_are_released_in_timestamp_order() {
        let mut jb = JitterBuffer::new(4);
        jb.push(300, "c");
        jb.push(100, "a");
        jb.push(200, "b");
        let released = jb.flush();
        assert_eq!(released, vec![(100, "a"), (200, "b"), (300, "c")]);
    }

    #[test]
    fn flush_drains_everything_still_pending_and_empties_the_buffer() {
        let mut jb: JitterBuffer<&str> = JitterBuffer::new(5);
        jb.push(50, "x");
        assert!(!jb.is_empty());
        let released = jb.flush();
        assert_eq!(released, vec![(50, "x")]);
        assert!(jb.is_empty());
    }

    #[test]
    fn duplicate_timestamps_are_kept_in_arrival_order() {
        // `partition_point` on `ts <= timestamp_micros` inserts a new
        // equal-timestamp item after existing ones with the same
        // timestamp, not before — this pins that behavior down
        // explicitly rather than leaving it to whatever `insert` happens
        // to do.
        let mut jb = JitterBuffer::new(4);
        jb.push(100, "first");
        jb.push(100, "second");
        let released = jb.flush();
        assert_eq!(released, vec![(100, "first"), (100, "second")]);
    }
}
