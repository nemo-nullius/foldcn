use std::str;

#[macro_use]
extern crate log;
extern crate env_logger;

fn fold(text: &str, lnlen: u8) -> String {
    let mut result = Vec::new();
    let mut buflen: u8;
    let mut word = Vec::new();
    let mut wordlen: u8;
    let mut wordno: u8; // word order number
    let mut word_after_ch;
    // get line_ending
    // if text contains "\r\n", then line_ending will be "\r\n",
    // else line_ending will be "\n".
    let line_ending = match text {
        txt if txt.contains("\r\n") => vec!['\r', '\n'],
        //txt if txt.contains("\r") => "\r", // unsupported by lines()
        _ => vec!['\n'],
    };
    // lines()
    let lines = text.lines();
    for ln in lines {
        debug!("{:?}", ln);
        let mut chars = ln.chars();
        buflen = 0;
        wordno = 0;
        // each line
        loop {
            // get next word
            loop {
                let ch = match chars.next() {
                    Some(x) => x,
                    None => {
                        word_after_ch = None;
                        break;
                    }
                };
                match ch {
                    c if [' ', '\t'].contains(&c)
                        || ((c as u32) > 255)
                        //|| ((c as u32) > 255 && (word.len() > 0) && (word[0] as u32) < 255)
                        // as for Chinese chars, each char is an individual word
                        || ((c as u32) < 255 && (word.len() > 0) && (word[0] as u32) > 255) =>
                    {
                        word_after_ch = Some(ch);
                        break;
                    }
                    _ => word.push(ch),
                };
            }
            wordno += 1;
            debug!(
                "WORD{:?}\tWAC {:?}\tBL {}\tWNO {}",
                word, word_after_ch, buflen, wordno
            );
            wordlen = word.len() as u8;
            match buflen {
                bl if bl + wordlen <= lnlen => {
                    result.append(&mut word);
                    match word_after_ch {
                        Some(c) => {
                            word.push(c);
                            buflen += wordlen;
                        }
                        None => break,
                    }
                }
                bl if bl + wordlen > lnlen => {
                    if wordno != 1 {
                        // buflen == 0 at this time
                        result.extend(&line_ending);
                    }
                    match wordlen {
                        // append word
                        wl if wl <= lnlen => {
                            result.append(&mut word);
                        }
                        wl if wl > lnlen => {
                            for i in 0..wordlen {
                                result.push(word[i as usize]);
                                if i != 0 && (i + 1) != wordlen && (i + 1) % lnlen == 0 {
                                    // there is no line_ending in the last line though it equals lnlen
                                    result.extend(&line_ending);
                                }
                            }
                            word.clear();
                            // redefine wordlen
                            wordlen = match wordlen % lnlen {
                                0 => lnlen,
                                x => x,
                            };
                        }
                        _ => {}
                    }
                    match word_after_ch {
                        Some(c) => {
                            word.push(c);
                            buflen = wordlen;
                        }
                        None => break,
                    }
                }
                _ => {}
            }
        }
        result.extend(&line_ending); // line_ending is not moved/changed
    }
    result.into_iter().collect()
}

fn test() {
    let txt = "What's your a name? abcdefghijklmnopqrstuvwxyz 道可道非常道\r\ngoodgood可名非常名";
    debug!("{}", txt);
    debug!("{:?}", fold(txt, 5));
    debug!("{}", fold(txt, 5));
}

use std::fs::File;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
    use std::env;

    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Bad command.\nfoldcn <input filename> <output filename> (<line length>).\n");
        return Ok(());
    } else {
        let filename_input = &args[1];
        let filename_output = &args[2];
        let line_length = match &args {
            ag if ag.len() >= 4 => match ag[3].parse::<u8>() {
                Ok(n) => n,
                Err(_) => {
                    println!("Bad line length. Set default to 80.");
                    80
                }
            },
            _ => {
                println!("Line length not set. Set default to 80.");
                80
            }
        };
        let mut file_input = File::open(filename_input)?;
        let mut contents = String::new();
        file_input.read_to_string(&mut contents)?;

        let mut file_output = File::create(filename_output)?;
        let folded = fold(&contents, line_length);
        file_output.write_all(folded.as_bytes())?;

        Ok(())
    }
}
