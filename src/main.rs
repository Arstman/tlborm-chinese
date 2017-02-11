//! Dirty fix for rustbook's parsing errors
#![allow(warnings)]
use std::io::{Read, Write};

fn main() {
    println!("Start fixing...");
    let mut cmd_args = std::env::args_os();
    let book_path = cmd_args.nth(1).expect("Require the book path!");
    let entries = std::fs::read_dir(book_path).expect("Wtf?!");
    for entry in entries {
        if let Ok(entry) = entry {
            let os_filename = entry.file_name();
            let filename = os_filename.to_str().expect("impossible");
            if filename.ends_with(".html") {
                //println!("Parsing {}...", filename);
                let mut buffer = String::with_capacity(entry.metadata().expect("no metadata").len() as usize);
                {
                    let f = std::fs::File::open(entry.path()).expect("file couldn't have not exist!");
                    (&f).read_to_string(&mut buffer).expect("???");
                }
                let mut buffer_new = Vec::with_capacity(buffer.len());
                process(buffer.as_bytes(), &mut buffer_new);
                let f = std::fs::File::create(entry.path()).expect("Coundn't create the file!");
                (&f).write_all(&buffer_new).expect("Coundn't write");
            }
        }
    }
    println!("Done!");
}

fn process(buffer: &[u8], buffer_new: &mut Vec<u8>) {
    let mut index = 0 as usize;
    let mut state = 0;
    let mut temp = Vec::new();
    while index < buffer.len() {
        if state == 0 {
            match try_inspect(buffer, index, 3) {
                Ok(seg) if seg == "<p>".as_bytes() => {
                    println!("Hit!");
                    state = 1;
                    index += 3;
                    buffer_new.write(seg);
                },
                _ => {
                    let next = try_read(buffer, &mut index, 1).expect("zzz1");
                    buffer_new.write(next);
                },
            }
        }else if state == 1 {
            if let Ok(seg) = try_inspect(buffer, index, 4) {
                if seg == "</p>".as_bytes() {
                    index += 4;
                    state = 0;
                    buffer_new.write(seg);
                    continue;
                }
            }
            match try_inspect(buffer, index, 6) {
                Ok(seg) if seg == "<code>".as_bytes() => {
                    state = 2;
                    index += 6;
                    buffer_new.write(seg);
                },
                Ok(seg) if seg == "<pre c".as_bytes() => {
                    //println!("Enter span");
                    state = 5;
                    index += 6;
                    buffer_new.write(seg);
                },
                Ok(seg) if seg == "<style".as_bytes() => {
                    //println!("Enter style");
                    state = 6;
                    index += 6;
                    buffer_new.write(seg);
                },
                _ => {
                    let next = try_read(buffer, &mut index, 1).expect("zzz2");
                    if next == "*".as_bytes() {
                        temp.clear();
                        let nnext = try_inspect(buffer, index, 1).expect("zzz3");
                        if nnext == "*".as_bytes() {
                            index += 1;
                            state = 4;
                            // buffer_new.write("<b>".as_bytes());
                            } else {
                            state = 3;
                            // buffer_new.write("<em>".as_bytes());
                        }
                    } else {
                        buffer_new.write(next);
                    }                   
                },
            }
        }else if state == 2 {
            match try_inspect(buffer, index, 7) {
                Ok(seg) if seg == "</code>".as_bytes() => {
                    state = 1;
                    index += 7;
                    buffer_new.write(seg);
                },
                _ => {
                    let next = try_read(buffer, &mut index, 1).expect("zzz");
                    buffer_new.write(next);
                },
            }
        }else if state == 3 {
            let next = try_read(buffer, &mut index, 1).expect("zzz");
            if next == "*".as_bytes() {
                buffer_new.write("<em>".as_bytes());
                buffer_new.write(&temp);
                buffer_new.write("</em>".as_bytes());
                state = 1;
            } else if next == "/n".as_bytes() {
                buffer_new.write("*".as_bytes());
                buffer_new.write(&temp);
                buffer_new.write("/n".as_bytes());
                state = 1;
            }else {
                temp.write(next);
            }
        }else if state == 4 {
            match try_inspect(buffer, index, 2) {
                Ok(seg) if seg == "**".as_bytes() => {
                    buffer_new.write("<b>".as_bytes());
                    buffer_new.write(&temp);
                    buffer_new.write("</b>".as_bytes());
                    index += 2;
                    state = 1;
                },
                _ => {
                    let next = try_read(buffer, &mut index, 1).expect("zzz");
                    if next == "/n".as_bytes() {
                        buffer_new.write("**".as_bytes());
                        buffer_new.write(&temp);
                        buffer_new.write("/n".as_bytes());
                    } else {
                        temp.write(next);
                    }
                },
            }
        }else if state == 5 {
            match try_inspect(buffer, index, 6) {
                Ok(seg) if seg == "</pre>".as_bytes() => {
                    state = 1;
                    index += 6;
                    //println!("Quit span");
                    buffer_new.write(seg);
                },
                _ => {
                    let next = try_read(buffer, &mut index, 1).expect("zzz");
                    buffer_new.write(next);
                },
            }
        }else if state == 6 {
            match try_inspect(buffer, index, 8) {
                Ok(seg) if seg == "</style>".as_bytes() => {
                    state = 1;
                    index += 8;
                    //println!("Quit style");
                    buffer_new.write(seg);
                },
                _ => {
                    let next = try_read(buffer, &mut index, 1).expect("zzz");
                    buffer_new.write(next);
                },
            }
        }
    }
}

fn try_read<'a>(line: &'a [u8], index: &mut usize, size: usize) -> Result<&'a [u8], ()> {
    if *index + size > line.len() {
        Err(())
    } else {
        *index += size;
        // println!("{:?}", &line[*index-size..*index]);
        Ok(&line[*index-size..*index])
    }
}

fn try_inspect<'a>(line: &'a [u8], index: usize, size: usize) -> Result<&'a [u8], ()> {
    if index + size > line.len() {
        Err(())
    } else {
        // println!("{:?}", &line[index..index+size]);
        Ok(&line[index..index+size])
    }
}