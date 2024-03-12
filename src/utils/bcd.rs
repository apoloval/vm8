pub fn add_byte(a: u8, b: u8) -> u8 {
    let t1 = a + 0x06;
    let t2 = t1.wrapping_add(b);
    let t3 = t1 ^ b;
    let t4 = !(t2 ^ t3) & 0x10;
    let t5 = (t4 >> 2) | (t4 >> 3);
    return t2 - t5;
}

pub fn add_word(a: u16, b: u16) -> u16 {
    let t1 = a + 0x0666;
    let t2 = t1.wrapping_add(b);
    let t3 = t1 ^ b;
    let t4 = !(t2 ^ t3) & 0x1110;
    let t5 = (t4 >> 2) | (t4 >> 3);
    return t2 - t5;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bcd_add() {
        assert_eq!(add_byte(0x00, 0x00), 0x00);
        assert_eq!(add_byte(0x01, 0x01), 0x02);
        assert_eq!(add_byte(0x09, 0x01), 0x10);
        assert_eq!(add_byte(0x09, 0x02), 0x11);
        assert_eq!(add_byte(0x09, 0x03), 0x12);
        assert_eq!(add_byte(0x09, 0x09), 0x18);
        assert_eq!(add_byte(0x39, 0x49), 0x88);
    }

    #[test]
    fn test_bcd_add_word() {
        assert_eq!(add_word(0x0000, 0x0000), 0x0000);
        assert_eq!(add_word(0x0001, 0x0001), 0x0002);
        assert_eq!(add_word(0x0009, 0x0001), 0x0010);
        assert_eq!(add_word(0x0009, 0x0002), 0x0011);
        assert_eq!(add_word(0x0009, 0x0003), 0x0012);
        assert_eq!(add_word(0x0009, 0x0009), 0x0018);
        assert_eq!(add_word(0x8639, 0x1249), 0x9888);
    }
}
