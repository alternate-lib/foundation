pub trait TransactionScope {
    type Error: std::error::Error;

    fn commit(self) -> impl Future<Output = Result<(), Self::Error>>;

    fn rollback(self) -> impl Future<Output = Result<(), Self::Error>>;
}

pub trait TransactionScopeFactory {
    type Scope: TransactionScope;

    fn begin(
        &self,
    ) -> impl Future<Output = Result<Self::Scope, <Self::Scope as TransactionScope>::Error>>;
}
