
use futures_util::stream::StreamExt;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use crate::{println, print, serial_println, serial_print};
use core::{pin::Pin, task::{Poll, Context}};
use futures_util::{task::AtomicWaker, stream::Stream};

// make sure initialization occurs outside of interrupt handler
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

// only local to lib.rs
// must be only be able to be called as soon as the queue is initialized
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            serial_println!("[warning] scancode queue is full");
        } else {
            WAKER.wake();
        }
    } else {
        serial_println!("[warning] scancode queue uninitialized");
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100))
            .expect("initialized more than once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE.try_get().expect("not initialized");

        // queue is not empty, just directly fetch a scancode
        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker()); // store a waker for an item

        match queue.pop() {
            Some(queue_scancode) => {
                WAKER.take();
                Poll::Ready(Some(queue_scancode))
            }
            None => Poll::Pending,
        }
    }
}


pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(ScancodeSet1::new(),
        layouts::Us104Key, HandleControl::Ignore);

    while let Some(scancode) = scancodes.next().await {
        serial_println!("yelow");
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => serial_print!("{}", character),
                    DecodedKey::RawKey(key) => serial_print!("{:?}", key),
                }
            }
        }
    }
}
