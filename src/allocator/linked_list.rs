use core::{mem, ptr};

extern crate alloc;

use alloc::alloc::{GlobalAlloc, Layout};

use super::{align_up, Locked};

struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    const fn new(size: usize) -> Self {
        ListNode { size, next: None }
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

pub struct LinkedListAllocator {
    head: ListNode,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.return_free_region(heap_start, heap_size);
    }

    fn size_align(layout: Layout) -> (usize, usize) {
        // Ensures that each allocated block is at least large enough to store a
        // list node; allocating a region smaller than this would make it
        // incapable of serving as a node in our free list, should this
        // allocation be freed later on.

        // If necessary, increases the layout's alignment to that of `ListNode`.
        // Also, this rounds up the size to a multiple of said alignment, so
        // that the next (neighboring) memory block will also have proper
        // alignment.
        let layout = layout
            .align_to(mem::align_of::<ListNode>())
            .expect("Adjusting alignment failed.")
            .pad_to_align();

        // Enforces a minimum allocation size for book-keeping.
        let size = layout.size().max(mem::size_of::<ListNode>());

        (size, layout.align())
    }

    unsafe fn return_free_region(&mut self, address: usize, size: usize) {
        // Ensures that `address` is properly aligned for the `ListNode` type.
        assert_eq!(align_up(address, mem::align_of::<ListNode>()), address);

        // Ensures that `size` is sufficient to store a `ListNode` struct.
        assert!(size >= mem::size_of::<ListNode>());

        // Allocates a new list node on the stack.
        let mut node = ListNode::new(size);

        // Splices our free list nodes behind `node`
        // (`head.next` here becomes `None` after the `take()`).
        node.next = self.head.next.take();

        // Copies our (stack) node struct to the heap (at `address`).
        let node_ptr = address as *mut ListNode;
        node_ptr.write(node);

        // Updates `head` to point to our latest list node.
        self.head.next = Some(&mut *node_ptr);
    }

    fn take_free_region(
        &mut self,
        size: usize,
        align: usize,
    ) -> Option<(&'static mut ListNode, usize)> {
        // Pointer to the node we're currently visiting.
        let mut current_node = &mut self.head;

        // Traverse the free list to find a large enough free region with the
        // correct alignment.
        while let Some(ref mut region) = current_node.next {
            if let Ok(alloc_start) = Self::is_usable_free_region(&region, size, align) {
                // We can use this region (and remove its node from the free
                // list).
                let next_next_node = region.next.take();

                let result = Some((current_node.next.take().unwrap(), alloc_start));

                current_node.next = next_next_node;

                return result;
            } else {
                current_node = current_node.next.as_mut().unwrap();
            }
        }

        None
    }

    fn is_usable_free_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
        // Start address for the region
        let alloc_start = align_up(region.start_addr(), align);

        // End address of the requested allocation size, beginning at `alloc_start`.
        // (Includes an overflow check).
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        // Check if the requested allocation is larger than this free region.
        if alloc_end > region.end_addr() {
            return Err(());
        }

        // If the allocation fits in this region, compute how much space would
        // still be unused.
        let excess_size = region.end_addr() - alloc_end;

        // If there's unused space remaining, but not enough of it to store a
        // new free list node (i.e., book-keeping data), then this region isn't
        // suitable for allocating (we don't want to accumulate unused memory).
        if excess_size > 0 && excess_size < mem::size_of::<ListNode>() {
            return Err(());
        }

        // This region will work, so use it!
        Ok(alloc_start)
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Performs layout adjustments.
        let (size, align) = LinkedListAllocator::size_align(layout);

        let mut allocator = self.lock();

        // Search for a suitable region in our free list.
        if let Some((region, alloc_start)) = allocator.take_free_region(size, align) {
            // Checks whether or not we can split this region before allocating.
            let alloc_end = alloc_start.checked_add(size).expect("Overflow");
            let excess_size = region.end_addr() - alloc_end;

            if excess_size > 0 {
                // At this point we know there to be enough excess space to fit
                // another free list node (i.e., we can split the node).
                allocator.return_free_region(alloc_end, excess_size);
            }

            // Returns the start address for this allocation.
            alloc_start as *mut u8
        } else {
            // Not enough heap memory to service this alloc request.
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Performs layout adjustments.
        let (size, _) = LinkedListAllocator::size_align(layout);

        // Returns this region to the free list.
        self.lock().return_free_region(ptr as usize, size);
    }
}
