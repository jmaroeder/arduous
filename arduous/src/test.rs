pub struct StatusRegister<'a> {
    byte: &'a BitSlice<Lsb0, u8>,
    i: &'a bool,
    t: &'a bool,
    h: &'a bool,
    s: &'a bool,
    v: &'a bool,
    n: &'a bool,
    z: &'a bool,
    c: &'a bool,
}

impl StatusRegister<'_> {
    pub fn new(byte: &u8) -> StatusRegister {
        let byte = byte.view_bits::<Lsb0>();
        StatusRegister {
            byte,
            i: &byte[7],
            t: &byte[6],
            h: &byte[5],
            s: &byte[4],
            v: &byte[3],
            n: &byte[2],
            z: &byte[1],
            c: &byte[0],
        }
    }
}
