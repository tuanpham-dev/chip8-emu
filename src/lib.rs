use rand::Rng;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const START_ADDRESS: u16 = 0x200;
const RAM_SIZE: usize = 4096;
const NUM_REGISTER_V: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Chip8 {
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    ram: [u8; RAM_SIZE],
    program_counter: u16,
    register_v: [u8; NUM_REGISTER_V],
    register_i: u16,
    delay_timer: u8,
    sound_timer: u8,
    stack_pointer: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    is_debug: bool
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip = Self {
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            ram: [0; RAM_SIZE],
            program_counter: START_ADDRESS,
            register_v: [0; NUM_REGISTER_V],
            register_i: 0,
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            is_debug: false
        };

        chip.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        chip
    }

    pub fn reset(&mut self) {
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.ram = [0; RAM_SIZE];
        self.program_counter = START_ADDRESS;
        self.register_v = [0; NUM_REGISTER_V];
        self.register_i = 0;
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.stack_pointer = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        self.is_debug = false;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDRESS as usize;
        let end = start + data.len();

        self.ram[start..end].copy_from_slice(data);
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, key_index: usize, is_pressed: bool) {
        self.keys[key_index] = is_pressed;
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn is_beeping(&self) -> bool {
        self.sound_timer > 0
    }

    pub fn tick(&mut self) {
        let opcode = self.fetch();

        if self.is_debug {
            self.execute_with_debug(opcode);
        } else {
            self.execute(opcode);
        }
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.program_counter as usize] as u16;
        let lower_byte = self.ram[(self.program_counter + 1) as usize] as u16;
        self.program_counter += 2;

        (higher_byte << 8) | lower_byte
    }

    fn execute(&mut self, opcode: u16) {
        let digit1 = (opcode & 0xF000) >> 12;
        let digit2 = (opcode & 0x0F00) >> 8;
        let digit3 = (opcode & 0x00F0) >> 4;
        let digit4 = opcode & 0x000F;
        let nnn = opcode & 0x0FFF;
        let nn = (opcode & 0x00FF) as u8;
        let x = digit2 as usize;
        let y = digit3 as usize;

        match (digit1, digit2, digit3, digit4) {
            // NOP
            (0, 0, 0, 0) => {
                return
            },
            // CLS
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },
            // RET
            (0, 0, 0xE, 0xE) => {
                let return_address = self.stack_pop();
                self.program_counter = return_address;
            },
            // JMP NNN
            (1, _, _, _) => {
                self.program_counter = nnn;
            },
            // CALL NNN
            (2, _, _, _) => {
                self.stack_push(self.program_counter);
                self.program_counter = nnn;
            },
            // SKIP IF VX == NN
            (3, _, _, _) => {
                if self.register_v[x] == nn {
                    self.program_counter += 2;
                }
            },
            // SKIP IF VX != NN
            (4, _, _, _) => {
                if self.register_v[x] != nn {
                    self.program_counter += 2;
                }
            },
            // SKIP IF VX == VY
            (5, _, _, _) => {
                if self.register_v[x] == self.register_v[y] {
                    self.program_counter += 2;
                }
            },
            // VX = NN
            (6, _, _, _) => {
                self.register_v[x] = nn;
            },
            // VX += NN
            (7, _, _, _) => {
                self.register_v[x] = self.register_v[x].wrapping_add(nn);
            },
            // VX = VY
            (8, _, _, 0) => {
                self.register_v[x] = self.register_v[y];
            },
            // VX |= VY
            (8, _, _, 1) => {
                self.register_v[x] |= self.register_v[y];
            },
            // VX &= VY
            (8, _, _, 2) => {
                self.register_v[x] &= self.register_v[y];
            },
            // VX ^= VY
            (8, _, _, 3) => {
                self.register_v[x] ^= self.register_v[y];
            },
            // VX += VY
            (8, _, _, 4) => {
                let (value, carry) = self.register_v[x].overflowing_add(self.register_v[y]);

                self.register_v[x] = value;
                self.register_v[0xF] = carry as u8;
            },
            // VX -= VY
            (8, _, _, 5) => {
                let (value, borrow) = self.register_v[x].overflowing_sub(self.register_v[y]);

                self.register_v[x] = value;
                self.register_v[0xF] = !borrow as u8;
            },
            // VX >>= 1
            (8, _, _, 6) => {
                self.register_v[0xF] = self.register_v[x] & 0x0001;
                self.register_v[x] >>= 1;
            },
            // VX = VY - VX
            (8, _, _, 7) => {
                let (value, borrow) = self.register_v[y].overflowing_sub(self.register_v[x]);

                self.register_v[x] = value;
                self.register_v[0xF] = !borrow as u8;
            },
            // VX <<= 1
            (8, _, _, 0x0E) => {
                self.register_v[0xF] = (self.register_v[x] >> 7) & 0x01;
                self.register_v[x] <<= 1;
            },
            // SKIP IF VX != VY
            (9, _, _, 0) => {
                if self.register_v[x] != self.register_v[y] {
                    self.program_counter += 2;
                }
            },
            // I = NNN
            (0xA, _, _, _) => {
                self.register_i = nnn;
            },
            // JMP V0 + NNN
            (0xB, _, _, _) => {
                self.program_counter = (self.register_v[0] as u16) + nnn;
            },
            // VX = rand() & NN
            (0xC, _, _, _) => {
                let rng: u8 = rand::thread_rng().gen();
                self.register_v[x] = rng & nn;
            },
            // DRAW
            (0xD, _, _, _) => {
                // get the (x, y) coordinates from the sprite
                let x_coordinate = self.register_v[x] as u16;
                let y_coordinate = self.register_v[y] as u16;

                // the last digit determins how many rows high the spirte is
                let num_rows = digit4;

                // keep track if any pixels were flipped
                let mut flipped = false;

                // interate over each row of the sprite
                for y_line in 0..num_rows {
                    // determine which memory address the row's data is stored
                    let address = self.register_i + y_line as u16;
                    let pixels = self.ram[address as usize];


                    // iterate over each column in the row
                    for x_line in 0..8 {
                        // use a mask to fetch current pixel's bit. Only flip if a 1
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            // sprites should wrap around screen, so apply modulo
                            let x = (x_coordinate + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coordinate + y_line) as usize % SCREEN_HEIGHT;

                            // get the pixel's index in the 1D screen array
                            let index = x + SCREEN_WIDTH * y;
                            // check if we're about to flip the pixel and set
                            flipped |= self.screen[index];
                            self.screen[index] ^= true;
                        }
                    }
                }

                // populate VF register
                self.register_v[0xF] = flipped as u8;
            },
            // SKIP KEY PRESS
            (0xE, _, 9, 0xE) => {
                let key = self.keys[self.register_v[x] as usize];

                if key {
                    self.program_counter += 2;
                }
            },
            // SKIP KEY RELEASE
            (0xE, _, 0xA, 1) => {
                let key = self.keys[self.register_v[x] as usize];

                if !key {
                    self.program_counter += 2;
                }
            },
            // VX = DT,
            (0xF, _, 0, 7) => {
                self.register_v[x] = self.delay_timer;
            },
            // WAIT KEY
            (0xF, _, 0, 0xA) => {
                self.register_v[x] = self.delay_timer;
                let mut is_pressed = false;

                for i in 0..NUM_KEYS {
                    if self.keys[i] {
                        self.register_v[x] = i as u8;
                        is_pressed = true;
                        break;
                    }
                }

                if !is_pressed {
                    self.program_counter -= 2;
                }
            },
            // DT = VX
            (0xF, _, 1, 5) => {
                self.delay_timer = self.register_v[x];
            },
            // ST = VX
            (0xF, _, 1, 8) => {
                self.sound_timer = self.register_v[x];
            },
            // I += VX
            (0xF, _, 1, 0xE) => {
                self.register_i = self.register_i.wrapping_add(self.register_v[x] as u16);
            },
            // I = FONT
            (0xF, _, 2, 9) => {
                self.register_i = self.register_v[x] as u16 * 5;
            },
            // BCD
            (0xF, _, 3, 3) => {
                let vx = self.register_v[x] as f32;

                // fetch the hundreds digit by dividing by 100 and tossing the decimal
                let hundreds = (vx / 100.0).floor() as u8;
                // fetch the tens digit by dividing by 10, tossing the ones digit and the decimal
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                // fetch the ones digit by tossing the hundreds and the tens
                let ones = (vx % 10.0) as u8;

                let i = self.register_i as usize;
                self.ram[i] = hundreds;
                self.ram[i + 1] = tens;
                self.ram[i + 2] = ones;
            },
            // STORE V0 - VX
            (0xF, _, 5, 5) => {
                let i = self.register_i as usize;

                for index in 0..=x {
                    self.ram[i + index] = self.register_v[index];
                }
            },
            // LOAD V0 - VX
            (0xF, _, 6, 5) => {
                let i = self.register_i as usize;

                for index in 0..=x {
                    self.register_v[index] = self.ram[i + index];
                }
            },
            _ => unimplemented!("Unimplemented opcode: {:#04x}", opcode)
        }
    }

    fn execute_with_debug(&mut self, opcode: u16) {
        let digit1 = (opcode & 0xF000) >> 12;
        let digit2 = (opcode & 0x0F00) >> 8;
        let digit3 = (opcode & 0x00F0) >> 4;
        let digit4 = opcode & 0x000F;
        let nnn = opcode & 0x0FFF;
        let nn = (opcode & 0x00FF) as u8;
        let x = digit2 as usize;
        let y = digit3 as usize;

        match (digit1, digit2, digit3, digit4) {
            // NOP
            (0, 0, 0, 0) => {
                println!("{:#04x} NOP", opcode);
                return
            },
            // CLS
            (0, 0, 0xE, 0) => {
                println!("{:#04x} CLS", opcode);
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },
            // RET
            (0, 0, 0xE, 0xE) => {
                println!("{:#04x} RET", opcode);
                let return_address = self.stack_pop();
                self.program_counter = return_address;
            },
            // JMP NNN
            (1, _, _, _) => {
                println!("{:#04x} JMP {:#04x}", opcode, nnn);
                self.program_counter = nnn;
            },
            // CALL NNN
            (2, _, _, _) => {
                println!("{:#04x} CALL {:#04x}", opcode, nnn);
                self.stack_push(self.program_counter);
                self.program_counter = nnn;
            },
            // SKIP IF VX == NN
            (3, _, _, _) => {
                println!("{:#04x} SE V{}, {:#02x}", opcode, x, nn);
                if self.register_v[x] == nn {
                    self.program_counter += 2;
                }
            },
            // SKIP IF VX != NN
            (4, _, _, _) => {
                println!("{:#04x} SNE V{}, {:#02x}", opcode, x, nn);
                if self.register_v[x] != nn {
                    self.program_counter += 2;
                }
            },
            // SKIP IF VX == VY
            (5, _, _, _) => {
                println!("{:#04x} SE V{}, V{}", opcode, x, y);
                if self.register_v[x] == self.register_v[y] {
                    self.program_counter += 2;
                }
            },
            // VX = NN
            (6, _, _, _) => {
                println!("{:#04x} LD V{}, {:#02x}", opcode, x, nn);
                self.register_v[x] = nn;
            },
            // VX += NN
            (7, _, _, _) => {
                println!("{:#04x} ADD V{}, {:#02x}", opcode, x, nn);
                self.register_v[x] = self.register_v[x].wrapping_add(nn);
            },
            // VX = VY
            (8, _, _, 0) => {
                println!("{:#04x} LD V{}, V{}", opcode, x, y);
                self.register_v[x] = self.register_v[y];
            },
            // VX |= VY
            (8, _, _, 1) => {
                println!("{:#04x} OR V{}, V{}", opcode, x, y);
                self.register_v[x] |= self.register_v[y];
            },
            // VX &= VY
            (8, _, _, 2) => {
                println!("{:#04x} AND V{}, V{}", opcode, x, y);
                self.register_v[x] &= self.register_v[y];
            },
            // VX ^= VY
            (8, _, _, 3) => {
                println!("{:#04x} XOR V{}, V{}", opcode, x, y);
                self.register_v[x] ^= self.register_v[y];
            },
            // VX += VY
            (8, _, _, 4) => {
                println!("{:#04x} ADD V{}, V{}", opcode, x, y);
                let (value, carry) = self.register_v[x].overflowing_add(self.register_v[y]);

                self.register_v[x] = value;
                self.register_v[0xF] = carry as u8;
            },
            // VX -= VY
            (8, _, _, 5) => {
                println!("{:#04x} SUB V{}, V{}", opcode, x, y);
                let (value, borrow) = self.register_v[x].overflowing_sub(self.register_v[y]);

                self.register_v[x] = value;
                self.register_v[0xF] = !borrow as u8;
            },
            // VX >>= 1
            (8, _, _, 6) => {
                println!("{:#04x} SHR V{}", opcode, x);
                self.register_v[0xF] = self.register_v[x] & 0x0001;
                self.register_v[x] >>= 1;
            },
            // VX = VY - VX
            (8, _, _, 7) => {
                println!("{:#04x} SUBN V{}, V{}", opcode, x, y);
                let (value, borrow) = self.register_v[y].overflowing_sub(self.register_v[x]);

                self.register_v[x] = value;
                self.register_v[0xF] = !borrow as u8;
            },
            // VX <<= 1
            (8, _, _, 0x0E) => {
                println!("{:#04x} SHL V{}", opcode, x);
                self.register_v[0xF] = (self.register_v[x] >> 7) & 0x01;
                self.register_v[x] <<= 1;
            },
            // SKIP IF VX != VY
            (9, _, _, 0) => {
                println!("{:#04x} SNE V{}, V{}", opcode, x, y);
                if self.register_v[x] != self.register_v[y] {
                    self.program_counter += 2;
                }
            },
            // I = NNN
            (0xA, _, _, _) => {
                println!("{:#04x} LD I, {:#04x}", opcode, nnn);
                self.register_i = nnn;
            },
            // JMP V0 + NNN
            (0xB, _, _, _) => {
                println!("{:#04x} JMP V0, {:#04x}", opcode, nnn);
                self.program_counter = (self.register_v[0] as u16) + nnn;
            },
            // VX = rand() & NN
            (0xC, _, _, _) => {
                println!("{:#04x} RND V{}, {:#02x}", opcode, x, nn);
                let rng: u8 = rand::thread_rng().gen();
                self.register_v[x] = rng & nn;
            },
            // DRAW
            (0xD, _, _, _) => {
                println!("{:#04x} DRW V{}, V{}, {:#01x}", opcode, x, y, digit4);
                // get the (x, y) coordinates from the sprite
                let x_coordinate = self.register_v[x] as u16;
                let y_coordinate = self.register_v[y] as u16;

                // the last digit determins how many rows high the spirte is
                let num_rows = digit4;

                // keep track if any pixels were flipped
                let mut flipped = false;

                // interate over each row of the sprite
                for y_line in 0..num_rows {
                    // determine which memory address the row's data is stored
                    let address = self.register_i + y_line as u16;
                    let pixels = self.ram[address as usize];


                    // iterate over each column in the row
                    for x_line in 0..8 {
                        // use a mask to fetch current pixel's bit. Only flip if a 1
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            // sprites should wrap around screen, so apply modulo
                            let x = (x_coordinate + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coordinate + y_line) as usize % SCREEN_HEIGHT;

                            // get the pixel's index in the 1D screen array
                            let index = x + SCREEN_WIDTH * y;
                            // check if we're about to flip the pixel and set
                            flipped |= self.screen[index];
                            self.screen[index] ^= true;
                        }
                    }
                }

                // populate VF register
                self.register_v[0xF] = flipped as u8;
            },
            // SKIP KEY PRESS
            (0xE, _, 9, 0xE) => {
                println!("{:#04x} SKP V{}", opcode, x);
                let key = self.keys[self.register_v[x] as usize];

                if key {
                    self.program_counter += 2;
                }
            },
            // SKIP KEY RELEASE
            (0xE, _, 0xA, 1) => {
                println!("{:#04x} SKNP V{}", opcode, x);
                let key = self.keys[self.register_v[x] as usize];

                if !key {
                    self.program_counter += 2;
                }
            },
            // VX = DT,
            (0xF, _, 0, 7) => {
                println!("{:#04x} LD V{}, DT", opcode, x);
                self.register_v[x] = self.delay_timer;
            },
            // WAIT KEY
            (0xF, _, 0, 0xA) => {
                println!("{:#04x} LD V{}, K", opcode, x);
                self.register_v[x] = self.delay_timer;
                let mut is_pressed = false;

                for i in 0..NUM_KEYS {
                    if self.keys[i] {
                        self.register_v[x] = i as u8;
                        is_pressed = true;
                        break;
                    }
                }

                if !is_pressed {
                    self.program_counter -= 2;
                }
            },
            // DT = VX
            (0xF, _, 1, 5) => {
                println!("{:#04x} LD DT, V{}", opcode, x);
                self.delay_timer = self.register_v[x];
            },
            // ST = VX
            (0xF, _, 1, 8) => {
                println!("{:#04x} LD ST, V{}", opcode, x);
                self.sound_timer = self.register_v[x];
            },
            // I += VX
            (0xF, _, 1, 0xE) => {
                println!("{:#04x} ADD I, V{}", opcode, x);
                self.register_i = self.register_i.wrapping_add(self.register_v[x] as u16);
            },
            // I = FONT
            (0xF, _, 2, 9) => {
                println!("{:#04x} LD F, V{}", opcode, x);
                self.register_i = self.register_v[x] as u16 * 5;
            },
            // BCD
            (0xF, _, 3, 3) => {
                println!("{:#04x} LD B, V{}", opcode, x);
                let vx = self.register_v[x] as f32;

                // fetch the hundreds digit by dividing by 100 and tossing the decimal
                let hundreds = (vx / 100.0).floor() as u8;
                // fetch the tens digit by dividing by 10, tossing the ones digit and the decimal
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                // fetch the ones digit by tossing the hundreds and the tens
                let ones = (vx % 10.0) as u8;

                let i = self.register_i as usize;
                self.ram[i] = hundreds;
                self.ram[i + 1] = tens;
                self.ram[i + 2] = ones;
            },
            // STORE V0 - VX
            (0xF, _, 5, 5) => {
                println!("{:#04x} LD [I], V{}", opcode, x);
                let i = self.register_i as usize;

                for index in 0..=x {
                    self.ram[i + index] = self.register_v[index];
                }
            },
            // LOAD V0 - VX
            (0xF, _, 6, 5) => {
                println!("{:#04x} LD V{}, [I]", opcode, x);
                let i = self.register_i as usize;

                for index in 0..=x {
                    self.register_v[index] = self.ram[i + index];
                }
            },
            _ => unimplemented!("Unimplemented opcode: {:#04x}", opcode)
        }
    }

    fn stack_push(&mut self, data: u16) {
        self.stack[self.stack_pointer as usize] = data;
        self.stack_pointer += 1;
    }

    fn stack_pop(&mut self) -> u16 {
        self.stack_pointer -= 1;

        self.stack[self.stack_pointer as usize]
    }
}