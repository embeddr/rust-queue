use std::cmp::min;
use std::fmt;
use std::mem::MaybeUninit;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex, MutexGuard,
};

use crate::typed_queue::{QueueError, TypedQueue};

// Queue data to be protected via mutex
struct QueueData<T: Copy, const CAPACITY: usize> {
    head: usize,
    tail: usize,
    buffer: [MaybeUninit<T>; CAPACITY],
}

impl<T: Copy, const CAPACITY: usize> Default for QueueData<T, CAPACITY> {
    fn default() -> Self {
        QueueData {
            head: 0,
            tail: 0,
            buffer: [MaybeUninit::uninit(); CAPACITY],
        }
    }
}

// Wrapper providing immutable reference to element in container. Holds a lock until dropped.
pub struct RefGuard<'a, T: Copy, const CAPACITY: usize> {
    guard: MutexGuard<'a, QueueData<T, CAPACITY>>,
    index: usize,
}

impl<'a, T: Copy, const CAPACITY: usize> RefGuard<'a, T, CAPACITY> {
    fn new(guard: MutexGuard<'a, QueueData<T, CAPACITY>>, index: usize) -> Self {
        Self { guard, index }
    }
}

impl<'a, T: Copy, const CAPACITY: usize> std::ops::Deref for RefGuard<'a, T, CAPACITY> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.guard.buffer[self.index].assume_init_ref() }
    }
}

impl<'a, T: Copy + fmt::Debug, const CAPACITY: usize> fmt::Debug for RefGuard<'a, T, CAPACITY> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.guard.buffer[self.index], f)
    }
}

// Thread-safe typed queue struct with generic capacity.
pub struct ThreadSafeTypedQueue<T: Copy, const CAPACITY: usize> {
    // Size is stored as an atomic separately from protected_data so that it can be read without
    // needing to acquire a lock. This speeds up functions like size() and related.
    size: AtomicUsize,
    protected_data: Mutex<QueueData<T, CAPACITY>>,
}

impl<T: Copy, const CAPACITY: usize> ThreadSafeTypedQueue<T, CAPACITY> {
    pub fn new() -> Self {
        ThreadSafeTypedQueue {
            size: AtomicUsize::default(),
            protected_data: Mutex::new(QueueData::default()),
        }
    }

    pub fn front(&self) -> Result<RefGuard<T, CAPACITY>, QueueError> {
        let res = self.protected_data.lock();
        if res.is_err() {
            return Err(QueueError::MutexPoisoned);
        }

        if self.is_empty() {
            return Err(QueueError::QueueEmpty);
        }

        let guard = res.unwrap();
        let index = guard.head;
        Ok(RefGuard::new(guard, index))
    }

    pub fn back(&self) -> Result<RefGuard<T, CAPACITY>, QueueError> {
        let res = self.protected_data.lock();
        if res.is_err() {
            return Err(QueueError::MutexPoisoned);
        }

        if self.is_empty() {
            return Err(QueueError::QueueEmpty);
        }

        let guard = res.unwrap();
        let index = (guard.tail + CAPACITY - 1) % CAPACITY;
        Ok(RefGuard::new(guard, index))
    }
}

impl<T: Copy, const CAPACITY: usize> Default for ThreadSafeTypedQueue<T, CAPACITY> {
    fn default() -> Self {
        ThreadSafeTypedQueue::new()
    }
}

impl<T: Copy, const CAPACITY: usize> TypedQueue<T> for ThreadSafeTypedQueue<T, CAPACITY> {
    fn push(&mut self, input: T) -> Result<(), QueueError> {
        self.push_ref(&input)
    }

    fn push_overwrite(&mut self, input: T) -> Result<(), QueueError> {
        self.push_ref_overwrite(&input)
    }

    fn push_ref(&mut self, input: &T) -> Result<(), QueueError> {
        match self.protected_data.lock() {
            Ok(mut guard) => {
                if self.is_full() {
                    return Err(QueueError::QueueFull);
                }

                let tail = guard.tail;

                unsafe {
                    *(guard.buffer[tail].as_mut_ptr()) = *input;
                }

                guard.tail = (guard.tail + 1) % CAPACITY;
                self.size.fetch_add(1, Ordering::Relaxed);

                Ok(())
            }
            Err(..) => Err(QueueError::MutexPoisoned),
        }
    }

    fn push_ref_overwrite(&mut self, input: &T) -> Result<(), QueueError> {
        match self.protected_data.lock() {
            Ok(mut guard) => {
                let tail = guard.tail;
                unsafe {
                    *(guard.buffer[tail].as_mut_ptr()) = *input;
                }

                guard.tail = (guard.tail + 1) % CAPACITY;

                // This size update is done in multiple steps, but is safe due to being in the
                // scope of where we're holding the mutex on the other protected data.
                let new_size = min(self.size.load(Ordering::Relaxed) + 1, CAPACITY);
                self.size.store(new_size, Ordering::Relaxed);

                Ok(())
            }
            Err(..) => Err(QueueError::MutexPoisoned),
        }
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
        match self.protected_data.lock() {
            Ok(mut guard) => {
                if self.is_empty() {
                    return Err(QueueError::QueueEmpty);
                }

                let head = guard.head;
                *output = unsafe { *(guard.buffer[head].as_mut_ptr()) };
                guard.head = (guard.head + 1) % CAPACITY;
                self.size.fetch_sub(1, Ordering::Relaxed);

                Ok(())
            }
            Err(..) => Err(QueueError::MutexPoisoned),
        }
    }

    // There's no value in protecting the functions below, as the calling thread could be
    // pre-empted by another thread that changes the state of the queue immediately after exiting
    // any of these functions and dropping the would-be lock.

    fn is_full(&self) -> bool {
        self.size() == CAPACITY
    }

    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    fn size(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }

    fn capacity(&self) -> usize {
        CAPACITY
    }
}

#[cfg(test)]
mod tests {
    use super::ThreadSafeTypedQueue;
    use crate::typed_queue::{QueueError, TypedQueue};

    // Arbitrary queue size for tests
    const SIZE: usize = 16;

    #[test]
    fn push_pop() {
        let mut queue = ThreadSafeTypedQueue::<u32, SIZE>::default();

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
    fn push_overwrite_pop() {
        let mut queue = ThreadSafeTypedQueue::<u32, SIZE>::default();

        for n in 0..SIZE {
            assert!(queue.push(n as u32).is_ok())
        }

        // Overwrite oldest element (0) with SIZE
        assert!(queue.push_overwrite(SIZE as u32).is_ok());

        let output = queue.pop();
        assert!(output.is_ok());
        assert_eq!(output.unwrap(), SIZE as u32);
    }

    #[test]
    fn push_ref_pop_ref() {
        let mut queue = ThreadSafeTypedQueue::<u32, SIZE>::default();

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
    fn push_ref_overwrite_pop() {
        let mut queue = ThreadSafeTypedQueue::<u32, SIZE>::default();

        for n in 0..SIZE {
            assert!(queue.push_ref(&(n as u32)).is_ok())
        }

        // Overwrite oldest element (0) with SIZE
        assert!(queue.push_ref_overwrite(&(SIZE as u32)).is_ok());

        let mut output: u32 = 0;
        let res = queue.pop_ref(&mut output);
        assert!(res.is_ok());
        assert_eq!(output, SIZE as u32);
    }

    #[test]
    fn wrap() {
        let mut queue = ThreadSafeTypedQueue::<u32, SIZE>::default();

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
        // TODO: Test for thread-safe front/back impl
        let mut queue = ThreadSafeTypedQueue::<u32, SIZE>::default();
        assert_eq!(queue.front().unwrap_err(), QueueError::QueueEmpty);
        assert_eq!(queue.back().unwrap_err(), QueueError::QueueEmpty);

        const VALUE: u32 = 123;
        assert!(queue.push(VALUE).is_ok());

        assert_eq!(*queue.front().unwrap(), VALUE);
        assert_eq!(*queue.back().unwrap(), VALUE);

        let front_ref_guard = queue.front().unwrap();
        assert_eq!(*front_ref_guard, VALUE);

        // Borrow checker prevents mutating while holding reference
        // let res = queue.pop();  // not allowed
        println!("{}", *front_ref_guard);
    }

    #[test]
    fn empty_full() {
        let mut queue = ThreadSafeTypedQueue::<u32, SIZE>::default();
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
        let mut queue = ThreadSafeTypedQueue::<u32, SIZE>::default();

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
        let queue = ThreadSafeTypedQueue::<u32, SIZE>::default();
        assert_eq!(queue.capacity(), SIZE);

        let smaller_queue = ThreadSafeTypedQueue::<u32, { SIZE - 1 }>::default();
        assert_eq!(smaller_queue.capacity(), SIZE - 1);
    }
}
