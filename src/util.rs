pub fn delay(count: u32) {
    let mut count = count * 5000;
    while count != 0 {
        count -= 1;
    }
}
