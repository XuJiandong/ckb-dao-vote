extern crate alloc;
use core::convert::TryInto;
use molecule::lazy_reader::{Cursor, Error, NUMBER_SIZE};
#[derive(Clone)]
pub struct Uint64 {
    pub cursor: Cursor,
}
impl From<Cursor> for Uint64 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Uint64 {
    pub fn len(&self) -> usize {
        8
    }
}
impl Uint64 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1usize * index, 1usize)?;
        cur.try_into()
    }
}
impl Uint64 {
    pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixed_size(8usize)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct Bytes {
    pub cursor: Cursor,
}
impl From<Cursor> for Bytes {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Bytes {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.fixvec_length()
    }
}
impl Bytes {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.fixvec_slice_by_index(1usize, index)?;
        cur.try_into()
    }
}
pub struct BytesIterator {
    cur: Bytes,
    index: usize,
    len: usize,
}
impl core::iter::Iterator for BytesIterator {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl core::iter::IntoIterator for Bytes {
    type Item = u8;
    type IntoIter = BytesIterator;
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len().unwrap();
        Self::IntoIter {
            cur: self,
            index: 0,
            len,
        }
    }
}
pub struct BytesIteratorRef<'a> {
    cur: &'a Bytes,
    index: usize,
    len: usize,
}
impl<'a> core::iter::Iterator for BytesIteratorRef<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl Bytes {
    pub fn iter(&self) -> BytesIteratorRef {
        let len = self.len().unwrap();
        BytesIteratorRef {
            cur: &self,
            index: 0,
            len,
        }
    }
}
impl Bytes {
    pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixvec(1usize)?;
        Ok(())
    }
}
pub struct BytesOpt {
    pub cursor: Cursor,
}
impl From<Cursor> for BytesOpt {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
#[derive(Clone)]
pub struct String {
    pub cursor: Cursor,
}
impl From<Cursor> for String {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl String {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.fixvec_length()
    }
}
impl String {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.fixvec_slice_by_index(1usize, index)?;
        cur.try_into()
    }
}
pub struct StringIterator {
    cur: String,
    index: usize,
    len: usize,
}
impl core::iter::Iterator for StringIterator {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl core::iter::IntoIterator for String {
    type Item = u8;
    type IntoIter = StringIterator;
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len().unwrap();
        Self::IntoIter {
            cur: self,
            index: 0,
            len,
        }
    }
}
pub struct StringIteratorRef<'a> {
    cur: &'a String,
    index: usize,
    len: usize,
}
impl<'a> core::iter::Iterator for StringIteratorRef<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl String {
    pub fn iter(&self) -> StringIteratorRef {
        let len = self.len().unwrap();
        StringIteratorRef {
            cur: &self,
            index: 0,
            len,
        }
    }
}
impl String {
    pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixvec(1usize)?;
        Ok(())
    }
}
pub struct StringOpt {
    pub cursor: Cursor,
}
impl From<Cursor> for StringOpt {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
#[derive(Clone)]
pub struct StringVec {
    pub cursor: Cursor,
}
impl From<Cursor> for StringVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl StringVec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.dynvec_length()
    }
}
impl StringVec {
    pub fn get(&self, index: usize) -> Result<Cursor, Error> {
        let cur = self.cursor.dynvec_slice_by_index(index)?;
        cur.convert_to_rawbytes()
    }
}
pub struct StringVecIterator {
    cur: StringVec,
    index: usize,
    len: usize,
}
impl core::iter::Iterator for StringVecIterator {
    type Item = Cursor;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl core::iter::IntoIterator for StringVec {
    type Item = Cursor;
    type IntoIter = StringVecIterator;
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len().unwrap();
        Self::IntoIter {
            cur: self,
            index: 0,
            len,
        }
    }
}
pub struct StringVecIteratorRef<'a> {
    cur: &'a StringVec,
    index: usize,
    len: usize,
}
impl<'a> core::iter::Iterator for StringVecIteratorRef<'a> {
    type Item = Cursor;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl StringVec {
    pub fn iter(&self) -> StringVecIteratorRef {
        let len = self.len().unwrap();
        StringVecIteratorRef {
            cur: &self,
            index: 0,
            len,
        }
    }
}
impl StringVec {
    pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
        self.cursor.verify_dynvec()?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct Byte32 {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte32 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte32 {
    pub fn len(&self) -> usize {
        32
    }
}
impl Byte32 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1usize * index, 1usize)?;
        cur.try_into()
    }
}
impl Byte32 {
    pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixed_size(32usize)?;
        Ok(())
    }
}
pub struct Byte32Opt {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte32Opt {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
#[derive(Clone)]
pub struct VoteMeta {
    pub cursor: Cursor,
}
impl From<Cursor> for VoteMeta {
    fn from(cursor: Cursor) -> Self {
        VoteMeta { cursor }
    }
}
impl VoteMeta {
    pub fn smt_root_hash(&self) -> Result<Option<[u8; 32usize]>, Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            Ok(Some(cur.try_into()?))
        }
    }
}
impl VoteMeta {
    pub fn candidates(&self) -> Result<StringVec, Error> {
        let cur = self.cursor.table_slice_by_index(1usize)?;
        Ok(cur.into())
    }
}
impl VoteMeta {
    pub fn start_time(&self) -> Result<u64, Error> {
        let cur = self.cursor.table_slice_by_index(2usize)?;
        cur.try_into()
    }
}
impl VoteMeta {
    pub fn end_time(&self) -> Result<u64, Error> {
        let cur = self.cursor.table_slice_by_index(3usize)?;
        cur.try_into()
    }
}
impl VoteMeta {
    pub fn extra(&self) -> Result<Option<Cursor>, Error> {
        let cur = self.cursor.table_slice_by_index(4usize)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            let cur = cur.convert_to_rawbytes()?;
            Ok(Some(cur.into()))
        }
    }
}
impl VoteMeta {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(5usize, compatible)?;
        let val = self.smt_root_hash()?;
        if val.is_some() {
            let val = val.unwrap();
            Byte32::from(Cursor::try_from(val)?).verify(compatible)?;
        }
        self.candidates()?.verify(compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct VoteProof {
    pub cursor: Cursor,
}
impl From<Cursor> for VoteProof {
    fn from(cursor: Cursor) -> Self {
        VoteProof { cursor }
    }
}
impl VoteProof {
    pub fn lock_script_hash(&self) -> Result<[u8; 32usize], Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        cur.try_into()
    }
}
impl VoteProof {
    pub fn smt_proof(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(1usize)?;
        cur.convert_to_rawbytes()
    }
}
impl VoteProof {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(2usize, compatible)?;
        Byte32::from(Cursor::try_from(self.lock_script_hash()?)?).verify(compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct WitnessArgs {
    pub cursor: Cursor,
}
impl From<Cursor> for WitnessArgs {
    fn from(cursor: Cursor) -> Self {
        WitnessArgs { cursor }
    }
}
impl WitnessArgs {
    pub fn lock(&self) -> Result<Option<Cursor>, Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            let cur = cur.convert_to_rawbytes()?;
            Ok(Some(cur.into()))
        }
    }
}
impl WitnessArgs {
    pub fn input_type(&self) -> Result<Option<Cursor>, Error> {
        let cur = self.cursor.table_slice_by_index(1usize)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            let cur = cur.convert_to_rawbytes()?;
            Ok(Some(cur.into()))
        }
    }
}
impl WitnessArgs {
    pub fn output_type(&self) -> Result<Option<Cursor>, Error> {
        let cur = self.cursor.table_slice_by_index(2usize)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            let cur = cur.convert_to_rawbytes()?;
            Ok(Some(cur.into()))
        }
    }
}
impl WitnessArgs {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(3usize, compatible)?;
        Ok(())
    }
}
