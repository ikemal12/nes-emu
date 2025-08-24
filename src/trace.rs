use crate::cpu::AddressingMode;
use crate::cpu::Mem;
use crate::cpu::CPU;
use crate::opcodes;
use std::collections::HashMap;

pub fn trace(cpu: &CPU) -> String {
    let ref opcodes: HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPCODES_MAP;
    let code = cpu.mem_read(cpu.program_counter);
    
    let ops = match opcodes.get(&code) {
        Some(opcode) => opcode,
        None => {
            return format!(
                "{:04X}  {:02X}        *UNKNOWN                        A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
                cpu.program_counter, code, cpu.register_a, cpu.register_x, cpu.register_y, cpu.status.bits(), cpu.stack_pointer
            );
        }
    };
    let begin = cpu.program_counter;
    let mut hex_dump = vec![];
    hex_dump.push(code);

    let (mem_addr, stored_value) = match ops.mode {
        AddressingMode::Immediate | AddressingMode::NoneAddressing => (0, 0),
        _ => {
            let addr = match ops.mode {
                AddressingMode::ZeroPage => cpu.mem_read(begin + 1) as u16,
                AddressingMode::ZeroPage_X => {
                    let pos = cpu.mem_read(begin + 1);
                    pos.wrapping_add(cpu.register_x) as u16
                },
                AddressingMode::ZeroPage_Y => {
                    let pos = cpu.mem_read(begin + 1);
                    pos.wrapping_add(cpu.register_y) as u16
                },
                AddressingMode::Absolute => cpu.mem_read_u16(begin + 1),
                AddressingMode::Absolute_X => {
                    let base = cpu.mem_read_u16(begin + 1);
                    base.wrapping_add(cpu.register_x as u16)
                },
                AddressingMode::Absolute_Y => {
                    let base = cpu.mem_read_u16(begin + 1);
                    base.wrapping_add(cpu.register_y as u16)
                },
                AddressingMode::Indirect_X => {
                    let base = cpu.mem_read(begin + 1);
                    let ptr = base.wrapping_add(cpu.register_x);
                    let lo = cpu.mem_read(ptr as u16);
                    let hi = cpu.mem_read(ptr.wrapping_add(1) as u16);
                    (hi as u16) << 8 | (lo as u16)
                },
                AddressingMode::Indirect_Y => {
                    let base = cpu.mem_read(begin + 1);
                    let lo = cpu.mem_read(base as u16);
                    let hi = cpu.mem_read(base.wrapping_add(1) as u16);
                    let deref_base = (hi as u16) << 8 | (lo as u16);
                    deref_base.wrapping_add(cpu.register_y as u16)
                },
                _ => 0,
            };
            (addr, cpu.mem_read(addr))
        }
    };

    let tmp = match ops.len {
        1 => match ops.code {
            0x0a | 0x4a | 0x2a | 0x6a => format!("A "),
            _ => String::from(""),
        },
        2 => {
            let address: u8 = cpu.mem_read(begin + 1);
            hex_dump.push(address);

            match ops.mode {
                AddressingMode::Immediate => format!("#${:02X}", address),
                AddressingMode::ZeroPage => format!("${:02X} = {:02X}", mem_addr, stored_value),
                AddressingMode::ZeroPage_X => format!("${:02X},X @ {:02X} = {:02X}", address, mem_addr, stored_value),
                AddressingMode::ZeroPage_Y => format!("${:02X},Y @ {:02X} = {:02X}", address, mem_addr, stored_value),
                AddressingMode::Absolute => format!("${:04X} = {:02X}", mem_addr, stored_value),
                AddressingMode::Absolute_X => format!("${:04X},X @ {:04X} = {:02X}", (mem_addr.wrapping_sub(cpu.register_x as u16)), mem_addr, stored_value),
                AddressingMode::Absolute_Y => format!("${:04X},Y @ {:04X} = {:02X}", (mem_addr.wrapping_sub(cpu.register_y as u16)), mem_addr, stored_value),
                AddressingMode::Indirect_X => format!("(${:02X},X) @ {:02X} = {:04X} = {:02X}", address, (address.wrapping_add(cpu.register_x)), mem_addr, stored_value),
                AddressingMode::Indirect_Y => format!("(${:02X}),Y = {:04X} @ {:04X} = {:02X}", address, (mem_addr.wrapping_sub(cpu.register_y as u16)), mem_addr, stored_value),
                AddressingMode::NoneAddressing => {
                    let address = cpu.mem_read_u16(begin + 1);
                    hex_dump.push((address & 0xff) as u8);
                    hex_dump.push((address >> 8) as u8);
                    format!("${:04X}", address)
                }
            }
        },
        3 => {
            let address_lo = cpu.mem_read(begin + 1);
            let address_hi = cpu.mem_read(begin + 2);
            hex_dump.push(address_lo);
            hex_dump.push(address_hi);

            let address = cpu.mem_read_u16(begin + 1);

            match ops.mode {
                AddressingMode::NoneAddressing => {
                    if ops.code == 0x6c {
                        format!("(${:04X})", address)
                    } else {
                        format!("${:04X}", address)
                    }
                }
                AddressingMode::Absolute => format!("${:04X} = {:02X}", mem_addr, stored_value),
                AddressingMode::Absolute_X => format!("${:04X},X @ {:04X} = {:02X}", address, mem_addr, stored_value),
                AddressingMode::Absolute_Y => format!("${:04X},Y @ {:04X} = {:02X}", address, mem_addr, stored_value),
                _ => panic!("unexpected addressing mode {:?} has ops-len 3", ops.mode)
            }
        },
        _ => String::from(""),
    };

    let hex_str = hex_dump
        .iter()
        .map(|z| format!("{:02X}", z))
        .collect::<Vec<String>>()
        .join(" ");
    let asm_str = format!("{:04X}  {:8} {: >4} {}", begin, hex_str, ops.mnemonic, tmp)
        .trim()
        .to_string();

    format!(
        "{:47} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
        asm_str, cpu.register_a, cpu.register_x, cpu.register_y, cpu.status.bits(), cpu.stack_pointer
    )
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::bus::Bus;
    use crate::cartridge::test::test_rom;

    #[test]
    fn test_format_trace() {
        let mut bus = Bus::new(test_rom());
        bus.mem_write(100, 0xa2);
        bus.mem_write(101, 0x01);
        bus.mem_write(102, 0xca);
        bus.mem_write(103, 0x88);
        bus.mem_write(104, 0x00);

        let mut cpu = CPU::new(bus);
        cpu.program_counter = 0x64;
        cpu.register_a = 1;
        cpu.register_x = 2;
        cpu.register_y = 3;
        let mut result: Vec<String> = vec![];
        cpu.run_with_callback(|cpu| {
            result.push(trace(cpu));
        });
        assert_eq!(
            "0067  88        DEY                             A:01 X:00 Y:03 P:26 SP:FD",
           result[2]
        );
    }

    #[test]
    fn test_format_mem_access() {
        let mut bus = Bus::new(test_rom());
        // ORA ($33), Y
        bus.mem_write(100, 0x11);
        bus.mem_write(101, 0x33);

        // data
        bus.mem_write(0x33, 00);
        bus.mem_write(0x34, 04);

        // target cell
        bus.mem_write(0x400, 0xAA);

        let mut cpu = CPU::new(bus);
        cpu.program_counter = 0x64;
        cpu.register_y = 0;
        let mut result: Vec<String> = vec![];
        cpu.run_with_callback(|cpu| {
            result.push(trace(cpu));
        });
        assert_eq!(
           "0064  11 33     ORA ($33),Y = 0400 @ 0400 = AA  A:00 X:00 Y:00 P:24 SP:FD",
           result[0]
        );
    }
}