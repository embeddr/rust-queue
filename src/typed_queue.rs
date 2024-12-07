// Trait for a fixed-capacity queue that stores with a generic type in FIFO fashion. Provides
// overwriting and non-overwriting APIs.
pub trait TypedQueue<T: Copy> {
    /// Push an element to the queue by value. Returns None if successful, or else
    /// QueueError::QueueFull.
    fn push(&mut self, input: T) -> Result<(), QueueError>;

    /// Push an element to the queue by value. Overwrite the oldest value if the queue is full.
    fn push_overwrite(&mut self, input: T);

    /// Push an element to the queue by reference. Returns None if successful, or else
    /// QueueError::QueueFull.
    fn push_ref(&mut self, input: &T) -> Result<(), QueueError>;

    /// Push an element to the queue by reference. Overwrite the oldest value if the queue is full.
    fn push_ref_overwrite(&mut self, input: &T);

    /// Pop an element from the queue by value. Returns the element if successful, or else
    /// QueueError::QueueEmpty.
    fn pop(&mut self) -> Result<T, QueueError>;

    /// Pop an element from the queue by reference. Returns the element if successful, or else
    /// QueueError::QueueEmpty.
    fn pop_ref(&mut self, output: &mut T) -> Result<(), QueueError>;

    // Get a reference to the oldest (next to be popped) element in the queue, if any exists.
    fn front(&self) -> Result<&T, QueueError>;

    // Get a reference to the newest (most recently pushed) element in the queue, if any exists.
    fn back(&self) -> Result<&T, QueueError>;

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
