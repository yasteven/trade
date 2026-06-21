
// trade/vsta/src/lens/mod.rs

// Layers: 
// L.0 base => basic controls for input / display
// L.1 core => dsta specific controlls for building forms
// L.2 form => controls to interact with backend system
// L.3 lens => combinations of forms to 

// This layer combines the forms into their respective categories the
// main menu would  set of forms that combine core and base and iced
// controls into a coherent view to represent the user interface with
// the multiple actions we can have with the trade/usta backend core.

// e.g., v_dr_r (has all the v_buzz_make forms), v_buzz - view buzz's
//       v_head
/* Eventually we'll have
   v_saly - vieing all the sally bots
   v_stea - view all the stealth bots
   v_swat - view all the swat bots
   v_apex - heads up display of Most important info from everywhere
*/

// L.4 - main.rs

pub mod v_dr_r;
pub mod v_buzz;
pub mod v_stev;
// TODO: we need v_stev; v_steve needs to be a control that displays a colum of 100x100 .jpg button selections - just one for now: amy, just like the v_dr_r. 
// TODO: then we need a new form::v_nico/ that can send async to a struct of tokio mpsc Senders made with allm::new() -> "aimd"; aimd has async ::send_message_and_await_reply("string message", tokio_mpsc sender to send the markdown reply to<markdown String>) -> Result<(),Err> ; 
//   crate aimd also has async ::send_message_and_get_reply("string message", ) -> .md markdown in .txt; 
//   and nico needs to render a scrolalbe list of markdowns of the messages from us and the aimd
pub mod v_log_notes;

pub use v_dr_r::*;
pub use v_buzz::*;
pub use v_log_notes::*;