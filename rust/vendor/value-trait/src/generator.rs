// This is mostly taken from json-rust's codegen
// as it seems to perform well and it makes sense to see
// if we can adopt the approach
//
// https://github.com/maciejhirsz/json-rust/blob/master/src/codegen.rs

macro_rules! stry {
    ($e:expr) => {
        match $e {
            ::std::result::Result::Ok(val) => val,
            ::std::result::Result::Err(err) => return ::std::result::Result::Err(err),
        }
    };
}

use std::io;
use std::io::Write;
use std::marker::PhantomData;
use std::ptr;

use crate::Value;

const QU: u8 = b'"';
const BS: u8 = b'\\';
const BB: u8 = b'b';
const TT: u8 = b't';
const NN: u8 = b'n';
const FF: u8 = b'f';
const RR: u8 = b'r';
const UU: u8 = b'u';
const __: u8 = 0;

// Look up table for characters that need escaping in a product string
pub(crate) static ESCAPED: [u8; 256] = [
    // 0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    UU, UU, UU, UU, UU, UU, UU, UU, BB, TT, NN, UU, FF, RR, UU, UU, // 0
    UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // 1
    __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
    __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
];

// taken from https://github.com/serde-rs/json/blob/4354fc3eb2232ee0ba9a9a23acce107a980a6dc0/src/ser.rs#L1790
// This is called rarely
#[inline(never)]
fn u_encode<W>(w: &mut W, byte: u8) -> io::Result<()>
where
    W: Write,
{
    static HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";
    let bytes = [
        b'\\',
        b'u',
        b'0',
        b'0',
        HEX_DIGITS[(byte >> 4) as usize],
        HEX_DIGITS[(byte & 0xF) as usize],
    ];
    w.write_all(&bytes)
}

/// Base generator trait
pub trait BaseGenerator {
    /// The writer
    type T: Write;

    /// returns teh writer
    fn get_writer(&mut self) -> &mut Self::T;

    /// Write a slice
    /// # Errors
    /// if the write fails
    #[inline(always)]
    fn write(&mut self, slice: &[u8]) -> io::Result<()> {
        self.get_writer().write_all(slice)
    }

    /// Write a char
    /// # Errors
    /// if the write fails
    #[inline(always)]
    fn write_char(&mut self, ch: u8) -> io::Result<()> {
        self.get_writer().write_all(&[ch])
    }

    /// write with minimum
    /// # Errors
    /// if the write fails
    fn write_min(&mut self, slice: &[u8], min: u8) -> io::Result<()>;

    /// writes new line
    /// # Errors
    /// if the write fails
    #[inline(always)]
    fn new_line(&mut self) -> io::Result<()> {
        Ok(())
    }

    /// indents one step
    #[inline(always)]
    fn indent(&mut self) {}

    /// dedents one step
    #[inline(always)]
    fn dedent(&mut self) {}

    /// Writes a string with escape sequences
    /// # Errors
    /// if the write fails
    #[inline(never)]
    fn write_string_complex(&mut self, string: &[u8], mut start: usize) -> io::Result<()> {
        stry!(self.write(&string[..start]));

        for (index, ch) in string.iter().enumerate().skip(start) {
            let escape = ESCAPED[*ch as usize];
            if escape > 0 {
                stry!(self.write(&string[start..index]));
                if escape == b'u' {
                    stry!(u_encode(self.get_writer(), *ch));
                } else {
                    stry!(self.write(&[b'\\', escape]));
                };
                start = index + 1;
            }
        }
        self.write(&string[start..])
    }

    /// writes a string
    /// # Errors
    /// if the write fails
    #[inline(always)]
    fn write_string(&mut self, string: &str) -> io::Result<()> {
        stry!(self.write_char(b'"'));
        stry!(self.write_string_content(string));
        self.write_char(b'"')
    }

    /// writes a string
    /// # Errors
    /// if the write fails
    #[inline(always)]
    fn write_string_content(&mut self, string: &str) -> io::Result<()> {
        let mut string = string.as_bytes();

        unsafe {
            // Looking at the table above the lower 5 bits are entirely
            // quote characters that gives us a bitmask of 0x1f for that
            // region, only quote (`"`) and backslash (`\`) are not in
            // this range.
            stry!(self.write_str_simd(&mut string));
        }
        // Legacy code to handle the remainder of the code
        for (index, ch) in string.iter().enumerate() {
            if ESCAPED[*ch as usize] > 0 {
                return self.write_string_complex(string, index);
            }
        }
        self.write(string)
    }

    /// writes a simple string (usually short and non escaped)
    /// This means we can skip the simd accelerated writing which is
    /// expensive on short strings.
    /// # Errors
    /// if the write fails
    #[inline(always)]
    fn write_simple_string(&mut self, string: &str) -> io::Result<()> {
        stry!(self.write_char(b'"'));

        stry!(self.write_simple_str_content(string));
        self.write_char(b'"')
    }

    /// writes a simple string content  (usually short and non escaped)
    /// This means we can skip the simd accelerated writing which is
    /// expensive on short strings.
    /// # Errors
    /// if the write fails
    #[inline(always)]
    fn write_simple_str_content(&mut self, string: &str) -> io::Result<()> {
        let string = string.as_bytes();
        // Legacy code to handle the remainder of the code
        for (index, ch) in string.iter().enumerate() {
            if ESCAPED[*ch as usize] > 0 {
                return self.write_string_complex(string, index);
            }
        }
        self.write(string)
    }

    /// writes a float value
    /// # Errors
    /// if the write fails
    #[inline(always)]
    fn write_float(&mut self, num: f64) -> io::Result<()> {
        let mut buffer = ryu::Buffer::new();
        let s = buffer.format_finite(num);
        self.get_writer().write_all(s.as_bytes())
    }

    /// writes an integer value
    /// # Errors
    /// if the write fails
    #[inline(always)]
    fn write_int<I: itoa::Integer>(&mut self, num: I) -> io::Result<()> {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(num);
        self.get_writer().write_all(s.as_bytes())
    }

    #[cfg(target_feature = "avx2")]
    #[inline(always)]
    #[allow(clippy::cast_possible_wrap, clippy::cast_ptr_alignment)]
    /// Writes a string with simd-acceleration
    /// # Safety
    /// This function is unsafe because it uses simd instructions
    /// # Errors
    ///  if the write fails
    unsafe fn write_str_simd(&mut self, string: &mut &[u8]) -> io::Result<()> {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::{
            __m256i, _mm256_and_si256, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8,
            _mm256_or_si256, _mm256_set1_epi8, _mm256_xor_si256,
        };
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::{
            __m256i, _mm256_and_si256, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8,
            _mm256_or_si256, _mm256_set1_epi8, _mm256_xor_si256,
        };

        let writer = self.get_writer();
        let mut idx = 0;
        let zero = _mm256_set1_epi8(0);
        let lower_quote_range = _mm256_set1_epi8(0x1F_i8);
        let quote = _mm256_set1_epi8(b'"' as i8);
        let backslash = _mm256_set1_epi8(b'\\' as i8);
        while string.len() - idx >= 32 {
            // Load 32 bytes of data;
            let data: __m256i = _mm256_loadu_si256(string.as_ptr().add(idx).cast::<__m256i>());
            // Test the data against being backslash and quote.
            let bs_or_quote = _mm256_or_si256(
                _mm256_cmpeq_epi8(data, backslash),
                _mm256_cmpeq_epi8(data, quote),
            );
            // Now mask the data with the quote range (0x1F).
            let in_quote_range = _mm256_and_si256(data, lower_quote_range);
            // then test of the data is unchanged. aka: xor it with the
            // Any field that was inside the quote range it will be zero
            // now.
            let is_unchanged = _mm256_xor_si256(data, in_quote_range);
            let in_range = _mm256_cmpeq_epi8(is_unchanged, zero);
            let quote_bits = _mm256_movemask_epi8(_mm256_or_si256(bs_or_quote, in_range));
            if quote_bits == 0 {
                idx += 32;
            } else {
                let quote_dist = quote_bits.trailing_zeros() as usize;
                stry!(writer.write_all(string.get_unchecked(0..idx + quote_dist)));

                let ch = string[idx + quote_dist];
                match ESCAPED[ch as usize] {
                    b'u' => stry!(u_encode(writer, ch)),
                    escape => stry!(writer.write_all(&[b'\\', escape])),
                };

                *string = string.get_unchecked(idx + quote_dist + 1..);
                idx = 0;
            }
        }
        stry!(writer.write_all(&string[0..idx]));
        *string = string.get_unchecked(idx..);
        Ok(())
    }

    #[cfg(all(not(target_feature = "avx2"), target_feature = "sse2"))]
    #[inline(always)]
    #[allow(clippy::cast_possible_wrap, clippy::cast_ptr_alignment)]
    /// Writes a string with simd-acceleration
    /// # Safety
    /// This function is unsafe because it uses simd instructions
    /// # Errors
    ///  if the write fails
    unsafe fn write_str_simd(&mut self, string: &mut &[u8]) -> io::Result<()> {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::{
            __m128i, _mm_and_si128, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8,
            _mm_or_si128, _mm_set1_epi8, _mm_xor_si128,
        };
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::{
            __m128i, _mm_and_si128, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8,
            _mm_or_si128, _mm_set1_epi8, _mm_xor_si128,
        };

        let writer = self.get_writer();
        let mut idx = 0;
        let zero = _mm_set1_epi8(0);
        let lower_quote_range = _mm_set1_epi8(0x1F_i8);
        let quote = _mm_set1_epi8(b'"' as i8);
        let backslash = _mm_set1_epi8(b'\\' as i8);
        while string.len() - idx > 16 {
            // Load 16 bytes of data;
            let data: __m128i = _mm_loadu_si128(string.as_ptr().add(idx).cast::<__m128i>());
            // Test the data against being backslash and quote.
            let bs_or_quote =
                _mm_or_si128(_mm_cmpeq_epi8(data, backslash), _mm_cmpeq_epi8(data, quote));
            // Now mask the data with the quote range (0x1F).
            let in_quote_range = _mm_and_si128(data, lower_quote_range);
            // then test of the data is unchanged. aka: xor it with the
            // Any field that was inside the quote range it will be zero
            // now.
            let is_unchanged = _mm_xor_si128(data, in_quote_range);
            let in_range = _mm_cmpeq_epi8(is_unchanged, zero);
            let quote_bits = _mm_movemask_epi8(_mm_or_si128(bs_or_quote, in_range));
            if quote_bits == 0 {
                idx += 16;
            } else {
                let quote_dist = quote_bits.trailing_zeros() as usize;
                stry!(writer.write_all(&string[0..idx + quote_dist]));

                let ch = string[idx + quote_dist];
                match ESCAPED[ch as usize] {
                    b'u' => stry!(u_encode(writer, ch)),
                    escape => stry!(writer.write_all(&[b'\\', escape])),
                }

                *string = &string[idx + quote_dist + 1..];
                idx = 0;
            }
        }
        stry!(writer.write_all(&string[0..idx]));
        *string = &string[idx..];
        Ok(())
    }

    #[cfg(all(
        not(target_arch = "aarch64"),
        not(all(target_arch = "wasm32", target_feature = "simd128")),
        not(any(target_feature = "avx2", target_feature = "sse2")),
        feature = "allow-non-simd"
    ))]
    #[inline(always)]
    /// Writes a string with simd-acceleration (not really, as the architecture doesn't support it)
    /// # Safety
    /// This function is unsafe because it uses simd instructions
    /// # Errors
    ///  if the write fails
    unsafe fn write_str_simd(&mut self, string: &mut &[u8]) -> io::Result<()> {
        self.write_simple_string(std::str::from_utf8_unchecked(string))
    }

    #[cfg(all(
        not(target_arch = "aarch64"),
        not(all(target_arch = "wasm32", target_feature = "simd128")),
        not(any(target_feature = "avx2", target_feature = "sse2")),
        not(feature = "allow-non-simd")
    ))]
    unsafe fn write_str_simd(&mut self, string: &mut &[u8]) -> io::Result<()> {
        compile_error!("write_str_simd not supported on the current architecture")
    }
    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    /// Writes a string with simd-acceleration
    /// # Safety
    /// This function is unsafe because it uses simd instructions
    /// # Errors
    ///  if the write fails
    unsafe fn write_str_simd(&mut self, string: &mut &[u8]) -> io::Result<()> {
        use std::arch::aarch64::{
            uint8x16_t, vandq_u8, vceqq_u8, vdupq_n_u8, veorq_u8, vgetq_lane_u16, vld1q_u8,
            vorrq_u8, vpaddq_u8, vreinterpretq_u16_u8,
        };
        use std::mem;

        #[inline(always)]
        unsafe fn bit_mask() -> uint8x16_t {
            mem::transmute([
                0x01_u8, 0x02, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80, 0x01, 0x02, 0x4, 0x8, 0x10, 0x20,
                0x40, 0x80,
            ])
        }

        #[inline(always)]
        unsafe fn neon_movemask(input: uint8x16_t) -> u16 {
            let simd_input: uint8x16_t = vandq_u8(input, bit_mask());
            let tmp: uint8x16_t = vpaddq_u8(simd_input, simd_input);
            let tmp = vpaddq_u8(tmp, tmp);
            let tmp = vpaddq_u8(tmp, tmp);

            vgetq_lane_u16(vreinterpretq_u16_u8(tmp), 0)
        }

        let writer = self.get_writer();
        // The case where we have a 16+ byte block
        // we repeate the same logic as above but with
        // only 16 bytes
        let mut idx = 0;
        let zero = vdupq_n_u8(0);
        let lower_quote_range = vdupq_n_u8(0x1F);
        let quote = vdupq_n_u8(b'"');
        let backslash = vdupq_n_u8(b'\\');
        while string.len() - idx > 16 {
            // Load 16 bytes of data;
            let data: uint8x16_t = vld1q_u8(string.as_ptr().add(idx));
            // Test the data against being backslash and quote.
            let bs_or_quote = vorrq_u8(vceqq_u8(data, backslash), vceqq_u8(data, quote));
            // Now mask the data with the quote range (0x1F).
            let in_quote_range = vandq_u8(data, lower_quote_range);
            // then test of the data is unchanged. aka: xor it with the
            // Any field that was inside the quote range it will be zero
            // now.
            let is_unchanged = veorq_u8(data, in_quote_range);
            let in_range = vceqq_u8(is_unchanged, zero);
            let quote_bits = neon_movemask(vorrq_u8(bs_or_quote, in_range));
            if quote_bits == 0 {
                idx += 16;
            } else {
                let quote_dist = quote_bits.trailing_zeros() as usize;
                stry!(writer.write_all(&string[0..idx + quote_dist]));
                let ch = string[idx + quote_dist];
                match ESCAPED[ch as usize] {
                    b'u' => stry!(u_encode(writer, ch)),
                    escape => stry!(writer.write_all(&[b'\\', escape])),
                }

                *string = &string[idx + quote_dist + 1..];
                idx = 0;
            }
        }
        stry!(writer.write_all(&string[0..idx]));
        *string = &string[idx..];
        Ok(())
    }

    #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
    #[inline(always)]
    /// Writes a string with simd-acceleration
    /// # Safety
    /// This function is unsafe because it uses simd instructions
    /// # Errors
    ///  if the write fails
    unsafe fn write_str_simd(&mut self, string: &mut &[u8]) -> io::Result<()> {
        let writer = self.get_writer();
        use std::arch::wasm32::{
            u8x16_bitmask, u8x16_eq, u8x16_splat, v128, v128_and, v128_load, v128_or, v128_xor,
        };

        // The case where we have a 16+ byte block
        // we repeat the same logic as above but with
        // only 16 bytes
        let mut idx = 0;
        let zero = u8x16_splat(0);
        let lower_quote_range = u8x16_splat(0x1F);
        let quote = u8x16_splat(b'"');
        let backslash = u8x16_splat(b'\\');
        while string.len() - idx > 16 {
            // Load 16 bytes of data;
            let data = v128_load(string.as_ptr().add(idx).cast::<v128>());
            // Test the data against being backslash and quote.
            let bs_or_quote = v128_or(u8x16_eq(data, backslash), u8x16_eq(data, quote));
            // Now mask the data with the quote range (0x1F).
            let in_quote_range = v128_and(data, lower_quote_range);
            // then test of the data is unchanged. aka: xor it with the
            // Any field that was inside the quote range it will be zero
            // now.
            let is_unchanged = v128_xor(data, in_quote_range);
            let in_range = u8x16_eq(is_unchanged, zero);
            let quote_bits = u8x16_bitmask(v128_or(bs_or_quote, in_range));
            if quote_bits == 0 {
                idx += 16;
            } else {
                let quote_dist = quote_bits.trailing_zeros() as usize;
                stry!(writer.write_all(&string[0..idx + quote_dist]));
                let ch = string[idx + quote_dist];
                match ESCAPED[ch as usize] {
                    b'u' => stry!(u_encode(writer, ch)),
                    escape => stry!(writer.write_all(&[b'\\', escape])),
                }

                *string = &string[idx + quote_dist + 1..];
                idx = 0;
            }
        }
        stry!(writer.write_all(&string[0..idx]));
        *string = &string[idx..];
        Ok(())
    }
}

///  Simple dump Generator
pub struct DumpGenerator<VT: Value> {
    _value: PhantomData<VT>,
    code: Vec<u8>,
}

impl<VT: Value> Default for DumpGenerator<VT> {
    fn default() -> Self {
        Self {
            _value: PhantomData,
            code: Vec::with_capacity(1024),
        }
    }
}

impl<VT: Value> DumpGenerator<VT> {
    /// Creates a new generator
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the data as a String
    #[must_use]
    pub fn consume(self) -> String {
        // Original strings were unicode, numbers are all ASCII,
        // therefore this is safe.
        unsafe { String::from_utf8_unchecked(self.code) }
    }
}

impl<VT: Value> BaseGenerator for DumpGenerator<VT> {
    type T = Vec<u8>;

    #[inline(always)]
    fn write(&mut self, slice: &[u8]) -> io::Result<()> {
        extend_from_slice(&mut self.code, slice);
        Ok(())
    }
    #[inline(always)]
    fn write_char(&mut self, ch: u8) -> io::Result<()> {
        self.code.push(ch);
        Ok(())
    }

    #[inline(always)]
    fn get_writer(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }

    #[inline(always)]
    fn write_min(&mut self, _: &[u8], min: u8) -> io::Result<()> {
        self.code.push(min);
        Ok(())
    }
}

/// Pretty Generator
pub struct PrettyGenerator<V: Value> {
    code: Vec<u8>,
    dent: u16,
    spaces_per_indent: u16,
    _value: PhantomData<V>,
}

impl<V: Value> PrettyGenerator<V> {
    /// Creates a new pretty priting generator
    #[must_use]
    pub fn new(spaces: u16) -> Self {
        Self {
            code: Vec::with_capacity(1024),
            dent: 0,
            spaces_per_indent: spaces,
            _value: PhantomData,
        }
    }

    /// Returns the data as a String
    #[must_use]
    pub fn consume(self) -> String {
        unsafe { String::from_utf8_unchecked(self.code) }
    }
}

impl<V: Value> BaseGenerator for PrettyGenerator<V> {
    type T = Vec<u8>;
    #[inline(always)]
    fn write(&mut self, slice: &[u8]) -> io::Result<()> {
        extend_from_slice(&mut self.code, slice);
        Ok(())
    }

    #[inline(always)]
    fn write_char(&mut self, ch: u8) -> io::Result<()> {
        self.code.push(ch);
        Ok(())
    }

    #[inline(always)]
    fn get_writer(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }

    #[inline(always)]
    fn write_min(&mut self, slice: &[u8], _: u8) -> io::Result<()> {
        extend_from_slice(&mut self.code, slice);
        Ok(())
    }

    fn new_line(&mut self) -> io::Result<()> {
        self.code.push(b'\n');
        self.code.resize(
            self.code.len() + (self.dent * self.spaces_per_indent) as usize,
            b' ',
        );
        Ok(())
    }

    fn indent(&mut self) {
        self.dent += 1;
    }

    fn dedent(&mut self) {
        self.dent -= 1;
    }
}

/// Writer Generator
pub struct WriterGenerator<'w, W: 'w + Write, V: Value> {
    writer: &'w mut W,
    _value: PhantomData<V>,
}

impl<'w, W, V> WriterGenerator<'w, W, V>
where
    W: 'w + Write,
    V: Value,
{
    /// Creates a new generator
    pub fn new(writer: &'w mut W) -> Self {
        WriterGenerator {
            writer,
            _value: PhantomData,
        }
    }
}

impl<'w, W, V> BaseGenerator for WriterGenerator<'w, W, V>
where
    W: Write,
    V: Value,
{
    type T = W;

    #[inline(always)]
    fn get_writer(&mut self) -> &mut W {
        self.writer
    }

    #[inline(always)]
    fn write_min(&mut self, _: &[u8], min: u8) -> io::Result<()> {
        self.writer.write_all(&[min])
    }
}

/// Pretty Writer Generator

pub struct PrettyWriterGenerator<'w, W, V>
where
    W: 'w + Write,
    V: Value,
{
    writer: &'w mut W,
    dent: u16,
    spaces_per_indent: u16,
    _value: PhantomData<V>,
}

impl<'w, W, V> PrettyWriterGenerator<'w, W, V>
where
    W: 'w + Write,
    V: Value,
{
    /// Creates a new generator
    pub fn new(writer: &'w mut W, spaces_per_indent: u16) -> Self {
        PrettyWriterGenerator {
            writer,
            dent: 0,
            spaces_per_indent,
            _value: PhantomData,
        }
    }
}

impl<'w, W, V> BaseGenerator for PrettyWriterGenerator<'w, W, V>
where
    W: Write,
    V: Value,
{
    type T = W;

    #[inline(always)]
    fn get_writer(&mut self) -> &mut W {
        self.writer
    }

    #[inline(always)]
    fn write_min(&mut self, slice: &[u8], _: u8) -> io::Result<()> {
        self.writer.write_all(slice)
    }

    fn new_line(&mut self) -> io::Result<()> {
        stry!(self.write_char(b'\n'));
        for _ in 0..(self.dent * self.spaces_per_indent) {
            stry!(self.write_char(b' '));
        }
        Ok(())
    }

    fn indent(&mut self) {
        self.dent += 1;
    }

    fn dedent(&mut self) {
        self.dent -= 1;
    }
}

// From: https://github.com/dtolnay/fastwrite/blob/master/src/lib.rs#L68
//
// LLVM is not able to lower `Vec::extend_from_slice` into a memcpy, so this
// helps eke out that last bit of performance.
#[inline(always)]
#[allow(clippy::uninit_vec)]
pub(crate) fn extend_from_slice(dst: &mut Vec<u8>, src: &[u8]) {
    let dst_len = dst.len();
    let src_len = src.len();

    dst.reserve(src_len);

    unsafe {
        // We would have failed if `reserve` overflowed
        dst.set_len(dst_len + src_len);

        ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr().add(dst_len), src_len);
    }
}
