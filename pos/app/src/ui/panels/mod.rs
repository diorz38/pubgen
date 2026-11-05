// Studio workspace panels — port of panels::{left, center, right, bottom}.

pub mod bottom;
pub mod center;
pub mod left;
pub mod right;

pub use bottom::bottom_bar;
pub use center::center;
pub use left::left;
pub use right::right;
