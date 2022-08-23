use crate::devset::DevSet;

use winapi::um::setupapi::*;

pub struct DevInterfaceData<'a>(&'a DevSet, SP_DEVICE_INTERFACE_DATA);

