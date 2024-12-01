pub mod inline {
    use std::mem::MaybeUninit;

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub enum QueueError {
        QueueEmpty,
        QueueFull,
    }

    // Not sure if it's possible to use the more permissive Clone trait here...
    #[derive(Debug, Copy, Clone)]
    pub struct Queue<T: Copy, const CAPACITY: usize> {
        size: usize,
        head: usize,
        tail: usize,
        buffer: [MaybeUninit<T>; CAPACITY],
    }

    impl<T: Copy, const CAPACITY: usize> Queue<T, CAPACITY> {
        /// Create a new inline queue for the specified type and of the specified capacity.
        pub fn new() -> Queue<T, CAPACITY> {
            unsafe {
                Queue {
                    size: 0,
                    head: 0,
                    tail: 0,
                    buffer: [MaybeUninit::uninit().assume_init(); CAPACITY],
                }
            }
        }

        /// Push an element to the queue. Returns None or QueueError::QueueFull.
        pub fn push(&mut self, input: T) -> Result<(), QueueError> {
            self.push_ref(&input)
        }

        /// Push an element reference to the queue. Returns None or QueueError::QueueFull.
        ///
        /// This may be preferable over `push()` for types that are expensive to copy, as it
        /// eliminates the extra copy incurred by passing by value.
        pub fn push_ref(&mut self, input: &T) -> Result<(), QueueError> {
            if self.full() {
                return Err(QueueError::QueueFull);
            }

            unsafe {
                *(self.buffer[self.tail].as_mut_ptr()) = *input;
            }

            self.tail = (self.tail + 1) % CAPACITY;
            self.size += 1;

            Ok(())
        }

        /// Pop an element from the queue. Returns the element or QueueError::QueueEmpty.
        pub fn pop(&mut self) -> Result<T, QueueError> {
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

        /// Pop an element reference from the queue. Returns the element or QueueError::QueueEmpty.
        ///
        /// This may be preferable over `pop()` for types that are expensive to copy, as it
        /// eliminates the extra copy incurred by returning the result by value.
        pub fn pop_ref(&mut self, output: &mut T) -> Result<(), QueueError> {
                if self.empty() {
                return Err(QueueError::QueueEmpty);
            }

            *output = unsafe { *(self.buffer[self.head].as_mut_ptr()) };
            self.head = (self.head + 1) % CAPACITY;
            self.size -= 1;

            Ok(())
        }

        /// Check if the queue is full.
        pub fn full(&self) -> bool {
            self.size() == CAPACITY
        }

        /// Check if the queue is empty.
        pub fn empty(&self) -> bool {
            self.size() == 0
        }

        /// Get the current number of elements in the queue.
        pub fn size(&self) -> usize {
            self.size
        }
    }

    impl<T: Copy, const CAPACITY: usize> Default for Queue<T, CAPACITY> {
        fn default() -> Self {
            Queue::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::inline::Queue;
    use super::inline::QueueError;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn new() {
        let queue = Queue::<u32, 16>::new();
        println!("{:?}", queue);
    }

    #[test]
    fn default() {
        let queue = Queue::<u32, 16>::default();
        println!("{:?}", queue);
    }

    #[test]
    fn push_pop() {
        const SIZE: usize = 16;
        let mut queue = Queue::<u32, SIZE>::default();

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
        const SIZE: usize = 16;
        let mut queue = Queue::<u32, SIZE>::default();

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
    fn size() {
        const SIZE: usize = 16;
        let mut queue = Queue::<u32, SIZE>::default();

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
    fn empty_full() {
        const SIZE: usize = 16;
        let mut queue = Queue::<u32, SIZE>::default();
        assert!(queue.empty());

        for n in 0..SIZE {
            assert!(!queue.full());
            assert!(queue.push(n as u32).is_ok());
            assert!(!queue.empty());
        }

        assert!(queue.full());
        let res = queue.push(0);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), QueueError::QueueFull);

        for _ in 0..SIZE {
            assert!(!queue.empty());
            assert!(queue.pop().is_ok());
            assert!(!queue.full());
        }

        let output = queue.pop();
        assert!(output.is_err());
        assert_eq!(output.unwrap_err(), QueueError::QueueEmpty);
    }

    #[test]
    fn wrap() {
        const SIZE: usize = 16;
        let mut queue = Queue::<u32, SIZE>::default();

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
}
