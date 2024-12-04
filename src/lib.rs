pub mod inline {
    use std::mem::MaybeUninit;

    // TODO: Add iterator
    // TOOD: Add overwrite arg to push fxns

    // Trait for a queue that works with a fixed type as specified by the generic parameter. The
    // queue fails to push when full, rather than overwriting the oldest element.
    pub trait TypedQueue<T: Copy> {
        /// Push an element to the queue by value. Returns None if successful, or else
        /// QueueError::QueueFull.
        fn push(&mut self, input: T) -> Result<(), QueueError>;

        /// Push an element to the queue by reference. Returns None if successful, or else
        /// QueueError::QueueFull.
        ///
        /// This may be preferable over `push()` for types that are expensive to copy, as it
        /// eliminates the extra copy incurred by passing by value.
        fn push_ref(&mut self, input: &T) -> Result<(), QueueError>;

        /// Pop an element from the queue by value. Returns the element if successful, or else
        /// QueueError::QueueEmpty.
        fn pop(&mut self) -> Result<T, QueueError>;

        /// Pop an element from the queue by reference. Returns the element if successful, or else
        /// QueueError::QueueEmpty.
        ///
        /// This may be preferable over `pop()` for types that are expensive to copy, as it
        /// eliminates the extra copy incurred by returning the result by value.
        fn pop_ref(&mut self, output: &mut T) -> Result<(), QueueError>;

        fn front(&mut self) -> Result<&T, QueueError>;

        fn back(&mut self) -> Result<&T, QueueError>;

        /// Check if the queue is full.
        fn is_full(&self) -> bool;

        /// Check if the queue is empty.
        fn is_empty(&self) -> bool;

        /// Get the current number of elements in the queue.
        fn size(&self) -> usize;

        // Get the maximum number of elements the queue can hold.
        fn capacity(&self) -> usize;
    }

    /// Enum indicating why a queue operation failed.
    #[derive(Debug, Clone, Eq, PartialEq)]
    pub enum QueueError {
        /// The pop operation has failed due to the queue being empty.
        QueueEmpty,
        /// The push operation has failed due to the queue being full.
        QueueFull,
    }

    // Basic typed queue struct. The implementation for this struct is NOT thread-safe.
    #[derive(Debug, Copy, Clone)]
    pub struct BasicTypedQueue<T: Copy, const CAPACITY: usize> {
        size: usize,
        head: usize,
        tail: usize,
        buffer: [MaybeUninit<T>; CAPACITY],
    }

    impl<T: Copy, const CAPACITY: usize> BasicTypedQueue<T, CAPACITY> {
        /// Create a new inline queue for the specified type and of the specified capacity.
        pub fn new() -> BasicTypedQueue<T, CAPACITY> {
            BasicTypedQueue {
                size: 0,
                head: 0,
                tail: 0,
                buffer: [MaybeUninit::uninit(); CAPACITY],
            }
        }
    }

    impl<T: Copy, const CAPACITY: usize> Default for BasicTypedQueue<T, CAPACITY> {
        fn default() -> Self {
            BasicTypedQueue::new()
        }
    }

    impl<T: Copy, const CAPACITY: usize> TypedQueue<T> for BasicTypedQueue<T, CAPACITY> {
        fn push(&mut self, input: T) -> Result<(), QueueError> {
            self.push_ref(&input)
        }

        fn push_ref(&mut self, input: &T) -> Result<(), QueueError> {
            if self.is_full() {
                return Err(QueueError::QueueFull);
            }

            unsafe {
                *(self.buffer[self.tail].as_mut_ptr()) = *input;
            }

            self.tail = (self.tail + 1) % CAPACITY;
            self.size += 1;

            Ok(())
        }

        fn pop(&mut self) -> Result<T, QueueError> {
            let mut value = MaybeUninit::<T>::uninit();
            // We can safely pass the uninit value into `pop_ref()` because we know `pop_ref()` will
            // not read the value, only copy into it. If `pop_ref()` fails, we never access `value`.
            unsafe {
                match self.pop_ref(value.assume_init_mut()) {
                    Ok(()) => Ok(value.assume_init()),
                    Err(e) => Err(e),
                }
            }
        }

        fn pop_ref(&mut self, output: &mut T) -> Result<(), QueueError> {
            if self.is_empty() {
                return Err(QueueError::QueueEmpty);
            }

            *output = unsafe { *(self.buffer[self.head].as_mut_ptr()) };
            self.head = (self.head + 1) % CAPACITY;
            self.size -= 1;

            Ok(())
        }

        fn front(&mut self) -> Result<&T, QueueError> {
            if self.is_empty() {
                return Err(QueueError::QueueEmpty);
            }

            Ok(unsafe { self.buffer[self.head].assume_init_ref() })
        }

        fn back(&mut self) -> Result<&T, QueueError> {
            if self.is_empty() {
                return Err(QueueError::QueueEmpty);
            }

            let back_idx = (self.tail + CAPACITY - 1) % CAPACITY;
            Ok(unsafe { self.buffer[back_idx].assume_init_ref() })
        }

        fn is_full(&self) -> bool {
            self.size() == CAPACITY
        }

        fn is_empty(&self) -> bool {
            self.size() == 0
        }

        fn size(&self) -> usize {
            self.size
        }

        fn capacity(&self) -> usize {
            CAPACITY
        }
    }
}

#[cfg(test)]
mod tests {
    use super::inline::{BasicTypedQueue, QueueError, TypedQueue};

    // Arbitrary queue size for tests
    const SIZE: usize = 16;

    #[test]
    fn new() {
        let queue = BasicTypedQueue::<u32, SIZE>::new();
        println!("{:?}", queue);
    }

    #[test]
    fn default() {
        let queue = BasicTypedQueue::<u32, SIZE>::default();
        println!("{:?}", queue);
    }

    #[test]
    fn push_pop() {
        let mut queue = BasicTypedQueue::<u32, SIZE>::default();

        for n in 0..SIZE {
            assert!(queue.push(n as u32).is_ok())
        }

        for n in 0..SIZE {
            let output = queue.pop();
            assert!(output.is_ok());
            assert_eq!(output.unwrap(), n as u32);
        }
    }

    #[test]
    fn push_ref_pop_ref() {
        let mut queue = BasicTypedQueue::<u32, SIZE>::default();

        for n in 0..SIZE {
            assert!(queue.push_ref(&(n as u32)).is_ok())
        }

        for n in 0..SIZE {
            let mut output: u32 = 0;
            let res = queue.pop_ref(&mut output);
            assert!(res.is_ok());
            assert_eq!(output, n as u32);
        }
    }

    #[test]
    fn wrap() {
        let mut queue = BasicTypedQueue::<u32, SIZE>::default();

        // push/pop half capacity to move head/tail to SIZE/2
        for n in 0..SIZE / 2 {
            assert!(queue.push(n as u32).is_ok());
        }

        for _ in 0..SIZE / 2 {
            assert!(queue.pop().is_ok());
        }

        // Now perform full push/pop check to verify wrap logic
        for n in 0..SIZE {
            assert!(queue.push(n as u32).is_ok())
        }

        for n in 0..SIZE {
            let output = queue.pop();
            assert!(output.is_ok());
            assert_eq!(output.unwrap(), n as u32);
        }
    }

    #[test]
    fn front_back() {
        let mut queue = BasicTypedQueue::<u32, SIZE>::default();
        assert_eq!(queue.front().unwrap_err(), QueueError::QueueEmpty);
        assert_eq!(queue.back().unwrap_err(), QueueError::QueueEmpty);

        const VALUE: u32 = 123;
        assert!(queue.push(VALUE).is_ok());
        assert_eq!(*queue.front().unwrap(), VALUE);
        assert_eq!(*queue.back().unwrap(), VALUE);
    }

    #[test]
    fn empty_full() {
        let mut queue = BasicTypedQueue::<u32, SIZE>::default();
        assert!(queue.is_empty());

        for n in 0..SIZE {
            assert!(!queue.is_full());
            assert!(queue.push(n as u32).is_ok());
            assert!(!queue.is_empty());
        }

        assert!(queue.is_full());
        let res = queue.push(0);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), QueueError::QueueFull);

        for _ in 0..SIZE {
            assert!(!queue.is_empty());
            assert!(queue.pop().is_ok());
            assert!(!queue.is_full());
        }

        let output = queue.pop();
        assert!(output.is_err());
        assert_eq!(output.unwrap_err(), QueueError::QueueEmpty);
    }

    #[test]
    fn size() {
        let mut queue = BasicTypedQueue::<u32, SIZE>::default();

        for n in 0..SIZE {
            assert_eq!(n, queue.size());
            assert!(queue.push(n as u32).is_ok());
        }

        for n in 0..SIZE {
            assert_eq!((SIZE - n), queue.size());
            assert!(queue.pop().is_ok());
        }
    }

    #[test]
    fn capacity() {
        let queue = BasicTypedQueue::<u32, SIZE>::default();
        assert_eq!(queue.capacity(), SIZE);

        let smaller_queue = BasicTypedQueue::<u32, { SIZE - 1 }>::default();
        assert_eq!(smaller_queue.capacity(), SIZE - 1);
    }
}
