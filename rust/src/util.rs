//
//  OpenSlide, a library for reading whole slide image files
//
//  Copyright (c) 2020 Carnegie Mellon University
//  All rights reserved.
//
//  OpenSlide is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Lesser General Public License as
//  published by the Free Software Foundation, version 2.1.
//
//  OpenSlide is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//  GNU Lesser General Public License for more details.
//
//  You should have received a copy of the GNU Lesser General Public
//  License along with OpenSlide. If not, see
//  <http://www.gnu.org/licenses/>.
//
// SPDX-Licence-Identifier: LGPL-2.1-only
//

//! Various utility functions/classes for openslide-internals

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

/// FileSlice implements only limited functionality; consecutively read a
/// finite amount of data starting at a specific offset. If we need a more
/// extensive implementation we could use slice::ioSlice instead.
pub struct FileSlice {
    file: File,
    remaining: usize,
}

impl FileSlice {
    pub fn new(mut file: File, offset: u64, length: i64) -> io::Result<FileSlice> {
        let remaining = if length < 0 {
            std::usize::MAX
        } else {
            length as usize
        };
        file.seek(SeekFrom::Start(offset))?;
        Ok(FileSlice { file, remaining })
    }
}

impl Read for FileSlice {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let toread = std::cmp::min(self.remaining, buf.len());
        let nread = self.file.read(&mut buf[..toread])?;
        self.remaining -= nread;
        Ok(nread)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fileslice() -> io::Result<()> {
        let mut f = FileSlice::new(File::open("src/util.rs")?, 7, 20)?;
        let mut buf: [u8; 32] = [0; 32];
        let len = f.read(&mut buf)?;

        assert_eq!(&buf[..len], b"OpenSlide, a library");
        Ok(())
    }
}
