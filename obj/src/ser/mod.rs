use crate::Error;
use serde::ser::Error as _;
use std::io::Write;

struct Serializer<'a> {
    buf: &'a mut dyn Write,
}

impl<'a> Serializer<'a> {
    pub const fn new(buf: &'a mut dyn Write) -> Self {
        Self { buf }
    }

    pub fn end(self) -> Result<&'a mut dyn Write, Error> {
        self.buf.flush()?;
        Ok(self.buf)
    }
}

struct TupleSerializer<'a> {
    remaining: usize,
    buf: &'a mut Serializer<'a>,
}

impl<'a> serde::ser::SerializeTuple for TupleSerializer<'a> {
    type Ok = &'a mut Serializer<'a>;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.remaining -= 1;
        if self.remaining > 0 {
            write!(self.buf.buf, " ")?;
        }
        value.serialize(self.buf)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.buf)
    }
}

impl<'a> serde::ser::Serializer for Serializer<'a> {
    type Ok = &'a mut dyn Write;
    type Error = Error;
    type SerializeSeq = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        write!(self.buf, "{v}")?;
        Ok(self.buf)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        write!(self.buf, "{v}")?;
        Ok(self.buf)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        write!(self.buf, "{v}")?;
        Ok(self.buf)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        write!(self.buf, "{v}")?;
        Ok(self.buf)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        write!(self.buf, "{v}")?;
        Ok(self.buf)
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        write!(self.buf, "{v}")?;
        Ok(self.buf)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        write!(self.buf, "{v}")?;
        Ok(self.buf)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        write!(self.buf, "{v}")?;
        Ok(self.buf)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        write!(self.buf, "{v}")?;
        Ok(self.buf)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        write!(self.buf, "{v}")?;
        Ok(self.buf)
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        write!(self.buf, "{v}")?;
        Ok(self.buf)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        write!(self.buf, "{v}")?;
        Ok(self.buf)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        write!(self.buf, "{v}")?;
        Ok(self.buf)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        write!(self.buf, "{v}")?;
        Ok(self.buf)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        write!(self.buf, "{v:?}")?;
        Ok(self.buf)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.buf.write_all(v)?;
        Ok(self.buf)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.buf)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom("unsupported type"))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(format_args!("unsupported type: {name}")))
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(format_args!("unsupported type: {name}")))
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        Err(Error::custom(format_args!("unsupported type: {name}")))
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        Err(Error::custom(format_args!("unsupported type: {name}")))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {}

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::custom(format_args!("unsupported type: {name}")))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::custom(format_args!("unsupported type: {name}")))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Error::custom(format_args!("unsupported type: {name}")))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::custom(format_args!("unsupported type: {name}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[test]
    fn test0() {
        let mut buf = Vec::new();
        true.serialize(Serializer::new(&mut buf)).unwrap();
        assert_eq!(&buf, b"true");
    }

    #[test]
    fn test1() {
        let mut buf = Vec::new();
        (1, 5).serialize(Serializer::new(&mut buf)).unwrap();
        assert_eq!(&buf, b"1 5");
    }
}
