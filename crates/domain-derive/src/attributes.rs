use darling::{FromMeta, util::Flag};
use syn::Meta;

pub fn parse_flag(meta: &Meta) -> Result<Flag, darling::Error> {
    if matches!(meta, Meta::Path(_)) {
        Flag::from_meta(meta)
    } else {
        Err(darling::Error::custom("expected a flag without a value or arguments").with_span(meta))
    }
}
