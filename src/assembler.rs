/*

    cls                             - clear screen
    ret                             - return

    jp <nnn | .label>               - jump to address
    call <nnn | .label>             - call subrouting
    se vx, <vy | nn>                - skip if equal
    sne vx, <vy | nn>               - skip if not equal

    gt vx, vy                       - vf = vx > vy
    gte vx, vy                      - vf = vx >= vy
    lt vx, vy                       - vf = vx < vy
    lte vx, vy                      - vf = vx <= vy

    ld vx, <nn | vy | dt>           - load nn, vy, or dt into vx
    ldi, nnn                        - load addr into I
    ld dt, vx                       - load vx into dt
    ld st, vx                       - load vx into st
    ldsprt vx                       - load sprite version of vx into I
    ldbcd vx                        - load bcd version of vx into mem
    dumpreg vx                      - load registers v0-vx into memory
    ldreg   vx                      - load registers v0-vx from memory
    getkey vx                       - stores key press val in vx

    add vx, <nn | vy>               - add vx to whatever and save in vx
    addi vx                         - i += vx
    sub vx, vy                      - vx -= vy
    subn vx, vy                     - vx = vy - vx
    shr vx                          - vx >> 1
    shl vx                          - vx << 1

    rnd vx, nn                      - random number in vx
    drw vx, vy, n                   - n-byte sprite drawn from mem i to vx, vy
    skp vx                          - if key with value vx is pressed, skip next instruction
    sknp vx                         - if key with value vx is not pressed, skip next
*/

use std::collections::HashMap;


pub fn assemble(input: &str) -> Vec<u8> {
    let ins = get_instructions(input);
    let mut label_table: HashMap<String, u16> = HashMap::new();
    let dirs = separate_into_directions(ins, &mut label_table);
    translate(dirs, label_table)
}

fn is_label(s: &str) -> bool {
    if s.is_empty() || s.chars().nth(0).unwrap() != '.' {false} else {true}
}

fn is_number(s: &str) -> bool {
    if s.is_empty() || !char::is_numeric(s.chars().nth(0).unwrap()) {false} else {true}
}

fn is_register(s: &str) -> bool {
    if s.is_empty() {return false;}
    let c = s.chars().nth(0).unwrap();
    if c == 'v' {return true;}
    if s == "dt" || s == "st" {return true;}
    return false;
}

fn get_register_num(s: &str) -> u8 {
    let num: u8 = s[1..].parse().expect("Could not parse register number.");
    if num >= 16 {panic!("Register number must be between 0 and 16")}
    num
}

fn translate(input: Vec<Vec<String>>, labels: HashMap<String, u16>) -> Vec<u8> {
    let mut ins: Vec<u8> = vec![];

    for line in input {
        match line[0].as_str() {
            "cls" => ins.append(&mut vec![0x00, 0xE0]),
            "ret" => ins.append(&mut vec![0x00, 0xEE]),
            "jp" => {
                let mut location: u16;
                if is_label(&line[1]) {
                    if !labels.contains_key(&line[1]) {panic!("No label {}", line[1]);}
                    location = labels[&line[1]] + 0x200;
                } else {location = line[1].parse().expect("Could not parse number into 12 bits.")}
                location &= 0xFFF;

                ins.append(&mut vec![(0x01 << 4) | (location >> 8) as u8, (location & 0xFF) as u8])
            },
            "call" => {
                let mut location: u16;
                if is_label(&line[1]) {
                    if !labels.contains_key(&line[1]) {panic!("No label {}", line[1]);}
                    location = labels[&line[1]] + 0x200;
                } else {location = line[1].parse().expect("Could not parse number into 12 bits.")}
                location &= 0xFFF;
                ins.append(&mut vec![(0x02 << 4) | (location >> 8) as u8, (location & 0xFF) as u8]);
            }
            "se" => {
                if is_register(&line[2]) {
                    let x_reg = get_register_num(&line[1]);
                    let y_reg = get_register_num(&line[2]);
                    ins.append(&mut vec![(0x5 << 4) | (x_reg & 0xF), ((y_reg << 4) & 0xF0) | 0x0]);
                } else {
                    let x_reg = get_register_num(&line[1]);
                    let nn: u8 = line[2].parse().unwrap();
                    ins.append(&mut vec![(0x3 << 4) | (x_reg & 0xF), nn]);
                }
            },
            "sne" => {
                if is_register(&line[2]) {
                    let x_reg = get_register_num(&line[1]);
                    let y_reg = get_register_num(&line[2]);
                    ins.append(&mut vec![(0x9 << 4) | (x_reg & 0xF), ((y_reg << 4) & 0xF0) | 0x0]);
                } else {
                    let x_reg = get_register_num(&line[1]);
                    let nn: u8 = line[2].parse().unwrap();
                    ins.append(&mut vec![(0x4 << 4) | (x_reg & 0xF), nn]);
                }
            },
            "gt" => {
                let x_reg = get_register_num(&line[1]);
                let y_reg = get_register_num(&line[2]);
                ins.append(&mut vec![0x90 | (x_reg & 0xF), (y_reg << 4) | 0x1]);
            },
            "gte" => {
                let x_reg = get_register_num(&line[1]);
                let y_reg = get_register_num(&line[2]);
                ins.append(&mut vec![0x90 | (x_reg & 0xF), (y_reg << 4) | 0x2]);
            },
            "lt" => {
                let x_reg = get_register_num(&line[1]);
                let y_reg = get_register_num(&line[2]);
                ins.append(&mut vec![0x90 | (x_reg & 0xF), (y_reg << 4) | 0x3]);
            },
            "lte" => {
                let x_reg = get_register_num(&line[1]);
                let y_reg = get_register_num(&line[2]);
                ins.append(&mut vec![0x90 | (x_reg & 0xF), (y_reg << 4) | 0x4]);
            },
            "ld" => {
                if is_register(&line[1]) {
                    let x_reg = get_register_num(&line[1]);

                    if is_number(&line[2]) {
                        let num: u8 = line[2].parse().unwrap();
                        ins.append(&mut vec![(0x6 << 4) | (x_reg & 0xF), num]);

                    } else if is_register(&line[2]) {
                        let y_reg = get_register_num(&line[2]);
                        ins.append(&mut vec![(0x08 << 4) | (x_reg & 0xF), ((y_reg << 4) & 0xF0) | 0x0]);
                    } else {
                        ins.append(&mut vec![(0xF << 4) | (x_reg & 0xF), 0x07]);
                    }

                } else if line[1].chars().nth(0).unwrap() == 'd' {
                    let x_reg = get_register_num(&line[1]);
                    ins.append(&mut vec![(0xF << 4) | (x_reg & 0xF), 0x15]);
                } else if line[1].chars().nth(0).unwrap() == 's' {
                    let x_reg = get_register_num(&line[1]);
                    ins.append(&mut vec![(0xF << 4) | (x_reg & 0xF), 0x18]);
                }
            },
            "ldi" => {
                let n: u16 = line[1].parse().unwrap();
                if n >= 4096 {panic!("Address must be between 0 and 4095.")}
                ins.append(&mut vec![(0xA << 4) | ((n >> 8) as u8 & 0xF), (n & 0xFF) as u8]);
            },
            "ldsprt" => {
                let x_reg = get_register_num(&line[1]);
                ins.append(&mut vec![(0xF << 4) | (x_reg & 0xF), 0x29]);
            },
            "ldbcd" => {
                let x_reg = get_register_num(&line[1]);
                ins.append(&mut vec![(0xF << 4) | (x_reg & 0xF), 0x33]);
            },
            "dumpreg" => {
                let x_reg = get_register_num(&line[1]);
                ins.append(&mut vec![(0xF << 4) | (x_reg & 0xF), 0x55]);
            },
            "ldreg" => {
                let x_reg = get_register_num(&line[1]);
                ins.append(&mut vec![(0xF << 4) | (x_reg & 0xF), 0x65]);
            },
            "getkey" => {
                let x_reg = get_register_num(&line[1]);
                ins.append(&mut vec![(0xF << 4) | (x_reg & 0xF), 0x0A]);
            },
            "add" => {
                let x_reg = get_register_num(&line[1]);

                if is_register(&line[2]) {
                    let y_reg = get_register_num(&line[2]);
                    ins.append(&mut vec![(8 << 4) | (x_reg & 0xF), ((y_reg << 4) & 0xF0) | 0x4]);
                } else {
                    let num: u8 = line[2].parse().unwrap();
                    ins.append(&mut vec![0x70 | (x_reg & 0xF), num]);
                }
            },
            "addi" => {
                let x_reg = get_register_num(&line[1]);
                ins.append(&mut vec![(0xF << 4) | (x_reg & 0xF), 0x1E]);
            },
            "sub" => {
                let x_reg = get_register_num(&line[1]);
                let y_reg = get_register_num(&line[2]);

                ins.append(&mut vec![0x80 | (x_reg & 0xF), ((y_reg << 4) & 0xF0) | 0x5]);
            },
            "subn" => {
                let x_reg = get_register_num(&line[1]);
                let y_reg = get_register_num(&line[2]);

                ins.append(&mut vec![0x80 | (x_reg & 0xF), ((y_reg << 4) & 0xF0) | 0x7]);
            },
            "shr" => {
                let x_reg = get_register_num(&line[1]);
                ins.append(&mut vec![0x80 | (x_reg & 0xF), 0x06]);
            },
            "shl" => {
                let x_reg = get_register_num(&line[1]);
                ins.append(&mut vec![0x80 | (x_reg & 0xF), 0x0E]);
            },
            "rnd" => {
                let x_reg = get_register_num(&line[1]);
                let num: u8 = line[2].parse().unwrap();
                ins.append(&mut vec![0xC0 | (x_reg & 0xF), num]);
            },
            "drw" => {
                let x_reg = get_register_num(&line[1]);
                let y_reg = get_register_num(&line[2]);
                let num: u8 = line[3].parse().unwrap();
                if num > 10 {panic!("Sprites have a maximum length of 10.")}

                ins.append(&mut vec![0xD0 | (x_reg & 0xF), ((y_reg << 4) & 0xF0) | (num & 0x0F)]);
            },
            "skp" => {
                let x_reg = get_register_num(&line[1]);
                ins.append(&mut vec![(0xE << 4) | (x_reg & 0xF), 0x9E]);
            },
            "sknp" => {
                let x_reg = get_register_num(&line[1]);
                ins.append(&mut vec![(0xE << 4) | (x_reg & 0xF), 0xA1]);
            },
            _ => panic!("Unknown instruction {}", line[0])
        }
    }

    ins
}


fn separate_into_directions(input: Vec<String>, label_table: &mut HashMap<String, u16>) -> Vec<Vec<String>> {
    let mut ins: Vec<Vec<String>> = vec![];
    let mut pos = 0;
    let mut instruction_num: u16 = 0;

    let advance = |pos: &mut usize, input: &Vec<String>| -> String {
        *pos += 1;
        if *pos >= input.len() {panic!("Unexpected end to input stream.")}
        input[*pos].clone()
    };

    while pos < input.len() {
        if input[pos].chars().nth(0).unwrap() == '.' {
            if label_table.contains_key(&input[pos]) {panic!("Label {} was already defined.", input[pos]);}
            label_table.insert(input[pos].clone(), instruction_num);
            pos += 1;
        } else {
            match input[pos].as_str() {
                "cls" => {
                    ins.push(vec!["cls".to_string()]);
                },
                "ret" => {
                    ins.push(vec!["ret".to_string()]);
                },
                "call" => {
                    let next = advance(&mut pos, &input);
                    if !is_number(&next) && !is_label(&next) {panic!("Expected number of label after call.")}
                    ins.push(vec!["call".to_string(), next]);
                }
                "jp" => {
                    let next = advance(&mut pos, &input);
                    if !is_number(&next) && !is_label(&next) {panic!("Expected number or label after jmp.")}
                    ins.push(vec!["jp".to_string(), next]);
                },
                "se" => {
                    let vx = advance(&mut pos, &input);
                    if !is_register(&vx) {panic!("Expected register after se.")}
                    let next = advance(&mut pos, &input);
                    if !is_register(&next) && !is_number(&next) {panic!("Expected number or register on se.")}
                    ins.push(vec!["se".to_string(), vx, next]);
                },
                "sne" => {
                    let vx = advance(&mut pos, &input);
                    if !is_register(&vx) {panic!("Expected register after sne.")}
                    let next = advance(&mut pos, &input);
                    if !is_register(&next) && !is_number(&next) {panic!("Expected number or register on sne.")}
                    ins.push(vec!["sne".to_string(), vx, next]);
                },
                "gt" => {
                    let vx = advance(&mut pos, &input);
                    let vy = advance(&mut pos, &input);
                    if !is_register(&vx) || !is_register(&vy) {panic!("gt arguments are both registers.")}
                    ins.push(vec!["gt".to_string(),vx, vy]);
                },
                "gte" => {
                    let vx = advance(&mut pos, &input);
                    let vy = advance(&mut pos, &input);
                    if !is_register(&vx) || !is_register(&vy) {panic!("gte arguments are both registers.")}
                    ins.push(vec!["gte".to_string(),vx, vy]);
                },
                "lt" => {
                    let vx = advance(&mut pos, &input);
                    let vy = advance(&mut pos, &input);
                    if !is_register(&vx) || !is_register(&vy) {panic!("lt arguments are both registers.")}
                    ins.push(vec!["lt".to_string(),vx, vy]);
                },
                "lte" => {
                    let vx = advance(&mut pos, &input);
                    let vy = advance(&mut pos, &input);
                    if !is_register(&vx) || !is_register(&vy) {panic!("lte arguments are both registers.")}
                    ins.push(vec!["lte".to_string(),vx, vy]);
                },
                "ld" => {
                    let reg = advance(&mut pos, &input);
                    if !is_register(&reg) {panic!("Expected register after ld.")}
                    let next = advance(&mut pos, &input);
                    if !is_register(&next) && !is_number(&next) { panic!("Expected number or register in ld"); }
                    ins.push(vec!["ld".to_string(), reg, next]);
                },
                "ldi" => {
                    let num = advance(&mut pos, &input);
                    if !is_number(&num) {panic!("Expected number in ldi.")};
                    ins.push(vec!["ldi".to_string(), num]);
                },
                "ldsprt" => {
                    let reg = advance(&mut pos, &input);
                    if !is_register(&reg) {panic!("Expected register in ldsprt.")};
                    ins.push(vec!["ldsprt".to_string(), reg]);
                },
                "ldbcd" => {
                    let reg = advance(&mut pos, &input);
                    if !is_register(&reg) {panic!("Expected register in ldbcd.")};
                    ins.push(vec!["ldbcd".to_string(), reg]);
                },
                "dumpreg" => {
                    let reg = advance(&mut pos, &input);
                    if !is_register(&reg) {panic!("Expected register in dumpreg.")};
                    ins.push(vec!["dumpreg".to_string(), reg]);
                },
                "ldreg" => {
                    let reg = advance(&mut pos, &input);
                    if !is_register(&reg) {panic!("Expected register in ldreg.")};
                    ins.push(vec!["ldreg".to_string(), reg]);
                },
                "getkey" => {
                    let reg = advance(&mut pos, &input);
                    if !is_register(&reg) {panic!("Expected register in getkey.")};
                    ins.push(vec!["getkey".to_string(), reg]);
                },
                "add" => {
                    let reg = advance(&mut pos, &input);
                    if !is_register(&reg) {panic!("Expected register in add")};
                    let next = advance(&mut pos, &input);
                    if !is_register(&next) && !is_number(&next) {panic!("Expected number or register in add.")}
                    ins.push(vec!["add".to_string(), reg, next]);
                }
                "addi" => {
                    let reg = advance(&mut pos, &input);
                    if !is_register(&reg) {panic!("Expected register in addi.")};
                    ins.push(vec!["addi".to_string(), reg]);
                },
                "sub" => {
                    let reg = advance(&mut pos, &input);
                    if !is_register(&reg) {panic!("Expected register in sub")};
                    let next = advance(&mut pos, &input);
                    if !is_register(&next) {panic!("Expected register in sub.")}
                    ins.push(vec!["sub".to_string(), reg, next]);
                },
                "subn" => {
                    let reg = advance(&mut pos, &input);
                    if !is_register(&reg) {panic!("Expected register in subn")};
                    let next = advance(&mut pos, &input);
                    if !is_register(&next) {panic!("Expected register in subn.")}
                    ins.push(vec!["subn".to_string(), reg, next]);
                },
                "shr" => {
                    let reg = advance(&mut pos, &input);
                    if !is_register(&reg) {panic!("Expected register in shr.")};
                    ins.push(vec!["shr".to_string(), reg]);
                },
                "shl" => {
                    let reg = advance(&mut pos, &input);
                    if !is_register(&reg) {panic!("Expected register in shl.")};
                    ins.push(vec!["shl".to_string(), reg]);
                },
                "rnd" => {
                    let reg = advance(&mut pos, &input);
                    if !is_register(&reg) {panic!("Expected register in rnd")};
                    let next = advance(&mut pos, &input);
                    if !is_number(&next) {panic!("Expected number in rnd.")}
                    ins.push(vec!["rnd".to_string(), reg, next]);
                },
                "drw" => {
                    let vx = advance(&mut pos, &input);
                    let vy = advance(&mut pos, &input);
                    if !is_register(&vx) && !is_register(&vy) {panic!("Expected two register inputs to drw")};
                    let num = advance(&mut pos, &input);
                    if !is_number(&num) {panic!("Expected number in drw")}
                    ins.push(vec!["drw".to_string(), vx, vy, num]);
                },
                "skp" => {
                    let reg = advance(&mut pos, &input);
                    if !is_register(&reg) {panic!("Expected register in skp.")};
                    ins.push(vec!["skp".to_string(), reg]);
                },
                "sknp" => {
                    let reg = advance(&mut pos, &input);
                    if !is_register(&reg) {panic!("Expected register in sknp.")};
                    ins.push(vec!["sknp".to_string(), reg]);
                },
                _ => panic!("Unknown instruction {}", input[pos]),
            }
            pos += 1;
            instruction_num += 2;
        }
    }


    ins
}


fn get_label(input: &str, pos: &mut usize) -> String {
    let mut word = String::new();

    word.push('.');
    *pos += 1;

    let mut ch = input.chars().nth(*pos).unwrap();
    while *pos < input.len() && (char::is_alphanumeric(ch) || ch == '_') {
        word.push(ch);
        *pos += 1;
        if *pos >= input.len() {break;}
        ch = input.chars().nth(*pos).unwrap();
    }

    word
}

fn get_number(input: &str, pos: &mut usize) -> String {
    let mut word = String::new();

    while *pos < input.len() && char::is_numeric(input.chars().nth(*pos).unwrap()) {
        word.push(input.chars().nth(*pos).unwrap());
        *pos += 1;
    }

    word
}

fn get_iden(input: &str, pos: &mut usize) -> String {
    let mut word = String::new();

    let mut ch = input.chars().nth(*pos).unwrap();
    while *pos < input.len() && (char::is_alphanumeric(ch) || ch == '_') {
        word.push(ch);
        *pos += 1;
        if *pos >= input.len() {break;}
        ch = input.chars().nth(*pos).unwrap();
    }

    word
}

fn get_instructions(input: &str) -> Vec<String> {
    let mut pos = 0;
    let mut ins: Vec<String> = vec![];


    while pos < input.len() {
        let ch = input.chars().nth(pos).unwrap();

        match ch {
            '.' => ins.push(get_label(input, &mut pos)),
            ';' => {
                let mut curr_char = input.chars().nth(pos).unwrap();
                while pos < input.len() && curr_char != '\n' {
                    pos += 1;
                    if pos >= input.len() {break;}
                    curr_char = input.chars().nth(pos).unwrap();
                }
            }
            _ if char::is_numeric(ch) => ins.push(get_number(input, &mut pos)),
            _ if char::is_alphabetic(ch) => ins.push(get_iden(input, &mut pos)),
            _ => pos += 1,
        }
    }


    ins

}

