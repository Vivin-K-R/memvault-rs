use std::alloc::Layout;
use std::ptr::NonNull;

pub struct Arena {
    buf: Vec<u8>,
    pos: usize,
}

impl Arena {
    pub fn with_capacity(cap: usize) -> Self {
        Self { buf: vec![0u8; cap], pos: 0 }
    }

    pub fn capacity(&self) -> usize { self.buf.len() }
    pub fn used(&self) -> usize { self.pos }
    pub fn remaining(&self) -> usize { self.capacity().saturating_sub(self.pos) }

    pub fn reset(&mut self) {
        self.pos = 0;
    }

    fn align_up(x: usize, align: usize) -> usize {
        debug_assert!(align.is_power_of_two());
        (x + (align - 1)) & !(align - 1)
    }

    pub fn alloc_bytes(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        let start = Self::align_up(self.pos, layout.align());
        let end = start.checked_add(layout.size())?;
        if end > self.buf.len() {
            return None;
        }
        self.pos = end;
        let p = unsafe { self.buf.as_mut_ptr().add(start) };
        NonNull::new(p)
    }

    pub fn alloc_slice<T: Copy>(&mut self, n: usize) -> Option<&mut [T]> {
        let layout = Layout::array::<T>(n).ok()?;
        let p = self.alloc_bytes(layout)?;
        let ptr = p.as_ptr() as *mut T;
        let slice = unsafe { std::slice::from_raw_parts_mut(ptr, n) };
        Some(slice)
    }
}

#[derive(Debug)]
pub enum RbError {
    Full,
    Empty,
}

pub struct RingBuffer<T: Copy> {
    buf: Vec<T>,
    head: usize,
    tail: usize,
    full: bool,
}

impl<T: Copy> RingBuffer<T> {
    pub fn with_capacity(cap: usize, init: T) -> Self {
        assert!(cap > 0);
        Self {
            buf: vec![init; cap],
            head: 0,
            tail: 0,
            full: false,
        }
    }

    pub fn capacity(&self) -> usize { self.buf.len() }
    pub fn is_empty(&self) -> bool { !self.full && self.head == self.tail }
    pub fn is_full(&self) -> bool { self.full }

    pub fn len(&self) -> usize {
        if self.full {
            self.capacity()
        } else if self.tail >= self.head {
            self.tail - self.head
        } else {
            self.capacity() - (self.head - self.tail)
        }
    }

    pub fn push(&mut self, v: T) -> Result<(), RbError> {
        if self.full {
            return Err(RbError::Full);
        }
        self.buf[self.tail] = v;
        self.tail = (self.tail + 1) % self.capacity();
        self.full = self.tail == self.head;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<T, RbError> {
        if self.is_empty() {
            return Err(RbError::Empty);
        }
        let v = self.buf[self.head];
        self.head = (self.head + 1) % self.capacity();
        self.full = false;
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena_basic() {
        let mut a = Arena::with_capacity(1024);

        // should start empty
        assert_eq!(a.used(), 0);
        assert_eq!(a.remaining(), 1024);

        // allocate 4 u32s (16 bytes)
        let xs: &mut [u32] = a.alloc_slice::<u32>(4).unwrap();
        xs[0] = 10;
        xs[1] = 20;
        xs[2] = 30;
        xs[3] = 40;

        assert_eq!(xs[0], 10);
        assert_eq!(xs[3], 40);
        assert!(a.used() > 0);

        // reset should bring used back to 0
        a.reset();
        assert_eq!(a.used(), 0);
    }

    #[test]
    fn arena_out_of_memory() {
        let mut a = Arena::with_capacity(8); // tiny arena, only 8 bytes

        // u64 = 8 bytes, should succeed
        let r1 = a.alloc_slice::<u64>(1);
        assert!(r1.is_some());

        // no space left, should return None not crash
        let r2 = a.alloc_slice::<u64>(1);
        assert!(r2.is_none());
    }

    #[test]
    fn ring_basic() {
        let mut rb = RingBuffer::with_capacity(4, 0u32);

        // should start empty
        assert!(rb.is_empty());
        assert_eq!(rb.len(), 0);

        // fill it up
        rb.push(1).unwrap();
        rb.push(2).unwrap();
        rb.push(3).unwrap();
        rb.push(4).unwrap();

        // should be full now
        assert!(rb.is_full());
        assert_eq!(rb.len(), 4);

        // pushing to a full buffer returns error, not crash
        assert!(rb.push(5).is_err());

        // pop in order (FIFO)
        assert_eq!(rb.pop().unwrap(), 1);
        assert_eq!(rb.pop().unwrap(), 2);
    }

    #[test]
    fn ring_wraparound() {
        let mut rb = RingBuffer::with_capacity(3, 0u32);

        // fill
        rb.push(10).unwrap();
        rb.push(20).unwrap();
        rb.push(30).unwrap();

        // drain two
        assert_eq!(rb.pop().unwrap(), 10);
        assert_eq!(rb.pop().unwrap(), 20);

        // push two more — this forces wraparound
        rb.push(40).unwrap();
        rb.push(50).unwrap();

        // should still read in correct order
        assert_eq!(rb.pop().unwrap(), 30);
        assert_eq!(rb.pop().unwrap(), 40);
        assert_eq!(rb.pop().unwrap(), 50);

        // now empty
        assert!(rb.is_empty());
        assert!(rb.pop().is_err());
    }

    #[test]
    fn ring_invariants() {
        let mut rb = RingBuffer::with_capacity(3, 0u32);

        // len always stays within 0..=capacity
        rb.push(10).unwrap();
        rb.push(20).unwrap();
        rb.push(30).unwrap();
        assert!(rb.len() <= rb.capacity());

        rb.pop().unwrap();
        rb.push(40).unwrap();
        assert!(rb.len() <= rb.capacity());
    }
}