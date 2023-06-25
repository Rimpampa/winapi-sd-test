use std::marker::PhantomData;
use std::mem::{size_of, size_of_val, zeroed};
use std::ptr::null_mut;

use winapi::shared::devpropdef::*;
use winapi::shared::guiddef::*;
use winapi::shared::ntdef::{FALSE, TRUE};
use winapi::um::setupapi::*;

use crate::devprop::DevProperty;
use crate::{devset::DevInterfaceSet, win};

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
    /// A ghost reference to the device set wrapper, to take advantage of the borrow checker
    _marker: PhantomData<&'a DevInterfaceSet>,
}

impl DevInterfaceData<'_> {
    /// Returns a [`zeroed`] value of the [`SP_DEVICE_INTERFACE_DATA`] type
    ///
    /// This function also intializes the `cbSize` field with the correct size
    pub(crate) fn raw_zeroed() -> SP_DEVICE_INTERFACE_DATA {
        SP_DEVICE_INTERFACE_DATA {
            cbSize: size_of::<SP_DEVICE_INTERFACE_DATA>().try_into().unwrap(),
            // SAFETY: this struct can be zero initialized
            ..unsafe { zeroed() }
        }
    }

    /// Constructs a new wrapper around the given values
    ///
    /// # Safety
    ///
    /// The values must comply to the invariants of the wrapper: [`Self`]
    pub(crate) unsafe fn from_raw(set: &DevInterfaceSet, data: SP_DEVICE_INTERFACE_DATA) -> Self {
        Self {
            handle: set.handle,
            data,
            _marker: PhantomData,
        }
    }

    /// Returns whether or not the device interface described by this data is active
    pub fn is_active(&self) -> bool {
        (self.data.Flags & SPINT_ACTIVE) == SPINT_ACTIVE
    }

    /// Returns whether or not the device interface described by this data is the default for it's class
    pub fn is_default(&self) -> bool {
        (self.data.Flags & SPINT_DEFAULT) == SPINT_DEFAULT
    }

    /// Returns whether or not the device interface described by this data is removed
    // TODO: what does it mean for it to be removed?
    pub fn is_removed(&self) -> bool {
        (self.data.Flags & SPINT_REMOVED) == SPINT_REMOVED
    }

    /// Returns the path of the device interface described by this data instance
    pub fn fetch_path(&self) -> win::Result<Vec<u8>> {
        let mut raw_size = 0;

        // SAFETY:
        // https://docs.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacedetailw#parameters
        // `DeviceInfoSet`: is a valid handle because of the invariants of Self
        // `DeviceInterfaceData`: is correctly initialized because of the invariants of Self
        // `DeviceInterfaceDetailData`: must be null if DeviceInterfaceDetailDataSize is 0
        // `DeviceInterfaceDetailDataSize`: can be 0
        // `RequiredSize`: is a pointer to a valid, mutable, DWORD
        // `DeviceInfoData`: can always be null
        let result = unsafe {
            SetupDiGetDeviceInterfaceDetailW(
                self.handle,
                // NOTE: for some obscure reason it wants a *mut T even tho it doesn't modify the value
                &mut SP_DEVICE_INTERFACE_DATA { ..self.data },
                null_mut(),
                0,
                &mut raw_size,
                null_mut(),
            )
        };
        // NOTE: this is expected to fail because of DeviceInterfaceDetailDataSize = 0
        //       and, for the same reason, the error is expected to be `ERROR_INSUFFICIENT_BUFFER`
        assert_eq!(result, FALSE.into());
        match win::Error::get() {
            win::Error::INSUFFICIENT_BUFFER => (), // Ok
            e => return Err(e),
        }

        let raw_usize = raw_size.try_into().unwrap();
        assert!(raw_usize >= size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>());

        let mut raw = vec![0u8; raw_usize];
        // SAFETY:
        // the size check for the structure have been made right above this line
        // tramsmuting between an array of bytes and a struct it's allowed
        // TODO: other things? don't know
        let (prefix, details, _) =
            unsafe { raw.align_to_mut::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() };

        assert!(prefix.is_empty());
        assert!(!details.is_empty());

        let details = &mut details[0];
        details.cbSize = size_of_val(details).try_into().unwrap();

        // SAFETY:
        // https://docs.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacedetailw#parameters
        // `DeviceInfoSet`: is a valid handle because of the invariants of Self
        // `DeviceInterfaceData`: is correctly initialized because of the invariants of Self
        // `DeviceInterfaceDetailData`: is a valid SP_DEVICE_INTERFACE_DETAIL_DATA_W
        //                              with the required size (allocated just aboce here)
        // `DeviceInterfaceDetailDataSize`: is the required size returned from the previous call,
        //                                  and the size of the actual SP_DEVICE_INTERFACE_DETAIL_DATA_W
        // `RequiredSize`: can always be null
        // `DeviceInfoData`: can always be null
        let result = unsafe {
            SetupDiGetDeviceInterfaceDetailW(
                self.handle,
                // NOTE: for some obscure reason this wants a *mut T even tho it doesn't modify the value
                &mut SP_DEVICE_INTERFACE_DATA { ..self.data },
                details,
                raw_size,
                null_mut(),
                null_mut(),
            )
        };
        if result != TRUE.into() {
            return Err(win::Error::get());
        }
        // NOTE: from now on details can't be accessed, this is why the raw buffer can be modified
        //       without taking care of the struct layout
        let fixed_size_part_size = size_of_val(details) - size_of_val(&details.DevicePath);
        raw.copy_within(fixed_size_part_size..raw_usize, 0);
        raw.truncate(raw_usize - fixed_size_part_size);
        Ok(raw)
    }

    pub fn fetch_property_keys(&self) -> win::Result<Vec<DEVPROPKEY>> {
        let mut size = 0;

        // SAFETY:
        // https://docs.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacepropertykeys#parameters
        // `DeviceInfoSet`: is a valid handle because of the invariants of Self
        // `DeviceInterfaceData`: is correctly initialized because of the invariants of Self
        // `PropertyKeyArray`: can be null if `PropertyKeyCount` is 0
        // `PropertyKeyCount`: 0 is a valid value
        // `RequiredPropertyKeyCount`: is a valid pointer to a DWORD
        // `Flags`: must be 0
        let result = unsafe {
            SetupDiGetDeviceInterfacePropertyKeys(
                self.handle,
                &mut SP_DEVICE_INTERFACE_DATA { ..self.data },
                null_mut(),
                0,
                &mut size,
                0,
            )
        };
        // NOTE: this is expected to fail because of DeviceInterfaceDetailDataSize = 0
        //       and, for the same reason, the error is expected to be `ERROR_INSUFFICIENT_BUFFER`
        assert_eq!(result, FALSE.into());
        match win::Error::get() {
            win::Error::INSUFFICIENT_BUFFER => (), // Ok
            e => return Err(e),
        }

        // SAFETY: the DEVPROPKEY struct can be zero initialized
        let mut properties = vec![unsafe { zeroed() }; size as usize];

        // SAFETY:
        // https://docs.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacepropertykeys#parameters
        // `DeviceInfoSet`: is a valid handle because of the invariants of Self
        // `DeviceInterfaceData`: is correctly initialized because of the invariants of Self
        // `PropertyKeyArray`: is the pointer to a valid array of `PropertyKeyCount` elemenets
        // `PropertyKeyCount`: is the value returned by the previous call
        // `RequiredPropertyKeyCount`: can always be null
        // `Flags`: must be 0
        let result = unsafe {
            SetupDiGetDeviceInterfacePropertyKeys(
                self.handle,
                &mut SP_DEVICE_INTERFACE_DATA { ..self.data },
                properties.as_mut_ptr(),
                size,
                null_mut(),
                0,
            )
        };
        if result != TRUE.into() {
            return Err(win::Error::get());
        }
        Ok(properties)
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
        assert_eq!(result, FALSE.into());
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
        if result != TRUE.into() {
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
