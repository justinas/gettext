extern crate byteorder;

use std::mem;

use self::byteorder::{ByteOrder, BigEndian, LittleEndian};

/// According to the given magic number of a MO file,
/// returns the function which reads a `u32` in the relevant endianness.
fn get_read_u32_fn(magic: &[u8]) -> Option<fn(&[u8]) -> u32> {
    if magic == [0xde, 0x12, 0x04, 0x95] {
        Some(LittleEndian::read_u32)
    } else if magic == [0x95, 0x04, 0x12, 0xde] {
        Some(BigEndian::read_u32)
    } else {
        None
    }
}

#[test]
fn test_get_read_u32_fn() {
    assert!(get_read_u32_fn(&[]).is_none());
    assert!(get_read_u32_fn(&[0xde, 0x12, 0x04, 0x95, 0x00]).is_none());

    {
        let le_ptr: *const ();
        let ret_ptr;
        unsafe {
            le_ptr = mem::transmute(LittleEndian::read_u32);
            ret_ptr = mem::transmute(get_read_u32_fn(&[0xde, 0x12, 0x04, 0x95]).unwrap());
        }
        assert_eq!(le_ptr, ret_ptr);
    }

    {
        let be_ptr: *const ();
        let ret_ptr;
        unsafe {
            be_ptr = mem::transmute(BigEndian::read_u32);
            ret_ptr = mem::transmute(get_read_u32_fn(&[0x95, 0x04, 0x12, 0xde]).unwrap());
        }
        assert_eq!(be_ptr, ret_ptr);
    }
}
