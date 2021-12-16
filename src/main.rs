use std::vec::Vec;
use std::iter::Peekable;

pub fn main() {
    let input = include_str!("input.txt");
    let packet = decode(convert(input));

    println!("problem 16 part 1: {}", calc_version_sum(&packet));
    println!("problem 16 part 2: {}", calc_result(&packet));
}

pub fn calc_version_sum(p: &Packet) -> u64 {
    return match p {
        Packet::Literal(l)  => l.version,
        Packet::Operator(o) => {
            let mut sum = o.version;
            for op in &o.packets {
                sum += calc_version_sum(op);
            }
            sum
        },
    }
}

pub fn calc_result(p: &Packet) -> u64 {
    return match p {
        Packet::Literal(l)  => l.value,
        Packet::Operator(o) => { 
            match o.type_id {
                // sum
                0 => {
                    let mut sum = 0;
                    for op in &o.packets {
                        sum += calc_result(op);
                    }
                    sum
                },
                // product
                1 => {
                    let mut product = 1;
                    for op in &o.packets {
                        product *= calc_result(op);
                    }
                    product
                },
                // min
                2 => {
                    let mut minimum = u64::MAX;
                    for op in &o.packets {
                        minimum = std::cmp::min(calc_result(op), minimum);
                    }
                    minimum
                },
                // max
                3 => {
                    let mut maximum = u64::MIN;
                    for op in &o.packets {
                        maximum = std::cmp::max(calc_result(op), maximum);
                    }
                    maximum
                },
                // gt
                5 => {
                    let first = calc_result(o.packets.get(0).expect("has first"));
                    let second = calc_result(o.packets.get(1).expect("has second"));

                    match first > second {
                        true => 1,
                        false => 0,
                    }
                },
                // lt
                6 => {
                    let first = calc_result(o.packets.get(0).expect("has first"));
                    let second = calc_result(o.packets.get(1).expect("has second"));

                    match first < second {
                        true => 1,
                        false => 0,
                    }
                },
                // eq
                7 => {
                    let first = calc_result(o.packets.get(0).expect("has first"));
                    let second = calc_result(o.packets.get(1).expect("has second"));

                    match first == second {
                        true => 1,
                        false => 0,
                    }
                },
                _ => panic!("unknown type")
            }
        },
    }
}

pub fn convert(hex: &str) -> Vec<u64> {
    let mut res = Vec::new();
    for c in hex.trim().chars() {
        match c {
            '0' => { res.append(&mut vec![0,0,0,0]) },
            '1' => { res.append(&mut vec![0,0,0,1]) },
            '2' => { res.append(&mut vec![0,0,1,0]) },
            '3' => { res.append(&mut vec![0,0,1,1]) },
            '4' => { res.append(&mut vec![0,1,0,0]) },
            '5' => { res.append(&mut vec![0,1,0,1]) },
            '6' => { res.append(&mut vec![0,1,1,0]) },
            '7' => { res.append(&mut vec![0,1,1,1]) },
            '8' => { res.append(&mut vec![1,0,0,0]) },
            '9' => { res.append(&mut vec![1,0,0,1]) },
            'A' => { res.append(&mut vec![1,0,1,0]) },
            'B' => { res.append(&mut vec![1,0,1,1]) },
            'C' => { res.append(&mut vec![1,1,0,0]) },
            'D' => { res.append(&mut vec![1,1,0,1]) },
            'E' => { res.append(&mut vec![1,1,1,0]) },
            'F' => { res.append(&mut vec![1,1,1,1]) },
             _  => { panic!("Unkown hex={}", c) },
        }
    }

    return res;
}

pub fn decode(bitstream: Vec<u64>) -> Packet {
    let mut bitstream_itr = bitstream.into_iter().peekable();
    return match decode_packet(&mut bitstream_itr) {
        Some(p) => p,
        None    => panic!("expected outer packet"),
    }
}

pub fn decode_packets<I: Iterator<Item = u64>>(itr: &mut Peekable<I>) -> Vec<Packet> {

    let mut res = Vec::new();
    loop {
        match decode_packet(itr) {
            Some(p) => { res.push(p); },
            None    => break
        }
    }

    return res;
}

pub fn decode_packet<I: Iterator<Item = u64>>(itr: &mut Peekable<I>) -> Option<Packet> {
    // get version digits
    let version = extend_number(0, itr, 3)?;
    let type_id = extend_number(0, itr, 3)?;

    return match type_id {
        4 => literal_packet(version, type_id, itr),
        _ => operator_packet(version, type_id, itr),
    };
}

pub fn literal_packet<I: Iterator<Item = u64>>(version: u64, type_id: u64, itr: &mut Peekable<I>) -> Option<Packet> {
    let mut literal = 0;
    // process 5-tuples that start with 1
    while itr.peek() == Some(&1) {
        itr.next()?;
        literal = extend_number(literal, itr, 4)?;
    }

    // process last tuple
    itr.next()?;
    literal = extend_number(literal, itr, 4)?;

    let p = LiteralPacket {
        version: version,
        type_id: type_id,
        value: literal,
    };

    return Some(Packet::Literal(p));
}

pub fn operator_packet<I: Iterator<Item = u64>>(version: u64, type_id: u64, itr: &mut Peekable<I>) -> Option<Packet> {
    let length_type_id = extend_number(0, itr, 1)?;
    if length_type_id == 0 {
        let length = extend_number(0, itr, 15)?;

        let p = OperatorPacket {
            version: version,
            type_id: type_id,
            packets: decode_packets(&mut itr.take(length as usize).peekable()),
        };

        return Some(Packet::Operator(p));
    }

    let num_packets = extend_number(0, itr, 11)?;
    let mut packets = Vec::new();
    for _ in 0..num_packets {
        packets.push(decode_packet(itr)?);
    }

    let p = OperatorPacket {
        version: version,
        type_id: type_id,
        packets: packets,
    };

    return Some(Packet::Operator(p));
}

pub fn extend_number<I: Iterator<Item = u64>>(num: u64, itr: &mut Peekable<I>, take: u64) -> Option<u64> {
    let mut value = num;
    for _ in 0..take {
        value *= 2;
        value += itr.next()?;
    }

    return Some(value);
}

#[derive(Debug)]
pub enum Packet {
    Literal(LiteralPacket),
    Operator(OperatorPacket),
}

#[derive(Debug)]
pub struct LiteralPacket {
    version: u64,
    type_id: u64,
    value: u64,
}

#[derive(Debug)]
pub struct OperatorPacket {
    version: u64,
    type_id: u64,
    packets: Vec<Packet>
}
