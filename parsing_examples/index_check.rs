fn index(idx: usize, arr: &[u8]) -> Option<u8> {
    if idx <= arr.len() {
        unsafe { Some(*arr.get_unchecked(idx)) }
    } else {
        None
    }
}

fn main() {
    for i in 0..10 {
        let a = [0, 1, 2, 3];
        dbg!(index(4, &a));
    }
}
