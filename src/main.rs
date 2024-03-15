use windows_sys::core::GUID;
use windows_sys::Win32::Devices::Properties::*;
use windows_sys::Win32::System::Ioctl::*;

use devprop::DevProperty;
use devset::DevInterfaceSet;
use sd_formatter::devprop;
use sd_formatter::devset;

fn main() {
    let devset = DevInterfaceSet::fetch_present().unwrap();

    let verbose = std::env::args().nth(1).filter(|s| s == "-v").is_some();

    for &Named {
        value: guid,
        name,
        descr,
    } in GUIDS
    {
        println!(
            "[{}] {}",
            DevProperty::Guid(guid),
            if verbose { name } else { descr }
        );
        for data in devset.enumerate(&guid).map(Result::unwrap) {
            let path = data.fetch_path().unwrap().to_utf8();

            let active = if data.is_active() {
                " active"
            } else {
                " inactive"
            };
            let default = if data.is_default() { " default" } else { "" };
            let removed = if data.is_removed() { " removed" } else { "" };

            println!("\"{path}\"{active}{default}{removed}");

            for prop in data.fetch_property_keys().unwrap().into_vec() {
                let name = DEVPKEYS.iter().find_map(|Named { value, name, descr }| {
                    eq_devpropkey(*value, prop).then_some(if verbose { name } else { descr })
                });
                let val = data.fetch_property(&prop).unwrap();
                match name {
                    Some(name) => println!("  {name} = {val}"),
                    None => println!(
                        "  [{}]::{} = {val}",
                        DevProperty::Guid(prop.fmtid),
                        prop.pid
                    ),
                }
            }
        }
    }
}

fn eq_devpropkey(rhs: DEVPROPKEY, lhs: DEVPROPKEY) -> bool {
    rhs.fmtid.data1 == lhs.fmtid.data1
        && rhs.fmtid.data2 == lhs.fmtid.data2
        && rhs.fmtid.data3 == lhs.fmtid.data3
        && rhs.fmtid.data4 == lhs.fmtid.data4
        && rhs.pid == lhs.pid
}

#[derive(Clone, Copy)]
struct Named<'a, T> {
    value: T,
    name: &'a str,
    descr: &'a str,
}

macro_rules! with_name {
    ([$($value:ident = $descr:literal),* $(,)?]) => {
        [ $( Named { value: $value, name: stringify!($value), descr: $descr } ),* ]
    }
}

const GUIDS: &[Named<GUID>] = &with_name!([
    GUID_DEVINTERFACE_DISK = "Disk",
    GUID_DEVINTERFACE_CDROM = "Cdrom",
    GUID_DEVINTERFACE_PARTITION = "Partition",
    GUID_DEVINTERFACE_TAPE = "Tape",
    GUID_DEVINTERFACE_WRITEONCEDISK = "Writeoncedisk",
    GUID_DEVINTERFACE_VOLUME = "Volume",
    GUID_DEVINTERFACE_MEDIUMCHANGER = "Mediumchanger",
    GUID_DEVINTERFACE_FLOPPY = "Floppy",
    GUID_DEVINTERFACE_CDCHANGER = "Cdchanger",
    GUID_DEVINTERFACE_STORAGEPORT = "Storageport",
    GUID_DEVINTERFACE_VMLUN = "Vmlun",
    GUID_DEVINTERFACE_SES = "Ses",
    GUID_DEVINTERFACE_SERVICE_VOLUME = "Service volume",
    GUID_DEVINTERFACE_HIDDEN_VOLUME = "Hidden volume",
    GUID_DEVINTERFACE_UNIFIED_ACCESS_RPMB = "Unified access rpmb",
    GUID_DEVINTERFACE_SCM_PHYSICAL_DEVICE = "Scm physical device",
    GUID_DEVINTERFACE_COMPORT = "Comport",
    GUID_DEVINTERFACE_SERENUM_BUS_ENUMERATOR = "Serenum bus enumerator",
]);

const DEVPKEYS: &[Named<DEVPROPKEY>] = &with_name!([
    DEVPKEY_Device_AdditionalSoftwareRequested = "Device: Additional software requested",
    DEVPKEY_Device_Address = "Device: Address",
    DEVPKEY_Device_BaseContainerId = "Device: Base container id",
    DEVPKEY_Device_BiosDeviceName = "Device: Bios device name",
    DEVPKEY_Device_BusNumber = "Device: Bus number",
    DEVPKEY_Device_BusRelations = "Device: Bus relations",
    DEVPKEY_Device_BusReportedDeviceDesc = "Device: Bus reported device desc",
    DEVPKEY_Device_BusTypeGuid = "Device: Bus type guid",
    DEVPKEY_Device_Capabilities = "Device: Capabilities",
    DEVPKEY_Device_Characteristics = "Device: Characteristics",
    DEVPKEY_Device_Children = "Device: Children",
    DEVPKEY_Device_Class = "Device: Class",
    DEVPKEY_Device_ClassGuid = "Device: Class guid",
    DEVPKEY_Device_CompatibleIds = "Device: Compatible ids",
    DEVPKEY_Device_ConfigFlags = "Device: Config flags",
    DEVPKEY_Device_ConfigurationId = "Device: Configuration id",
    DEVPKEY_Device_ContainerId = "Device: Container id",
    DEVPKEY_Device_DebuggerSafe = "Device: Debugger safe",
    DEVPKEY_Device_DependencyDependents = "Device: Dependency dependents",
    DEVPKEY_Device_DependencyProviders = "Device: Dependency providers",
    DEVPKEY_Device_DeviceDesc = "Device: Device desc",
    DEVPKEY_Device_DevNodeStatus = "Device: Dev node status",
    DEVPKEY_Device_DevType = "Device: Dev type",
    DEVPKEY_Device_DHP_Rebalance_Policy = "Device: Dhp rebalance policy",
    DEVPKEY_Device_Driver = "Device: Driver",
    DEVPKEY_Device_DriverCoInstallers = "Device: Driver co installers",
    DEVPKEY_Device_DriverDate = "Device: Driver date",
    DEVPKEY_Device_DriverDesc = "Device: Driver desc",
    DEVPKEY_Device_DriverInfPath = "Device: Driver inf path",
    DEVPKEY_Device_DriverInfSection = "Device: Driver inf section",
    DEVPKEY_Device_DriverInfSectionExt = "Device: Driver inf section ext",
    DEVPKEY_Device_DriverLogoLevel = "Device: Driver logo level",
    DEVPKEY_Device_DriverProblemDesc = "Device: Driver problem desc",
    DEVPKEY_Device_DriverPropPageProvider = "Device: Driver prop page provider",
    DEVPKEY_Device_DriverProvider = "Device: Driver provider",
    DEVPKEY_Device_DriverRank = "Device: Driver rank",
    DEVPKEY_Device_DriverVersion = "Device: Driver version",
    DEVPKEY_Device_EjectionRelations = "Device: Ejection relations",
    DEVPKEY_Device_EnumeratorName = "Device: Enumerator name",
    DEVPKEY_Device_Exclusive = "Device: Exclusive",
    DEVPKEY_Device_ExtendedConfigurationIds = "Device: Extended configuration ids",
    DEVPKEY_Device_FirmwareDate = "Device: Firmware date",
    DEVPKEY_Device_FirmwareRevision = "Device: Firmware revision",
    DEVPKEY_Device_FirmwareVersion = "Device: Firmware version",
    DEVPKEY_Device_FirstInstallDate = "Device: First install date",
    DEVPKEY_Device_FriendlyName = "Device: Friendly name",
    DEVPKEY_Device_FriendlyNameAttributes = "Device: Friendly name attributes",
    DEVPKEY_Device_GenericDriverInstalled = "Device: Generic driver installed",
    DEVPKEY_Device_HardwareIds = "Device: Hardware ids",
    DEVPKEY_Device_HasProblem = "Device: Has problem",
    DEVPKEY_Device_InLocalMachineContainer = "Device: In local machine container",
    DEVPKEY_Device_InstallDate = "Device: Install date",
    DEVPKEY_Device_InstallState = "Device: Install state",
    DEVPKEY_Device_InstanceId = "Device: Instance id",
    DEVPKEY_Device_IsAssociateableByUserAction = "Device: Is associateable by user action",
    DEVPKEY_Device_IsPresent = "Device: Is present",
    DEVPKEY_Device_IsRebootRequired = "Device: Is reboot required",
    DEVPKEY_Device_LastArrivalDate = "Device: Last arrival date",
    DEVPKEY_Device_LastRemovalDate = "Device: Last removal date",
    DEVPKEY_Device_Legacy = "Device: Legacy",
    DEVPKEY_Device_LegacyBusType = "Device: Legacy bus type",
    DEVPKEY_Device_LocationInfo = "Device: Location info",
    DEVPKEY_Device_LocationPaths = "Device: Location paths",
    DEVPKEY_Device_LowerFilters = "Device: Lower filters",
    DEVPKEY_Device_Manufacturer = "Device: Manufacturer",
    DEVPKEY_Device_ManufacturerAttributes = "Device: Manufacturer attributes",
    DEVPKEY_Device_MatchingDeviceId = "Device: Matching device id",
    DEVPKEY_Device_Model = "Device: Model",
    DEVPKEY_Device_ModelId = "Device: Model id",
    DEVPKEY_Device_NoConnectSound = "Device: No connect sound",
    DEVPKEY_Device_Numa_Node = "Device: Numa node",
    DEVPKEY_Device_Numa_Proximity_Domain = "Device: Numa proximity domain",
    DEVPKEY_Device_Parent = "Device: Parent",
    DEVPKEY_Device_PDOName = "Device: Pdo name",
    DEVPKEY_Device_PhysicalDeviceLocation = "Device: Physical device location",
    DEVPKEY_Device_PostInstallInProgress = "Device: Post install in progress",
    DEVPKEY_Device_PowerData = "Device: Power data",
    DEVPKEY_Device_PowerRelations = "Device: Power relations",
    DEVPKEY_Device_PresenceNotForDevice = "Device: Presence not for device",
    DEVPKEY_Device_ProblemCode = "Device: Problem code",
    DEVPKEY_Device_ProblemStatus = "Device: Problem status",
    DEVPKEY_Device_RemovalPolicy = "Device: Removal policy",
    DEVPKEY_Device_RemovalPolicyDefault = "Device: Removal policy default",
    DEVPKEY_Device_RemovalPolicyOverride = "Device: Removal policy override",
    DEVPKEY_Device_RemovalRelations = "Device: Removal relations",
    DEVPKEY_Device_Reported = "Device: Reported",
    DEVPKEY_Device_ReportedDeviceIdsHash = "Device: Reported device ids hash",
    DEVPKEY_Device_ResourcePickerExceptions = "Device: Resource picker exceptions",
    DEVPKEY_Device_ResourcePickerTags = "Device: Resource picker tags",
    DEVPKEY_Device_SafeRemovalRequired = "Device: Safe removal required",
    DEVPKEY_Device_SafeRemovalRequiredOverride = "Device: Safe removal required override",
    DEVPKEY_Device_Security = "Device: Security",
    DEVPKEY_Device_SecuritySDS = "Device: Security sds",
    DEVPKEY_Device_Service = "Device: Service",
    DEVPKEY_Device_SessionId = "Device: Session id",
    DEVPKEY_Device_ShowInUninstallUI = "Device: Show in uninstall ui",
    DEVPKEY_Device_Siblings = "Device: Siblings",
    DEVPKEY_Device_SignalStrength = "Device: Signal strength",
    DEVPKEY_Device_SoftRestartSupported = "Device: Soft restart supported",
    DEVPKEY_Device_Stack = "Device: Stack",
    DEVPKEY_Device_TransportRelations = "Device: Transport relations",
    DEVPKEY_Device_UINumber = "Device: Ui number",
    DEVPKEY_Device_UINumberDescFormat = "Device: Ui number desc format",
    DEVPKEY_Device_UpperFilters = "Device: Upper filters",
    DEVPKEY_DeviceClass_Characteristics = "Device class: Characteristics",
    DEVPKEY_DeviceClass_ClassCoInstallers = "Device class: Class co installers",
    DEVPKEY_DeviceClass_ClassInstaller = "Device class: Class installer",
    DEVPKEY_DeviceClass_ClassName = "Device class: Class name",
    DEVPKEY_DeviceClass_DefaultService = "Device class: Default service",
    DEVPKEY_DeviceClass_DevType = "Device class: Dev type",
    DEVPKEY_DeviceClass_DHPRebalanceOptOut = "Device class: Dhp rebalance opt out",
    DEVPKEY_DeviceClass_Exclusive = "Device class: Exclusive",
    DEVPKEY_DeviceClass_Icon = "Device class: Icon",
    DEVPKEY_DeviceClass_IconPath = "Device class: Icon path",
    DEVPKEY_DeviceClass_LowerFilters = "Device class: Lower filters",
    DEVPKEY_DeviceClass_Name = "Device class: Name",
    DEVPKEY_DeviceClass_NoDisplayClass = "Device class: No display class",
    DEVPKEY_DeviceClass_NoInstallClass = "Device class: No install class",
    DEVPKEY_DeviceClass_NoUseClass = "Device class: No use class",
    DEVPKEY_DeviceClass_PropPageProvider = "Device class: Prop page provider",
    DEVPKEY_DeviceClass_Security = "Device class: Security",
    DEVPKEY_DeviceClass_SecuritySDS = "Device class: Security sds",
    DEVPKEY_DeviceClass_SilentInstall = "Device class: Silent install",
    DEVPKEY_DeviceClass_UpperFilters = "Device class: Upper filters",
    DEVPKEY_DeviceContainer_Address = "Device container: Address",
    DEVPKEY_DeviceContainer_AlwaysShowDeviceAsConnected =
        "Device container: Always show device as connected",
    DEVPKEY_DeviceContainer_AssociationArray = "Device container: Association array",
    DEVPKEY_DeviceContainer_BaselineExperienceId = "Device container: Baseline experience id",
    DEVPKEY_DeviceContainer_Category = "Device container: Category",
    DEVPKEY_DeviceContainer_Category_Desc_Plural = "Device container: Category desc plural",
    DEVPKEY_DeviceContainer_Category_Desc_Singular = "Device container: Category desc singular",
    DEVPKEY_DeviceContainer_Category_Icon = "Device container: Category icon",
    DEVPKEY_DeviceContainer_CategoryGroup_Desc = "Device container: Category group desc",
    DEVPKEY_DeviceContainer_CategoryGroup_Icon = "Device container: Category group icon",
    DEVPKEY_DeviceContainer_ConfigFlags = "Device container: Config flags",
    DEVPKEY_DeviceContainer_CustomPrivilegedPackageFamilyNames =
        "Device container: Custom privileged package family names",
    DEVPKEY_DeviceContainer_DeviceDescription1 = "Device container: Device description1",
    DEVPKEY_DeviceContainer_DeviceDescription2 = "Device container: Device description2",
    DEVPKEY_DeviceContainer_DeviceFunctionSubRank = "Device container: Device function sub rank",
    DEVPKEY_DeviceContainer_DiscoveryMethod = "Device container: Discovery method",
    DEVPKEY_DeviceContainer_ExperienceId = "Device container: Experience id",
    DEVPKEY_DeviceContainer_FriendlyName = "Device container: Friendly name",
    DEVPKEY_DeviceContainer_HasProblem = "Device container: Has problem",
    DEVPKEY_DeviceContainer_Icon = "Device container: Icon",
    DEVPKEY_DeviceContainer_InstallInProgress = "Device container: Install in progress",
    DEVPKEY_DeviceContainer_IsAuthenticated = "Device container: Is authenticated",
    DEVPKEY_DeviceContainer_IsConnected = "Device container: Is connected",
    DEVPKEY_DeviceContainer_IsDefaultDevice = "Device container: Is default device",
    DEVPKEY_DeviceContainer_IsDeviceUniquelyIdentifiable =
        "Device container: Is device uniquely identifiable",
    DEVPKEY_DeviceContainer_IsEncrypted = "Device container: Is encrypted",
    DEVPKEY_DeviceContainer_IsLocalMachine = "Device container: Is local machine",
    DEVPKEY_DeviceContainer_IsMetadataSearchInProgress =
        "Device container: Is metadata search in progress",
    DEVPKEY_DeviceContainer_IsNetworkDevice = "Device container: Is network device",
    DEVPKEY_DeviceContainer_IsNotInterestingForDisplay =
        "Device container: Is not interesting for display",
    DEVPKEY_DeviceContainer_IsPaired = "Device container: Is paired",
    DEVPKEY_DeviceContainer_IsRebootRequired = "Device container: Is reboot required",
    DEVPKEY_DeviceContainer_IsSharedDevice = "Device container: Is shared device",
    DEVPKEY_DeviceContainer_IsShowInDisconnectedState =
        "Device container: Is show in disconnected state",
    DEVPKEY_DeviceContainer_Last_Connected = "Device container: Last connected",
    DEVPKEY_DeviceContainer_Last_Seen = "Device container: Last seen",
    DEVPKEY_DeviceContainer_LaunchDeviceStageFromExplorer =
        "Device container: Launch device stage from explorer",
    DEVPKEY_DeviceContainer_LaunchDeviceStageOnDeviceConnect =
        "Device container: Launch device stage on device connect",
    DEVPKEY_DeviceContainer_Manufacturer = "Device container: Manufacturer",
    DEVPKEY_DeviceContainer_MetadataCabinet = "Device container: Metadata cabinet",
    DEVPKEY_DeviceContainer_MetadataChecksum = "Device container: Metadata checksum",
    DEVPKEY_DeviceContainer_MetadataPath = "Device container: Metadata path",
    DEVPKEY_DeviceContainer_ModelName = "Device container: Model name",
    DEVPKEY_DeviceContainer_ModelNumber = "Device container: Model number",
    DEVPKEY_DeviceContainer_PrimaryCategory = "Device container: Primary category",
    DEVPKEY_DeviceContainer_PrivilegedPackageFamilyNames =
        "Device container: Privileged package family names",
    DEVPKEY_DeviceContainer_RequiresPairingElevation =
        "Device container: Requires pairing elevation",
    DEVPKEY_DeviceContainer_RequiresUninstallElevation =
        "Device container: Requires uninstall elevation",
    DEVPKEY_DeviceContainer_UnpairUninstall = "Device container: Unpair uninstall",
    DEVPKEY_DeviceContainer_Version = "Device container: Version",
    DEVPKEY_DeviceContainer_AlwaysShowDeviceAsConnected =
        "Device display: Always show device as connected",
    DEVPKEY_DeviceContainer_Category = "Device display: Category",
    DEVPKEY_DeviceContainer_DiscoveryMethod = "Device display: Discovery method",
    DEVPKEY_DeviceContainer_IsNetworkDevice = "Device display: Is network device",
    DEVPKEY_DeviceContainer_IsNotInterestingForDisplay =
        "Device display: Is not interesting for display",
    DEVPKEY_DeviceContainer_IsShowInDisconnectedState =
        "Device display: Is show in disconnected state",
    DEVPKEY_DeviceContainer_RequiresUninstallElevation =
        "Device display: Requires uninstall elevation",
    DEVPKEY_DeviceContainer_UnpairUninstall = "Device display: Unpair uninstall",
    DEVPKEY_DeviceInterface_ClassGuid = "Class guid",
    DEVPKEY_DeviceInterface_Enabled = "Enabled",
    DEVPKEY_DeviceInterface_FriendlyName = "Friendly name",
    DEVPKEY_DeviceInterface_ReferenceString = "Reference string",
    DEVPKEY_DeviceInterface_Restricted = "Restricted",
    DEVPKEY_DeviceInterfaceClass_DefaultInterface = "Class: Default interface",
    DEVPKEY_DeviceInterfaceClass_Name = "Device interface class: Name",
    DEVPKEY_DevQuery_ObjectType = "Device query: Object type",
    DEVPKEY_DrvPkg_BrandingIcon = "DrvPkg: Branding icon",
    DEVPKEY_DrvPkg_DetailedDescription = "DrvPkg: Detailed description",
    DEVPKEY_DrvPkg_DocumentationLink = "DrvPkg: Documentation link",
    DEVPKEY_DrvPkg_Icon = "DrvPkg: Icon",
    DEVPKEY_DrvPkg_Model = "DrvPkg: Model",
    DEVPKEY_DrvPkg_VendorWebSite = "DrvPkg: Vendor web site",
    DEVPKEY_NAME = "Name",
    DEVPKEY_Device_Numa_Proximity_Domain = "Numa proximity domain",
    DEVPKEY_Storage_Disk_Number = "Storage: Disk number",
    DEVPKEY_Storage_Gpt_Name = "Storage: Gpt name",
    DEVPKEY_Storage_Gpt_Type = "Storage: Gpt type",
    DEVPKEY_Storage_Mbr_Type = "Storage: Mbr type",
    DEVPKEY_Storage_Partition_Number = "Storage: Partition number",
    DEVPKEY_Storage_Portable = "Storage: Portable",
    DEVPKEY_Storage_Removable_Media = "Storage: Removable media",
    DEVPKEY_Storage_System_Critical = "Storage: System critical",
]);
