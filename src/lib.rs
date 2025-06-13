use std::{collections::VecDeque, fmt::Debug};

#[derive(Clone, Debug)]
struct BufferElement<F: PartialOrd + Clone + Debug> {
    index: usize,
    value: F,
}

/// Data structure to keep track of the max value over a ring buffer.
/// Extension of a deque but for a new entry it will:
///   - Remove all elements that are now outside of the window.
///   - Remove all elements less than the new entry in value.
///   - Return the highest value.
///
/// This keeps the deque sorted and set to only the buffer giving
/// efficiently returning of the max value.
/// If two values are equal in their ordering, the newest value will be kept.
#[derive(Clone, Debug)]
pub struct MaxDetector<F: PartialOrd + Clone + Debug> {
    deque: VecDeque<BufferElement<F>>,
    buffer_size: usize,
    next_index: usize,
}

impl<F: PartialOrd + Clone + Debug> MaxDetector<F> {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            buffer_size,
            deque: VecDeque::default(),
            next_index: 0,
        }
    }

    /// Add new element to buffer and return highest value.
    pub fn next(&mut self, value: F) -> F {
        let deque = &mut self.deque;
        let buffer_size = self.buffer_size;
        let next_index = self.next_index;
        // Remove values no longer in the buffer.
        // An element will only stay in the buffer long enough to require removal if its value is
        // the max value.
        // Therefore we only need to check the max value element (back of queue).
        if deque.back().map(|it| it.index) == Some(next_index) {
            deque.pop_back();
        }
        if deque.is_empty() {
            deque.push_back(BufferElement {
                index: next_index,
                value,
            });
        } else if deque.back().unwrap().value <= value {
            // New value is larger than max value.
            // Remove all other elements.
            deque.clear();
            deque.push_back(BufferElement {
                index: next_index,
                value,
            });
        } else {
            // Add element to queue from left.
            // Remove all elements with a value less than or equal to this entry.
            // This is okay as this value is larger and newer.
            // This also keeps the queue sorted and only retaining relevant elements.
            while value >= deque.front().unwrap().value {
                deque.pop_front();
            }
            deque.push_front(BufferElement {
                index: next_index,
                value,
            });
        }
        // Update next index in ring buffer.
        self.next_index = (next_index + 1) % buffer_size;
        // Return max value.
        deque.back().unwrap().value.to_owned()
    }

    /// Get current max value in buffer.
    pub fn current(&self) -> Option<F> {
        let value = self.deque.back()?;
        Some(value.value.to_owned())
    }
}

#[cfg(test)]
mod test {
    use super::MaxDetector;

    #[derive(PartialEq, Debug, Clone)]
    struct StringStruct {
        value: String,
    }

    impl PartialOrd for StringStruct {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.value.len().cmp(&other.value.len()))
        }
    }

    #[test]
    fn tracks_max_ascending_list() {
        let array = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5];
        let mut detector = MaxDetector::new(10);
        let detected_maxes = array.map(|it| detector.next(it));
        assert_eq!(detected_maxes, array);
    }

    #[test]
    fn tracks_max_descending_list() {
        let array = [0.5, 0.4, 0.3, 0.2, 0.1];
        let mut detector = MaxDetector::new(10);
        let detected_maxes = array.map(|it| detector.next(it));
        assert_eq!(detected_maxes, [0.5; 5])
    }

    #[test]
    fn max_outside_of_buffer_is_removed() {
        let array = [0.5, 0.4, 0.3, 0.2, 0.1];
        let mut detector = MaxDetector::new(4);
        for value in array {
            detector.next(value);
        }
        let expected = Some(0.4);
        assert_eq!(detector.current(), expected)
    }

    #[test]
    fn detector_handles_array_larger_than_buffer() {
        let array = [0.8, 0.1, 0.3, 0.2, 0.1, 0.6, 0.2];
        let mut detector = MaxDetector::new(4);
        for value in array {
            detector.next(value);
        }
        let expected = Some(0.6);
        assert_eq!(detector.current(), expected)
    }

    #[test]
    fn detector_handles_usize() {
        let array = [2, 5, 1, 0, 2, 3, 1, 0];
        let mut detector = MaxDetector::<usize>::new(4);
        for value in array {
            detector.next(value);
        }
        let expected = Some(3);
        assert_eq!(detector.current(), expected)
    }

    #[test]
    fn detector_handles_negatives() {
        let array = [2, -5, 1, 0, -2, 3, -1, 0];
        let mut detector = MaxDetector::<isize>::new(4);
        for value in array {
            detector.next(value);
        }
        let expected = Some(3);
        assert_eq!(detector.current(), expected)
    }

    #[test]
    fn new_max_is_detected() {
        let array = [0.5, 0.0, 0.1, 0.0, 0.8];
        let mut detector = MaxDetector::new(10);
        for value in array {
            detector.next(value);
        }
        let expected = Some(0.8);
        assert_eq!(detector.current(), expected);
    }

    #[test]
    fn empty_buffer_returns_none() {
        let detector = MaxDetector::<f32>::new(10);
        assert_eq!(detector.current(), None);
    }

    #[test]
    fn detector_handles_custom_type() {
        let values = [
            StringStruct {
                value: "abc".into(),
            },
            StringStruct { value: "a".into() },
            StringStruct { value: "ab".into() },
        ];

        let mut detector = MaxDetector::new(5);
        for value in values {
            detector.next(value);
        }
        let expected = Some(StringStruct {
            value: "abc".into(),
        });
        assert_eq!(detector.current(), expected);
    }

    #[test]
    fn detector_keeps_newest_of_equal_values() {
        let values = [
            StringStruct {
                value: "abc".into(),
            },
            StringStruct { value: "a".into() },
            StringStruct {
                value: "def".into(),
            },
        ];

        let mut detector = MaxDetector::new(5);
        for value in values {
            detector.next(value);
        }
        let expected = Some(StringStruct {
            value: "def".into(),
        });
        assert_eq!(detector.current(), expected);
    }
}
