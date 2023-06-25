use std::iter;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::{null, null_mut};

use winapi::shared::ntdef::TRUE;
use winapi::shared::{guiddef::*, minwindef::DWORD};
use winapi::um::{handleapi::*, setupapi::*};

use crate::{devdata::DevInterfaceData, win};

pub struct DevInterfaceSet {
    /// Handle to the device interface set
    pub(crate) handle: HDEVINFO,
    /// Marker to tell rustc that this struct doesn't implement [`Send`]
    _marker: PhantomData<*const ()>,
}

impl DevInterfaceSet {
    fn fetch(additional_flags: DWORD) -> win::Result<Self> {
        // SAFETY: NULL is allowed for all the parameters
        // https://docs.microsoft.com/en-gb/windows/win32/api/setupapi/nf-setupapi-setupdigetclassdevsw?redirectedfrom=MSDN#parameters
        let handle = unsafe {
            SetupDiGetClassDevsW(
                null(),
                null(),
                null_mut(),
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
        Self::fetch(DIGCF_PRESENT)
    }

    /// Creates a new device set containing all the device interface classes
    // TODO: expand
    pub fn fetch_all() -> win::Result<Self> {
        Self::fetch(0)
    }

    /// Returns an iterator over all the data of the device interfaces listed in the set
    ///
    /// The GUID parameter filters which device interface class will be included
    pub fn enumerate(&self, guid: GUID) -> impl Iterator<Item = win::Result<DevInterfaceData<'_>>> {
        iter::zip(0.., iter::repeat(DevInterfaceData::raw_zeroed())).map_while(
            move |(i, mut data)| {
                unsafe { SetupDiEnumDeviceInterfaces(self.handle, null_mut(), &guid, i, &mut data) }
                    .eq(&TRUE.into())
                    .then(|| Some(unsafe { DevInterfaceData::from_raw(self, data) }))
                    .ok_or_else(win::Error::get)
                    .or_else(|err| {
                        (err == win::Error::NO_MORE_ITEMS)
                            .then_some(None)
                            .ok_or(err)
                    })
                    .transpose()
            },
        )
    }
}

impl Drop for DevInterfaceSet {
    fn drop(&mut self) {
        // SAFETY: the pointers is the same returned by `SetupDiGetClassDevsW` and it must be deleted like this according to the remarks
        // https://docs.microsoft.com/en-gb/windows/win32/api/setupapi/nf-setupapi-setupdigetclassdevsw?redirectedfrom=MSDN#remarks
        unsafe { SetupDiDestroyDeviceInfoList(self.handle) };
    }
}

impl Deref for DevInterfaceSet {
    type Target = HDEVINFO;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}
