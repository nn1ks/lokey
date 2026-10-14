#![no_std]
#![feature(doc_cfg)]

#[cfg(feature = "component")]
mod component;
pub mod protocol;
#[cfg(feature = "usb")]
pub mod usb;

use arrayvec::ArrayVec;
use capnp::message::{Builder, ReaderOptions, SingleSegmentAllocator, TypedReader};
use capnp::serialize::{NoAllocSliceSegments, read_message_no_alloc};
#[cfg(feature = "component")]
pub use component::OdynCore;
use lokey::util::declare_const_for_feature_group;

declare_const_for_feature_group!(
    MAX_PAYLOAD_SIZE,
    default = 32,
    [
        ("max-payload-size-64", 64),
        ("max-payload-size-128", 128),
        ("max-payload-size-256", 256),
    ]
);

pub const MAX_MESSAGE_SIZE: usize = MAX_PAYLOAD_SIZE + 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub sequence_number: u32,
    pub id: u32,
    pub payload: ArrayVec<u8, MAX_PAYLOAD_SIZE>,
}

impl Message {
    pub fn encode(&self) -> capnp::Result<ArrayVec<u8, MAX_MESSAGE_SIZE>> {
        encode::<protocol::message::Owned, MAX_MESSAGE_SIZE>(|builder| {
            builder.set_sequence_number(self.sequence_number);
            builder.set_id(self.id);
            builder.set_payload(&self.payload);
        })
    }

    pub fn decode(data: &[u8]) -> capnp::Result<Self> {
        let mut buf = [0u8; MAX_MESSAGE_SIZE];
        let reader = decode::<protocol::message::Owned>(data, &mut buf)?;
        let root = reader.get()?;
        let payload = root.get_payload()?;
        Ok(Self {
            sequence_number: root.get_sequence_number(),
            id: root.get_id(),
            payload: ArrayVec::try_from(payload)
                .map_err(|_| capnp::Error::from_kind(capnp::ErrorKind::BufferNotLargeEnough))?,
        })
    }

    pub fn error_response(sequence_number: u32, error_code: u32) -> capnp::Result<Self> {
        let response = encode::<protocol::error_response::Owned, MAX_PAYLOAD_SIZE>(|builder| {
            builder.set_code(error_code);
        })?;
        Ok(Self {
            sequence_number,
            id: protocol::message_id::ERROR_RESPONSE,
            payload: response,
        })
    }
}

struct ArrayVecWriter<const N: usize>(ArrayVec<u8, N>);

impl<const N: usize> capnp::io::Write for ArrayVecWriter<N> {
    fn write_all(&mut self, buf: &[u8]) -> capnp::Result<()> {
        self.0
            .try_extend_from_slice(buf)
            .map_err(|_| capnp::Error::from_kind(capnp::ErrorKind::BufferNotLargeEnough))
    }
}

pub fn encode<T, const N: usize>(
    build: impl for<'a> FnOnce(&mut T::Builder<'a>),
) -> capnp::Result<ArrayVec<u8, N>>
where
    T: capnp::traits::Owned,
{
    let mut segment = [0u8; N];
    let allocator = SingleSegmentAllocator::new(&mut segment);
    let mut capnp_message = Builder::new(allocator);
    {
        let mut root = capnp_message.init_root::<T::Builder<'_>>();
        build(&mut root);
    }

    let mut writer = ArrayVecWriter(ArrayVec::<u8, N>::new());
    capnp::serialize::write_message(&mut writer, &capnp_message)?;
    Ok(writer.0)
}

pub fn decode<'a, T>(
    data: &'a [u8],
    buf: &'a mut [u8],
) -> capnp::Result<TypedReader<NoAllocSliceSegments<'a>, T>>
where
    T: capnp::traits::Owned,
{
    let reader = read_message_no_alloc(data, buf, ReaderOptions::new())?;
    Ok(reader.into_typed())
}
