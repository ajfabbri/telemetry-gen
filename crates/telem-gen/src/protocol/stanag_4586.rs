use nom::{
    bytes::complete::{tag, take},
    number::complete::be_u32,
    Finish, IResult,
};

use crate::TGResult;

/// Message Wrapper from STANAG 4586 Ed. 2.5 Feb 2007.
///
/// Each message shall contain the version identification of the Interface Definition Document
/// (IDD) from which its structure was defined. This version identification shall be placed in a
/// fixed 10 byte field and filled with a null-terminated string of ASCII characters. ("2.5")
///
/// The instance identifier shall uniquely identify every instance of a message of a given
/// type. Instance identifiers are used by the system to keep streaming data coordinated, and
/// to identify dropped messages of a given type at the application level. Instance identifier
/// numbers shall not be reused unless other provisions for avoiding identifier ambiguity are
/// provided in the message body.
///
/// The length shall be a 32-bit unsigned integer of the number of bytes in the “Message
/// Data”. The length shall be any number between 1 and 538.
///
/// The instance identifier shall uniquely identify every instance of a message of a given type.
/// Instance identifiers are used by the system to keep streaming data coordinated, and to identify
/// dropped messages of a given type at the application level. Instance identifier numbers shall
/// not be reused unless other provisions for avoiding identifier ambiguity are provided in the
/// message body.
///
/// The purpose of Packet Sequence Number was to provide a means for segmenting data from a single
/// message into sequences of blocks of a maximum length. This field is not used and shall contain
/// “- 1”.
#[derive(Debug, Default)]
pub struct WrapperHeader {
    pub idd: [u8; 10],     // 10
    pub msg_instance: u32, // 14
    pub msg_type: u32,     // 18
    pub msg_length: u32,   // 22
    pub stream_id: u32,    // 26
    pub packet_seq: u32,   // 30
}

/// Checksum shall be employed to determine the presence of errors during
/// transmission or handling of messages.
///
/// The checksum shall be a 4-byte unsigned integer and calculated by simple, byte-wise unsigned
/// binary addition of all data contained in the message excluding the checksum, and truncated to
/// four bytes
#[derive(Debug)]
pub struct WrapperFooter {
    pub checksum: u32,
}

pub const IDD_2_5: [u8; 10] = *b"2.5\0\0\0\0\0\0\0"; // checksum = 50 + 46 + 53 = 149

/// For vehicle specific messages (private), the type numbers shall be greater than 2000.
pub const MSG_TYPE_VEHICLE_SPECIFIC1: u32 = 2001;

#[derive(Debug)]
pub struct Message<'a> {
    pub header: WrapperHeader,
    pub payload: &'a [u8],
    pub footer: WrapperFooter,
}

pub fn parse(bytes: &[u8]) -> TGResult<Message<'_>> {
    let nom_res = nom_parse(bytes);
    nom_res.finish().map(|(_, msg)| msg).map_err(|e| e.into())
}

fn nom_parse(bytes: &[u8]) -> IResult<&[u8], Message<'_>> {
    let (rest, idd) = tag(IDD_2_5)(bytes)?;
    let (rest, msg_instance) = be_u32(rest)?;
    let (rest, msg_type) = be_u32(rest)?;
    let (rest, msg_length) = be_u32(rest)?;
    let (rest, stream_id) = be_u32(rest)?;
    let (rest, packet_seq) = be_u32(rest)?;
    let (rest, payload) = take(msg_length)(rest)?;
    let (rest, checksum) = be_u32(rest)?;
    let header = WrapperHeader {
        idd: idd.try_into().expect("fixed width idd field"),
        msg_instance,
        msg_type,
        msg_length,
        stream_id,
        packet_seq,
    };
    let footer = WrapperFooter { checksum };
    Ok((
        rest,
        Message {
            header,
            payload,
            footer,
        },
    ))
}

/// STANAG 4586 checksum: Bytewise addition truncated to 4 bytes.
pub fn checksum(bytes: &[u8]) -> u32 {
    let mut sum: u64 = 0;
    for byte in bytes {
        sum += *byte as u64;
    }
    sum as u32 // Rust truncates on cast to smaller unsigned types
}

#[cfg(test)]
mod test {
    use nom::AsBytes;

    use crate::lazy_init_tracing;

    use super::*;

    // We don't have specs for actual message types. For now just test wrapper serde with arbitrary
    // payload bytes.
    #[test]
    fn test_stanag_wrapper_hello_serde() {
        lazy_init_tracing();
        let payload = b"Hello";
        let msg_bytes = [
            IDD_2_5.as_bytes(),
            &[0, 0, 0, 0], // msg_instance
            &MSG_TYPE_VEHICLE_SPECIFIC1.to_be_bytes(),
            &[
                0, 0, 0, 5, // msg_length
                0, 0, 0, 1, // stream_id
                0xff, 0xff, 0xff, 0xff, // packet_seq = -1 in twos compliment
            ],
            payload,
        ]
        .concat();

        let msg_bytes = [msg_bytes.as_slice(), &checksum(&msg_bytes).to_be_bytes()].concat();

        let msg = parse(&msg_bytes).unwrap();
        assert_eq!(msg.payload, payload);
        assert_eq!(msg.header.msg_length, 5);
        let mut csum = 149; // IDD_2_5
        csum += 216; // msg_type = 2001 = byte(7) + byte(209)
        csum += 5; // msg len
        csum += 1; // stream id
        csum += 0xff * 4; // packet seq
        csum += 500; // "Hello" = 72 + 101 + 108 + 108 + 111
        assert_eq!(msg.footer.checksum, csum);
    }

    #[test]
    fn test_stanag_checksum() {
        let test_cases = [
            (vec![0x01, 0x01, 0x01, 0x01, 0x00, 0x00], 4),
            (vec![0xff, 0xff, 0xff, 0xff, 0x01, 0x01], 255 * 4 + 2),
            (vec![], 0),
            (vec![0x00, 0x00], 0),
            (vec![0x00, 0x01, 0x02], 3),
        ];
        for (bytes, expected) in test_cases.iter() {
            let sum = checksum(bytes);
            assert_eq!(sum, *expected, "checksum({:?}) = {}", bytes, sum);
        }
    }
}
