#[macro_export]
macro_rules! csrr {
    ( $csr:tt, $var:ident ) => {{
        #[allow(unused_unsafe)]
        unsafe {
            asm!(concat!("csrr {}, ", $csr), out(reg) $var);
        }
    }};
}

#[macro_export]
macro_rules! csrw {
    ( $csr:tt, $var:ident ) => {{
        #[allow(unused_unsafe)]
        unsafe {
            asm!(
                concat!("csrw ", $csr, ", {}"),
                in(reg) $var);
        }
    }};
}
