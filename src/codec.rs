//! (De)compression and enciphering/deciphering.

use std::convert::TryFrom;
#[cfg(feature = "rs3")]
use std::io::BufReader;
use std::io::{self, Read, Write};

use bzip2::{read::BzDecoder, write::BzEncoder};
use flate2::{bufread::GzDecoder, write::GzEncoder};
#[cfg(feature = "rs3")]
use lzma_rs::{compress, decompress, lzma_compress_with_options, lzma_decompress_with_options};
use nom::{
    combinator::cond,
    number::complete::{be_i16, be_u32, be_u8},
};

use crate::{error::CompressionUnsupported, xtea};

use std::marker::PhantomData;

/// Supported compression types.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Compression {
    None,
    Bzip2,
    Gzip,
    #[cfg(feature = "rs3")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rs3")))]
    Lzma,
}

/// Marker struct conveying `State` of a [`Buffer`](Buffer).
pub struct Encoded;
/// Marker struct conveying `State` of a [`Buffer`](Buffer).
pub struct Decoded;

/// Primary way to store bytes for encoding and decoding.
pub struct Buffer<State> {
    compression: Compression,
    buffer: Vec<u8>,
    version: Option<i16>,
    keys: Option<[u32; 4]>,
    _state: PhantomData<State>,
}

impl Buffer<Decoded> {
    /// Encodes the buffer, consuming self and returning a `Buffer<Encoded>`.
    ///
    /// The following process takes place when encoding:
    /// 1. Compress the buffer with the selected compression format.
    /// 2. Allocate a new buffer.
    /// 3. Push the compression type as a byte into the new buffer.
    /// 4. Push the length (u32) into the buffer of the compressed data from step 1.
    /// 5. If a compression type was selected (and not `Compression::None`) insert the uncompressed length as u32.
    /// 6. Extend the buffer with the compressed data.
    /// 7. Add the `version` as i16 if present.
    /// 8. Encode complete.
    ///
    /// **NOTE: When compressing with gzip the header is removed
    /// before the compressed data is returned.
    /// The encoded buffer will not contain the gzip header.**
    ///
    /// # Errors
    ///
    /// Returns an error if the data couldn't be compressed or is invalid.
    pub fn encode(self) -> crate::Result<Buffer<Encoded>> {
        let decompressed_len = self.buffer.len();
        let mut compressed_data = match self.compression {
            Compression::None => self.buffer,
            Compression::Bzip2 => compress_bzip2(&self.buffer)?,
            Compression::Gzip => compress_gzip(&self.buffer)?,
            #[cfg(feature = "rs3")]
            Compression::Lzma => compress_lzma(&self.buffer)?,
        };
        if let Some(keys) = &self.keys {
            xtea::encipher(&mut compressed_data, keys);
        }
        let mut buffer = Vec::with_capacity(compressed_data.len() + 11);
        buffer.write_all(&[self.compression as u8])?;
        buffer.write_all(&u32::to_be_bytes(compressed_data.len() as u32))?;
        if self.compression != Compression::None {
            buffer.write_all(&u32::to_be_bytes(decompressed_len as u32))?;
        }
        buffer.extend(compressed_data);
        if let Some(version) = self.version {
            buffer.write_all(&i16::to_be_bytes(version))?;
        }

        Ok(Buffer {
            compression: self.compression,
            buffer,
            version: self.version,
            keys: self.keys,
            _state: PhantomData,
        })
    }
}

impl Buffer<Encoded> {
    /// Decodes the buffer, consuming self and returning a `Buffer<Decoded>`.
    ///
    /// The following process takes place when decoding:
    /// 1. Read the first byte to determine which compression type should be used to decompress.
    /// 2. Read the length of the rest of the buffer.
    /// 3. Decompress the remaining bytes.
    ///
    /// # Panics
    /// 
    /// When data can't be decompressed using LZMA this function panics.
    /// 
    /// # Errors
    ///
    /// Returns an error if the remaining bytes couldn't be decompressed.
    pub fn decode(self) -> crate::Result<Buffer<Decoded>> {
        let (buffer, compression) = be_u8(self.buffer.as_slice())?;
        let compression = Compression::try_from(compression)?;

        let (buffer, compressed_len) = be_u32(buffer)?;
        let compressed_len = compressed_len as usize;

        let mut buffer = std::borrow::Cow::from(buffer);
        if let Some(keys) = self.keys {
            xtea::decipher(buffer.to_mut(), &keys);
        }

        let (version, buffer) = match compression {
            Compression::None => decompress_none(&buffer, compressed_len)?,
            Compression::Bzip2 => decompress_bzip2(&buffer, compressed_len)?,
            Compression::Gzip => decompress_gzip(&buffer, compressed_len)?,
            #[cfg(feature = "rs3")]
            Compression::Lzma => decompress_lzma(&buffer, compressed_len)?,
        };

        Ok(Buffer {
            compression,
            buffer,
            version,
            keys: self.keys,
            _state: PhantomData,
        })
    }
}

impl<State> Buffer<State> {
    /// Set the compression format for this buffer returning a new instance of `Self`.
    pub fn with_compression(mut self, compression: Compression) -> Self {
        self.compression = compression;
        self
    }

    /// Set the version for this buffer returning a new instance of `Self`.
    pub fn with_version(mut self, version: i16) -> Self {
        self.version = Some(version);
        self
    }

    /// Set the xtea keys for this buffer returning a new instance of `Self`.
    pub fn with_xtea_keys(mut self, keys: [u32; 4]) -> Self {
        self.keys = Some(keys);
        self
    }

    /// Convert the `Buffer` with its current state into a raw `Vec<u8>`.
    #[inline]
    pub fn finalize(self) -> Vec<u8> {
        self.buffer
    }
}

impl<State> Default for Buffer<State> {
    fn default() -> Self {
        Self {
            compression: Compression::None,
            buffer: Vec::new(),
            version: None,
            keys: None,
            _state: PhantomData,
        }
    }
}

impl<State> std::fmt::Debug for Buffer<State> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer")
            .field("compression", &self.compression)
            .field("keys", &self.keys)
            .field("version", &self.version)
            .field("buffer", &self.buffer)
            .finish()
    }
}

impl<State> From<&[u8]> for Buffer<State> {
    fn from(buffer: &[u8]) -> Self {
        Self {
            buffer: Vec::from(buffer),
            ..Self::default()
        }
    }
}

impl<State> From<Vec<u8>> for Buffer<State> {
    fn from(buffer: Vec<u8>) -> Self {
        Self {
            buffer,
            ..Self::default()
        }
    }
}

impl<State> std::ops::Deref for Buffer<State> {
    type Target = Vec<u8>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<State> std::ops::DerefMut for Buffer<State> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

impl<State> std::convert::AsRef<[u8]> for Buffer<State> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.buffer.as_slice()
    }
}

impl<State> std::io::Write for Buffer<State> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.buffer.write(buffer)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buffer.flush()
    }
}

fn compress_bzip2(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut compressor = BzEncoder::new(Vec::with_capacity(data.len()), bzip2::Compression::fast());
    compressor.write_all(data)?;
    let mut compressed_data = compressor.finish()?;
    compressed_data.drain(..4);

    Ok(compressed_data)
}

fn compress_gzip(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut compressor =
        GzEncoder::new(Vec::with_capacity(data.len()), flate2::Compression::best());
    compressor.write_all(data)?;
    let compressed_data: Vec<u8> = compressor.finish()?;

    Ok(compressed_data)
}

#[cfg(feature = "rs3")]
fn compress_lzma(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut input = std::io::BufReader::new(data);
    let mut output = Vec::with_capacity(data.len());
    let options = compress::Options {
        unpacked_size: compress::UnpackedSize::SkipWritingToHeader,
    };

    lzma_compress_with_options(&mut input, &mut output, &options)?;

    Ok(output)
}

fn decompress_none(buffer: &[u8], len: usize) -> crate::Result<(Option<i16>, Vec<u8>)> {
    let (buffer, data) = nom::bytes::complete::take(len)(buffer)?;
    let (_, version) = cond(buffer.len() >= 2, be_i16)(buffer)?;

    Ok((version, data.to_vec()))
}

fn decompress_bzip2(buffer: &[u8], len: usize) -> crate::Result<(Option<i16>, Vec<u8>)> {
    let (buffer, decompressed_len) = be_u32(buffer)?;
    let (buffer, data) = nom::bytes::complete::take(len)(buffer)?;
    let (_, version) = cond(buffer.len() >= 2, be_i16)(buffer)?;

    let mut compressed_data = data.to_vec();
    compressed_data[4..len].copy_from_slice(&data[..len - 4]);
    compressed_data[..4].copy_from_slice(b"BZh1");

    let mut decompressor = BzDecoder::new(compressed_data.as_slice());
    let mut decompressed_data = vec![0; decompressed_len as usize];
    decompressor.read_exact(&mut decompressed_data)?;

    Ok((version, decompressed_data))
}

fn decompress_gzip(buffer: &[u8], len: usize) -> crate::Result<(Option<i16>, Vec<u8>)> {
    let (buffer, decompressed_len) = be_u32(buffer)?;
    let (buffer, data) = nom::bytes::complete::take(len)(buffer)?;
    let (_, version) = cond(buffer.len() >= 2, be_i16)(buffer)?;

    let mut decompressor = GzDecoder::new(data);
    let mut decompressed_data = vec![0; decompressed_len as usize];
    decompressor.read_exact(&mut decompressed_data)?;

    Ok((version, decompressed_data))
}

#[cfg(feature = "rs3")]
fn decompress_lzma(buffer: &[u8], len: usize) -> crate::Result<(Option<i16>, Vec<u8>)> {
    let (buffer, decompressed_len) = be_u32(buffer)?;
    let (buffer, data) = nom::bytes::complete::take(len)(buffer)?;
    let (_, version) = cond(buffer.len() >= 2, be_i16)(buffer)?;

    let mut decompressed_data = Vec::with_capacity(decompressed_len as usize);
    let mut wrapper = BufReader::new(data);
    let options = decompress::Options {
        unpacked_size: decompress::UnpackedSize::UseProvided(Some(decompressed_len as u64)),
        ..decompress::Options::default()
    };

    lzma_decompress_with_options(&mut wrapper, &mut decompressed_data, &options).unwrap();

    Ok((version, decompressed_data))
}

impl Default for Compression {
    #[inline]
    fn default() -> Self {
        Self::None
    }
}

impl From<Compression> for u8 {
    fn from(compression: Compression) -> Self {
        match compression {
            Compression::None => 0,
            Compression::Bzip2 => 1,
            Compression::Gzip => 2,
            #[cfg(feature = "rs3")]
            Compression::Lzma => 3,
        }
    }
}

impl std::convert::TryFrom<u8> for Compression {
    type Error = CompressionUnsupported;

    fn try_from(compression: u8) -> Result<Self, Self::Error> {
        match compression {
            0 => Ok(Self::None),
            1 => Ok(Self::Bzip2),
            2 => Ok(Self::Gzip),
            #[cfg(feature = "rs3")]
            3 => Ok(Self::Lzma),
            _ => Err(CompressionUnsupported(compression)),
        }
    }
}
