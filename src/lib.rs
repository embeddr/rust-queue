pub mod queue {
    use std::mem::MaybeUninit;

    // TODO: Define Queue trait and then try multiple implementations?

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub enum QueueError {
        QueueEmpty,
        QueueFull,
    }

    // Note: Not sure if it's possible to use the more permissive Clone trait here...
    #[derive(Debug)]
    pub struct Queue<T: Copy, const CAPACITY: usize> {
        size: usize,
        head: usize,
        tail: usize,
        buffer: [MaybeUninit<T>; CAPACITY],
    }

    impl<T: Copy, const CAPACITY: usize> Queue<T, CAPACITY> {
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

        pub fn push(&mut self, input: &T) -> Result<(), QueueError> {
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

        // TODO: Create reference versions of pop and push
        pub fn pop(&mut self) -> Result<T, QueueError> {
            if self.empty() {
                return Err(QueueError::QueueEmpty);
            }

            let out_ptr = self.buffer[self.head].as_mut_ptr();
            self.head = (self.head + 1) % CAPACITY;
            self.size -= 1;

            Ok(unsafe {*out_ptr})
        }

        pub fn full(&self) -> bool {
            self.size() == CAPACITY
        }

        pub fn empty(&self) -> bool {
            self.size() == 0 
        }

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
    use super::queue::*;

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
    fn enqueue_dequeue() {
        const SIZE: usize = 16;
        let mut queue = Queue::<u32, SIZE>::default();

        for n in 0..SIZE {
            assert!(queue.push(&(n as u32)).is_ok())
        }

        for n in 0..SIZE {
            let output = queue.pop();
            assert!(output.is_ok());
            assert_eq!(output.unwrap(), n as u32);
        }
    }

    #[test]
    fn size() {
        const SIZE: usize = 16;
        let mut queue = Queue::<u32, SIZE>::default();

        for n in 0..SIZE {
            assert_eq!(n, queue.size());
            assert!(queue.push(&(n as u32)).is_ok());
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
            assert!(queue.push(&(n as u32)).is_ok());
            assert!(!queue.empty());
        }

        assert!(queue.full());
        let res = queue.push(&0);
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

        // Enqueue/dequeue half capacity to move head/tail to SIZE/2
        for n in 0..SIZE/2 {
            assert!(queue.push(&(n as u32)).is_ok());
        }

        for _ in 0..SIZE/2 {
            assert!(queue.pop().is_ok());
        }

        // Now perform full enqueue/dequeue check to verify wrap logic 
        for n in 0..SIZE {
            assert!(queue.push(&(n as u32)).is_ok())
        }

        for n in 0..SIZE {
            let output = queue.pop();
            assert!(output.is_ok());
            assert_eq!(output.unwrap(), n as u32);
        }
    }

}
