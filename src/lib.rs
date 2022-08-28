use std::fs::File;
use std::io::{self, Read, Write, Seek};
use std::cmp;

const BUFFER_SIZE: usize = 1024;

pub fn tail(path: String, lines: u64) -> Result<(), io::Error> {
    let mut file = File::open(path)?;
    let (start_offset, end_offset) = rfind_count(&mut file, b'\n', lines, true)?;
    print_file(&mut file, start_offset + 1, end_offset)?;
    Ok(())
}

fn rfind_count(f: &mut File, byte: u8, count: u64, ignore_last: bool) -> Result<(u64, u64), io::Error> {
    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let file_size = f.metadata()?.len();

    let mut start_offset = file_size;
    let mut end_offset;
    let mut counter = 0;
    loop {
        end_offset = start_offset;
        if end_offset == 0 {
            return Ok((0, file_size));
        }

        let buf_size;
        if end_offset < BUFFER_SIZE as u64 {
            start_offset = 0;
            buf_size = end_offset as usize;
        } else {
            start_offset = end_offset - BUFFER_SIZE as u64;
            buf_size = BUFFER_SIZE;
        }

        f.seek(io::SeekFrom::Start(start_offset))?;
        f.read_exact(&mut buffer[..buf_size])?;

        for i in (0..buf_size - 1).rev() {
            if buffer[i] == byte && (!ignore_last || end_offset != file_size || i != buf_size - 1){
                counter += 1;
                if counter == count {
                    return Ok((start_offset + i as u64, file_size));
                }
            }
        }
    }
}

fn print_file(f: &mut File, offset: u64, end: u64) -> Result<(), io::Error> {
    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    f.seek(io::SeekFrom::Start(offset))?;

    let mut current_offset = offset;
    while current_offset < end {
        let buf_size = cmp::min(BUFFER_SIZE, (end - current_offset) as usize);
        f.read_exact(&mut buffer[..buf_size])?;
        io::stdout().write_all(&buffer[..buf_size])?;
        current_offset += buf_size as u64;
    }

    Ok(())
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

