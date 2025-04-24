use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

// for communicating with UART to actual console
lazy_static! {
    pub static ref SERIAL_ONE: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

// below is an implementation of printing to this other IO port
// instead of VGA text

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL_ONE.lock().write_fmt(args).expect("Printing to serial failed");
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
    concat!($fmt, "\n"), $($arg)*));
}
