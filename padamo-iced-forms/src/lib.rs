pub mod traits;
pub mod basetypes;
pub mod double_entry_state;
pub mod options;
pub mod vectors;
pub mod refs;
pub mod buttons;

// pub mod unit;
pub use traits::{IcedForm,IcedFormBuffer};
pub use padamo_iced_forms_derive::IcedForm;
pub use traits::ActionOrUpdate;
pub use buttons::{Action, ActionTrait};

#[macro_export]
macro_rules! make_action {
    ($struct_name:ident,$action_enum:ident, $variant:ident) => {
        #[derive(Clone,Debug,Default)]
        struct $struct_name;

        impl padamo_iced_forms::ActionTrait<$action_enum> for $struct_name{
            fn make()->$action_enum {
                $action_enum::$variant
            }
        }
    }
}

// make_action!(A,B);
