use std::slice;

use crate::AggregateRoot;

pub trait Repository<AR: AggregateRoot + Sync>: Sync
where
    AR::Id: Send,
{
    type Error: std::error::Error;

    fn find_by_ids(
        &self,
        ids: &[AR::Id],
    ) -> impl Future<Output = Result<Vec<AR>, RepositoryError<Self::Error>>> + Send;

    fn find_by_id(
        &self,
        id: AR::Id,
    ) -> impl Future<Output = Result<Option<AR>, RepositoryError<Self::Error>>> + Send {
        async move {
            self.find_by_ids(&[id])
                .await
                .map(|mut entities| entities.pop())
        }
    }

    fn batch_save(
        &self,
        entities: &[AR],
    ) -> impl Future<Output = Result<(), RepositoryError<Self::Error>>> + Send;

    fn save(
        &self,
        entity: &AR,
    ) -> impl Future<Output = Result<(), RepositoryError<Self::Error>>> + Send {
        async move { self.batch_save(slice::from_ref(entity)).await }
    }

    fn delete_by_ids(
        &self,
        ids: &[AR::Id],
    ) -> impl Future<Output = Result<(), RepositoryError<Self::Error>>> + Send;

    fn delete_by_id(
        &self,
        id: AR::Id,
    ) -> impl Future<Output = Result<(), RepositoryError<Self::Error>>> + Send {
        async move { self.delete_by_ids(&[id]).await }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError<InnerErr: std::error::Error> {
    #[error("entity not found")]
    NotFound,

    #[error("duplicate entity")]
    Duplicate,

    #[error("stale entity version")]
    StaleVersion,

    #[error(transparent)]
    Other(#[from] InnerErr),
}
