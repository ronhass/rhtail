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
    let start_offset = match start_offset {
        None => 0,
        Some(n) => n + 1,
    };
    print_file(input, output, start_offset, end_offset)?;
    Ok(())
}

struct RevReader<R> {
    inner: R,
    inner_offset: u64,
    buffer: [u8; BUFFER_SIZE],
    buffer_offset: u64,
    size: u64,
}

impl<R: Read + Seek> RevReader<R> {
    fn new(inner: R) -> Result<RevReader<R>, io::Error> {
        let mut r = RevReader {
            inner,
            inner_offset: 0,
            buffer: [0; BUFFER_SIZE],
            buffer_offset: 0,
            size: 0
        };
        r.size = r.inner.seek(SeekFrom::End(0))?;
        r.inner_offset = r.size;
        Ok(r)
    }

    fn read_byte(&mut self) -> Result<Option<u8>, io::Error> {
        if self.buffer_offset == 0 {
            if self.inner_offset == 0 {
                return Ok(None);
            }
            let _buffer_offset = cmp::min(BUFFER_SIZE as u64, self.inner_offset);
            let _inner_offset = self.inner_offset - _buffer_offset;

            self.inner.seek(SeekFrom::Start(_inner_offset))?;
            self.inner_offset = _inner_offset;

            self.inner.read_exact(&mut self.buffer[0.._buffer_offset as usize])?;
            self.buffer_offset = _buffer_offset;
        }
        self.buffer_offset -= 1;
        Ok(Some(self.buffer[self.buffer_offset as usize]))
    }

    fn offset(&self) -> u64 {
        return self.inner_offset + self.buffer_offset;
    }
}

fn rfind_count<R>(input: &mut R, byte: u8, count: u64, ignore_last: bool) -> Result<(Option<u64>, u64), io::Error>
where
    R: Read + Seek,
{
    let mut rev_reader = RevReader::new(input)?;
    let input_size = rev_reader.size;
    let mut counter = 0;

    if ignore_last {
        if rev_reader.read_byte()? == None {
            return Ok((None, input_size));
        }
    }

    loop {
        match rev_reader.read_byte()? {
            None => return Ok((None, input_size)),
            Some(c) if c == byte => counter += 1,
            _ => (),
        }
        
        if counter == count {
            return Ok((Some(rev_reader.offset()), input_size));
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
        assert_eq!(offset, Some(3));
        assert_eq!(size, 12);
        Ok(())
    }

    #[test]
    fn test_rfind_count_empty_line() -> Result<(), io::Error> {
        use std::io::Write;
        let mut tmp_file = tempfile::tempfile()?;
        write!(tmp_file, "1\n2\n3\n4\n5\n")?;
        let (offset, size) = rfind_count(&mut tmp_file, b'\n', 2, true)?;
        assert_eq!(offset, Some(5));
        assert_eq!(size, 10);
        Ok(())
    }

    #[test]
    fn test_rfind_count_not_found() -> Result<(), io::Error> {
        use std::io::Write;
        let mut tmp_file = tempfile::tempfile()?;
        write!(tmp_file, "123456")?;
        let (offset, size) = rfind_count(&mut tmp_file, b'1', 2, false)?;
        assert_eq!(offset, None);
        assert_eq!(size, 6);
        Ok(())
    }
}

