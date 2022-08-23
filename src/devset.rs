use std::iter;
use std::marker::PhantomData;
use std::mem::{size_of, size_of_val, zeroed};
use std::ops::Deref;
use std::ptr::{null, null_mut};

use winapi::shared::devpropdef::*;
use winapi::shared::ntdef::{FALSE, TRUE};
use winapi::shared::winerror::{ERROR_INSUFFICIENT_BUFFER, ERROR_NO_MORE_ITEMS};
use winapi::shared::{guiddef::*, minwindef::DWORD};
use winapi::um::{errhandlingapi::*, handleapi::*, setupapi::*};

pub struct DevInterfaceSet {
    handle: HDEVINFO,
}

impl DevInterfaceSet {
    fn fetch(additional_flags: DWORD) -> Result<Self, DWORD> {
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
        (handle != INVALID_HANDLE_VALUE)
            .then(|| Self { handle })
            // SAFETY: how can this be unsafe?
            .ok_or_else(|| unsafe { GetLastError() })
    }

    /// Creates a new device set containing all the device interface classes currently present
    // TODO: expand
    pub fn fetch_present() -> Result<Self, DWORD> {
        Self::fetch(DIGCF_PRESENT)
    }

    /// Creates a new device set containing all the device interface classes
    // TODO: expand
    pub fn fetch_all() -> Result<Self, DWORD> {
        Self::fetch(0)
    }

    /// Returns an iterator over all the data of the device interfaces listed in the set
    ///
    /// The GUID parameter filters which device interface class will be included
    pub fn enumerate(
        &self,
        guid: GUID,
    ) -> impl Iterator<Item = Result<DevInterfaceData<'_>, DWORD>> {
        iter::zip(0.., iter::repeat(DevInterfaceData::raw_zeroed())).map_while(
            move |(i, mut data)| {
                unsafe { SetupDiEnumDeviceInterfaces(self.handle, null_mut(), &guid, i, &mut data) }
                    .eq(&TRUE.into())
                    .then(|| Some(unsafe { DevInterfaceData::from_raw(self, data) }))
                    .ok_or_else(|| unsafe { GetLastError() })
                    .or_else(|err| (err == ERROR_NO_MORE_ITEMS).then(|| None).ok_or(err))
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
    fn raw_zeroed() -> SP_DEVICE_INTERFACE_DATA {
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
    unsafe fn from_raw(set: &DevInterfaceSet, data: SP_DEVICE_INTERFACE_DATA) -> Self {
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
    pub fn fetch_path(&self) -> Result<Vec<u8>, DWORD> {
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
        // SAFETY: how can this be unsafe?
        match unsafe { GetLastError() } {
            ERROR_INSUFFICIENT_BUFFER => (), // Ok
            err => return Err(err),
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
            // SAFETY: how can this be unsafe?
            return Err(unsafe { GetLastError() });
        }
        // NOTE: from now on details can't be accessed, this is why the raw buffer can be modified
        //       without taking care of the struct layout
        let fixed_size_part_size = size_of_val(details) - size_of_val(&details.DevicePath);
        raw.copy_within(fixed_size_part_size..raw_usize, 0);
        raw.truncate(raw_usize - fixed_size_part_size);
        Ok(raw)
    }

    pub fn fetch_property_keys(&self) -> Result<Vec<DEVPROPKEY>, DWORD> {
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
        // SAFETY: how can this be unsafe?
        match unsafe { GetLastError() } {
            ERROR_INSUFFICIENT_BUFFER => (), // Ok
            err => return Err(err),
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
            // SAFETY: how can this be unsafe?
            return Err(unsafe { GetLastError() });
        }
        Ok(properties)
    }

    pub fn fetch_property_value(&self, property: DEVPROPKEY) -> Result<DevProperty, DWORD> {
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
        // SAFETY: how can this be unsafe?
        match unsafe { GetLastError() } {
            ERROR_INSUFFICIENT_BUFFER => (), // Ok
            err => return Err(err),
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
            // SAFETY: how can this be unsafe?
            return Err(unsafe { GetLastError() });
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
        let guidconv = |v: &[u8]| GuidWrap(GUID {
            Data1: u32conv(&v[0..4]),
            Data2: u16conv(&v[4..6]),
            Data3: u16conv(&v[6..8]),
            Data4: v[8..16].try_into().unwrap(),
        });

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

#[derive(Debug)]
pub enum DevProperty {
    Empty,
    Null,
    Bool(bool),
    BoolArray(Vec<bool>),
    String(String),
    I8(i8),
    I8Array(Vec<i8>),
    U8(u8),
    U8Array(Vec<u8>),
    I16(i16),
    I16Array(Vec<i16>),
    U16(u16),
    U16Array(Vec<u16>),
    I32(i32),
    I32Array(Vec<i32>),
    U32(u32),
    U32Array(Vec<u32>),
    I64(i64),
    I64Array(Vec<i64>),
    U64(u64),
    U64Array(Vec<u64>),
    F32(f32),
    F32Array(Vec<f32>),
    F64(f64),
    F64Array(Vec<f64>),
    Binary(Vec<u8>),
    Guid(GuidWrap),
    GuidArray(Vec<GuidWrap>),
    Unsupported(DEVPROPTYPE),
}

impl std::fmt::Display for DevProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DevProperty::Empty => write!(f, "#EMPTY"),
            DevProperty::Null => write!(f, "#NULL"),
            DevProperty::Bool(v) => write!(f, "{v}"),
            DevProperty::BoolArray(v) => write!(f, "{v:?}"),
            DevProperty::String(v) => write!(f, "{v}"),
            DevProperty::I8(v) => write!(f, "{v}"),
            DevProperty::I8Array(v) => write!(f, "{v:?}"),
            DevProperty::U8(v) => write!(f, "{v}"),
            DevProperty::U8Array(v) => write!(f, "{v:?}"),
            DevProperty::I16(v) => write!(f, "{v}"),
            DevProperty::I16Array(v) => write!(f, "{v:?}"),
            DevProperty::U16(v) => write!(f, "{v}"),
            DevProperty::U16Array(v) => write!(f, "{v:?}"),
            DevProperty::I32(v) => write!(f, "{v}"),
            DevProperty::I32Array(v) => write!(f, "{v:?}"),
            DevProperty::U32(v) => write!(f, "{v}"),
            DevProperty::U32Array(v) => write!(f, "{v:?}"),
            DevProperty::I64(v) => write!(f, "{v}"),
            DevProperty::I64Array(v) => write!(f, "{v:?}"),
            DevProperty::U64(v) => write!(f, "{v}"),
            DevProperty::U64Array(v) => write!(f, "{v:?}"),
            DevProperty::F32(v) => write!(f, "{v}"),
            DevProperty::F32Array(v) => write!(f, "{v:?}"),
            DevProperty::F64(v) => write!(f, "{v}"),
            DevProperty::F64Array(v) => write!(f, "{v:?}"),
            DevProperty::Binary(v) => v.iter().try_for_each(|v| write!(f, "{v:02x}")),
            DevProperty::Guid(v) => write!(f, "{v}"),
            DevProperty::GuidArray(v) => write!(f, "{v:?}"),
            DevProperty::Unsupported(v) => write!(f, "#UNSUP{{{v}}}"),
        }
    }
}

pub struct GuidWrap(pub GUID);

impl std::fmt::Debug for GuidWrap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Guid")
            .field("Data1", &self.0.Data1)
            .field("Data2", &self.0.Data2)
            .field("Data3", &self.0.Data3)
            .field("Data4", &self.0.Data4)
            .finish()
    }
}

impl std::fmt::Display for GuidWrap {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let GUID {
            Data1: a,
            Data2: b,
            Data3: c,
            Data4: [d, e, f, g, h, i, j, k],
        } = self.0;
        write!(
            fmt,
            "{a:08x}-{b:04x}-{c:04x}-{d:02x}{e:02x}-{f:02x}{g:02x}{h:02x}{i:02x}{j:02x}{k:02x}"
        )
    }
}
