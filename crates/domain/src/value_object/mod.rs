pub use as_view::AsView;

pub mod as_view;
pub mod string;
pub mod validator;

pub trait ValueObject: AsView {
    type Raw;
    type Error: std::error::Error;

    fn validate(value: &Self::View) -> Result<(), Self::Error>;

    fn into_raw(self) -> Self::Raw;
}
