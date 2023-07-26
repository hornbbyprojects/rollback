use crate::game::commands::TimedCommand;

use super::*;
use alkahest::{serialize_to_vec, SerializeRef, Formula, private::BareFormula, deserialize, Deserialize};
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use std::{io::{self, Write, Read, ErrorKind}, sync::mpsc::{Sender, self, Receiver}};
use std::thread;



pub fn serialize_item<W: Write, ItemType: SerializeRef<ItemType> + Formula + BareFormula>(out: &mut W, item: &ItemType) -> io::Result<()> {
    let mut buffer = Vec::new();
    serialize_to_vec::<ItemType, _>(item, &mut buffer);
    out.write_u32::<BigEndian>(buffer.len() as u32)?;
    out.write_all(&buffer)?;
    Ok(())
}
pub fn deserialize_item<R: Read, ItemType: Formula + for<'a> Deserialize<'a, ItemType>>(in_stream: &mut R) -> io::Result<ItemType> {
    let len = in_stream.read_u32::<BigEndian>()?;
    let mut buffer = vec![0u8; len as usize];
    in_stream.read_exact(&mut buffer)?;
    deserialize::<ItemType, ItemType>(&mut buffer).map_err(|e|io::Error::new(ErrorKind::InvalidData, format!("{:?}", e)))
}

pub fn net_thread(connection: TcpStream) -> (Sender<TimedCommand>, Receiver<TimedCommand>) {
    let (from_other_sender, from_other_receiver) = mpsc::channel();
    let (to_other_sender, to_other_receiver) = mpsc::channel();
    let in_stream = connection.try_clone().expect("Couldn't create input network stream");
    thread::spawn(||input_thread(from_other_sender, in_stream));
    thread::spawn(||output_thread(to_other_receiver, connection));
    (to_other_sender, from_other_receiver)
}
fn output_thread(command_receiver: Receiver<TimedCommand>, mut out_stream: TcpStream) -> io::Result<()> {
    loop {
        for command in command_receiver.try_iter() {
            serialize_item(&mut out_stream, &command)?;
        }
        out_stream.flush()?;
    }
}
fn input_thread(command_sender: Sender<TimedCommand>, mut in_stream: TcpStream) -> io::Result<()> {
    loop {
        let command = deserialize_item::<_, TimedCommand>(&mut in_stream)?;
        command_sender.send(command).map_err(|e|io::Error::new(ErrorKind::Other, e))?;
    }
}
