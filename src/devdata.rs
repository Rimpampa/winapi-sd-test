use core::marker::PhantomData;
use core::mem::{align_of, size_of, MaybeUninit};
use core::num::NonZeroUsize;
use core::ptr::{addr_of_mut, null_mut};

use utf16string::LittleEndian;

use winapi::shared::devpropdef::*;
use winapi::shared::guiddef::*;
use winapi::shared::minwindef::{DWORD, FALSE, TRUE};
use winapi::um::setupapi::*;

use crate::devprop::DevProperty;
use crate::devset::DevInterfaceSet;
use crate::win;

/// A wrapper around the [`SP_DEVICE_INTERFACE_DATA`] struct from the [`winapi`]
///
/// # Invariants
///
/// The `handle` lives as long as the ghost reference in `_marker`
///
/// The `data` is retrieved from a call to [`SetupDiEnumDeviceInterfaces()`]
/// to which the same handle as `handle` was given
pub struct DevInterfaceData<'a> {
    /// The handle to the device set from which this data was retreived
    handle: HDEVINFO,
    /// The data returned by the [`SetupDiEnumDeviceInterfaces`] function
    data: SP_DEVICE_INTERFACE_DATA,
    /// Ghost reference to the [`DevInterfaceSet`] from which this data
    /// was fetched
    ///
    /// This is needed because it binds the lifetime of a value of this type
    /// to the lifetime of the [`DevInterfaceSet`] from which the `handle`
    /// was taken from
    _marker: PhantomData<&'a DevInterfaceSet>,
}

impl<'a> DevInterfaceData<'a> {
    /// Retrieves the data of the device interface with the given [`GUID`]
    ///
    /// The GUID parameter filters which device interface class will be included
    pub fn fetch(set: &'a DevInterfaceSet, index: u32, guid: &GUID) -> win::Result<Option<Self>> {
        use SP_DEVICE_INTERFACE_DATA as Data;
        const SIZE: u32 = size_of::<Data>() as u32;

        let mut data = MaybeUninit::<Data>::uninit();
        // NOTE: This is required by `SetupDiEnumDeviceInterfaces`
        // SAFETY: thanks to `addr_of_mut!` no reference to uninitialized data is created
        unsafe { addr_of_mut!((*data.as_mut_ptr()).cbSize).write(SIZE) };

        // SAFETY:
        // https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdienumdeviceinterfaces#parameters
        // - `DeviceInfoSet = set.handle` is assured to be valid by the invariants of `DevInterfaceSet`
        // - `[optional] DeviceInfoData` can be null
        // - `InterfaceClassGuid` is a valid pointer to a `GUID`
        // - `[out] DeviceInterfaceData` is a valid pointer to an `SP_DEVICE_INTERFACE_DATA`,
        //   also this has been done:
        //   > The caller must set `DeviceInterfaceData.cbSize` to `sizeof(SP_DEVICE_INTERFACE_DATA)`
        //   > before calling this function.
        //   (the other fields can remain uninitialized)
        let result = unsafe {
            SetupDiEnumDeviceInterfaces(set.handle, null_mut(), guid, index, data.as_mut_ptr())
        };
        match result {
            TRUE => Ok(Some(Self {
                handle: set.handle,
                // SAFETY:
                // https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdienumdeviceinterfaces#parameters
                // in `[out] DeviceInterfaceData`:
                // > A pointer to a caller-allocated buffer that contains, on successful return,
                // > a completed SP_DEVICE_INTERFACE_DATA.
                // https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdienumdeviceinterfaces#return-value
                // in **Return Value**:
                // > SetupDiEnumDeviceInterfaces returns TRUE if the function completed without error.
                // Here the return value is `TRUE` so it is ok to assume that the value is initialized
                data: unsafe { data.assume_init() },
                _marker: PhantomData,
            })),
            _ => match win::Error::get() {
                win::Error::NO_MORE_ITEMS => Ok(None),
                e => Err(e),
            },
        }
    }

    /// Checks if the [`SP_DEVICE_INTERFACE_DATA::flags`](SP_DEVICE_INTERFACE_DATA) contains
    /// the given flag (or flags)
    fn is(&self, flag: DWORD) -> bool {
        (self.data.Flags & flag) == flag
    }

    /// Returns whether or not the device interface described by this data is active
    // TODO: extend explanation
    pub fn is_active(&self) -> bool {
        self.is(SPINT_ACTIVE)
    }

    /// Returns whether or not the device interface described by this data is the default for it's class
    // TODO: extend explanation
    pub fn is_default(&self) -> bool {
        self.is(SPINT_DEFAULT)
    }

    /// Returns whether or not the device interface described by this data is removed
    // TODO: what does it mean for it to be removed?
    pub fn is_removed(&self) -> bool {
        self.is(SPINT_REMOVED)
    }

    /// Returns the path of the device interface described by this data instance
    ///
    /// This path can be used in the windows API functions to refer to this device
    pub fn fetch_path(&self) -> win::Result<utf16string::WString<LittleEndian>> {
        use SP_DEVICE_INTERFACE_DETAIL_DATA_W as Data;
        const SIZE: DWORD = size_of::<Data>() as DWORD;

        let mut size = MaybeUninit::uninit();

        // SAFETY:
        // https://docs.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacedetailw#parameters
        // - `DeviceInfoSet = set.handle` is assured to be valid by the invariants of `Self`
        // - `DeviceInterfaceData` is assured to be valid by the invariants of `Self`
        // - `[optional] DeviceInterfaceDetailData` can be null
        //   > This parameter must be NULL if `DeviceInterfaceDetailSize` is zero
        // - `DeviceInterfaceDetailDataSize` can be 0
        //   > This parameter must be zero if `DeviceInterfaceDetailData` is NULL
        // - `[out] RequiredSize` is a valid pointer to an (uninitialized) mutable DWORD
        // - `[optional] DeviceInfoData` can be null
        let result = unsafe {
            SetupDiGetDeviceInterfaceDetailW(
                self.handle,
                // NOTE: for some obscure reason it wants a *mut T even tho it doesn't modify the value
                <*const _>::cast_mut(&self.data),
                null_mut(),
                0,
                size.as_mut_ptr(),
                null_mut(),
            )
        };
        // NOTE:
        // https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacedetailw#remarks
        // This is expected to fail with `ERROR_INSUFFICIENT_BUFFER` because we are requesting the size
        assert_eq!(result, FALSE);
        match win::Error::get() {
            win::Error::INSUFFICIENT_BUFFER => (), // Ok
            e => return Err(e),
        }
        // SAFETY:
        // https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacedetailw#remarks
        // > **Remarks**
        // > Get the required buffer size. Call `SetupDiGetDeviceInterfaceDetail` with a
        // > **NULL** `DeviceInterfaceDetailData` pointer, a `DeviceInterfaceDetailDataSize` of zero,
        // > and a valid `RequiredSize` variable. In response to such a call, this function returns
        // > the required buffer size at `RequiredSize` and fails with `GetLastError` returning
        // > `ERROR_INSUFFICIENT_BUFFER`.
        // All of the requirements are met so its safe to assume `size` to be initialized
        let size = unsafe { size.assume_init() };
        debug_assert!(size >= SIZE);

        let mut raw = crate::alloc_slice_with_align(
            size.try_into().ok().and_then(NonZeroUsize::new).unwrap(),
            align_of::<Data>(),
        );
        let ptr = raw.as_mut_ptr() as *mut Data;
        // NOTE: This is required by `SetupDiGetDeviceInterfaceDetailW`
        // SAFETY: thanks to `addr_of_mut!` no reference to uninitialized data is created
        unsafe { addr_of_mut!((*ptr).cbSize).write(SIZE) };

        let mut new_size = MaybeUninit::uninit();

        // SAFETY:
        // https://docs.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacedetailw#parameters
        // - `DeviceInfoSet = set.handle` is assured to be valid by the invariants of `Self`
        // - `DeviceInterfaceData` is assured to be valid by the invariants of `Self`
        // - `DeviceInterfaceDetailData` is a non-null and correctly aligned pointer
        //   to an `SP_DEVICE_INTERFACE_DETAIL_DATA_W`, and this was done:
        //   > the caller must set `DeviceInterfaceDetailData.cbSize` to
        //   > `sizeof(SP_DEVICE_INTERFACE_DETAIL_DATA)` before calling this function.
        // - `DeviceInterfaceDetailDataSize` is the size returned from the previous call
        // - `[optional] RequiredSize` can be null
        // - `[optional] DeviceInfoData` can be null
        let result = unsafe {
            SetupDiGetDeviceInterfaceDetailW(
                self.handle,
                // NOTE: for some obscure reason this wants a *mut T even tho it doesn't modify the value
                <*const _>::cast_mut(&self.data),
                ptr,
                size,
                new_size.as_mut_ptr(),
                null_mut(),
            )
        };
        if result != TRUE {
            return Err(win::Error::get());
        }
        // SAFETY:
        // https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacedetailw#parameters
        // > [out] RequiredSize
        // > [...] receives the required size of the DeviceInterfaceDetailData buffer
        // There is no indication of when this operation may not happen, but since the operation was
        // successful it's safe to assume that this was done
        // NOTE: this check is important for the following operation
        assert_eq!(size, unsafe { new_size.assume_init() });
        // SAFETY: the docs don't specify it explicitly but the data should be initialized now
        let mut vec = unsafe { raw.assume_init() }.into_vec();

        // Remove the `cbSize` from the data buffer, so that only the `DevicePath` remains
        const OFFSET: usize = core::mem::offset_of!(Data, DevicePath);
        vec.drain(..OFFSET);
        // TODO: handle the null-terminator

        // SAFETY: WinAPI functions that end with W are assured to return little-endian UTF-16 encoded strings
        // https://learn.microsoft.com/en-us/windows/win32/learnwin32/working-with-strings
        Ok(unsafe { utf16string::WString::from_utf16_unchecked(vec) })
    }

    /// Returns a list of all the properties of this device interface
    ///
    /// The value of these properties can be fetched with the [`fetch_property_value`] method
    pub fn fetch_property_keys(&self) -> win::Result<Box<[DEVPROPKEY]>> {
        let mut size = MaybeUninit::uninit();

        // SAFETY:
        // https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacepropertykeys#parameters
        // - `DeviceInfoSet = set.handle` is assured to be valid by the invariants of `Self`
        // - `DeviceInterfaceData` is assured to be valid by the invariants of `Self`
        // - `[optional] PropertyKeyArray` can be null
        // - `DeviceInterfaceDetailDataSize` must be 0
        //   > If `PropertyKeyArray` is NULL, PropertyKeyCount must be set to zero.
        // - `[out] RequiredPropertyKeyCount` is a valid pointer to an (uninitialized) mutable DWORD
        // - `Flags` must be 0
        let result = unsafe {
            SetupDiGetDeviceInterfacePropertyKeys(
                self.handle,
                // NOTE: for some obscure reason it wants a *mut T even tho it doesn't modify the value
                <*const _>::cast_mut(&self.data),
                null_mut(),
                0,
                size.as_mut_ptr(),
                0,
            )
        };
        // NOTE:
        // https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacepropertykeys#return-value
        // This is expected to fail with `ERROR_INSUFFICIENT_BUFFER` because we are requesting the size
        assert_eq!(result, FALSE);
        match win::Error::get() {
            win::Error::INSUFFICIENT_BUFFER => (), // Ok
            e => return Err(e),
        }
        // SAFETY:
        // https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacepropertykeys#parameters
        // > [out] RequiredPropertyKeyCount
        // > [...] receives the number of requested device property keys
        // There is no indication of when this operation may not happen, but it's assurred that on
        // `ERROR_INSUFFICIENT_BUFFER` this is always done (it wouldn't make sense otherwise)
        let size = unsafe { size.assume_init() };

        let mut properties = Box::new_uninit_slice(size.try_into().unwrap());
        let mut new_size = MaybeUninit::uninit();

        // SAFETY:
        // https://docs.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacepropertykeys#parameters
        // - `DeviceInfoSet = set.handle` is assured to be valid by the invariants of `Self`
        // - `DeviceInterfaceData` is assured to be valid by the invariants of `Self`
        // - `PropertyKeyArray` is the pointer to an array of `PropertyKeyCount` elemenets
        // - `PropertyKeyCount` is the value returned by the previous call
        // - `RequiredPropertyKeyCount` can be null
        // - `Flags` must be 0
        let result = unsafe {
            SetupDiGetDeviceInterfacePropertyKeys(
                self.handle,
                // NOTE: for some obscure reason this wants a *mut T even tho it doesn't modify the value
                <*const _>::cast_mut(&self.data),
                // NOTE: MaybeUninit as the same layout of the underlying type
                properties.as_mut_ptr() as _,
                size,
                new_size.as_mut_ptr(),
                0,
            )
        };
        if result != TRUE {
            return Err(win::Error::get());
        }
        // SAFETY:
        // https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacepropertykeys#parameters
        // > [out] RequiredPropertyKeyCount
        // > [...] receives the number of requested device property keys
        // There is no indication of when this operation may not happen, but since the operation was
        // successful it's safe to assume that this was done
        // NOTE: this check is important for the following operation
        assert_eq!(size, unsafe { new_size.assume_init() });
        // SAFETY: result == TRUE means that operation was successful, and being
        // `size` the exact amount of properties requested, it means that all the
        // values in `properties` where initialized.
        Ok(unsafe { properties.assume_init() })
    }

    pub fn fetch_property_value(&self, property: DEVPROPKEY) -> win::Result<DevProperty> {
        let mut prop_ty = 0;
        let mut size = 0;

        // SAFETY:
        // https://docs.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacepropertyw#parameters
        // `DeviceInfoSet`: is a valid handle because of the invariants of Self
        // `DeviceInterfaceData`: is correctly initialized because of the invariants of Self
        // `PropertyKey`: any value is allowed (if the property is wrong an error is returned)
        // `PropertyType`: a valid pointer to a `DEVPROPTYPE`
        // `PropertyBuffer`: can be null if `PropertyBufferSize` is 0
        // `PropertyBufferSize`: must be 0 if `PropertyBuffer` is null
        // `RequiredSize`: is a valid pointer to a `DWORD`
        // `Flags`: must be 0
        let result = unsafe {
            SetupDiGetDeviceInterfacePropertyW(
                self.handle,
                &mut SP_DEVICE_INTERFACE_DATA { ..self.data },
                &property,
                &mut prop_ty,
                null_mut(),
                0,
                &mut size,
                0,
            )
        };
        // NOTE: this is expected to fail because of DeviceInterfaceDetailDataSize = 0
        //       and, for the same reason, the error is expected to be `ERROR_INSUFFICIENT_BUFFER`
        assert_eq!(result, FALSE);
        match win::Error::get() {
            win::Error::INSUFFICIENT_BUFFER => (), // Ok
            e => return Err(e),
        }
        let mut raw = vec![0u8; size as usize];

        // SAFETY:
        // https://docs.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacepropertyw#parameters
        // `DeviceInfoSet`: is a valid handle because of the invariants of Self
        // `DeviceInterfaceData`: is correctly initialized because of the invariants of Self
        // `PropertyKey`: any value is allowed (if the property is wrong an error is returned)
        // `PropertyType`: a valid pointer to a `DEVPROPTYPE`
        // `PropertyBuffer`: can be null if `PropertyBufferSize` is 0
        // `PropertyBufferSize`: must be 0 if `PropertyBuffer` is null
        // `RequiredSize`: is a valid pointer to a `DWORD`
        // `Flags`: must be 0
        let result = unsafe {
            SetupDiGetDeviceInterfacePropertyW(
                self.handle,
                &mut SP_DEVICE_INTERFACE_DATA { ..self.data },
                &property,
                &mut prop_ty,
                raw.as_mut_ptr(),
                size,
                null_mut(),
                0,
            )
        };
        if result != TRUE {
            return Err(win::Error::get());
        }

        use DevProperty as P;

        let i16conv = |v: &[u8]| i16::from_ne_bytes([v[0], v[1]]);
        let u16conv = |v: &[u8]| u16::from_ne_bytes([v[0], v[1]]);
        let i32conv = |v: &[u8]| i32::from_ne_bytes(v[0..4].try_into().unwrap());
        let u32conv = |v: &[u8]| u32::from_ne_bytes(v[0..4].try_into().unwrap());
        let i64conv = |v: &[u8]| i64::from_ne_bytes(v[0..8].try_into().unwrap());
        let u64conv = |v: &[u8]| u64::from_ne_bytes(v[0..8].try_into().unwrap());
        let f32conv = |v: &[u8]| f32::from_ne_bytes(v[0..4].try_into().unwrap());
        let f64conv = |v: &[u8]| f64::from_ne_bytes(v[0..8].try_into().unwrap());
        let guidconv = |v: &[u8]| GUID {
            Data1: u32conv(&v[0..4]),
            Data2: u16conv(&v[4..6]),
            Data3: u16conv(&v[6..8]),
            Data4: v[8..16].try_into().unwrap(),
        };

        fn arrconv<T>(arr: &[u8], f: impl Fn(&[u8]) -> T) -> Vec<T> {
            arr.chunks_exact(std::mem::size_of::<T>() / 8)
                .map(f)
                .collect()
        }

        use DEVPROP_TYPEMOD_ARRAY as ARR;

        Ok(
            match (prop_ty & DEVPROP_MASK_TYPEMOD, prop_ty & DEVPROP_MASK_TYPE) {
                (0, DEVPROP_TYPE_EMPTY) => P::Empty,
                (0, DEVPROP_TYPE_NULL) => P::Null,
                (0, DEVPROP_TYPE_BOOLEAN) => P::Bool(raw[0] as i8 == DEVPROP_TRUE),
                (0, DEVPROP_TYPE_STRING) => P::String(
                    // SAFETY: transmuting between plain data types doesn't cause any damage (if correctly aligned)
                    String::from_utf16(unsafe { raw.align_to() }.1.split_last().unwrap().1)
                        .unwrap(),
                ),
                (0, DEVPROP_TYPE_SBYTE) => P::I8(raw[0] as i8),
                (0, DEVPROP_TYPE_BYTE) => P::U8(raw[0]),
                (0, DEVPROP_TYPE_INT16) => P::I16(i16conv(&raw)),
                (0, DEVPROP_TYPE_UINT16) => P::U16(u16conv(&raw)),
                (0, DEVPROP_TYPE_INT32) => P::I32(i32conv(&raw)),
                (0, DEVPROP_TYPE_UINT32) => P::U32(u32conv(&raw)),
                (0, DEVPROP_TYPE_INT64) => P::I64(i64conv(&raw)),
                (0, DEVPROP_TYPE_UINT64) => P::U64(u64conv(&raw)),
                (0, DEVPROP_TYPE_FLOAT) => P::F32(f32conv(&raw)),
                (0, DEVPROP_TYPE_DOUBLE) => P::F64(f64conv(&raw)),
                (0, DEVPROP_TYPE_BINARY) => P::Binary(raw),
                (0, DEVPROP_TYPE_GUID) => P::Guid(guidconv(&raw)),
                (ARR, DEVPROP_TYPE_BOOLEAN) => {
                    P::BoolArray(raw.into_iter().map(|v| v as i8 == DEVPROP_TRUE).collect())
                }
                (ARR, DEVPROP_TYPE_SBYTE) => P::I8Array(raw.into_iter().map(|v| v as i8).collect()),
                (ARR, DEVPROP_TYPE_BYTE) => P::U8Array(raw),
                (ARR, DEVPROP_TYPE_INT16) => P::I16Array(arrconv(&raw, i16conv)),
                (ARR, DEVPROP_TYPE_UINT16) => P::U16Array(arrconv(&raw, u16conv)),
                (ARR, DEVPROP_TYPE_INT32) => P::I32Array(arrconv(&raw, i32conv)),
                (ARR, DEVPROP_TYPE_UINT32) => P::U32Array(arrconv(&raw, u32conv)),
                (ARR, DEVPROP_TYPE_INT64) => P::I64Array(arrconv(&raw, i64conv)),
                (ARR, DEVPROP_TYPE_UINT64) => P::U64Array(arrconv(&raw, u64conv)),
                (ARR, DEVPROP_TYPE_FLOAT) => P::F32Array(arrconv(&raw, f32conv)),
                (ARR, DEVPROP_TYPE_DOUBLE) => P::F64Array(arrconv(&raw, f64conv)),
                (ARR, DEVPROP_TYPE_GUID) => P::GuidArray(arrconv(&raw, guidconv)),
                _ => DevProperty::Unsupported(prop_ty),
            },
        )
    }
}
