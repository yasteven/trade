

// trade/vsta/src/core/mod.rs
// Layers: 
// L.0 base => basic controls for input / display
// L.1 core => dsta specific controls for building forms
// This layer is the trade data (dsta) visuals of basic components
// that are found inside as part of the major components the user
// manipulates in order to interact with the system.
//
// bot-agnostic
pub mod v_bot_common_info;
pub mod v_bot_action_time;
pub mod v_haggle_limits;
pub mod v_haggle_method;
pub mod v_haggle_action;
pub mod v_option_chain;  
pub mod v_emerald_collection;  

// bot-specific
pub mod v_buzz_abort_time;
pub mod v_stea_abort_time;
pub mod v_sally_enter_way;

pub mod v_swat_preset;
pub mod v_sally_preset; 


pub use v_bot_common_info::*;
pub use v_bot_action_time::*;
pub use v_haggle_limits::*;
pub use v_haggle_method::*;
pub use v_haggle_action::*;
pub use v_option_chain::*;  
pub use v_emerald_collection::*;  

pub use v_buzz_abort_time::*;
pub use v_stea_abort_time::*;
pub use v_sally_enter_way::*;


pub use v_swat_preset::*;
pub use v_sally_preset::*; 