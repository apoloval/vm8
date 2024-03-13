pub fn add_word(a: u16, b: u16) -> u16 {
    let t1 = a + 0x0666;
    let t2 = t1.wrapping_add(b);
    let t3 = t1 ^ b;
    let t4 = !(t2 ^ t3) & 0x1110;
    let t5 = (t4 >> 2) | (t4 >> 3);
    return t2.wrapping_sub(t5);
}

pub fn sub_word(a: u16, b: u16) -> u16 {
    add_word(a, neg_word(b))
}

pub fn neg_word(a: u16) -> u16 {
    let t1 = -(a as i16) as u16;
    let t2 = t1.wrapping_add(0xFFFF);
    let t3 = t2 ^ 1;
    let t4 = !(t2 ^ t3) & 0x1110;
    let t5 = (t4 >> 2) | (t4 >> 3);
    return t1.wrapping_sub(t5);
}

#[cfg(test)]
mod test {
    use super::*;

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

    #[test]
    fn test_bcd_sub_word() {
        assert_eq!(sub_word(0x0000, 0x0000), 0x0000);
        assert_eq!(sub_word(0x0001, 0x0000), 0x0001);
        assert_eq!(sub_word(0x0001, 0x0001), 0x0000);
        assert_eq!(sub_word(0x5678, 0x1234), 0x4444);
    }
}
