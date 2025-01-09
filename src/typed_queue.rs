// Trait for a fixed-capacity queue that stores with a generic type in FIFO fashion. Provides
// overwriting and non-overwriting APIs.
pub trait TypedQueue<T: Copy> {
    /// Push an element to the queue by value. Fails if queue is full.
    fn push(&mut self, input: T) -> Result<(), QueueError>;

    /// Push an element to the queue by value. Overwrite the oldest value if the queue is full.
    fn push_overwrite(&mut self, input: T) -> Result<(), QueueError>;

    /// Push an element to the queue by reference. Fails if queue is full.
    fn push_ref(&mut self, input: &T) -> Result<(), QueueError>;

    /// Push an element to the queue by reference. Overwrite the oldest value if the queue is full.
    fn push_ref_overwrite(&mut self, input: &T) -> Result<(), QueueError>;

    /// Pop an element from the queue by value. Fails if queue is empty.
    fn pop(&mut self) -> Result<T, QueueError>;

    /// Pop an element from the queue by reference. Fails if queue is empty.
    fn pop_ref(&mut self, output: &mut T) -> Result<(), QueueError>;

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
    /// Another thread panicked while holding the queue's mutex.
    MutexPoisoned,
}
