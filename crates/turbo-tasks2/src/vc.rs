use std::marker::PhantomData;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    debug::{ValueDebugFormat, ValueDebugFormatString, ValueDebugVc},
    trace::{TraceRawVcs, TraceRawVcsContext},
    CollectiblesFuture, CollectiblesSource, FromTaskInput, RawVc, ReadRawVcFuture, ReadRef,
    ResolveTypeError, TaskInput, TraitTypeId, ValueTraitVc, ValueTypeId,
};

pub trait Value: Sized + Send + Sync + 'static {
    type TraitTypeIdsIterator: Iterator<Item = TraitTypeId>;
    type Inner: Send + Sync;

    fn convert(content: Self::Inner) -> Self;
    fn update(content: Self) -> RawVc;
    fn read(node: &RawVc) -> ReadRawVcFuture<Self, Self::Inner>;
    fn strongly_consistent_read(node: &RawVc) -> ReadRawVcFuture<Self, Self::Inner>;

    fn get_value_type_id() -> ValueTypeId;

    fn get_trait_type_ids() -> Self::TraitTypeIdsIterator;
}

// pub trait ValueTrait: Sized + Send + Sync + 'static {
//     type TraitTypeIdsIterator: Iterator<Item = TraitTypeId>;
//     type Inner: Send + Sync;

//     fn convert(content: Self::Inner) -> Self;
//     fn update(content: Self) -> RawVc;
//     fn read(node: &RawVc) -> ReadRawVcFuture<Self, Self::Inner>;
//     fn strongly_consistent_read(node: &RawVc) -> ReadRawVcFuture<Self,
// Self::Inner>;

//     fn get_value_type_id() -> ValueTypeId;

//     fn get_trait_type_ids() -> Self::TraitTypeIdsIterator;
// }

pub trait AsDyn<T>: Value
where
    T: ?Sized,
{
}

pub trait ValueInto<T>: Value {
    fn value_into(vc: Vc<Self>) -> T;
}

pub trait ValueFrom<T>: Value {
    fn value_from(t: T) -> Vc<Self>;
}

pub trait ValueIntoVc: Value {}

// TODO(alexkirsz) Figure out a way to make some kind of `cell` public in the
// defining path.
pub trait ValueCell: Value {}

// TODO(alexkirsz) This should be a try/from? Or make sure RawVc is never pub.
impl<T> From<T> for RawVc
where
    T: Value,
    T: ValueIntoVc,
{
    fn from(content: T) -> Self {
        T::update(content)
    }
}

impl<T> From<T> for Vc<T>
where
    T: Value,
    T: ValueIntoVc,
{
    fn from(content: T) -> Self {
        Self {
            node: T::update(content),
            _phantom: PhantomData,
        }
    }
}

#[derive(Copy, Debug, PartialOrd, Ord, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Vc<T>
where
    T: ?Sized,
{
    // TODO(alexkirsz) Should be private (or undocumented), but value_impl needs this to be
    // accessible...
    pub node: RawVc,
    _phantom: PhantomData<T>,
}

unsafe impl<T> Send for Vc<T> where T: ?Sized {}
unsafe impl<T> Sync for Vc<T> where T: ?Sized {}

impl<T> Clone for Vc<T> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T> Vc<T>
where
    T: Value,
    T: ValueCell,
{
    pub fn cell(content: T::Inner) -> Self {
        Self {
            node: T::update(T::convert(content)),
            _phantom: PhantomData,
        }
    }
}

impl<T> Vc<T>
where
    T: Value,
{
    pub fn as_dyn<K>(&self) -> Vc<K>
    where
        T: AsDyn<K>,
        K: ?Sized,
    {
        Vc {
            node: self.node.clone(),
            _phantom: PhantomData,
        }
    }

    /// see [turbo_tasks::RawVc::resolve]
    async fn resolve(self) -> Result<Self> {
        Ok(Self {
            node: self.node.resolve().await?,
            _phantom: PhantomData,
        })
    }

    /// see [turbo_tasks::RawVc::resolve_strongly_consistent]
    async fn resolve_strongly_consistent(self) -> Result<Self> {
        Ok(Self {
            node: self.node.resolve_strongly_consistent().await?,
            _phantom: PhantomData,
        })
    }

    /// see [turbo_tasks::RawVc::cell_local]
    async fn cell_local(self) -> Result<Self> {
        Ok(Self {
            node: self.node.cell_local().await?,
            _phantom: PhantomData,
        })
    }

    async fn resolve_from(
        super_trait_vc: impl std::convert::Into<RawVc>,
    ) -> Result<Option<Self>, ResolveTypeError> {
        let raw_vc: RawVc = super_trait_vc.into();
        let raw_vc = raw_vc.resolve_value(T::get_value_type_id()).await?;
        Ok(raw_vc.map(|raw_vc| Vc {
            node: raw_vc,
            _phantom: PhantomData,
        }))
    }

    #[must_use]
    pub fn strongly_consistent(self) -> ReadRawVcFuture<T, T::Inner> {
        T::read(&self.node)
    }
}

impl<T> std::future::IntoFuture for Vc<T>
where
    T: Value,
{
    type Output = Result<ReadRef<T, T::Inner>>;
    type IntoFuture = ReadRawVcFuture<T, T::Inner>;
    fn into_future(self) -> Self::IntoFuture {
        /// SAFETY: Types are binary identical via #[repr(transparent)]
        unsafe {
            self.node.into_transparent_read::<T, T::Inner>()
        }
    }
}

impl<T> std::future::IntoFuture for &Vc<T>
where
    T: Value,
{
    type Output = Result<ReadRef<T, T::Inner>>;
    type IntoFuture = ReadRawVcFuture<T, T::Inner>;
    fn into_future(self) -> Self::IntoFuture {
        /// SAFETY: Types are binary identical via #[repr(transparent)]
        unsafe {
            self.node.into_transparent_read::<T, T::Inner>()
        }
    }
}

impl<T> CollectiblesSource for Vc<T>
where
    T: ?Sized,
{
    fn take_collectibles<Vt: ValueTraitVc>(self) -> CollectiblesFuture<Vt> {
        self.node.take_collectibles()
    }

    fn peek_collectibles<Vt: ValueTraitVc>(self) -> CollectiblesFuture<Vt> {
        self.node.peek_collectibles()
    }
}

impl<T> FromTaskInput<'_> for Vc<T>
where
    T: ?Sized,
{
    type Error = anyhow::Error;

    fn try_from(value: &TaskInput) -> Result<Self, Self::Error> {
        Ok(Self {
            node: value.try_into()?,
            _phantom: PhantomData,
        })
    }
}

impl<T> From<RawVc> for Vc<T>
where
    T: ?Sized,
{
    fn from(node: RawVc) -> Self {
        Self {
            node,
            _phantom: PhantomData,
        }
    }
}

impl<T> From<Vc<T>> for RawVc
where
    T: ?Sized,
{
    fn from(node_ref: Vc<T>) -> Self {
        node_ref.node
    }
}

impl<T> From<&Vc<T>> for RawVc
where
    T: ?Sized,
{
    fn from(node_ref: &Vc<T>) -> Self {
        node_ref.node.clone()
    }
}

impl<T> From<Vc<T>> for TaskInput
where
    T: ?Sized,
{
    fn from(node_ref: Vc<T>) -> Self {
        node_ref.node.into()
    }
}

impl<T> From<&Vc<T>> for TaskInput
where
    T: ?Sized,
{
    fn from(node_ref: &Vc<T>) -> Self {
        node_ref.node.clone().into()
    }
}

impl<T> TraceRawVcs for Vc<T>
where
    T: ?Sized,
{
    fn trace_raw_vcs(&self, context: &mut TraceRawVcsContext) {
        TraceRawVcs::trace_raw_vcs(&self.node, context);
    }
}

impl<T> ValueDebugFormat for Vc<T>
where
    T: ?Sized,
{
    fn value_debug_format(&self) -> ValueDebugFormatString {
        ValueDebugFormatString::Async(Box::pin(async move {
            Ok(
                if let Some(value_debug) = ValueDebugVc::resolve_from(self).await? {
                    value_debug.dbg().await?.to_string()
                } else {
                    // This case means `SelfVc` does not implement `ValueDebugVc`, which is not
                    // possible if this implementation exists.
                    unreachable!()
                },
            )
        }))
    }
}
