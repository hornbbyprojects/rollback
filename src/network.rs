use crate::game::commands::{Handshake, SetInputDelay, TimedCommand, TimingPacket};

use super::*;
use alkahest::{
    deserialize, private::BareFormula, serialize_to_vec, Deserialize, Formula, SerializeRef,
};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::thread;
use std::{
    io::{self, ErrorKind, Read, Write},
    sync::mpsc::{self, Receiver, Sender},
};

pub fn serialize_item<W: Write, ItemType: SerializeRef<ItemType> + Formula + BareFormula>(
    out: &mut W,
    item: &ItemType,
) -> io::Result<()> {
    let mut buffer = Vec::new();
    serialize_to_vec::<ItemType, _>(item, &mut buffer);
    out.write_u32::<BigEndian>(buffer.len() as u32)?;
    out.write_all(&buffer)?;
    Ok(())
}
pub fn deserialize_item<R: Read, ItemType: Formula + for<'a> Deserialize<'a, ItemType>>(
    in_stream: &mut R,
) -> io::Result<ItemType> {
    let len = in_stream.read_u32::<BigEndian>()?;
    let mut buffer = vec![0u8; len as usize];
    in_stream.read_exact(&mut buffer)?;
    deserialize::<ItemType, ItemType>(&mut buffer)
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, format!("{:?}", e)))
}

const TIMING_PACKET_COUNT: u64 = 10;
fn host_measure_timing(connection: &mut TcpStream) -> io::Result<u128> {
    let start_time = Instant::now();
    let mut max_elapsed: Duration = Duration::from_micros(1);
    let mut start_packet = start_time;
    for i in 0..TIMING_PACKET_COUNT + 1 {
        if i != 0 {
            let timing_packet: TimingPacket = deserialize_item(connection)?;
            let packet_elapsed = start_packet.elapsed();
            max_elapsed = max_elapsed.max(packet_elapsed);
            if timing_packet.sequence_number != i {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "Got wrong sequence number in timing packets",
                ));
            }
        }
        if i != TIMING_PACKET_COUNT {
            serialize_item(
                connection,
                &TimingPacket {
                    sequence_number: i + 1,
                },
            )?;
            connection.flush()?;
            start_packet = Instant::now();
        }
    }
    let average_elapsed = start_time.elapsed();
    println!(
        "Total elapsed time {} us, max {} us",
        average_elapsed.as_micros(),
        max_elapsed.as_micros()
    );
    Ok((max_elapsed.as_micros() / TICK_TIME.as_micros()) + 2)
}
fn client_measure_timing(connection: &mut TcpStream) -> io::Result<()> {
    for _ in 0..TIMING_PACKET_COUNT {
        let timing_packet: TimingPacket =
            deserialize_item(connection).expect("Unable to receive timing packet");
        serialize_item(connection, &timing_packet).expect("Unable to send timing packet");
        connection.flush()?;
    }
    Ok(())
}

pub fn net_thread(
    is_host: bool,
    my_name: String,
    mut connection: TcpStream,
) -> (
    Handshake,
    SetInputDelay,
    Sender<TimedCommand>,
    Receiver<TimedCommand>,
) {
    connection
        .set_nodelay(true)
        .expect("Couldn't disable Nagle's");
    serialize_item(&mut connection, &Handshake { my_name }).expect("Failed to write handshake");
    let handshake =
        deserialize_item::<_, Handshake>(&mut connection).expect("Failed to read handshake");
    let set_input_delay = if is_host {
        let input_delay =
            host_measure_timing(&mut connection).expect("Unable to measure host timing");
        let set_input_delay = SetInputDelay {
            input_delay: input_delay as u64,
        };
        serialize_item(&mut connection, &set_input_delay).expect("Unable to send input delay");
        set_input_delay
    } else {
        client_measure_timing(&mut connection).expect("Unable to work with server to measure lag");
        deserialize_item(&mut connection).expect("Unable to read input delay")
    };
    println!(
        "Setting input delay to {} frames",
        set_input_delay.input_delay
    );
    let (from_other_sender, from_other_receiver) = mpsc::channel();
    let (to_other_sender, to_other_receiver) = mpsc::channel();
    let in_stream = connection
        .try_clone()
        .expect("Couldn't create input network stream");
    thread::spawn(|| input_thread(from_other_sender, in_stream));
    thread::spawn(|| output_thread(to_other_receiver, connection));
    (
        handshake,
        set_input_delay,
        to_other_sender,
        from_other_receiver,
    )
}
fn output_thread(
    command_receiver: Receiver<TimedCommand>,
    mut out_stream: TcpStream,
) -> io::Result<()> {
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
        command_sender
            .send(command)
            .map_err(|e| io::Error::new(ErrorKind::Other, e))?;
    }
}
