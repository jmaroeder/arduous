use bitvec::prelude::*;

use std::{
    convert::TryInto,
    fmt::Debug,
    ops::{Index, IndexMut},
};

// Flash Memory Layout
pub const PROGRAM_MEMORY_SIZE: usize = 32768;
pub const PROGRAM_MEMORY_WORDS: usize = 16384;

// Data Memory Layout
pub const REGISTER_START: usize = 0x0000;
pub const REGISTER_SIZE: usize = 32;
pub const REGISTER_END: usize = REGISTER_START + REGISTER_SIZE - 1;
pub const REGISTER_W_INDEX: usize = 24;
pub const REGISTER_X_INDEX: usize = 26;
pub const REGISTER_Y_INDEX: usize = 28;
pub const REGISTER_Z_INDEX: usize = 30;
pub const IO_REGISTER_START: usize = 0x0020;
pub const IO_REGISTER_SIZE: usize = 64;
pub const IO_REGISTER_END: usize = IO_REGISTER_START + IO_REGISTER_SIZE - 1;
pub const EXT_IO_REGISTER_START: usize = 0x0060;
pub const EXT_IO_REGISTER_SIZE: usize = 160;
pub const EXT_IO_REGISTER_END: usize = EXT_IO_REGISTER_START + EXT_IO_REGISTER_SIZE - 1;
pub const SRAM_START: usize = 0x0100;
pub const SRAM_SIZE: usize = 2560;
pub const SRAM_END: usize = SRAM_START + SRAM_SIZE - 1;
pub const DATA_MEMORY_SIZE: usize = 2816;

// EEPROM Layout
pub const EEPROM_SIZE: usize = 1024;

#[derive(Debug, Default)]
struct StatusRegister {
    i: bool,
    t: bool,
    h: bool,
    s: bool,
    v: bool,
    n: bool,
    z: bool,
    c: bool,
}

/// Status Register indexing/conversion
impl StatusRegister {
    fn to_u8(&self) -> u8 {
        (self.i as u8) << 7
            | (self.t as u8) << 6
            | (self.h as u8) << 5
            | (self.s as u8) << 4
            | (self.v as u8) << 3
            | (self.n as u8) << 2
            | (self.z as u8) << 1
            | (self.c as u8) << 0
        // bitarr![Lsb0, u8; self.c, self.z, self.n, self.v, self.s, self.h, self.t, self.i].load::<u8>()
    }

    fn from_u8(value: u8) -> Self {
        let value = value.view_bits::<Lsb0>();
        Self {
            i: value[7],
            t: value[6],
            h: value[5],
            s: value[4],
            v: value[3],
            n: value[2],
            z: value[1],
            c: value[0],
        }
    }
}

impl Index<u8> for StatusRegister {
    type Output = bool;

    fn index(&self, index: u8) -> &Self::Output {
        match index {
            7 => &self.i,
            6 => &self.t,
            5 => &self.h,
            4 => &self.s,
            3 => &self.v,
            2 => &self.n,
            1 => &self.z,
            0 => &self.c,
            _ => unreachable!(),
        }
    }
}

impl IndexMut<u8> for StatusRegister {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        match index {
            7 => &mut self.i,
            6 => &mut self.t,
            5 => &mut self.h,
            4 => &mut self.s,
            3 => &mut self.v,
            2 => &mut self.n,
            1 => &mut self.z,
            0 => &mut self.c,
            _ => unreachable!(),
        }
    }
}

macro_rules! pair_access {
    () => {
        fn pair(&self, index: usize) -> u16 {
            u16::from_le_bytes(self.0[index..index + 2].try_into().unwrap())
        }

        fn set_pair(&mut self, index: usize, value: u16) {
            let bytes = value.to_le_bytes();
            self.0[index] = bytes[0];
            self.0[index + 1] = bytes[1];
        }
    };
}

macro_rules! simple_index {
    ( $s:ty, $( $t:ty ),* ) => {
        $(
            impl Index<$t> for $s {
                type Output = u8;
                fn index(&self, index: $t) -> &Self::Output {
                    &self.0[index as usize]
                }
            }
            impl IndexMut<$t> for $s {
                fn index_mut(&mut self, index: $t) -> &mut Self::Output {
                    &mut self.0[index as usize]
                }
            }
        )*
    };
}

#[derive(Debug, Default)]
/// General Register indexing
struct GeneralRegisters([u8; REGISTER_SIZE]);

impl GeneralRegisters {
    pair_access! {}

    fn w(&self) -> u16 {
        self.pair(REGISTER_W_INDEX)
    }

    fn set_w(&mut self, value: u16) {
        self.set_pair(REGISTER_W_INDEX, value);
    }

    fn x(&self) -> u16 {
        self.pair(REGISTER_X_INDEX)
    }

    fn set_x(&mut self, value: u16) {
        self.set_pair(REGISTER_X_INDEX, value);
    }

    fn y(&self) -> u16 {
        self.pair(REGISTER_Y_INDEX)
    }

    fn set_y(&mut self, value: u16) {
        self.set_pair(REGISTER_Y_INDEX, value);
    }

    fn z(&self) -> u16 {
        self.pair(REGISTER_Z_INDEX)
    }

    fn set_z(&mut self, value: u16) {
        self.set_pair(REGISTER_Z_INDEX, value);
    }
}

simple_index! {GeneralRegisters, usize, u8}

#[derive(Debug)]
struct Sram([u8; SRAM_SIZE]);

impl Sram {
    pair_access! {}
}

impl Default for Sram {
    fn default() -> Self {
        Self([0; SRAM_SIZE])
    }
}

simple_index! {Sram, usize, u16}

pub struct ATmega32u4 {
    /// Flash Program Memory
    program_memory: [u16; PROGRAM_MEMORY_WORDS],

    /// Data Memory
    regs: GeneralRegisters,
    // io_regs: [u8; IO_REGISTER_SIZE],
    sram: Sram,

    /// EEPROM
    eeprom: [u8; EEPROM_SIZE],

    /// Program Counter
    pc: u16,

    /// Stack Pointer
    sp: u16,

    /// Status Register
    status: StatusRegister,
}

impl ATmega32u4 {
    // Opcodes are performed with the do_OPCODE function, which returns the number
    // of cycles taken to perform the operation.
    // PC is not automatically incremented - each function must manually update

    /// Load data from data
    fn data_load(&self, addr: usize) -> u8 {
        match addr {
            REGISTER_START..=REGISTER_SIZE => self.regs[addr - REGISTER_START],
            IO_REGISTER_START..=IO_REGISTER_END => 0,
            SRAM_START..=SRAM_END => self.sram[addr - SRAM_START],
            _ => unreachable!(),
        }
    }

    fn data_store(&mut self, addr: usize, value: u8) {
        match addr {
            REGISTER_START..=REGISTER_SIZE => self.regs[addr - REGISTER_START] = value,
            IO_REGISTER_START..=IO_REGISTER_END => {}
            SRAM_START..=SRAM_END => self.sram[addr - SRAM_START] = value,
            _ => unreachable!(),
        }
    }

    /// Helper function used by do_adc and do_add to set status bits
    fn helper_add_status_flags(&mut self, d: u8, r: u8, c: bool, result: u8) {
        self.status.z = result == 0;

        let d = d.view_bits::<Lsb0>();
        let r = r.view_bits::<Lsb0>();
        let result = result.view_bits::<Lsb0>();

        self.status.v = r[7] == d[7] && r[7] != result[7];
        self.status.n = result[7];
        self.status.s = self.status.n != self.status.v;
        self.status.h = d[3] && r[3] || r[3] && !result[3] || !result[3] && d[3];
    }

    /// 5. Add with Carry (ADC Rd,Rr)
    fn do_adc(&mut self, rd: u8, rr: u8) -> usize {
        let (result, c0) = self.regs[rd].overflowing_add(self.regs[rr]);
        let (result, c1) = result.overflowing_add(self.status.c as u8);
        let c = c0 || c1;
        self.helper_add_status_flags(self.regs[rd], self.regs[rr], c, result);
        self.status.c = c;
        self.regs[rd] = result;

        self.pc += 1;
        1
    }

    /// 6. Add without Carry (ADD Rd,Rr)
    fn do_add(&mut self, rd: u8, rr: u8) -> usize {
        let (result, c) = self.regs[rd].overflowing_add(self.regs[rr]);
        self.regs[rd] = result;
        self.helper_add_status_flags(self.regs[rd], self.regs[rr], c, result);
        self.status.c = c;

        self.pc += 1;
        1
    }

    /// 7. Add Immediate to Word (ADIW Rd+1:Rd,K)
    fn do_adiw(&mut self, rd: u8, k: u8) -> usize {
        let rd = rd as usize;
        let d = self.regs.pair(rd);
        let (result, c) = d.overflowing_add(k as u16);
        self.regs.set_pair(rd, result);
        self.status.z = result == 0;

        let d = d.view_bits::<Lsb0>();
        let result = result.view_bits::<Lsb0>();
        self.status.v = !d[15] && result[15];
        self.status.n = result[15];
        self.status.s = self.status.n != self.status.v;
        self.status.c = c;

        self.pc += 1;
        2 // This is from the ATmega32u4 datasheet, which differs from the AVR Instruction Set Manual!
          // TODO: verify this is true
    }

    /// 8. Logical AND (AND Rd,Rr)
    fn do_and(&mut self, rd: u8, rr: u8) -> usize {
        let result = self.regs[rd] & self.regs[rr];
        self.status.v = false;
        self.status.n = result.view_bits::<Lsb0>()[7];
        self.status.s = self.status.n != self.status.v;
        self.status.z = result == 0;

        self.pc += 1;
        1
    }

    /// 9. Logical AND with Immediate (ANDI Rd,K)
    fn do_andi(&mut self, rd: u8, k: u8) -> usize {
        let result = self.regs[rd] & k;
        self.status.v = false;
        self.status.n = result.view_bits::<Lsb0>()[7];
        self.status.s = self.status.n != self.status.v;
        self.status.z = result == 0;
        self.regs[rd] = result;

        self.pc += 1;
        1
    }

    /// 10. Arithmetic Shift Right (ASR Rd)
    fn do_asr(&mut self, rd: u8) -> usize {
        let result = self.regs[rd] | (self.regs[rd] & 0b1000_0000);
        self.status.n = result.view_bits::<Lsb0>()[7];
        self.status.c = self.regs[rd].view_bits::<Lsb0>()[0];
        self.status.s = self.status.n != self.status.v;
        self.status.v = self.status.n != self.status.c;
        self.status.z = result == 0;
        self.regs[rd] = result;

        self.pc += 1;
        1
    }

    /// 11. Bit Clear in SREG (BCLR s)
    fn do_bclr(&mut self, s: u8) -> usize {
        self.status[s] = false;

        self.pc += 1;
        1
    }

    /// 12. Bit Load from the T Flag in SREG to a Bit in Register (BLD Rd,b)
    fn do_bld(&mut self, rd: u8, b: u8) -> usize {
        self.regs[rd]
            .view_bits_mut::<Lsb0>()
            .set(b as usize, self.status.t);

        self.pc += 1;
        1
    }

    /// 13. Branch if Bit in SREG is Cleared (BRBC s,k)
    fn do_brbc(&mut self, k: i8, s: u8) -> usize {
        if !self.status[s] {
            self.pc += (k + 1) as u16;
            2
        } else {
            self.pc += 1;
            1
        }
    }

    /// 14. BRBS – Branch if Bit in SREG is Set
    fn do_brbs(&mut self, k: i8, s: u8) -> usize {
        if self.status[s] {
            self.pc += (k + 1) as u16;
            2
        } else {
            self.pc += 1;
            1
        }
    }

    // 15. Branch if Carry Cleared (BRCC k) == BRBC C
    // 16. Branch if Carry Set (BRCS k) == BRBS C

    /// 17. Break (BREAK)
    fn do_break(&mut self) -> usize {
        panic!("BREAK is not implemented");
    }

    // 18. Branch if Equal (BREQ k) == BRBS Z
    // 19. Branch if Greater or Equal (Signed) (BRGE k) == BRBC S
    // 20. Branch if Half Carry Flag is Cleared (BRHC k) == BRBC H
    // 21. Branch if Half Carry Flag is Set (BRHS k) == BRBS H
    // 22. Branch if Global Interrupt is Disabled (BRID k) == BRBC I
    // 23. Branch if Global Interrupt is Enabled (BRIE k) == BRBS I
    // 24. Branch if Lower (Unsigned) (BRLO k) == BRBS C
    // 25. Branch if Less Than (Signed) (BRLT k) == BRBS S
    // 26. Branch if Minus (BRMI k) == BRBS N
    // 27. Branch if Not Equal (BRNE k) == BRBC Z
    // 28. Branch if Plus (BRPL k) == BRBC N
    // 29. Branch if Same or Higher (Unsigned) (BRSH k) == BRBC C
    // 30. Branch if the T Flag is Cleared (BRTC k) == BRBC T
    // 31. Branch if the T Flag is Set (BRTS k) == BRBS T
    // 32. Branch if Overflow Cleared (BRVC k) == BRBC V
    // 33. Branch if Overflow Set (BRVS k) == BRBS V

    /// 34. Bit Set in SREG (BSET s)
    fn do_bset(&mut self, s: u8) -> usize {
        self.status[s] = true;

        self.pc += 1;
        1
    }

    /// 35. Bit Store from Bit in Register to T Flag in SREG (BST Rd,b)
    fn do_bst(&mut self, rd: u8, b: u8) -> usize {
        self.status.t = self.regs[rd].view_bits::<Lsb0>()[b as usize];

        self.pc += 1;
        1
    }

    /// Push a 16-bit word onto the stack
    fn push_u16(&mut self, value: u16) {
        self.sram.set_pair((self.sp - 1) as usize, value);
        self.sp -= 2;
    }

    /// 36. Long Call to a Subroutine (CALL k)
    fn do_call(&mut self, k: u16) -> usize {
        self.push_u16(self.pc + 2);

        self.pc = k;
        5 // TODO: verify this
    }

    /// 37. Clear Bit in I/O Register (CBI A,b)
    fn do_cbi(&mut self, a: u8, b: u8) -> usize {
        let a = a as usize;
        let value = self.data_load(IO_REGISTER_START + a);
        self.data_store(IO_REGISTER_START + a, value & !(1 << b));

        self.pc += 1;
        2
    }

    // 38. Clear Bits in Register (CBR Rd, K) OK == ANDI with K complemented
    // 39. Clear Carry Flag (CLC) OK == BCLR C
    // 40. Clear Half Carry Flag (CLH) OK == BCLR H
    // 41. Clear Global Interrupt Flag (CLI) OK == BCLR I
    // 42. Clear Negative Flag (CLN) OK == BCLR N
    // 43. Clear Register (CLR Rd) OK == EOR Rd, Rd
    // 44. Clear Signed Flag (CLS) OK == BCLR S
    // 45. Clear T Flag (CLT) OK == BCLR T
    // 46. Clear Overflow Flag (CLV) OK == BCLR V
    // 47. Clear Zero Flag (CLZ) OK == BCLR Z

    /// 48. One's Complement (COM Rd)
    fn do_com(&mut self, rd: u8) -> usize {
        let (result, _) = 0xffu8.overflowing_sub(self.regs[rd]);
        self.status.n = result.view_bits::<Lsb0>()[7] != false;
        self.status.c = true;
        self.status.v = false;
        self.status.s = self.status.n != self.status.v;
        self.status.z = result == 0;
        self.regs[rd] = result;

        self.pc += 1;
        1
    }

    /// Helper function used by compare ops to set status bits
    fn helper_cp_status_flags(&mut self, d: u8, r: u8, result: u8) {
        let d = d.view_bits::<Lsb0>();
        let r = r.view_bits::<Lsb0>();
        let result = result.view_bits::<Lsb0>();

        self.status.h = !d[3] && r[3] || r[3] && result[3] || result[3] && !d[3];
        self.status.v = d[7] && !r[7] && !result[7] || !d[7] && r[7] && result[7];
        self.status.n = result[7];
        self.status.s = self.status.n != self.status.v;
    }

    /// 49. Compare (CP Rd,Rr)
    fn do_cp(&mut self, rd: u8, rr: u8) -> usize {
        let (result, c) = self.regs[rd].overflowing_sub(self.regs[rr]);
        self.helper_cp_status_flags(self.regs[rd], self.regs[rr], result);
        self.status.z = result == 0;
        self.status.c = c;

        self.pc += 1;
        1
    }

    /// 50. Compare with Carry (CPC Rd,Rr)
    fn do_cpc(&mut self, rd: u8, rr: u8) -> usize {
        let (result, c0) = self.regs[rd].overflowing_sub(self.regs[rr]);
        let (result, c1) = result.overflowing_sub(self.status.c as u8);
        self.helper_cp_status_flags(self.regs[rd], self.regs[rr], result);
        if result != 0 {
            self.status.z = false;
        }
        self.status.c = c0 || c1;

        self.pc += 1;
        1
    }

    /// 51. Compare with Immediate (CPI Rd,K)
    fn do_cpi(&mut self, rd: u8, k: u8) -> usize {
        let (result, c) = self.regs[rd].overflowing_sub(k);
        self.helper_cp_status_flags(self.regs[rd], k, result);
        self.status.z = result == 0;
        self.status.c = c;

        self.pc += 1;
        1
    }

    /// 52. Compare Skip if Equal (CPSE Rd,Rr)
    fn do_cpse(&mut self, rd: u8, rr: u8, next_op_len: u8) -> usize {
        if self.regs[rd] == self.regs[rr] {
            self.pc += 1 + next_op_len as u16;
            1 + next_op_len as usize
        } else {
            self.pc += 1;
            1
        }
    }

    // 53. Decrement (DEC Rd)
    // fn do_dec(&mut self, rd: u8) -> usize {}
}
