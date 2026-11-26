use std::collections::VecDeque;
use std::time::{Duration, Instant};

const WINDOW: Duration = Duration::from_millis(500);
const SPREAD_KEYS: usize = 9;
const PILE_KEYS: usize = 4;
const HOLD_LIMIT: Duration = Duration::from_millis(1500);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Down(u32),
    Up(u32),
    Overflow,
}

#[derive(Default)]
pub struct Detector {
    recent: VecDeque<(Instant, u32)>,
    held: Vec<(u32, Instant)>,
}

impl Detector {
    pub fn push(&mut self, now: Instant, edge: Edge) -> bool {
        self.held.retain(|&(_, at)| now.saturating_duration_since(at) <= HOLD_LIMIT);
        let key = match edge {
            Edge::Up(key) => {
                self.held.retain(|&(k, _)| k != key);
                return false;
            }
            Edge::Overflow => return self.fire(),
            Edge::Down(key) => key,
        };
        if !self.held.iter().any(|&(k, _)| k == key) {
            self.held.push((key, now));
        }
        self.recent.push_back((now, key));
        while let Some(&(at, _)) = self.recent.front() {
            if now.saturating_duration_since(at) > WINDOW {
                self.recent.pop_front();
            } else {
                break;
            }
        }
        if self.held.len() >= PILE_KEYS || self.distinct_recent() >= SPREAD_KEYS {
            return self.fire();
        }
        false
    }

    fn distinct_recent(&self) -> usize {
        let mut keys: Vec<u32> = self.recent.iter().map(|&(_, k)| k).collect();
        keys.sort_unstable();
        keys.dedup();
        keys.len()
    }

    fn fire(&mut self) -> bool {
        self.recent.clear();
        self.held.clear();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Run {
        d: Detector,
        start: Instant,
        hits: usize,
    }

    impl Run {
        fn new() -> Self {
            Run { d: Detector::default(), start: Instant::now(), hits: 0 }
        }
        fn at(&mut self, ms: u64, edge: Edge) {
            if self.d.push(self.start + Duration::from_millis(ms), edge) {
                self.hits += 1;
            }
        }
        fn tap(&mut self, ms: u64, key: u32, hold_ms: u64) {
            self.at(ms, Edge::Down(key));
            self.at(ms + hold_ms, Edge::Up(key));
        }
    }

    fn typing(keys: &[u32], gap_ms: u64, hold_ms: u64) -> usize {
        let mut events: Vec<(u64, Edge)> = Vec::new();
        for (i, k) in keys.iter().enumerate() {
            let at = i as u64 * gap_ms;
            events.push((at, Edge::Down(*k)));
            events.push((at + hold_ms, Edge::Up(*k)));
        }
        events.sort_by_key(|(at, e)| (*at, matches!(e, Edge::Down(_))));
        let mut run = Run::new();
        for (at, e) in events {
            run.at(at, e);
        }
        run.hits
    }

    #[test]
    fn a_cloth_pressing_four_keys_at_once_is_detected() {
        let mut run = Run::new();
        for (i, key) in [10, 11, 12, 13].into_iter().enumerate() {
            run.at(i as u64 * 15, Edge::Down(key));
        }
        assert_eq!(run.hits, 1);
    }

    #[test]
    fn a_slow_wipe_holding_a_few_keys_at_a_time_is_detected() {
        let mut run = Run::new();
        for step in 0..6u32 {
            let at = step as u64 * 120;
            for key in step * 3..step * 3 + 3 {
                run.at(at, Edge::Down(key));
            }
            for key in step * 3..step * 3 + 3 {
                run.at(at + 100, Edge::Up(key));
            }
        }
        assert!(run.hits >= 1);
    }

    #[test]
    fn a_fast_spread_of_single_presses_is_detected() {
        let keys: Vec<u32> = (0..12).collect();
        assert!(typing(&keys, 30, 20) >= 1);
    }

    #[test]
    fn the_keyboard_reporting_too_many_keys_is_detected() {
        let mut run = Run::new();
        run.at(0, Edge::Overflow);
        assert_eq!(run.hits, 1);
    }

    #[test]
    fn fast_typing_with_rollover_is_not_detected() {
        let keys: Vec<u32> = (0..80).map(|i| (i * 7) % 26).collect();
        assert_eq!(typing(&keys, 70, 150), 0);
    }

    #[test]
    fn very_fast_typing_is_not_detected() {
        let keys: Vec<u32> = (0..80).map(|i| (i * 5) % 26).collect();
        assert_eq!(typing(&keys, 65, 110), 0);
    }

    #[test]
    fn mashing_one_key_is_not_detected() {
        assert_eq!(typing(&[7; 60], 20, 10), 0);
    }

    #[test]
    fn slow_presses_are_not_detected() {
        let keys: Vec<u32> = (0..30).collect();
        assert_eq!(typing(&keys, 400, 100), 0);
    }

    #[test]
    fn three_keys_held_together_is_not_detected() {
        let mut run = Run::new();
        for key in [13, 0, 49] {
            run.at(0, Edge::Down(key));
        }
        for key in [13, 0, 49] {
            run.at(800, Edge::Up(key));
        }
        assert_eq!(run.hits, 0);
    }

    #[test]
    fn a_key_whose_release_was_missed_is_forgotten() {
        let mut run = Run::new();
        run.at(0, Edge::Down(1));
        run.at(0, Edge::Down(2));
        run.at(0, Edge::Down(3));
        run.at(2000, Edge::Down(4));
        run.at(2000, Edge::Down(5));
        assert_eq!(run.hits, 0);
    }

    #[test]
    fn released_keys_do_not_count_towards_a_pile() {
        let mut run = Run::new();
        for (i, key) in [1, 2, 3, 4, 5, 6].into_iter().enumerate() {
            run.tap(i as u64 * 200, key, 50);
        }
        assert_eq!(run.hits, 0);
    }

    #[test]
    fn a_detection_starts_the_count_again() {
        let mut run = Run::new();
        for key in 0..8u32 {
            run.at(key as u64 * 10, Edge::Down(key));
        }
        assert_eq!(run.hits, 2);
    }
}
