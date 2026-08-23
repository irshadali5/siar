//! §29 "Resume Bitmap", §30 "Range-Based Resume".

/// §29: which chunks (by index) have been durably received, tracked as
/// a real bitset (`Vec<bool>` — simple and correct; a packed `u64`
/// bitset would be more compact but this crate has no size-at-scale
/// requirement forcing that yet) rather than re-deriving completeness
/// from what's on disk every time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeBitmap {
    received: Vec<bool>,
}

impl ResumeBitmap {
    pub fn new(chunk_count: usize) -> Self {
        Self { received: vec![false; chunk_count] }
    }

    pub fn mark_received(&mut self, chunk_index: u32) {
        if let Some(slot) = self.received.get_mut(chunk_index as usize) {
            *slot = true;
        }
    }

    pub fn is_received(&self, chunk_index: u32) -> bool {
        self.received.get(chunk_index as usize).copied().unwrap_or(false)
    }

    pub fn is_complete(&self) -> bool {
        self.received.iter().all(|&r| r)
    }

    pub fn received_count(&self) -> usize {
        self.received.iter().filter(|&&r| r).count()
    }

    /// §30: contiguous runs of missing chunk indices, as `[start, end)`
    /// half-open ranges — what a resume request actually asks a peer
    /// for ("send me chunks 12 through 47"), rather than one request
    /// per missing chunk.
    pub fn missing_ranges(&self) -> Vec<(u32, u32)> {
        let mut ranges = Vec::new();
        let mut run_start: Option<u32> = None;
        for (index, &received) in self.received.iter().enumerate() {
            let index = index as u32;
            if !received {
                if run_start.is_none() {
                    run_start = Some(index);
                }
            } else if let Some(start) = run_start.take() {
                ranges.push((start, index));
            }
        }
        if let Some(start) = run_start {
            ranges.push((start, self.received.len() as u32));
        }
        ranges
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_bitmap_is_entirely_missing() {
        let bitmap = ResumeBitmap::new(5);
        assert!(!bitmap.is_complete());
        assert_eq!(bitmap.missing_ranges(), vec![(0, 5)]);
    }

    #[test]
    fn marking_every_chunk_received_completes_the_bitmap() {
        let mut bitmap = ResumeBitmap::new(3);
        bitmap.mark_received(0);
        bitmap.mark_received(1);
        bitmap.mark_received(2);
        assert!(bitmap.is_complete());
        assert!(bitmap.missing_ranges().is_empty());
    }

    #[test]
    fn missing_ranges_groups_contiguous_gaps() {
        let mut bitmap = ResumeBitmap::new(10);
        for i in [0, 1, 2, 5, 6, 9] {
            bitmap.mark_received(i);
        }
        // received: 0,1,2,_,_,5,6,_,_,9 → missing: [3,5), [7,9)
        assert_eq!(bitmap.missing_ranges(), vec![(3, 5), (7, 9)]);
        assert_eq!(bitmap.received_count(), 6);
    }
}
