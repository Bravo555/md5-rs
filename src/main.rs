use std::{env, fs, mem};

fn main() {
    let mut args = env::args().skip(1);
    let filename = args.next().expect("usage: ./progname file");

    let data = fs::read(filename).unwrap();
    // perform md5 upon read data
    println!("{}", md5(&data));
}

fn md5(data: &Vec<u8>) -> u128 {
    // 512 bits is 64 bytes (u8)
    if data.len() % 64 != 0 {
        let data = add_padding(&data);
    }

    unimplemented!()
}

fn add_padding(data: &Vec<u8>) -> Vec<u8> {
    use std::convert::{TryFrom, TryInto};
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

    data.extend_from_slice(&msg_size.to_be_bytes());

    println!("{:#?}\n{}", data, data.len());
    unimplemented!()
}
