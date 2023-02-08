pub mod queue {
    #[derive(Debug)]
    // TODO: Probably don't actually need Default...
    pub struct Queue<T: Default + Copy, const CAPACITY: usize> {
        size: usize,
        head: usize,
        tail: usize,
        buffer: [T; CAPACITY],
    }

    impl<T: Default + Copy, const CAPACITY: usize> Queue<T, CAPACITY> {
        pub fn new() -> Queue<T, CAPACITY> {
            Queue {
                size: 0,
                head: 0,
                tail: 0,
                buffer: [T::default(); CAPACITY],
            }
        }

        pub fn push(&mut self, input: &T) -> bool {
            if self.full() {
                return false;
            }

            self.buffer[self.tail] = *input;
            self.tail = (self.tail + 1) % CAPACITY;
            self.size += 1;
            true
        }

        pub fn pop(&mut self, output: &mut T) -> bool {
            if self.empty() {
                return false;
            }

            *output = self.buffer[self.head];
            self.head = (self.head + 1) % CAPACITY;
            self.size -= 1;
            true
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

    // This is super verbose... how can it be simplified? Grouped with the above?
    impl<T: Default + Copy, const CAPACITY: usize> Default for Queue<T, CAPACITY> {
        fn default() -> Queue<T, CAPACITY> {
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
            assert!(queue.push(&(n as u32)))
        }

        for n in 0..SIZE {
            let mut output: u32 = 0;
            assert!(queue.pop(&mut output));
            assert_eq!(output, n as u32);
        }
    }

    #[test]
    fn size() {
        const SIZE: usize = 16;
        let mut queue = Queue::<u32, SIZE>::default();

        for n in 0..SIZE {
            assert_eq!(n, queue.size());
            queue.push(&(n as u32));
        }

        for n in 0..SIZE {
            let mut output: u32 = 0;
            assert_eq!((SIZE - n), queue.size());
            queue.pop(&mut output);
        }
    }

    #[test]
    fn empty_full() {
        const SIZE: usize = 16;
        let mut queue = Queue::<u32, SIZE>::default();
        assert!(queue.empty());

        for n in 0..SIZE {
            assert!(!queue.full());
            queue.push(&(n as u32));
            assert!(!queue.empty());
        }

        assert!(queue.full());
        assert!(!queue.push(&0));

        for _ in 0..SIZE {
            assert!(!queue.empty());
            let mut output: u32 = 0;
            queue.pop(&mut output);
            assert!(!queue.full());
        }

        let mut output: u32 = 0;
        assert!(!queue.pop(&mut output));
        assert!(queue.empty());
    }

    #[test]
    fn wrap() {
        const SIZE: usize = 16;
        let mut queue = Queue::<u32, SIZE>::default();

        // Enqueue/dequeue half capacity to move head/tail to SIZE/2
        for n in 0..SIZE/2 {
            queue.push(&(n as u32));
        }

        for _ in 0..SIZE/2 {
            let mut output: u32 = 0;
            queue.pop(&mut output);
        }

        // Now perform full enqueue/dequeue check to verify wrap logic 
        for n in 0..SIZE {
            assert!(queue.push(&(n as u32)))
        }

        for n in 0..SIZE {
            let mut output: u32 = 0;
            assert!(queue.pop(&mut output));
            assert_eq!(output, n as u32);
        }
    }

}
