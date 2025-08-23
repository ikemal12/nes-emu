use crate::cpu::AddressingMode;
use crate::cpu::Mem;
use crate::cpu::CPU;
use crate::opcodes;
use std::collections::HashMap;

pub fn trace(cpu: &CPU) -> String {
    let ref opcodes = HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPCODES_MAP;
    let code = cpu.mem_read(cpu.program_counter);
    let ops = opscodes.get(&code).unwrap();
    let begin = cpu.program_counter;
    let mut hex_dump = vec![];
    hex_dump.push(code);

    let (mem_addr, stored_value) = match ops.mode {
        AddressingMode::Immediate | AddressingMode::NoneAddressing => (0, 0),
        _ => {
            let addr = cpu.get_absolute_address(&ops.mode, begin + 1);
            (addr, cpu.mem_read(addr))
        }
    };

    let tmp = match.ops.len {
        1 => match.ops.code {
            0x0a | 0x4a | 0x2a | 0x6a => format!("A "),
            _ => String::from(""),
        },
        2 => {
            let address: u8 = cpu.mem_read(begin + 1);
        }
    }
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