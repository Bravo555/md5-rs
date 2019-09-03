use std::{convert::TryInto, env, fs, mem};

fn main() {
    let mut args = env::args().skip(1);
    let filename = args.next().expect("usage: ./progname file");

    let data = "a".bytes().collect();
    // perform md5 upon read data
    let result = md5(&data);

    println!(
        "{:x}{:x}{:x}{:x}",
        result[0], result[1], result[2], result[3]
    );
}

fn md5(data: &Vec<u8>) -> [u32; 4] {
    // 512 bits is 64 bytes (u8)
    let data = add_padding(&data);

    // group the data into a word (u32) sized chunks
    let data: Vec<u32> = data
        .chunks_exact(4)
        .map(|chunk| {
            if let &[a, b, c, d] = chunk {
                u32::from_le_bytes([a, b, c, d])
            } else {
                // not possible as we have ensured we have multiple of 64 elements in the vector
                unreachable!()
            }
        })
        .collect();
    let data = data.chunks_exact(16);

    let mut state = Md5State::new();

    let f1 = |x: u32, y: u32, z: u32| -> u32 { (x & y) | (!x & z) };
    let f2 = |x: u32, y: u32, z: u32| -> u32 { (x & z) | (y & !z) };
    let f3 = |x: u32, y: u32, z: u32| -> u32 { x ^ y ^ z };
    let f4 = |x: u32, y: u32, z: u32| -> u32 { y ^ (x | !z) };

    let t: Vec<_> = (0..64)
        .map(|i| (2u64.pow(32) as f64 * ((i + 1) as f64).sin().abs()).floor() as u32)
        .collect();

    let shifts = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5,
        9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10,
        15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];

    for block in data {
        let mut current_state = state.clone();

        // ROUND 1
        //  /* Let [abcd k s i] denote the operation
        //       a = b + ((a + F(b,c,d) + X[k] + T[i]) <<< s). */
        //  /* Do the following 16 operations. */
        //  [ABCD  0  7  1]  [DABC  1 12  2]  [CDAB  2 17  3]  [BCDA  3 22  4]
        //  [ABCD  4  7  5]  [DABC  5 12  6]  [CDAB  6 17  7]  [BCDA  7 22  8]
        //  [ABCD  8  7  9]  [DABC  9 12 10]  [CDAB 10 17 11]  [BCDA 11 22 12]
        //  [ABCD 12  7 13]  [DABC 13 12 14]  [CDAB 14 17 15]  [BCDA 15 22 16]
        for i in 0..64 {
            match i {
                0..=15 => {
                    let g = i;
                    current_state.round(f1, block[g], t[i], shifts[i])
                }
                16..=31 => {
                    let g = (5 * i + 1) % 16;
                    current_state.round(f2, block[g], t[i], shifts[i])
                }
                32..=47 => {
                    let g = (3 * i + 5) % 16;
                    current_state.round(f3, block[g], t[i], shifts[i])
                }
                48..=63 => {
                    let g = (7 * i) % 16;
                    current_state.round(f4, block[g], t[i], shifts[i])
                }
                _ => {}
            }
        }

        state = state.update_state(current_state);
    }

    let a = state.a.swap_bytes();
    let b = state.b.swap_bytes();
    let c = state.c.swap_bytes();
    let d = state.d.swap_bytes();

    [a, b, c, d]
}

fn add_padding(data: &Vec<u8>) -> Vec<u8> {
    let block_size = 512;

    let mut data = data.clone();
    let msg_size: u64 = (data.len() * mem::size_of::<u8>())
        .try_into()
        .expect("error during conversion message size from usize to u64");
    let wrapped_size = (msg_size + 64 + block_size) / block_size * block_size - 64;

    // appending `1` bit to the end of the message
    data.push(0x80);
    // filling the rest with `0` bits
    data.resize((wrapped_size / 8).try_into().unwrap(), 0x00);

    // append message length
    data.extend_from_slice(&msg_size.to_le_bytes());

    data
}

#[derive(Debug, Clone)]
struct Md5State {
    a: u32,
    b: u32,
    c: u32,
    d: u32,
}

impl Md5State {
    fn new() -> Md5State {
        Md5State {
            a: u32::from_le_bytes([0x01, 0x23, 0x45, 0x67]),
            b: u32::from_le_bytes([0x89, 0xab, 0xcd, 0xef]),
            c: u32::from_le_bytes([0xfe, 0xdc, 0xba, 0x98]),
            d: u32::from_le_bytes([0x76, 0x54, 0x32, 0x10]),
        }
    }

    fn round<'a, F>(&mut self, f: F, msg_block: u32, key: u32, shift_by: u32)
    where
        F: Fn(u32, u32, u32) -> u32,
    {
        let e = self
            .a
            .wrapping_add(f(self.b, self.c, self.d))
            .wrapping_add(msg_block)
            .wrapping_add(key);

        self.a = self.d;
        self.d = self.c;
        self.c = self.b;
        self.b = self.b.wrapping_add(e.rotate_left(shift_by));
    }

    fn update_state(self, new_state: Md5State) -> Md5State {
        Md5State {
            a: self.a.wrapping_add(new_state.a),
            b: self.b.wrapping_add(new_state.b),
            c: self.c.wrapping_add(new_state.c),
            d: self.d.wrapping_add(new_state.d),
        }
    }
}
