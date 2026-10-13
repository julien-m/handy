//! Framing between Handy and its read-only snapshot helper process.
//!
//! Stream layout (integers little-endian):
//! 1. header: `HSPS`, version byte, 16-byte request nonce echoed back;
//! 2. `B` begin: source changeCount, then the inventory (types per item);
//! 3. `O` omitted: items and types left out of the inventory because they
//!    exceed the structural limits below (counts only);
//! 4. one `T` (data) or `S` (skipped) frame per inventoried type, declared
//!    types first and derived types last (see `pasteboard_types`);
//! 5. `E` end: changeCount observed after reading.
//!
//! Frames are progressive so the parent keeps every completed type when it
//! kills the helper at the deadline. The decoder bounds every length, so a
//! malformed stream can never make the parent allocate past the budget.

use std::io::{self, Read, Write};

use super::snapshot::SkipCause;

/// Stream magic.
pub const MAGIC: [u8; 4] = *b"HSPS";
/// Protocol version (2 added the omitted-counts frame).
pub const VERSION: u8 = 2;
/// Request nonce length in bytes.
pub const NONCE_LEN: usize = 16;
// Structural limits far above real pasteboards (a few items, tens of
// types); they only stop a malformed stream from forcing large allocations.
// The helper truncates its inventory to them and reports what it left out.
/// Longest accepted type identifier; longer names are left out of the
/// inventory by the helper (no real UTI comes close).
pub const MAX_UTI_BYTES: usize = 1024;
/// Most items inventoried (a Finder copy of more files is truncated).
pub const MAX_ITEMS: usize = 10_000;
/// Most types inventoried per item.
pub const MAX_TYPES_PER_ITEM: usize = 1_000;

/// Request nonce binding a helper response to its request.
pub type Nonce = [u8; NONCE_LEN];

/// One protocol frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    /// Inventory of the pasteboard when reading started.
    Begin {
        /// Source `changeCount`.
        change_count: i64,
        /// Type identifiers per item, in pasteboard order.
        inventory: Vec<Vec<String>>,
    },
    /// Data of one inventoried type.
    Data {
        /// Item index.
        item: usize,
        /// Type index within the item.
        type_index: usize,
        /// Raw bytes.
        data: Vec<u8>,
    },
    /// Items and types left out of the inventory (structural limits).
    Omitted {
        /// Items beyond [`MAX_ITEMS`].
        items: usize,
        /// Types beyond [`MAX_TYPES_PER_ITEM`] or [`MAX_UTI_BYTES`].
        types: usize,
    },
    /// One inventoried type the helper did not save.
    Skipped {
        /// Item index.
        item: usize,
        /// Type index within the item.
        type_index: usize,
        /// Why it was skipped.
        cause: SkipCause,
    },
    /// Reading finished.
    End {
        /// `changeCount` observed after reading.
        change_count: i64,
    },
}

/// Protocol violations detected by the parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolError {
    /// The underlying pipe failed.
    Io(io::ErrorKind),
    /// Wrong magic or version.
    BadHeader,
    /// The echoed nonce does not match the request.
    NonceMismatch,
    /// A length, index or tag is out of bounds.
    Malformed,
    /// A data frame exceeds the remaining byte budget.
    DataTooLarge,
}

impl From<io::Error> for ProtocolError {
    fn from(error: io::Error) -> Self {
        Self::Io(error.kind())
    }
}

/// Writes the stream header.
pub fn write_header(writer: &mut impl Write, nonce: &Nonce) -> io::Result<()> {
    writer.write_all(&MAGIC)?;
    writer.write_all(&[VERSION])?;
    writer.write_all(nonce)
}

/// Writes one frame.
pub fn write_frame(writer: &mut impl Write, frame: &Frame) -> io::Result<()> {
    match frame {
        Frame::Begin {
            change_count,
            inventory,
        } => write_begin(writer, *change_count, inventory),
        Frame::Data {
            item,
            type_index,
            data,
        } => write_data(writer, (*item, *type_index), data),
        Frame::Skipped {
            item,
            type_index,
            cause,
        } => {
            writer.write_all(b"S")?;
            write_index_pair(writer, *item, *type_index)?;
            writer.write_all(&[cause_code(*cause)])
        }
        Frame::Omitted { items, types } => write_omitted(writer, *items, *types),
        Frame::End { change_count } => {
            writer.write_all(b"E")?;
            writer.write_all(&change_count.to_le_bytes())
        }
    }
}

fn write_begin(
    writer: &mut impl Write,
    change_count: i64,
    inventory: &[Vec<String>],
) -> io::Result<()> {
    writer.write_all(b"B")?;
    writer.write_all(&change_count.to_le_bytes())?;
    writer.write_all(&(inventory.len() as u32).to_le_bytes())?;
    for types in inventory {
        writer.write_all(&(types.len() as u32).to_le_bytes())?;
        for uti in types {
            writer.write_all(&(uti.len() as u16).to_le_bytes())?;
            writer.write_all(uti.as_bytes())?;
        }
    }
    Ok(())
}

fn write_data(
    writer: &mut impl Write,
    (item, type_index): (usize, usize),
    data: &[u8],
) -> io::Result<()> {
    writer.write_all(b"T")?;
    write_index_pair(writer, item, type_index)?;
    writer.write_all(&(data.len() as u64).to_le_bytes())?;
    writer.write_all(data)
}

fn write_omitted(writer: &mut impl Write, items: usize, types: usize) -> io::Result<()> {
    writer.write_all(b"O")?;
    writer.write_all(&(items as u64).to_le_bytes())?;
    writer.write_all(&(types as u64).to_le_bytes())
}

fn write_index_pair(writer: &mut impl Write, item: usize, type_index: usize) -> io::Result<()> {
    writer.write_all(&(item as u32).to_le_bytes())?;
    writer.write_all(&(type_index as u32).to_le_bytes())
}

fn cause_code(cause: SkipCause) -> u8 {
    match cause {
        SkipCause::Oversize => 1,
        SkipCause::Timeout => 2,
        SkipCause::Unreadable => 3,
    }
}

fn cause_from_code(code: u8) -> Result<SkipCause, ProtocolError> {
    match code {
        1 => Ok(SkipCause::Oversize),
        2 => Ok(SkipCause::Timeout),
        3 => Ok(SkipCause::Unreadable),
        _ => Err(ProtocolError::Malformed),
    }
}

/// Reads and checks the stream header against the request nonce.
pub fn read_header(reader: &mut impl Read, nonce: &Nonce) -> Result<(), ProtocolError> {
    let mut magic = [0_u8; 5];
    reader.read_exact(&mut magic)?;
    if magic[..4] != MAGIC || magic[4] != VERSION {
        return Err(ProtocolError::BadHeader);
    }
    let mut echoed = [0_u8; NONCE_LEN];
    reader.read_exact(&mut echoed)?;
    if &echoed != nonce {
        return Err(ProtocolError::NonceMismatch);
    }
    Ok(())
}

/// Reads the next frame; `Ok(None)` on a clean end of stream. `max_data`
/// bounds the size of a data frame (the remaining byte budget).
pub fn read_frame(reader: &mut impl Read, max_data: usize) -> Result<Option<Frame>, ProtocolError> {
    let mut tag = [0_u8; 1];
    if reader.read(&mut tag)? == 0 {
        return Ok(None);
    }
    let frame = match &tag {
        b"B" => read_begin(reader)?,
        b"T" => read_data(reader, max_data)?,
        b"S" => {
            let (item, type_index) = read_index_pair(reader)?;
            let cause = cause_from_code(read_u8(reader)?)?;
            Frame::Skipped {
                item,
                type_index,
                cause,
            }
        }
        b"O" => Frame::Omitted {
            items: read_count(reader)?,
            types: read_count(reader)?,
        },
        b"E" => Frame::End {
            change_count: i64::from_le_bytes(read_array(reader)?),
        },
        _ => return Err(ProtocolError::Malformed),
    };
    Ok(Some(frame))
}

fn read_begin(reader: &mut impl Read) -> Result<Frame, ProtocolError> {
    let change_count = i64::from_le_bytes(read_array(reader)?);
    let item_count = bounded(read_u32(reader)?, MAX_ITEMS)?;
    let mut inventory = Vec::with_capacity(item_count);
    for _ in 0..item_count {
        let type_count = bounded(read_u32(reader)?, MAX_TYPES_PER_ITEM)?;
        let mut types = Vec::with_capacity(type_count);
        for _ in 0..type_count {
            let length = bounded(
                u32::from(u16::from_le_bytes(read_array(reader)?)),
                MAX_UTI_BYTES,
            )?;
            let mut bytes = vec![0_u8; length];
            reader.read_exact(&mut bytes)?;
            types.push(String::from_utf8(bytes).map_err(|_| ProtocolError::Malformed)?);
        }
        inventory.push(types);
    }
    Ok(Frame::Begin {
        change_count,
        inventory,
    })
}

fn read_data(reader: &mut impl Read, max_data: usize) -> Result<Frame, ProtocolError> {
    let (item, type_index) = read_index_pair(reader)?;
    let length = usize::try_from(u64::from_le_bytes(read_array(reader)?))
        .map_err(|_| ProtocolError::DataTooLarge)?;
    if length > max_data {
        return Err(ProtocolError::DataTooLarge);
    }
    let mut data = vec![0_u8; length];
    reader.read_exact(&mut data)?;
    Ok(Frame::Data {
        item,
        type_index,
        data,
    })
}

fn bounded(value: u32, limit: usize) -> Result<usize, ProtocolError> {
    let value = usize::try_from(value).map_err(|_| ProtocolError::Malformed)?;
    (value <= limit)
        .then_some(value)
        .ok_or(ProtocolError::Malformed)
}

fn read_index_pair(reader: &mut impl Read) -> Result<(usize, usize), ProtocolError> {
    let item = bounded(read_u32(reader)?, MAX_ITEMS)?;
    let type_index = bounded(read_u32(reader)?, MAX_TYPES_PER_ITEM)?;
    Ok((item, type_index))
}

fn read_count(reader: &mut impl Read) -> Result<usize, ProtocolError> {
    usize::try_from(u64::from_le_bytes(read_array(reader)?)).map_err(|_| ProtocolError::Malformed)
}

fn read_u8(reader: &mut impl Read) -> Result<u8, ProtocolError> {
    Ok(read_array::<1>(reader)?[0])
}

fn read_u32(reader: &mut impl Read) -> Result<u32, ProtocolError> {
    Ok(u32::from_le_bytes(read_array(reader)?))
}

fn read_array<const N: usize>(reader: &mut impl Read) -> Result<[u8; N], ProtocolError> {
    let mut bytes = [0_u8; N];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}
