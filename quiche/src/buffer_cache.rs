use std::sync::Arc;

use parking_lot::Mutex;

#[derive(Clone)]
pub struct BufferCache {
    inner: Arc<Mutex<Vec<Vec<u8>>>>,
    alloc_size: usize,
    max_entries: usize,
}

impl BufferCache {
    pub fn new(alloc_size: usize, max_entries: usize) -> Self {
        let mut entries = vec![];

        for _i in 0..max_entries {
            entries.push(vec![0; alloc_size]);
        }
        Self {
            inner: Arc::new(Mutex::new(entries)),
            alloc_size,
            max_entries,
        }
    }

    pub fn take(&self, requested_size: usize) -> Vec<u8> {
        if requested_size > self.alloc_size {
            return vec![0; requested_size];
        }

        let mut buffer_list = self.inner.lock();
        let mut buffer = if let Some(buffer) = buffer_list.pop() {
            buffer
        } else {
            vec![0; self.alloc_size]
        };

        // Safety: we've previously initialized the entire capacity with 0 when
        // creating the buffer, and requested_size is always less than or equal
        // to alloc_size here.
        //
        // TODO: There's actually a potential soundness hole here if free is
        // passed a buffer that was not originally allocated by the cache, and
        // it has uninitialized capacity _and_ its capacity happens to match our
        // alloc_size. To fix this, we'd need to wrap the Vec in a type that
        // keeps track of the fact that it came from this particular buffer
        // cache.
        unsafe {
            buffer.set_len(requested_size);
        }
        buffer
    }

    pub fn free(&self, buffer: Vec<u8>) {
        if buffer.capacity() != self.alloc_size {
            return;
        }

        let mut buffer_list = self.inner.lock();

        if buffer_list.len() < self.max_entries {
            buffer_list.push(buffer);
        }
    }
}
