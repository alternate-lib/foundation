use std::slice;

use crate::AggregateRoot;

pub trait Repository<AR: AggregateRoot + Sync>: Sync
where
    AR::Id: Send,
{
    type BackendErr: std::error::Error;

    fn batch_load(
        &self,
        ids: &[AR::Id],
    ) -> impl Future<Output = Result<Vec<AR>, RepositoryError<Self::BackendErr>>> + Send;

    fn load(
        &self,
        id: AR::Id,
    ) -> impl Future<Output = Result<Option<AR>, RepositoryError<Self::BackendErr>>> + Send {
        async move {
            self.batch_load(&[id])
                .await
                .map(|mut entities| entities.pop())
        }
    }

    fn batch_save(
        &self,
        entities: &[AR],
    ) -> impl Future<Output = Result<(), RepositoryError<Self::BackendErr>>> + Send;

    fn save(
        &self,
        entity: &AR,
    ) -> impl Future<Output = Result<(), RepositoryError<Self::BackendErr>>> + Send {
        async move { self.batch_save(slice::from_ref(entity)).await }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError<E: std::error::Error> {
    #[error("entity not found")]
    NotFound,

    #[error("duplicate entity")]
    Duplicate,

    #[error("stale entity version")]
    StaleVersion,

    #[error(transparent)]
    Backend(#[from] E),
}
