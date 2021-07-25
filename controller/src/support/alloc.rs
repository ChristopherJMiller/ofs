use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;
use core::ptr::{self, NonNull};

use avr_device::interrupt::{self, Mutex};
use linked_list_allocator::Heap;

pub struct SharedHeap(Mutex<RefCell<Heap>>);

impl SharedHeap {
  pub const fn empty() -> SharedHeap {
    SharedHeap(Mutex::new(RefCell::new(Heap::empty())))
  }

  pub fn init(&self, start_addr: usize, size: usize) {
    interrupt::free(|cs| unsafe {
      self.0.borrow(cs).borrow_mut().init(start_addr, size);
    });
  }
}

unsafe impl GlobalAlloc for SharedHeap {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    interrupt::free(|cs| {
      self
        .0
        .borrow(cs)
        .borrow_mut()
        .allocate_first_fit(layout)
        .ok()
        .map_or(ptr::null_mut(), |a| a.as_ptr())
    })
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    interrupt::free(|cs| {
      self
        .0
        .borrow(cs)
        .borrow_mut()
        .deallocate(NonNull::new_unchecked(ptr), layout)
    })
  }
}

#[alloc_error_handler]
fn on_oom(_layout: Layout) -> ! {
  loop {}
}

#[global_allocator]
pub static ALLOCATOR: SharedHeap = SharedHeap::empty();
