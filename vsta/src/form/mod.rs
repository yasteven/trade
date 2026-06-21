
// trade/vsta/src/form/mod.rs

// Layers: 
// L.0 base => basic controls for input / display
// L.1 core => dsta specific controlls for building forms
// L.2 form => 

// This layer is the set of forms that combine core and base and iced
// controls into a coherent view to represent the user interface with
// the multiple actions we can have with the trade/usta backend core.

// e.g., make_buzz_bomber, etc.
 
 pub mod v_buzz_make;
 pub mod v_stealth_make;
 pub mod v_sally_make;
 pub mod v_swat_make;
 pub mod v_ttai;
 pub mod v_nico; 
 
 pub use v_buzz_make::*;
 pub use v_stealth_make::*;
 pub use v_sally_make::*;
 pub use v_swat_make::*;
 pub use v_ttai::*;