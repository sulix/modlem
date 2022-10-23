pub fn read_byte(reader: &mut dyn std::io::Read) -> std::io::Result<u8> {
    let mut out_byte : u8 = 0;
    reader.read_exact(std::slice::from_mut(&mut out_byte))?;
    return Ok(out_byte);
}

pub fn read_le16(reader : &mut dyn std::io::Read) -> std::io::Result<u16> {
    let mut raw_bytes = [0 as u8; 2];
    reader.read_exact(&mut raw_bytes)?;
    return Ok((raw_bytes[1] as u16) << 8 | (raw_bytes[0] as u16));
}

pub fn read_le32(reader : &mut dyn std::io::Read) -> std::io::Result<u32> {
    let mut raw_bytes = [0 as u8; 4];
    reader.read_exact(&mut raw_bytes)?;
    return Ok((raw_bytes[3] as u32) << 24 | (raw_bytes[2] as u32) << 16 | (raw_bytes[1] as u32) << 8 | (raw_bytes[0] as u32));
}

pub fn read_be16(reader : &mut dyn std::io::Read) -> std::io::Result<u16> {
    let mut raw_bytes = [0 as u8; 2];
    reader.read_exact(&mut raw_bytes)?;
    return Ok((raw_bytes[0] as u16) << 8 | (raw_bytes[1] as u16));
}

pub fn read_be32(reader : &mut dyn std::io::Read) -> std::io::Result<u32> {
    let mut raw_bytes = [0 as u8; 4];
    reader.read_exact(&mut raw_bytes)?;
    return Ok((raw_bytes[0] as u32) << 24 | (raw_bytes[1] as u32) << 16 | (raw_bytes[2] as u32) << 8 | (raw_bytes[3] as u32));
}

pub fn write_byte(out_byte : u8, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    writer.write_all(std::slice::from_ref(&out_byte))
}

pub fn write_be16(out_val : u16, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    let raw_bytes = [(out_val >> 8) as u8, (out_val & 0xFF) as u8];
    writer.write_all(&raw_bytes)
}

pub fn write_be32(out_val : u32, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    let raw_bytes = [((out_val >> 24) )as u8, (out_val >> 16) as u8, (out_val >> 8) as u8, (out_val & 0xFF) as u8];
    writer.write_all(&raw_bytes)
}

pub fn write_le16(out_val : u16, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    let raw_bytes = [(out_val & 0xFF) as u8, (out_val >> 8) as u8];
    writer.write_all(&raw_bytes)
}

pub fn write_le32(out_val : u32, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    let raw_bytes = [(out_val) as u8, (out_val >> 8) as u8, (out_val >> 16) as u8, (out_val >> 24) as u8];
    writer.write_all(&raw_bytes)
}
