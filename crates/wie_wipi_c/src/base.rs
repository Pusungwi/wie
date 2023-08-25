use alloc::{boxed::Box, string::String};

use bytemuck::{Pod, Zeroable};

use wie_backend::{task::SleepFuture, Backend};
use wie_base::util::{read_null_terminated_string, ByteRead, ByteWrite};

use crate::method::{MethodBody, TypeConverter};

pub type CError = anyhow::Error;
pub type CResult<T> = anyhow::Result<T>;

pub type CMethodBody = Box<dyn MethodBody<CError>>;

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct CMemoryId(pub u32);

#[async_trait::async_trait(?Send)]
pub trait CContext: ByteRead + ByteWrite {
    fn alloc_raw(&mut self, size: u32) -> CResult<u32>;
    fn alloc(&mut self, size: u32) -> CResult<CMemoryId>;
    fn free(&mut self, memory: CMemoryId) -> CResult<()>;
    fn get_total_memory(&mut self) -> CResult<i32>;
    fn get_free_memory(&mut self) -> CResult<i32>;
    fn data_ptr(&self, memory: CMemoryId) -> CResult<u32>;
    fn register_function(&mut self, method: CMethodBody) -> CResult<u32>;
    async fn call_method(&mut self, address: u32, args: &[u32]) -> CResult<u32>;
    fn backend(&mut self) -> &mut Backend;
    fn spawn(&mut self, callback: CMethodBody) -> CResult<()>;
    fn sleep(&mut self, duration: u64) -> SleepFuture;
}

impl TypeConverter<u32> for u32 {
    fn to_rust(_: &mut dyn CContext, raw: u32) -> u32 {
        raw
    }

    fn from_rust(_: &mut dyn CContext, rust: u32) -> u32 {
        rust
    }
}

impl TypeConverter<CMemoryId> for CMemoryId {
    fn to_rust(_: &mut dyn CContext, raw: u32) -> CMemoryId {
        CMemoryId(raw)
    }

    fn from_rust(_: &mut dyn CContext, rust: CMemoryId) -> u32 {
        rust.0
    }
}

impl TypeConverter<i32> for i32 {
    fn to_rust(_: &mut dyn CContext, raw: u32) -> i32 {
        raw as _
    }

    fn from_rust(_: &mut dyn CContext, rust: i32) -> u32 {
        rust as _
    }
}

impl TypeConverter<()> for () {
    fn to_rust(_: &mut dyn CContext, _: u32) {}

    fn from_rust(_: &mut dyn CContext, _: ()) -> u32 {
        0
    }
}

impl TypeConverter<String> for String {
    fn to_rust(context: &mut dyn CContext, raw: u32) -> String {
        read_null_terminated_string(context, raw).unwrap()
    }

    fn from_rust(_: &mut dyn CContext, _: String) -> u32 {
        unimplemented!()
    }
}
