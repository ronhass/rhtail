use std::io::{self, Read, Write, Seek, SeekFrom};
use std::cmp;
use std::fs::File;

const BUFFER_SIZE: usize = 1024;

pub fn tail<R, W>(input: &mut R, output: &mut W, lines: u64) -> Result<(), io::Error>
where
    R: Read + Seek,
    W: Write,
{
    let (start_offset, end_offset) = rfind_count(input, b'\n', lines, true)?;
    print_file(input, output, start_offset + 1, end_offset)?;
    Ok(())
}

fn rfind_count<R>(input: &mut R, byte: u8, count: u64, ignore_last: bool) -> Result<(u64, u64), io::Error>
where
    R: Read + Seek,
{
    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let input_size = input.seek(SeekFrom::End(0))?;

    let mut start_offset = input_size;
    let mut end_offset;
    let mut counter = 0;
    loop {
        end_offset = start_offset;
        if end_offset == 0 {
            return Ok((0, input_size));
        }

        let buf_size;
        if end_offset < BUFFER_SIZE as u64 {
            start_offset = 0;
            buf_size = end_offset as usize;
        } else {
            start_offset = end_offset - BUFFER_SIZE as u64;
            buf_size = BUFFER_SIZE;
        }

        input.seek(SeekFrom::Start(start_offset))?;
        input.read_exact(&mut buffer[..buf_size])?;

        for i in (0..buf_size - 1).rev() {
            if buffer[i] == byte && (!ignore_last || end_offset != input_size || i != buf_size - 1){
                counter += 1;
                if counter == count {
                    return Ok((start_offset + i as u64, input_size));
                }
            }
        }
    }
}

fn print_file<R, W>(input: &mut R, output: &mut W, offset: u64, end: u64) -> Result<(), io::Error>
where
    R: Read + Seek,
    W: Write,
{
    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    input.seek(SeekFrom::Start(offset))?;

    let mut current_offset = offset;
    while current_offset < end {
        let buf_size = cmp::min(BUFFER_SIZE, (end - current_offset) as usize);
        input.read_exact(&mut buffer[..buf_size])?;
        output.write_all(&buffer[..buf_size])?;
        current_offset += buf_size as u64;
    }

    Ok(())
}

pub fn follow_file<W>(input_file: &mut File, output: &mut W) -> Result<(), io::Error>
where
    W: Write
{
    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let mut buf_size;

    // Naive implementation: busy waiting
    loop {
        buf_size = input_file.read(&mut buffer)?;
        if buf_size > 0 && buf_size <= buffer.len() {
            output.write_all(&buffer[..buf_size])?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rfind_count() -> Result<(), io::Error> {
        use std::io::Write;
        let mut tmp_file = tempfile::tempfile()?;
        write!(tmp_file, "Hello world!")?;
        let (offset, size) = rfind_count(&mut tmp_file, b'l', 2, false)?;
        assert_eq!(offset, 3);
        assert_eq!(size, 12);
        Ok(())
    }

    #[test]
    fn test_rfind_count_empty_line() -> Result<(), io::Error> {
        use std::io::Write;
        let mut tmp_file = tempfile::tempfile()?;
        write!(tmp_file, "1\n2\n3\n4\n5\n")?;
        let (offset, size) = rfind_count(&mut tmp_file, b'\n', 2, true)?;
        assert_eq!(offset, 5);
        assert_eq!(size, 10);
        Ok(())
    }
}

