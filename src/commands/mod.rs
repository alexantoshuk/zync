pub mod status;
pub mod sync;
pub mod turn_off;
pub mod turn_on;

pub use status::{cmd_default, cmd_login, cmd_status};
pub use sync::cmd_sync;
pub use turn_off::cmd_turn_off;
pub use turn_on::cmd_turn_on;
