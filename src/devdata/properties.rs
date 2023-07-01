use utf16string::WString;
use winapi::shared::devpropdef::{DEVPROPKEY, DEVPROPTYPE, DEVPROP_BOOLEAN, DEVPROP_TRUE};
use winapi::shared::minwindef::{DWORD, FALSE, TRUE};
use winapi::um::setupapi::SetupDiGetDeviceInterfacePropertyW;

use core::mem::{align_of, size_of, MaybeUninit};
use core::ptr::null_mut;

use crate::{devprop::DevProperty, win};

mod consts {
    use winapi::shared::devpropdef::*;

    pub const EMPTY: DEVPROPTYPE = DEVPROP_TYPE_EMPTY;
    pub const NULL: DEVPROPTYPE = DEVPROP_TYPE_NULL;
    pub const SBYTE: DEVPROPTYPE = DEVPROP_TYPE_SBYTE;
    pub const BYTE: DEVPROPTYPE = DEVPROP_TYPE_BYTE;
    pub const INT16: DEVPROPTYPE = DEVPROP_TYPE_INT16;
    pub const UINT16: DEVPROPTYPE = DEVPROP_TYPE_UINT16;
    pub const INT32: DEVPROPTYPE = DEVPROP_TYPE_INT32;
    pub const UINT32: DEVPROPTYPE = DEVPROP_TYPE_UINT32;
    pub const INT64: DEVPROPTYPE = DEVPROP_TYPE_INT64;
    pub const UINT64: DEVPROPTYPE = DEVPROP_TYPE_UINT64;
    pub const FLOAT: DEVPROPTYPE = DEVPROP_TYPE_FLOAT;
    pub const DOUBLE: DEVPROPTYPE = DEVPROP_TYPE_DOUBLE;
    pub const BOOLEAN: DEVPROPTYPE = DEVPROP_TYPE_BOOLEAN;
    pub const GUID: DEVPROPTYPE = DEVPROP_TYPE_GUID;
    // NOTE: there is no `BYTE_ARRAY` since `DEVPROP_TYPE_BINARY`
    // is defined as `DEVPROP_TYPEMOD_ARRAY | DEVPROP_TYPE_BYTE`
    pub const BINARY: DEVPROPTYPE = DEVPROP_TYPE_BINARY;
    pub const STRING: DEVPROPTYPE = DEVPROP_TYPE_STRING;
    pub const SBYTE_ARRAY: DEVPROPTYPE = DEVPROP_TYPEMOD_ARRAY | DEVPROP_TYPE_SBYTE;
    pub const INT16_ARRAY: DEVPROPTYPE = DEVPROP_TYPEMOD_ARRAY | DEVPROP_TYPE_INT16;
    pub const UINT16_ARRAY: DEVPROPTYPE = DEVPROP_TYPEMOD_ARRAY | DEVPROP_TYPE_UINT16;
    pub const INT32_ARRAY: DEVPROPTYPE = DEVPROP_TYPEMOD_ARRAY | DEVPROP_TYPE_INT32;
    pub const UINT32_ARRAY: DEVPROPTYPE = DEVPROP_TYPEMOD_ARRAY | DEVPROP_TYPE_UINT32;
    pub const INT64_ARRAY: DEVPROPTYPE = DEVPROP_TYPEMOD_ARRAY | DEVPROP_TYPE_INT64;
    pub const UINT64_ARRAY: DEVPROPTYPE = DEVPROP_TYPEMOD_ARRAY | DEVPROP_TYPE_UINT64;
    pub const FLOAT_ARRAY: DEVPROPTYPE = DEVPROP_TYPEMOD_ARRAY | DEVPROP_TYPE_FLOAT;
    pub const DOUBLE_ARRAY: DEVPROPTYPE = DEVPROP_TYPEMOD_ARRAY | DEVPROP_TYPE_DOUBLE;
    pub const BOOLEAN_ARRAY: DEVPROPTYPE = DEVPROP_TYPEMOD_ARRAY | DEVPROP_TYPE_BOOLEAN;
    pub const GUID_ARRAY: DEVPROPTYPE = DEVPROP_TYPEMOD_ARRAY | DEVPROP_TYPE_GUID;
}

impl super::DevInterfaceData<'_> {
    /// Returns the value of the property with the given key
    ///
    /// This function is able to retrieve only a set of known property value
    /// types, which can be seen in the discriminants of [`DevProperty`].
    ///
    /// In the case that the requested property is not present in this set,
    /// [`DevProperty::Unsupported`] is returned and the value of the property
    /// can be fetched using the [`fetch_property_info()`]
    /// method and then one of the unsafe methods
    /// [`Property::fetch()`] and [`Property::fetch_array()`]
    /// to get the actual value.
    // TODO: add panic section
    pub fn fetch_property(&self, property: &DEVPROPKEY) -> win::Result<DevProperty> {
        use DevProperty::*;
        let property = self.fetch_property_info(property)?;
        match property.ty {
            consts::EMPTY => Ok(Empty),
            consts::NULL => Ok(Null),
            // SAFETY: `DevPropkey::I8` contains a `i8 ≡ ???`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-sbyte
            // TODO: SBYTE seems like its not defined anywhere, and this↑ page has errors
            consts::SBYTE => unsafe { property.fetch() }.map(I8),
            // SAFETY: `DevPropkey::U8` contains a `u8 ≡ BYTE`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-byte
            consts::BYTE => unsafe { property.fetch() }.map(U8),
            // SAFETY: `DevPropkey::I16` contains a `i16 ≡ SHORT`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-int16
            consts::INT16 => unsafe { property.fetch() }.map(I16),
            // SAFETY: `DevPropkey::U16` contains a `u16 ≡ USHORT`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-uint16
            consts::UINT16 => unsafe { property.fetch() }.map(U16),
            // SAFETY: `DevPropkey::U32` contains a `u32 ≡ LONG`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-int32
            consts::INT32 => unsafe { property.fetch() }.map(I32),
            // SAFETY: `DevPropkey::U32` contains a `u32 ≡ ULONG`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-uint32
            consts::UINT32 => unsafe { property.fetch() }.map(U32),
            // SAFETY: `DevPropkey::U64` contains a `u64 ≡ LONG64`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-int64
            consts::INT64 => unsafe { property.fetch() }.map(I64),
            // SAFETY: `DevPropkey::U64` contains a `u64 ≡ ULONG64`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-uint64
            consts::UINT64 => unsafe { property.fetch() }.map(U64),
            // SAFETY: `DevPropkey::F32` contains a `f32 ≡ FLOAT`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-float
            consts::FLOAT => unsafe { property.fetch() }.map(F32),
            // SAFETY: `DevPropkey::F64` contains a `f64 ≡ DOUBLE`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-double
            consts::DOUBLE => unsafe { property.fetch() }.map(F64),
            // SAFETY: `DevPropkey::Guid` contains a `GUID`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-guid
            consts::GUID => unsafe { property.fetch() }.map(Guid),
            consts::BOOLEAN => {
                // SAFETY: `T` is `DEVPROP_BOOLEAN` which is the exact type of the property value:
                // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-boolean
                // NOTE: only after the fetch, the value is converted to a Rust bool
                unsafe { property.fetch() }.map(|b: DEVPROP_BOOLEAN| Bool(b == DEVPROP_TRUE))
            }
            // SAFETY: `DevPropkey::Binary` contains an array of `u8 ≡ BYTE`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-binary
            consts::BINARY => unsafe { property.fetch_array() }.map(Binary),
            // SAFETY: `DevPropkey::Binary` contains an array of `u8 ≡ BYTE`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-string
            consts::STRING => unsafe { property.fetch_array() }
                // SAFETY:
                // WinAPI functions that end with W are assured to return little-endian UTF-16 encoded strings
                // https://learn.microsoft.com/en-us/windows/win32/learnwin32/working-with-strings
                // TODO: handle the null-terminator
                .map(|bytes| unsafe { WString::from_utf16le_unchecked(bytes.into_vec()) })
                .map(String),
            // SAFETY: `DevPropkey::I8Array` contains an array of `i8 ≡ ???`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-sbyte-array
            // TODO: SBYTE seems like its not defined anywhere, and this↑ page has errors
            consts::SBYTE_ARRAY => unsafe { property.fetch_array() }.map(I8Array),
            // SAFETY: `DevPropkey::I16Array` contains an array of `i16 ≡ SHORT`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-int16
            consts::INT16_ARRAY => unsafe { property.fetch_array() }.map(I16Array),
            // SAFETY: `DevPropkey::U16Array` contains an array of `u16 ≡ USHORT`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-uint16
            consts::UINT16_ARRAY => unsafe { property.fetch_array() }.map(U16Array),
            // SAFETY: `DevPropkey::U32Array` contains an array of `u32 ≡ LONG`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-int32
            consts::INT32_ARRAY => unsafe { property.fetch_array() }.map(I32Array),
            // SAFETY: `DevPropkey::U32Array` contains an array of `u32 ≡ ULONG`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-uint32
            consts::UINT32_ARRAY => unsafe { property.fetch_array() }.map(U32Array),
            // SAFETY: `DevPropkey::U64Array` contains an array of `u64 ≡ LONG64`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-int64
            consts::INT64_ARRAY => unsafe { property.fetch_array() }.map(I64Array),
            // SAFETY: `DevPropkey::U64Array` contains an array of `u64 ≡ ULONG64`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-uint64
            consts::UINT64_ARRAY => unsafe { property.fetch_array() }.map(U64Array),
            // SAFETY: `DevPropkey::F32Array` contains an array of `f32 ≡ FLOAT`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-float
            consts::FLOAT_ARRAY => unsafe { property.fetch_array() }.map(F32Array),
            // SAFETY: `DevPropkey::F64Array` contains an array of `f64 ≡ DOUBLE`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-double
            consts::DOUBLE_ARRAY => unsafe { property.fetch_array() }.map(F64Array),
            // SAFETY: `DevPropkey::GuidArray` contains an array of `GUID`
            // which is the exact type of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-guid
            consts::GUID_ARRAY => unsafe { property.fetch_array() }.map(GuidArray),
            // SAFETY: `T` is `DEVPROP_BOOLEAN` (inferred from `winbools_to_bools`)
            // which is the exact type of the elements in the array of the property value:
            // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devprop-type-boolean
            consts::BOOLEAN_ARRAY => unsafe { property.fetch_array() }
                .map(crate::winbools_to_bools)
                .map(BoolArray),
            t => Ok(Unsupported(t)),
        }
    }

    /// Returns the [`Property`] describing the given property `key`
    // TODO: add panic section
    pub fn fetch_property_info<'a>(&'a self, key: &'a DEVPROPKEY) -> win::Result<Property<'a>> {
        let mut ty = MaybeUninit::uninit();
        let mut size = MaybeUninit::uninit();

        // SAFETY:
        // https://docs.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacepropertyw#parameters
        // - `DeviceInfoSet = set.handle` is assured to be valid by the invariants of `Self`
        // - `DeviceInterfaceData = self.data` is assured to be valid by the invariants of `Self`
        // - `PropertyKey` plain data, any value allowed
        // - `[out] PropertyType` is a valid pointer to an uninitialized `DEVPROPTYPE`
        // - `PropertyBuffer` can be null if `PropertyBufferSize` is 0
        // - `PropertyBufferSize` must be 0 if `PropertyBuffer` is null
        // - `[out] RequiredSize` is a valid pointer to an uninitialized `DWORD`
        // - `Flags` must be 0
        let result = unsafe {
            SetupDiGetDeviceInterfacePropertyW(
                self.handle,
                // NOTE: for some obscure reason this wants a *mut T even tho it doesn't modify the value
                <*const _>::cast_mut(&self.data),
                key,
                ty.as_mut_ptr(),
                null_mut(),
                0,
                size.as_mut_ptr(),
                0,
            )
        };
        // NOTE:
        // https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacepropertyw#return-value
        // This is expected to fail with `ERROR_INSUFFICIENT_BUFFER` because we are requesting the size
        // by setting `PropertyBuffer` to NULL and `PropertyBufferSize` to 0
        assert_eq!(result, FALSE);
        match win::Error::get() {
            win::Error::INSUFFICIENT_BUFFER => (), // Ok
            e => return Err(e),
        }
        // SAFETY: it is safe to assume that `size` has been initialized because:
        // https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacepropertyw#paramters
        // > `[out] RequiredSize`
        // > [...] receives the size, in bytes, of [...] the required buffer size,
        // > if the buffer is not large enough
        // last phrase practically means: "if the generated error is `ERROR_INSUFFICIENT_BUFFER`"
        let size = unsafe { size.assume_init() };
        // SAFETY: it is safe to assume that `type` has been initialized because:
        // https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacepropertyw#paramters
        // > `[out] PropertyType`
        // > [...] receives the property-data-type identifier of the requested device interface property
        // There is no indication of when this value is not populated, but it can be assumed that
        // for `ERROR_INSUFFICIENT_BUFFER` this value must be known (the property exists) and thus retreived
        let ty = unsafe { ty.assume_init() };

        Ok(Property {
            dev_data: self,
            key,
            ty,
            size,
        })
    }
}

/// A device interface property and its *metadata*
#[derive(Clone, Copy)]
pub struct Property<'a> {
    /// The [`DevInterfaceData`](super::DevInterfaceData) from which this proprety
    /// metadata was fetched
    dev_data: &'a super::DevInterfaceData<'a>,
    /// Key of this property
    key: &'a DEVPROPKEY,
    /// Type of the value of the property
    ty: DEVPROPTYPE,
    /// Size in bytes of the value of the property
    size: DWORD,
}

impl Property<'_> {
    /// Fetches the value of this property which is assumed to be a `T`
    ///
    /// # Safety
    ///
    /// The type of the [property](Property) **must** indicate that its value has the same memory
    /// layout of `T` when retrieved with [`SetupDiGetDeviceInterfacePropertyW`].
    ///
    /// More informations can be found in the
    /// [Device and Driver Installation > Reference](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/)
    // TODO: add panic section
    pub unsafe fn fetch<T: Sized>(self) -> win::Result<T> {
        assert_eq!(self.size, size_of::<T>().try_into().unwrap());

        let mut value = MaybeUninit::uninit();
        let mut ty = MaybeUninit::uninit();
        let mut size = MaybeUninit::uninit();

        // SAFETY:
        // https://docs.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacepropertyw#parameters
        // - `DeviceInfoSet = set.handle` is assured to be valid by the invariants of `Self`
        // - `DeviceInterfaceData = self.data` is assured to be valid by the invariants of `Self`
        // - `PropertyKey` plain data, any value allowed
        // - `[out] PropertyType` is a valid pointer to an uninitialized `DEVPROPTYPE`
        // - `PropertyBuffer` is a pointer to an array of at least `PropertyBufferSize` size
        // - `PropertyBufferSize` plain data, any value allowed
        // - `[out] RequiredSize` is a valid pointer to an uninitialized `DWORD`
        // - `Flags` must be 0
        let result = unsafe {
            SetupDiGetDeviceInterfacePropertyW(
                self.dev_data.handle,
                // NOTE: for some obscure reason this wants a *mut T even tho it doesn't modify the value
                <*const _>::cast_mut(&self.dev_data.data),
                self.key,
                ty.as_mut_ptr(),
                value.as_mut_ptr() as _,
                self.size,
                size.as_mut_ptr(),
                0,
            )
        };
        if result != TRUE {
            return Err(win::Error::get());
        }
        // SAFETY: it is safe to assume that `size` has been initialized because:
        // https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacepropertyw#paramters
        // > `[out] RequiredSize`
        // > [...] receives the size, in bytes, of [...] the device interface property if the property is retrieved
        // last phrase practically means: "if the return type is `TRUE`"
        // NOTE: this check is important for the following unsafe operations
        assert_eq!(self.size, unsafe { size.assume_init() });
        // SAFETY: it is safe to assume that `type` has been initialized because:
        // https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacepropertyw#paramters
        // > `[out] PropertyType`
        // > A pointer to a DEVPROPTYPE-typed variable that receives the property-data-type identifier
        // > of the requested device interface property
        // Here is implicit that this always happens when `SetupDiGetDeviceInterfacePropertyW` return `TRUE`
        // NOTE: this check is important for the following unsafe operations
        assert_eq!(self.ty, unsafe { ty.assume_init() });

        Ok(unsafe { value.assume_init() })
    }

    /// Fetches the value of this property which is assumed to be an array of `T`s
    ///
    /// # Safety
    ///
    /// The type of the [property](Property) **must** indicate that its value has the same memory
    /// layout of `[T]` when retrieved with [`SetupDiGetDeviceInterfacePropertyW`].
    ///
    /// More informations can be found in the
    /// [Device and Driver Installation > Reference](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/)
    // TODO: add panic section
    pub unsafe fn fetch_array<T: Sized>(&self) -> win::Result<Box<[T]>> {
        let size_usize = usize::try_from(self.size).unwrap();
        let len = size_usize / size_of::<T>();
        assert_eq!(size_usize % size_of::<T>(), 0);

        let mut raw =
            crate::alloc_slice_with_align(size_usize.try_into().unwrap(), align_of::<T>());
        let mut ty = MaybeUninit::uninit();
        let mut size = MaybeUninit::uninit();

        // SAFETY:
        // https://docs.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacepropertyw#parameters
        // - `DeviceInfoSet = set.handle` is assured to be valid by the invariants of `Self`
        // - `DeviceInterfaceData = self.data` is assured to be valid by the invariants of `Self`
        // - `PropertyKey` plain data, any value allowed
        // - `[out] PropertyType` is a valid pointer to an uninitialized `DEVPROPTYPE`
        // - `PropertyBuffer` is a pointer to an array of at least `PropertyBufferSize` size
        // - `PropertyBufferSize` plain data, any value allowed
        // - `[out] RequiredSize` is a valid pointer to an uninitialized `DWORD`
        // - `Flags` must be 0
        let result = unsafe {
            SetupDiGetDeviceInterfacePropertyW(
                self.dev_data.handle,
                // NOTE: for some obscure reason this wants a *mut T even tho it doesn't modify the value
                <*const _>::cast_mut(&self.dev_data.data),
                self.key,
                ty.as_mut_ptr(),
                raw.as_mut_ptr() as _,
                self.size,
                size.as_mut_ptr(),
                0,
            )
        };
        if result != TRUE {
            return Err(win::Error::get());
        }
        // SAFETY: it is safe to assume that `size` has been initialized because:
        // https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacepropertyw#paramters
        // > `[out] RequiredSize`
        // > [...] receives the size, in bytes, of [...] the device interface property if the property is retrieved
        // last phrase practically means: "if the return type is `TRUE`"
        // NOTE: this check is important for the following unsafe operations
        assert_eq!(self.ty, unsafe { ty.assume_init() });
        // SAFETY: it is safe to assume that `type` has been initialized because:
        // https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacepropertyw#paramters
        // > `[out] PropertyType`
        // > A pointer to a DEVPROPTYPE-typed variable that receives the property-data-type identifier
        // > of the requested device interface property
        // Here is implicit that this always happens when `SetupDiGetDeviceInterfacePropertyW` return `TRUE`
        // NOTE: this check is important for the following unsafe operations
        assert_eq!(self.size, unsafe { size.assume_init() });

        // SAFETY:
        // https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacepropertyw#paramters
        // > `[out] PropertyBuffer`
        // > A pointer to a buffer that receives the requested device interface property.
        // > `SetupDiGetDeviceInterfaceProperty` retrieves the requested property only if the buffer is large enough
        // > to hold all the property value data
        // Since no error was returned (i.e. `result == TRUE`) we can assume the data was initialized,
        // and since the `size` returned is the same size of the allocation, all the bytes are initialized
        let raw = unsafe { raw.assume_init() };
        let slice = Box::into_raw(raw).as_mut_ptr() as *mut T;
        // SAFETY: requirmenets derived from the **Memory Layout** section of alloc::boxed
        // https://doc.rust-lang.org/nightly/alloc/boxed/#memory-layout
        // The layout is correct as the alignment is guarateed by `alloc_slice_with_align`
        // and the length has been checked to be a multiple of the size of T
        // https://doc.rust-lang.org/reference/type-layout.html#array-layout
        Ok(unsafe { Box::from_raw(core::slice::from_raw_parts_mut(slice, len)) })
    }
}
