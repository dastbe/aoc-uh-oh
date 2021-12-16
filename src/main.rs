use std::vec::Vec;
use std::iter::Peekable;

pub fn main() {
    let packet = decode(vec![1,0,1,0]);
}

pub fn decode(bitstream: Vec<u64>) -> Packet {
    let mut bitstream_itr = bitstream.into_iter().peekable();
    return match decode_packet(&mut bitstream_itr) {
        Some(p) => p,
        None    => panic!("expected outer packet"),
    }
}

//pub fn decode_packets(bitstream: Vec<u64>) -> Vec<Packet> {
pub fn decode_packets<I: Iterator<Item = u64>>(itr: &mut Peekable<I>) -> Vec<Packet> {
    //let mut bitstream_itr = bitstream.into_iter().peekable();

    let mut res = Vec::new();
    loop {
        match decode_packet(itr) {
        //match decode_packet(&mut bitstream_itr) {
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
        _ => operator_packet(version, type_id, itr),
    };
}

pub fn operator_packet<I: Iterator<Item = u64>>(version: u64, type_id: u64, itr: &mut Peekable<I>) -> Option<Packet> {
    let length_type_id = extend_number(0, itr, 1)?;
    if length_type_id == 0 {
        let length = extend_number(0, itr, 15)?;

        /*let mut sub_bits = Vec::new();
        for _ in 0..length {
            sub_bits.push(itr.next()?);
        }*/

        let p = OperatorPacket {
            version: version,
            type_id: type_id,
            packets: decode_packets(&mut itr.take(length as usize).peekable()),
            //packets: decode_packets(sub_bits),
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
    Operator(OperatorPacket),
}

#[derive(Debug)]
pub struct OperatorPacket {
    version: u64,
    type_id: u64,
    packets: Vec<Packet>
}
