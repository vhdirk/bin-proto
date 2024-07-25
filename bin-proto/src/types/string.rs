use crate::{
    util, BitRead, BitWrite, ByteOrder, Error, ExternallyTaggedRead, ExternallyTaggedWrite,
    FlexibleArrayMemberRead, Result,
};

impl<Tag, Ctx> ExternallyTaggedRead<Tag, Ctx> for String
where
    Tag: TryInto<usize>,
{
    fn read(
        read: &mut dyn BitRead,
        byte_order: ByteOrder,
        ctx: &mut Ctx,
        tag: Tag,
    ) -> Result<Self> {
        let bytes: Vec<u8> = util::read_items(
            tag.try_into().map_err(|_| Error::TagConvert)?,
            read,
            byte_order,
            ctx,
        )?
        .collect();

        Ok(String::from_utf8(bytes)?)
    }
}

impl<Ctx> ExternallyTaggedWrite<Ctx> for String {
    fn write(&self, write: &mut dyn BitWrite, byte_order: ByteOrder, ctx: &mut Ctx) -> Result<()> {
        let bytes: Vec<u8> = str::bytes(self).collect();
        util::write_items::<Ctx, u8>(&bytes, write, byte_order, ctx)
    }
}

impl<Ctx> FlexibleArrayMemberRead<Ctx> for String {
    fn read(read: &mut dyn BitRead, byte_order: ByteOrder, ctx: &mut Ctx) -> Result<Self> {
        let bytes: Vec<u8> = util::read_items_to_eof(read, byte_order, ctx)?.collect();
        Ok(String::from_utf8(bytes)?)
    }
}

#[cfg(test)]
mod tests {
    test_externally_tagged!(String => [[b'a', b'b', b'c', b'd'], String::from("abcd")]);
}
