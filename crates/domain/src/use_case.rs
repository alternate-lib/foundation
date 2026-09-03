pub trait UseCase<C> {
    type Response;
    type Error: std::error::Error;

    fn execute(&self, cmd: C) -> impl Future<Output = Result<Self::Response, Self::Error>>;
}
