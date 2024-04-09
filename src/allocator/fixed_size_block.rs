extern crate alloc;

use core::mem;

use alloc::alloc::{GlobalAlloc, Layout};

use super::{linked_list::LinkedListAllocator, Locked};

struct ListNode {
    next: Option<&'static mut ListNode>,
}

// For allocations greater than 2KiB, we'll fall back to a linked list allocator.
//
// Note that we don't offer block sizes smaller than 8 bytes, as blocks at
// minimum need 64 bits to store their `next` pointer.
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

pub struct FixedSizeBlockAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    // fallback_allocator: linked_list_allocator::Heap,
    fallback_allocator: Locked<LinkedListAllocator>,
}

impl FixedSizeBlockAllocator {
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;

        Self {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: Locked::new(LinkedListAllocator::new()),
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        // We'll lazily initialize `list_heads` on demand, through `alloc()`.

        self.fallback_allocator.lock().init(heap_start, heap_size);
    }

    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        unsafe { self.fallback_allocator.alloc(layout) }
    }

    fn get_list_index(layout: &Layout) -> Option<usize> {
        // A layout's block size will be the maximum of its size and alignment.
        let required_block_size = layout.size().max(layout.align());

        // We'll use the returned index to index into `list_heads`.
        BLOCK_SIZES
            .iter()
            .position(|&block_size| block_size >= required_block_size)
    }
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();

        match FixedSizeBlockAllocator::get_list_index(&layout) {
            Some(index) => {
                // This allocation will fit inside one of our fixed block sizes.

                // Check if a list for this block size is already created.
                match allocator.list_heads[index].take() {
                    Some(head_node) => {
                        // Uses the head of the corresponding free list for the
                        // allocation, and updates the list head.
                        allocator.list_heads[index] = head_node.next.take();

                        head_node as *mut ListNode as *mut u8
                    }
                    None => {
                        // Uses the fallback allocator to perform a new
                        // allocation, sized and aligned to this block size.
                        let block_size = BLOCK_SIZES[index];
                        let block_align = block_size;

                        let layout = Layout::from_size_align(block_size, block_align).unwrap();

                        allocator.fallback_alloc(layout)
                    }
                }
            }
            None => allocator.fallback_alloc(layout),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();

        match FixedSizeBlockAllocator::get_list_index(&layout) {
            Some(index) => {
                // Creates a new head node on the stack, and links it to the
                // appropriate free list (i.e., existing head).
                let new_head = ListNode {
                    next: allocator.list_heads[index].take(),
                };

                // Verify that this block is large enough to hold a ListNode.
                let block_size = BLOCK_SIZES[index];
                assert!(mem::size_of::<ListNode>() <= block_size);
                assert!(mem::align_of::<ListNode>() <= block_size);

                // Copies our new head from the stack to the heap (at `ptr`).
                let new_head_ptr = ptr as *mut ListNode;
                new_head_ptr.write(new_head);

                // Updates corresponding free list's head to the new head.
                allocator.list_heads[index] = Some(&mut *new_head_ptr);
            }
            None => {
                // let ptr = NonNull::new(ptr).unwrap();

                allocator.fallback_allocator.dealloc(ptr, layout);
            }
        }
    }
}
