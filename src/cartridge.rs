const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
const PRG_ROM_PAGE_SIZE: usize = 16384;
const CHR_ROM_PAGE_SIZE: usize = 8192;

#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Mirroring { // PPU related, will be covered later
   VERTICAL,
   HORIZONTAL,
   FOUR_SCREEN,
}

pub struct Rom {
   pub prg_rom: Vec<u8>, // program rom: contains the code data for the game
   pub chr_rom: Vec<u8>, // "character" rom: contains the visual data for the game
   pub mapper: u8, // to provide access to extra memory in the rom
   pub screen_mirroring: Mirroring,
}

impl Rom {
    pub fn new(raw: &Vec<u8>) -> Result<Rom, String> {
        if &raw[0..4] != NES_TAG { // first four bits don't match NES format
            return Err("File is not in iNES file format".to_string());
        }
 
        let mapper = (raw[7] & 0b1111_0000) | (raw[6] >> 4);
        // four upper bits of 7th byte (control byte 2) 
        // and four upper of 6th byte (control byte 1) together give
        // the u8 mapper type
 
        let ines_ver = (raw[7] >> 2) & 0b11; // iNES vers. from control byte 2 (bit 3,2)
        if ines_ver != 0 {
            return Err("NES2.0 format is not supported".to_string());
        }

        // get mirroring type from CB 1 (byte 6)
        let four_screen = raw[6] & 0b1000 != 0;
        let vertical_mirroring = raw[6] & 0b1 != 0;
        let screen_mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FOUR_SCREEN,
            (false, true) => Mirroring::VERTICAL,
            (false, false) => Mirroring::HORIZONTAL,
        };
 
        let prg_rom_size = raw[4] as usize * PRG_ROM_PAGE_SIZE;
        let chr_rom_size = raw[5] as usize * CHR_ROM_PAGE_SIZE;
 
        let skip_trainer = raw[6] & 0b100 != 0;
        // gets whether trainer exists and if so whether it should be skipped or not.
 
        let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 }; // if exits, skip it. 
        // Set starting position of code after the header accordingly.
        let chr_rom_start = prg_rom_start + prg_rom_size; // always starts after the prg rom.

        // if all works correclty,
        Ok(Rom {
            prg_rom: raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom: raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec(),
            mapper: mapper,
            screen_mirroring: screen_mirroring,
        })
    }
}


// tests imported from book
pub mod test {

    use super::*;

    struct TestRom {
        header: Vec<u8>,
        trainer: Option<Vec<u8>>, // if it exists, vector type. otherwise None
        pgp_rom: Vec<u8>,
        chr_rom: Vec<u8>,
    }

    fn create_rom(rom: TestRom) -> Vec<u8> {
        let mut result = Vec::with_capacity(
            rom.header.len()
                + rom.trainer
                    .as_ref() // take as a reference
                    .map_or(0, |t| t.len()) // if false, 0. otherwise get the length of the trainer
                + rom.pgp_rom.len()
                + rom.chr_rom.len(),
        );

        result.extend(&rom.header); // add header to rom
        if let Some(t) = rom.trainer {
            result.extend(t); // if it exists, add trainer
        }
        result.extend(&rom.pgp_rom); // add program code
        result.extend(&rom.chr_rom); // add character code

        result
    }

    pub fn test_rom() -> Rom {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E, 0x45, 0x53, 0x1A, 0x02, 0x01, 0x31, 00, 00, 00, 00, 00, 00, 00, 00, 00,
            ],
            trainer: None,
            pgp_rom: vec![1; 2 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });

        Rom::new(&test_rom).unwrap()
    }

    #[test]
    fn test() {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E, 0x45, 0x53, 0x1A, 0x02, 0x01, 0x31, 00, 00, 00, 00, 00, 00, 00, 00, 00,
            ],
            trainer: None,
            pgp_rom: vec![1; 2 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });

        let rom: Rom = Rom::new(&test_rom).unwrap();

        assert_eq!(rom.chr_rom, vec![2; 1 * CHR_ROM_PAGE_SIZE]);
        assert_eq!(rom.prg_rom, vec![1; 2 * PRG_ROM_PAGE_SIZE]);
        assert_eq!(rom.mapper, 3);
        assert_eq!(rom.screen_mirroring, Mirroring::VERTICAL);
    }

    #[test]
    fn test_with_trainer() {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E,
                0x45,
                0x53,
                0x1A,
                0x02,
                0x01,
                0x31 | 0b100,
                00,
                00,
                00,
                00,
                00,
                00,
                00,
                00,
                00,
            ],
            trainer: Some(vec![0; 512]),
            pgp_rom: vec![1; 2 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });

        let rom: Rom = Rom::new(&test_rom).unwrap();

        assert_eq!(rom.chr_rom, vec!(2; 1 * CHR_ROM_PAGE_SIZE));
        assert_eq!(rom.prg_rom, vec!(1; 2 * PRG_ROM_PAGE_SIZE));
        assert_eq!(rom.mapper, 3);
        assert_eq!(rom.screen_mirroring, Mirroring::VERTICAL);
    }

    #[test]
    fn test_nes2_is_not_supported() {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E, 0x45, 0x53, 0x1A, 0x01, 0x01, 0x31, 0x8, 00, 00, 00, 00, 00, 00, 00, 00,
            ],
            trainer: None,
            pgp_rom: vec![1; 1 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });
        let rom = Rom::new(&test_rom);
        match rom {
            Result::Ok(_) => assert!(false, "should not load rom"),
            Result::Err(str) => assert_eq!(str, "NES2.0 format is not supported"),
        }
    }
}