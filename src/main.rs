use winapi::shared::devpkey::*;
use winapi::shared::devpropdef::*;
use winapi::shared::guiddef::GUID;
use winapi::um::winioctl::*;

use devprop::DevProperty;
use devset::DevInterfaceSet;
use sd_formatter::devprop;
use sd_formatter::devset;

fn main() {
    let devset = DevInterfaceSet::fetch_present().unwrap();

    for (name, guid) in GUIDS {
        println!("GUID: [{}] {name}", DevProperty::Guid(guid));
        for data in devset.enumerate(&guid).map(Result::unwrap) {
            let path = data.fetch_path().unwrap().to_utf8();

            let active = if data.is_active() { "+" } else { "-" };
            let default = if data.is_default() { "#" } else { " " };
            let removed = if data.is_removed() { "!" } else { " " };

            match data.fetch_property_value(DEVPKEY_Storage_Removable_Media) {
                Ok(DevProperty::Bool(true)) => (),
                _ => continue,
            }

            // match data.fetch_property_value(DEVPKEY_Storage_System_Critical) {
            //     Ok(DevProperty::Bool(true)) => (),
            //     _ => continue,
            // }

            println!("{removed}{default}{active}PATH: {path}");

            for prop in data.fetch_property_keys().unwrap().into_vec() {
                let name = DEVPKEYS
                    .into_iter()
                    .find_map(|(name, key)| IsEqualDevPropKey(&key, &prop).then_some(name));
                let val = data.fetch_property_value(prop).unwrap();
                match name {
                    Some(name) => println!("    PROP: {name} = {val}"),
                    None => println!(
                        "    PROP: {}::{} = {val}",
                        DevProperty::Guid(prop.fmtid),
                        prop.pid
                    ),
                }
            }
        }
    }
}

macro_rules! with_name {
    ($i:ident) => {
        (stringify!($i), $i)
    };

    ([$($i:ident),* $(,)?]) => {
        [ $( with_name!($i) ),* ]
    }
}

const GUIDS: [(&str, GUID); 18] = with_name!([
    GUID_DEVINTERFACE_DISK,
    GUID_DEVINTERFACE_CDROM,
    GUID_DEVINTERFACE_PARTITION,
    GUID_DEVINTERFACE_TAPE,
    GUID_DEVINTERFACE_WRITEONCEDISK,
    GUID_DEVINTERFACE_VOLUME,
    GUID_DEVINTERFACE_MEDIUMCHANGER,
    GUID_DEVINTERFACE_FLOPPY,
    GUID_DEVINTERFACE_CDCHANGER,
    GUID_DEVINTERFACE_STORAGEPORT,
    GUID_DEVINTERFACE_VMLUN,
    GUID_DEVINTERFACE_SES,
    GUID_DEVINTERFACE_SERVICE_VOLUME,
    GUID_DEVINTERFACE_HIDDEN_VOLUME,
    GUID_DEVINTERFACE_UNIFIED_ACCESS_RPMB,
    GUID_DEVINTERFACE_SCM_PHYSICAL_DEVICE,
    GUID_DEVINTERFACE_COMPORT,
    GUID_DEVINTERFACE_SERENUM_BUS_ENUMERATOR,
]);

#[allow(non_upper_case_globals)]
const DEVPKEY_Storage_Disk_Number: DEVPROPKEY = DEVPROPKEY {
    fmtid: GUID {
        Data1: 0x4d1ebee8,
        Data2: 0x0803,
        Data3: 0x4774,
        Data4: [0x98, 0x42, 0xb7, 0x7d, 0xb5, 0x02, 0x65, 0xe9],
    },
    pid: 5,
};

#[allow(non_upper_case_globals)]
const DEVPKEY_Storage_Partition_Number: DEVPROPKEY = DEVPROPKEY {
    fmtid: GUID {
        Data1: 0x4d1ebee8,
        Data2: 0x0803,
        Data3: 0x4774,
        Data4: [0x98, 0x42, 0xb7, 0x7d, 0xb5, 0x02, 0x65, 0xe9],
    },
    pid: 6,
};

#[allow(non_upper_case_globals)]
const DEVPKEY_Storage_Mbr_Type: DEVPROPKEY = DEVPROPKEY {
    fmtid: GUID {
        Data1: 0x4d1ebee8,
        Data2: 0x0803,
        Data3: 0x4774,
        Data4: [0x98, 0x42, 0xb7, 0x7d, 0xb5, 0x02, 0x65, 0xe9],
    },
    pid: 7,
};

#[allow(non_upper_case_globals)]
const DEVPKEY_Storage_Gpt_Type: DEVPROPKEY = DEVPROPKEY {
    fmtid: GUID {
        Data1: 0x4d1ebee8,
        Data2: 0x0803,
        Data3: 0x4774,
        Data4: [0x98, 0x42, 0xb7, 0x7d, 0xb5, 0x02, 0x65, 0xe9],
    },
    pid: 8,
};

#[allow(non_upper_case_globals)]
const DEVPKEY_Storage_Gpt_Name: DEVPROPKEY = DEVPROPKEY {
    fmtid: GUID {
        Data1: 0x4d1ebee8,
        Data2: 0x0803,
        Data3: 0x4774,
        Data4: [0x98, 0x42, 0xb7, 0x7d, 0xb5, 0x02, 0x65, 0xe9],
    },
    pid: 9,
};

const DEVPKEYS: [(&str, DEVPROPKEY); 197] = with_name!([
    DEVPKEY_NAME,
    DEVPKEY_Device_DeviceDesc,
    DEVPKEY_Device_HardwareIds,
    DEVPKEY_Device_CompatibleIds,
    DEVPKEY_Device_Service,
    DEVPKEY_Device_Class,
    DEVPKEY_Device_ClassGuid,
    DEVPKEY_Device_Driver,
    DEVPKEY_Device_ConfigFlags,
    DEVPKEY_Device_Manufacturer,
    DEVPKEY_Device_FriendlyName,
    DEVPKEY_Device_LocationInfo,
    DEVPKEY_Device_PDOName,
    DEVPKEY_Device_Capabilities,
    DEVPKEY_Device_UINumber,
    DEVPKEY_Device_UpperFilters,
    DEVPKEY_Device_LowerFilters,
    DEVPKEY_Device_BusTypeGuid,
    DEVPKEY_Device_LegacyBusType,
    DEVPKEY_Device_BusNumber,
    DEVPKEY_Device_EnumeratorName,
    DEVPKEY_Device_Security,
    DEVPKEY_Device_SecuritySDS,
    DEVPKEY_Device_DevType,
    DEVPKEY_Device_Exclusive,
    DEVPKEY_Device_Characteristics,
    DEVPKEY_Device_Address,
    DEVPKEY_Device_UINumberDescFormat,
    DEVPKEY_Device_PowerData,
    DEVPKEY_Device_RemovalPolicy,
    DEVPKEY_Device_RemovalPolicyDefault,
    DEVPKEY_Device_RemovalPolicyOverride,
    DEVPKEY_Device_InstallState,
    DEVPKEY_Device_LocationPaths,
    DEVPKEY_Device_BaseContainerId,
    DEVPKEY_Device_InstanceId,
    DEVPKEY_Device_DevNodeStatus,
    DEVPKEY_Device_ProblemCode,
    DEVPKEY_Device_EjectionRelations,
    DEVPKEY_Device_RemovalRelations,
    DEVPKEY_Device_PowerRelations,
    DEVPKEY_Device_BusRelations,
    DEVPKEY_Device_Parent,
    DEVPKEY_Device_Children,
    DEVPKEY_Device_Siblings,
    DEVPKEY_Device_TransportRelations,
    DEVPKEY_Device_ProblemStatus,
    DEVPKEY_Device_Reported,
    DEVPKEY_Device_Legacy,
    DEVPKEY_Device_ContainerId,
    DEVPKEY_Device_InLocalMachineContainer,
    DEVPKEY_Device_Model,
    DEVPKEY_Device_ModelId,
    DEVPKEY_Device_FriendlyNameAttributes,
    DEVPKEY_Device_ManufacturerAttributes,
    DEVPKEY_Device_PresenceNotForDevice,
    DEVPKEY_Device_SignalStrength,
    DEVPKEY_Device_IsAssociateableByUserAction,
    DEVPKEY_Device_ShowInUninstallUI,
    DEVPKEY_Device_Numa_Proximity_Domain,
    DEVPKEY_Device_DHP_Rebalance_Policy,
    DEVPKEY_Device_Numa_Node,
    DEVPKEY_Device_BusReportedDeviceDesc,
    DEVPKEY_Device_IsPresent,
    DEVPKEY_Device_HasProblem,
    DEVPKEY_Device_ConfigurationId,
    DEVPKEY_Device_ReportedDeviceIdsHash,
    DEVPKEY_Device_PhysicalDeviceLocation,
    DEVPKEY_Device_BiosDeviceName,
    DEVPKEY_Device_DriverProblemDesc,
    DEVPKEY_Device_DebuggerSafe,
    DEVPKEY_Device_PostInstallInProgress,
    DEVPKEY_Device_Stack,
    DEVPKEY_Device_ExtendedConfigurationIds,
    DEVPKEY_Device_IsRebootRequired,
    DEVPKEY_Device_FirmwareDate,
    DEVPKEY_Device_FirmwareVersion,
    DEVPKEY_Device_FirmwareRevision,
    DEVPKEY_Device_DependencyProviders,
    DEVPKEY_Device_DependencyDependents,
    DEVPKEY_Device_SoftRestartSupported,
    DEVPKEY_Device_SessionId,
    DEVPKEY_Device_InstallDate,
    DEVPKEY_Device_FirstInstallDate,
    DEVPKEY_Device_LastArrivalDate,
    DEVPKEY_Device_LastRemovalDate,
    DEVPKEY_Device_DriverDate,
    DEVPKEY_Device_DriverVersion,
    DEVPKEY_Device_DriverDesc,
    DEVPKEY_Device_DriverInfPath,
    DEVPKEY_Device_DriverInfSection,
    DEVPKEY_Device_DriverInfSectionExt,
    DEVPKEY_Device_MatchingDeviceId,
    DEVPKEY_Device_DriverProvider,
    DEVPKEY_Device_DriverPropPageProvider,
    DEVPKEY_Device_DriverCoInstallers,
    DEVPKEY_Device_ResourcePickerTags,
    DEVPKEY_Device_ResourcePickerExceptions,
    DEVPKEY_Device_DriverRank,
    DEVPKEY_Device_DriverLogoLevel,
    DEVPKEY_Device_NoConnectSound,
    DEVPKEY_Device_GenericDriverInstalled,
    DEVPKEY_Device_AdditionalSoftwareRequested,
    DEVPKEY_Device_SafeRemovalRequired,
    DEVPKEY_Device_SafeRemovalRequiredOverride,
    DEVPKEY_DrvPkg_Model,
    DEVPKEY_DrvPkg_VendorWebSite,
    DEVPKEY_DrvPkg_DetailedDescription,
    DEVPKEY_DrvPkg_DocumentationLink,
    DEVPKEY_DrvPkg_Icon,
    DEVPKEY_DrvPkg_BrandingIcon,
    DEVPKEY_DeviceClass_UpperFilters,
    DEVPKEY_DeviceClass_LowerFilters,
    DEVPKEY_DeviceClass_Security,
    DEVPKEY_DeviceClass_SecuritySDS,
    DEVPKEY_DeviceClass_DevType,
    DEVPKEY_DeviceClass_Exclusive,
    DEVPKEY_DeviceClass_Characteristics,
    DEVPKEY_DeviceClass_Name,
    DEVPKEY_DeviceClass_ClassName,
    DEVPKEY_DeviceClass_Icon,
    DEVPKEY_DeviceClass_ClassInstaller,
    DEVPKEY_DeviceClass_PropPageProvider,
    DEVPKEY_DeviceClass_NoInstallClass,
    DEVPKEY_DeviceClass_NoDisplayClass,
    DEVPKEY_DeviceClass_SilentInstall,
    DEVPKEY_DeviceClass_NoUseClass,
    DEVPKEY_DeviceClass_DefaultService,
    DEVPKEY_DeviceClass_IconPath,
    DEVPKEY_DeviceClass_DHPRebalanceOptOut,
    DEVPKEY_DeviceClass_ClassCoInstallers,
    DEVPKEY_DeviceInterface_FriendlyName,
    DEVPKEY_DeviceInterface_Enabled,
    DEVPKEY_DeviceInterface_ClassGuid,
    DEVPKEY_DeviceInterface_ReferenceString,
    DEVPKEY_DeviceInterface_Restricted,
    DEVPKEY_DeviceInterfaceClass_DefaultInterface,
    DEVPKEY_DeviceInterfaceClass_Name,
    DEVPKEY_DeviceContainer_Address,
    DEVPKEY_DeviceContainer_DiscoveryMethod,
    DEVPKEY_DeviceContainer_IsEncrypted,
    DEVPKEY_DeviceContainer_IsAuthenticated,
    DEVPKEY_DeviceContainer_IsConnected,
    DEVPKEY_DeviceContainer_IsPaired,
    DEVPKEY_DeviceContainer_Icon,
    DEVPKEY_DeviceContainer_Version,
    DEVPKEY_DeviceContainer_Last_Seen,
    DEVPKEY_DeviceContainer_Last_Connected,
    DEVPKEY_DeviceContainer_IsShowInDisconnectedState,
    DEVPKEY_DeviceContainer_IsLocalMachine,
    DEVPKEY_DeviceContainer_MetadataPath,
    DEVPKEY_DeviceContainer_IsMetadataSearchInProgress,
    DEVPKEY_DeviceContainer_MetadataChecksum,
    DEVPKEY_DeviceContainer_IsNotInterestingForDisplay,
    DEVPKEY_DeviceContainer_LaunchDeviceStageOnDeviceConnect,
    DEVPKEY_DeviceContainer_LaunchDeviceStageFromExplorer,
    DEVPKEY_DeviceContainer_BaselineExperienceId,
    DEVPKEY_DeviceContainer_IsDeviceUniquelyIdentifiable,
    DEVPKEY_DeviceContainer_AssociationArray,
    DEVPKEY_DeviceContainer_DeviceDescription1,
    DEVPKEY_DeviceContainer_DeviceDescription2,
    DEVPKEY_DeviceContainer_HasProblem,
    DEVPKEY_DeviceContainer_IsSharedDevice,
    DEVPKEY_DeviceContainer_IsNetworkDevice,
    DEVPKEY_DeviceContainer_IsDefaultDevice,
    DEVPKEY_DeviceContainer_MetadataCabinet,
    DEVPKEY_DeviceContainer_RequiresPairingElevation,
    DEVPKEY_DeviceContainer_ExperienceId,
    DEVPKEY_DeviceContainer_Category,
    DEVPKEY_DeviceContainer_Category_Desc_Singular,
    DEVPKEY_DeviceContainer_Category_Desc_Plural,
    DEVPKEY_DeviceContainer_Category_Icon,
    DEVPKEY_DeviceContainer_CategoryGroup_Desc,
    DEVPKEY_DeviceContainer_CategoryGroup_Icon,
    DEVPKEY_DeviceContainer_PrimaryCategory,
    DEVPKEY_DeviceContainer_UnpairUninstall,
    DEVPKEY_DeviceContainer_RequiresUninstallElevation,
    DEVPKEY_DeviceContainer_DeviceFunctionSubRank,
    DEVPKEY_DeviceContainer_AlwaysShowDeviceAsConnected,
    DEVPKEY_DeviceContainer_ConfigFlags,
    DEVPKEY_DeviceContainer_PrivilegedPackageFamilyNames,
    DEVPKEY_DeviceContainer_CustomPrivilegedPackageFamilyNames,
    DEVPKEY_DeviceContainer_IsRebootRequired,
    DEVPKEY_DeviceContainer_FriendlyName,
    DEVPKEY_DeviceContainer_Manufacturer,
    DEVPKEY_DeviceContainer_ModelName,
    DEVPKEY_DeviceContainer_ModelNumber,
    DEVPKEY_DeviceContainer_InstallInProgress,
    DEVPKEY_DevQuery_ObjectType,
    DEVPKEY_Storage_Portable,
    DEVPKEY_Storage_Removable_Media,
    DEVPKEY_Storage_System_Critical,
    DEVPKEY_Storage_Disk_Number,
    DEVPKEY_Storage_Partition_Number,
    DEVPKEY_Storage_Mbr_Type,
    DEVPKEY_Storage_Gpt_Type,
    DEVPKEY_Storage_Gpt_Name,
]);
