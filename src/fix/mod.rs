mod code_blocks;
mod doc_links;
mod headings;
mod rel_links;
mod summary;

pub use crate::fix::code_blocks::fix as code_blocks;
pub use crate::fix::doc_links::fix as doc_links;
pub use crate::fix::headings::fix as headings;
pub use crate::fix::rel_links::fix as rel_links;
pub use crate::fix::summary::fix as summary;
