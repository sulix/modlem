use binary_io::*;

/*
 * The Lemmings 'dat' format consists of several 'sections', concatenated together.
 * Each 'section' consists of a simple header (in big-endian format), followed by
 * compressed data, using a compression algorthim reminiscent of the Amiga 'PowerPacker'
 * format. This is basically a bit-aligned LZ variant, with the exciting twist that
 * the data is compressed from end to beginning. This allows the compressed and
 * decompressed data to share a buffer.
 */

pub struct DatSection
{
    uncomp_size : u32,
    comp_size : u32,
    checksum : u8,
    num_bits_in_first_byte: u8,
    byte_offset : u32,
    bit_offset: u32,
    comp_data : std::vec::Vec<u8>,
}

impl DatSection
{

    /// Create a new 'empty' DatSection, which encodes 0 bytes.
    pub fn new_empty() -> DatSection {
        DatSection {
            uncomp_size : 0,
            comp_size : 0,
            checksum : 0,
            num_bits_in_first_byte: 0,
            byte_offset: 0,
            bit_offset: 0,
            comp_data: std::vec::Vec::<u8>::new(),
        }
    }

    /// Add an extra num_bits bits (< 32) to the compressed stream.
    fn add_bits(&mut self, num_bits : usize, mut bits : u32) {
        // The current (possibly partially written) byte in the compressed data.
        let mut cur_byte = self.comp_data[self.byte_offset as usize];

        for _n in 0..num_bits {
            let bit = (bits & 1) as u8;
            bits >>= 1;
            self.bit_offset += 1;

            // We've written a whole byte, reset to a new one.
            if self.bit_offset > 8 {
                self.comp_data[self.byte_offset as usize] = cur_byte;
                self.byte_offset += 1;
                self.bit_offset = 1;
                self.comp_data.push(0);
                cur_byte = 0;
            }

            // Shift one bit in.
            cur_byte = (cur_byte << 1) | bit;
        }

        // Save the byte we've been working on back into the array.
        self.comp_data[self.byte_offset as usize] = cur_byte;
        assert!(bits == 0);
    }

    /// Create a new DatSection from uncompressed data. The data will be compressed.
    pub fn from_data(data : &[u8], uncomp_size: usize) -> DatSection {
        let mut dat_section = DatSection::new_empty();
        dat_section.comp_data.push(0);
        dat_section.uncomp_size = uncomp_size as u32;
        dat_section.bit_offset = 1;
        let mut i = 0;
        let mut last_uncomp_off = 0;
        while i < uncomp_size {

            // Attempt to find the best matches:
            // - overall (longest_len / longest_off)
            // - 2 bytes long (best_match_2)
            // - 3 bytes long (best_match_3)
            // - 4 bytes long (best_match_4)

            let mut j = i + 1;
            let mut longest_off = 0;
            let mut longest_len = 1;
            let mut best_match_2 : Option<usize> = None;
            let mut best_match_3 : Option<usize> = None;
            let mut best_match_4 : Option<usize> = None;
            while j < i + 4096 {
                let mut match_len = 0;
                if j >= uncomp_size {
                    break;
                }
                while match_len < 256 {
                    if j + match_len >= uncomp_size {
                        break;
                    }
                    if data[j+match_len] != data[i+match_len] {
                        break;
                    }
                    match_len += 1;
                    if match_len == 2 && j < i + 256 {
                        best_match_2 = Some(j)
                    } else if match_len == 3 && j < i + 512 {
                        best_match_3 = Some(j)
                    } else if match_len == 4 && j < i + 1024 {
                        best_match_4 = Some(j)
                    }
                }
                if match_len >= longest_len {
                    longest_len = match_len;
                    longest_off = j;
                }
                j += 1;
            }

            // If we have a match which is better than nothing.
            let mut have_match = false;
            if longest_len > 4 { have_match = true; }
            else if best_match_4 != None {have_match = true; }
            else if best_match_3 != None {have_match = true; }
            else if best_match_2 != None {have_match = true; }

            if have_match {
                // Flush any uncompressed / literal data.
                while last_uncomp_off < i {
                    // We can output at most 265 (256 + 9) bytes in a single literal.
                    let uncomp_len = std::cmp::min(i - last_uncomp_off, 265);
                    if uncomp_len <= 8 {
                        // Output a small literal (5 + 8*n bits)
                        for b in 0..uncomp_len {
                            dat_section.add_bits(8, data[last_uncomp_off+b] as u32);
                        }
                        dat_section.add_bits(3, (uncomp_len - 1) as u32);
                        dat_section.add_bits(2, 0);
                    } else {
                        // Output a big literal (11 + 8*n bits)
                        for b in 0..uncomp_len {
                            dat_section.add_bits(8, data[last_uncomp_off+b] as u32);
                        }
                        dat_section.add_bits(8, (uncomp_len - 9) as u32);
                        dat_section.add_bits(3, 7);
                    }
                    last_uncomp_off += uncomp_len;
                }

                // Find the most efficient of the matches we have, and use it.
                if longest_len > 4 {
                    // First, if there's a >4 byte match, output it as an n-byte match.
                    // This takes 23 bits to save at least 40 bits
                    let offset = (longest_off - i - 1) as u32;
                    dat_section.add_bits(12, offset);
                    dat_section.add_bits(8, (longest_len - 1) as u32);
                    dat_section.add_bits(3, 6);
                } else if best_match_4 != None {
                    // Otherwise, try a 4-byte match (13 bits to save 32)
                    dat_section.add_bits(10, (best_match_4.unwrap() - i - 1) as u32);
                    dat_section.add_bits(3, 5);
                    longest_len = 4;
                } else if best_match_3 != None {
                    // Or, a 3-byte match (12 bits to save 24)
                    dat_section.add_bits(9, (best_match_3.unwrap() - i - 1) as u32);
                    dat_section.add_bits(3, 4);
                    longest_len = 3;
                } else if best_match_2 != None {
                    // Or a 2-byte match (10 bits to save 16)
                    dat_section.add_bits(8, (best_match_2.unwrap() - i - 1) as u32);
                    dat_section.add_bits(2, 1);
                    longest_len = 2;
                } else {
                    // No match found. This shouldn't happen.
                    panic!("Should've found a match here, but none was >1 byte long");
                }
                i += longest_len;
                last_uncomp_off = i;
            } else {
                // Increase our literal count, to flush later.
                i += 1;
            }


        }
        while last_uncomp_off < i {
            // We can output at most 265 (256 + 9) bytes in a single literal.
            let uncomp_len = std::cmp::min(i - last_uncomp_off, 265);
            if uncomp_len <= 8 {
                // Output a small literal (5 + 8*n bits)
                for b in 0..uncomp_len {
                    dat_section.add_bits(8, data[last_uncomp_off+b] as u32);
                }
                dat_section.add_bits(3, (uncomp_len - 1) as u32);
                dat_section.add_bits(2, 0);
            } else {
                // Output a big literal (11 + 8*n bits)
                for b in 0..uncomp_len {
                    dat_section.add_bits(8, data[last_uncomp_off+b] as u32);
                }
                dat_section.add_bits(8, (uncomp_len - 9) as u32);
                dat_section.add_bits(3, 7);
            }
            last_uncomp_off += uncomp_len;
        }

        // Write out the metadata for the section: compressed length (plus header), and num_bits_in_first_byte
        dat_section.comp_size = dat_section.comp_data.len() as u32 + 10;
        dat_section.num_bits_in_first_byte = dat_section.bit_offset as u8;

        // Calculate the checksum.
        for b in &dat_section.comp_data {
            dat_section.checksum ^= b;
        }

        println!("Compressed Dat Section from {} bytes to {} bytes", dat_section.uncomp_size, dat_section.comp_size);
        dat_section
    }

    /// Reads a DatSection from a file and verifies the checksum (but doesn't decompress it)
    pub fn from_file(reader : &mut dyn std::io::Read) -> std::io::Result<DatSection> {
        let num_bits_in_first_byte = read_byte(reader)?;
        let checksum = read_byte(reader)?;
        let uncomp_size = read_be32(reader)?;
        let comp_size = read_be32(reader)?;

        let mut comp_data = std::vec::Vec::<u8>::new();
        comp_data.resize((comp_size - 10) as usize, 0);
        reader.read_exact(&mut comp_data)?;
        let mut data_checksum = 0;
        for b in &comp_data {
            data_checksum ^= b;
        }
        if data_checksum != checksum {
            println!("Expected checksum {:x}, got {:x}\n", checksum, data_checksum);
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "checksum invalid"))
        } else {
            Ok(DatSection {
                uncomp_size,
                comp_size : if num_bits_in_first_byte == 0 { comp_size-1 } else { comp_size },
                checksum,
                num_bits_in_first_byte: if num_bits_in_first_byte == 0 { 8 } else { num_bits_in_first_byte },
                byte_offset: comp_size - if num_bits_in_first_byte == 0 { 12 } else { 11 } as u32,
                bit_offset : 0 as u32,
                comp_data
            })
        }
    }

    /// Writes an already-compressed the section to a file.
    pub fn write(self, writer : &mut dyn std::io::Write) -> std::io::Result<()> {
        write_byte(self.num_bits_in_first_byte, writer)?;
        write_byte(self.checksum, writer)?; /* checksum */
        write_be32(self.uncomp_size, writer)?;
        write_be32(self.comp_size, writer)?;
        writer.write_all(self.comp_data.as_slice())?;
        Ok(())
    }

    /// Read 'bits' bits of compressed data, in reverse, from the compressed stream.
    fn read_bits(&mut self, bits: u32) -> u32 {
        let mut val : u32 = 0;
        for _n in 0..bits {
            let cur_byte = self.comp_data[self.byte_offset as usize];
            let bit = if ((1 << self.bit_offset) & cur_byte) != 0 { 1 } else { 0 };
            let bits_in_byte = if (self.byte_offset == self.comp_size - 11) && (self.num_bits_in_first_byte != 0) { self.num_bits_in_first_byte as u32 } else { 8 };
            self.bit_offset += 1;
            if self.bit_offset >= bits_in_byte {
                if self.byte_offset != 0 {
                    self.byte_offset -= 1;
                }
                self.bit_offset = 0;
            }
            val = (val << 1) | bit;
        }
        val
    }

    /// Decompressed a DatSection.
    pub fn decompress(&mut self) -> std::vec::Vec<u8> {
        let mut output : std::vec::Vec::<u8> = vec![0; self.uncomp_size as usize];
        let mut i = (self.uncomp_size - 1) as usize;

        while i > 0 {
            match self.read_bits(1) {
                0 => {
                    // Commands starting with '0' are two bits.
                    match self.read_bits(1) {
                        0 => {
                            // Raw bytes.
                            let len = self.read_bits(3) + 1;
                            i = i + 1;
                            for _b in 0..len {
                                i = i - 1;
                                output[i] = self.read_bits(8) as u8;
                            }
                        }
                        1 => {
                            // Two-byte reference
                            let raw_off = self.read_bits(8);
                            let off = i + 1 + raw_off as usize;
                            output[i] = output[off];
                            i -= 1;
                            output[i] = output[off-1];
                        }
                        _ => {
                            panic!("Bad two-bit command! A bit was neither 0 nor 1!");
                        }
                    }
                }
                1 => {
                    // Commands which start with a 1 are 3-bits

                    // Read the remaining two bits
                    let cmd = self.read_bits(2);

                    match cmd {
                        // '100' Three byte match
                        0 => {
                            let raw_off = self.read_bits(9);
                            let off = i + 1 + raw_off as usize;
                            output[i] = output[off];
                            i -= 1;
                            output[i] = output[off-1];
                            i -= 1;
                            output[i] = output[off-2];
                        }
                        // '101' Four byte match
                        1 => {
                            let off = i + 1 + self.read_bits(10) as usize;
                            output[i] = output[off];
                            i -= 1;
                            output[i] = output[off-1];
                            i -= 1;
                            output[i] = output[off-2];
                            i -= 1;
                            output[i] = output[off-3];
                        }
                        // '110' n-byte match
                        2 => {
                            let len = self.read_bits(8) + 1;
                            let raw_off = self.read_bits(12) as usize;
                            i += 1;
                            for _b in 0..len {
                                i -= 1;
                                output[i] = output[i + raw_off + 1];
                            }
                        }
                        // '111' big literal (8 bit length)
                        3 => {
                            let len = self.read_bits(8) + 9;
                            i += 1;
                            for _b in 0..len {
                                i -= 1;
                                output[i] = self.read_bits(8) as u8;
                            }
                        }
                        _ => {
                            panic!("Bad 3-bit command 1{}", cmd);
                        }
                    }
                }
                _ => {
                    panic!("Looks like we've got a bit whose value is not 0 or 1!");
                }
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }
        output
    }
}
