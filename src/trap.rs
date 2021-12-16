use rv64::csr::stvec::Stvec;

extern "C" {
    fn kernelvec();
}

// setup to take exceptions and traps in supervisor mode
pub fn init_harttrap() {
    let mut stvec = Stvec::from_bits(0);
    stvec.set_addr(kernelvec as u64);
    stvec.write();
}
