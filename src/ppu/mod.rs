pub struct NesPPU {
    pub chr_rom: Vec<u8>,
    pub vram: [u8; 2048],
    pub oam_data: [u8; 256],
    pub palette_table: [u8; 32],
    pub mirroring: Mirroring,
    pub addr: AddrRegister,
}

impl NesPPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        NesPPU {
            chr_rom: chr_rom,
            mirroring: mirroring,
            vram: [0; 2048],
            oam_data: [0; 64 * 4],
            palette_table: [0; 32],
        }
    }

    fn write_to_ppu_addr(&mut self, value: u8) {
        self.addr.update(value);
    }
}