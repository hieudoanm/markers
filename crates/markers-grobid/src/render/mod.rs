//! TEI rendering: tree helpers, inline text, block elements,
//! bibliography parsing, and document assembly.

mod blocks;
mod convert;
mod inline;
mod references;
mod tei;

pub use convert::{convert_tei_to_md, convert_tei_xml};
pub use inline::inline;
pub use references::{ParsedReference, extract_references, parse_reference};
pub use tei::{author_name, descendant_named, descendants_named, tei_root};

pub(crate) use blocks::render_children;
#[doc(hidden)]
pub use convert::convert_doc;
