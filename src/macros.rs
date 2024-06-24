macro_rules! impl_to_from_u32 {
    ($ident:ident) => {
        impl From<$ident> for u32 {
            fn from(reg: $ident) -> Self {
                reg.0
            }
        }

        impl From<u32> for $ident {
            fn from(int: u32) -> Self {
                Self(int)
            }
        }
    };
}

macro_rules! impl_register {
    ($ident:ident, $variant:ident) => {
        impl $crate::memory::Register for $ident {
            fn get_address() -> $crate::memory::SFRAddress {
                $crate::memory::SFRAddress::$variant
            }
        }
    };
}

macro_rules! software_clearable {
        ($register_name:ident, $clear_name:ident) => {
            concat_idents::concat_idents!(get_name = _, $register_name {
                pub fn $register_name(&self) -> bool {
                    self.get_name()
                }
            });

            concat_idents::concat_idents!(set_name = _set_, $register_name,  {
                pub fn $clear_name(&mut self) {
                    self.set_name(false)
                }
            });
        };
    }

macro_rules! software_settable {
        ($register_name:ident, $set_name:ident) => {
            concat_idents::concat_idents!(get_name = _, $register_name {
                pub fn $register_name(&self) -> bool {
                    self.get_name()
                }
            });

            concat_idents::concat_idents!(set_value_name = _set_, $register_name,  {
                pub fn $set_name(&mut self) {
                    self.set_value_name(true)
                }
            });
        };
    }

pub(crate) use impl_register;
pub(crate) use impl_to_from_u32;
pub(crate) use software_clearable;
pub(crate) use software_settable;
