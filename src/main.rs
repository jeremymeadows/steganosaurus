use std::io::{self, Seek, Read, Write};
use std::fs::{self, File};
use std::{env, process};

const BUF_SIZE: usize = 1024;

fn help() -> ! {
    println!(
        "Steganosaurus v0.1.0

Usage:
    steganosaurus MODE <INPUT> <OUTPUT> [MESSAGE]

Hide messages or files in plain sight with steganography.

Modes:
    encode    Encodes MESSAGE into the INPUT file and saves it to OUTPUT
    decode    Decodes the message from INPUT and writes it to OUTPUT

ARGS:
    INPUT      The input image to use.
    OUTPUT     The location to save the output image (in encode mode) or the
               decoded message (in decode mode).
    MESSAGE    The message to hide in the image (only in encode mode).
"
    );

    process::exit(0);
}

fn main() -> io::Result<()> {
    let (mode, in_file_name, out_file_name, msg) = argparse();

    let mut in_file = File::open(&in_file_name)?;
    in_file.seek(io::SeekFrom::Start(10))?;
    
    let mut out_file = File::create(out_file_name)?;
    let data = fs::read(in_file_name)?;

    let mut off = [0u8; 4];
    in_file.read_exact(&mut off)?;
    let off = u32::from_le_bytes(off) as usize;
    in_file.seek(io::SeekFrom::Start(off as u64))?;

    let mut buf_ndx = 0;
    let mut buf = [0; BUF_SIZE];

    match mode.as_str() {
        "encode" | "enc" | "e" => {
            let msg = msg.unwrap();

            let msg_data = msg.as_bytes();
            let mut msg_len = [0u8; 4];
            for i in 0..4 {
                msg_len[i] = ((msg.len() as u32 >> (8 * i)) & 0xff) as u8;
            }

            let mut msg = Vec::from(msg_len);
            msg.extend_from_slice(msg_data);

            let mut msg_ndx = 0;
            let mut msg_off = 0;

            out_file.write(&data[0..off])?;

            for byte in data[off..].to_vec() {
                if msg_ndx < msg.len() {
                    buf[buf_ndx] = (byte & 0xfc) | (0xff & ((msg[msg_ndx] >> (2 * msg_off)) & 0b11));
                    msg_off += 1;

                    if msg_off == 4 {
                        msg_ndx += 1;
                        msg_off = 0;
                    }
                } else {
                    buf[buf_ndx] = byte;
                }
                buf_ndx += 1;

                if buf_ndx == BUF_SIZE {
                    out_file.write(&buf)?;
                    buf_ndx = 0;
                }
            }
            out_file.write(&buf[0..buf_ndx])?;
        }
        "decode" | "dec" | "d" => {
            let data = &data[(off as usize)..];

            let mut msg_len = 0usize;
            for i in (0..16).step_by(4) {
                for j in 0..4 {
                    msg_len += (((data[i+j] & 0b11) << 2 * j) as usize) << (8 * (i / 4));
                }
            }

            for i in (16..((msg_len * 4) + 16)).step_by(4) {
                let mut x = 0;
                for j in 0..4 {
                    x += (data[i+j] & 0b11) << 2 * j;
                }

                buf[buf_ndx] = x;
                buf_ndx += 1;

                if buf_ndx == BUF_SIZE {
                    out_file.write(&buf)?;
                    buf_ndx = 0;
                }
            }
            out_file.write(&buf[0..buf_ndx])?;
        }
        _ => {}
    }

    Ok(())
}

fn argparse() -> (String, String, String, Option<String>) {
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 4 || args.len() > 5 {
        help();
    }
    let msg = match args[1].as_str() {
        "encode" | "enc" | "e" => Some(args[4].clone()),
        "decode" | "dec" | "d" => None,
        _ => help(),
    };

    (args[1].clone(), args[2].clone(), args[3].clone(), msg)
}
