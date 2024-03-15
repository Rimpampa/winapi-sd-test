use core::{marker::PhantomData, ptr::null};

use windows_sys::core::GUID;
use windows_sys::Win32::Devices::DeviceAndDriverInstallation::{
    SetupDiDestroyDeviceInfoList, SetupDiGetClassDevsW, DIGCF_ALLCLASSES, DIGCF_DEVICEINTERFACE,
    DIGCF_PRESENT, HDEVINFO,
};
use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;

use crate::{devdata::DevInterfaceData, win};

pub struct DevInterfaceSet {
    /// Handle to the device interface set
    pub(crate) handle: HDEVINFO,
    /// Marker to tell rustc that this struct doesn't implement [`Send`]
    _marker: PhantomData<*const ()>,
}

impl DevInterfaceSet {
    fn fetch_all(additional_flags: u32) -> win::Result<Self> {
        // SAFETY: NULL is allowed for all the parameters
        // https://docs.microsoft.com/en-gb/windows/win32/api/setupapi/nf-setupapi-setupdigetclassdevsw?redirectedfrom=MSDN#parameters
        let handle = unsafe {
            SetupDiGetClassDevsW(
                null(),
                null(),
                0,
                DIGCF_ALLCLASSES | DIGCF_DEVICEINTERFACE | additional_flags,
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(win::Error::get());
        }
        Ok(Self {
            handle,
            _marker: PhantomData,
        })
    }

    /// Creates a new device set containing all the device interface classes currently present
    // TODO: expand
    pub fn fetch_present() -> win::Result<Self> {
        Self::fetch_all(DIGCF_PRESENT)
    }

    /// Returns an iterator over all the data of the device interfaces listed in the set
    ///
    /// The GUID parameter filters which device interface class will be included
    pub fn enumerate<'a>(
        &'a self,
        guid: &'a GUID,
    ) -> impl Iterator<Item = win::Result<DevInterfaceData<'a>>> {
        (0..).map_while(|i| DevInterfaceData::fetch(self, i, guid).transpose())
    }
}

impl Drop for DevInterfaceSet {
    fn drop(&mut self) {
        // SAFETY: the pointers is the same returned by `SetupDiGetClassDevsW` and it must be deleted like this according to the remarks
        // https://docs.microsoft.com/en-gb/windows/win32/api/setupapi/nf-setupapi-setupdigetclassdevsw?redirectedfrom=MSDN#remarks
        unsafe { SetupDiDestroyDeviceInfoList(self.handle) };
    }
}
