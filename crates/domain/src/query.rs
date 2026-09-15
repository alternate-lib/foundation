pub trait QueryRepository<Dto: Cursor>: Sync
where
    Dto::Data: Send,
{
    type Query: Send;
    type Key: IntoQuery<Query = Self::Query> + Send;
    type Error: std::error::Error;

    fn query(
        &self,
        params: QueryParams<Self::Query, Dto>,
    ) -> impl Future<Output = Result<Vec<Dto>, Self::Error>> + Send;

    fn query_by_key(
        &self,
        key: Self::Key,
    ) -> impl Future<Output = Result<Option<Dto>, Self::Error>> + Send {
        async move {
            self.query(QueryParams {
                base: key.into_query(),
                cursor: None,
                limit: None,
            })
            .await
            .map(|mut v| v.pop())
        }
    }

    fn paginate(
        &self,
        params: QueryParams<Self::Query, Dto>,
    ) -> impl Future<Output = Result<Paginated<Dto>, Self::Error>> + Send {
        async move {
            let has_cursor = params.cursor.is_some();

            let mut items = self
                .query(QueryParams {
                    base: params.base,
                    cursor: params.cursor,
                    limit: params.limit.map(|l| l + 1),
                })
                .await?;

            let mut cursor = None;

            if let Some(limit) = params.limit
                && has_cursor
                && items.len() > limit
            {
                items = items.into_iter().take(limit).collect();
                if let Some(last) = items.last() {
                    cursor = Some(last.to_cursor());
                }
            }

            Ok(Paginated { items, cursor })
        }
    }
}

pub trait Cursor {
    type Data;

    fn to_cursor(&self) -> Self::Data;
}

impl Cursor for () {
    type Data = ();

    fn to_cursor(&self) -> Self::Data {}
}

#[derive(Debug)]
pub struct QueryParams<P, C: Cursor = ()> {
    pub base: P,
    pub cursor: Option<C::Data>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Paginated<I: Cursor> {
    pub items: Vec<I>,
    pub cursor: Option<I::Data>,
}

impl<I: Cursor> Default for Paginated<I> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            cursor: None,
        }
    }
}

pub trait IntoQuery {
    type Query;

    fn into_query(self) -> Self::Query;
}
