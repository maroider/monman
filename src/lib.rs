use std::mem;

use winapi::{
    shared::windef::POINTL,
    um::{
        wingdi::{
            DEVMODEW, DISPLAY_DEVICEW, DISPLAY_DEVICE_ACTIVE, DISPLAY_DEVICE_MIRRORING_DRIVER,
            DISPLAY_DEVICE_MODESPRUNED, DISPLAY_DEVICE_PRIMARY_DEVICE, DISPLAY_DEVICE_REMOVABLE,
            DISPLAY_DEVICE_VGA_COMPATIBLE, DMDO_180, DMDO_270, DMDO_90, DMDO_DEFAULT,
            DM_BITSPERPEL, DM_COLLATE, DM_COLOR, DM_COPIES, DM_DEFAULTSOURCE,
            DM_DISPLAYFIXEDOUTPUT, DM_DISPLAYFLAGS, DM_DISPLAYFREQUENCY, DM_DISPLAYORIENTATION,
            DM_DITHERTYPE, DM_DUPLEX, DM_FORMNAME, DM_ICMINTENT, DM_ICMMETHOD, DM_INTERLACED,
            DM_LOGPIXELS, DM_MEDIATYPE, DM_NUP, DM_ORIENTATION, DM_PANNINGHEIGHT, DM_PANNINGWIDTH,
            DM_PAPERLENGTH, DM_PAPERSIZE, DM_PAPERWIDTH, DM_PELSHEIGHT, DM_PELSWIDTH, DM_POSITION,
            DM_PRINTQUALITY, DM_SCALE, DM_TTOPTION, DM_YRESOLUTION,
        },
        winuser::{
            ChangeDisplaySettingsW, EnumDisplayDevicesW, EnumDisplaySettingsW, CDS_FULLSCREEN,
            DISP_CHANGE_BADDUALVIEW, DISP_CHANGE_BADFLAGS, DISP_CHANGE_BADMODE,
            DISP_CHANGE_BADPARAM, DISP_CHANGE_FAILED, DISP_CHANGE_NOTUPDATED, DISP_CHANGE_RESTART,
            DISP_CHANGE_SUCCESSFUL, ENUM_CURRENT_SETTINGS, ENUM_REGISTRY_SETTINGS,
        },
    },
};

pub struct DisplayAdapters {
    adapters: Vec<DisplayAdapter>,
}

impl DisplayAdapters {
    pub fn new() -> Option<Self> {
        let mut adapters = Vec::new();

        for i in 0.. {
            if let Some(adapter) = DisplayAdapter::nth(i) {
                adapters.push(adapter);
            } else {
                break;
            }
        }

        if adapters.is_empty() {
            None
        } else {
            Some(Self { adapters })
        }
    }

    pub fn nth(&self, n: usize) -> Option<&DisplayAdapter> {
        self.adapters.get(n)
    }

    pub fn active(&self) -> impl Iterator<Item = &DisplayAdapter> {
        self.adapters
            .iter()
            .filter(|adapter| adapter.state.active())
    }

    pub fn iter(&self) -> impl Iterator<Item = &DisplayAdapter> {
        self.adapters.iter()
    }
}

pub struct DisplayAdapter {
    pub name: String,
    pub string: String,
    pub state: DisplayState,
    pub id: String,
    pub key: String,
    raw: DISPLAY_DEVICEW,
}

impl DisplayAdapter {
    pub fn nth(n: u32) -> Option<Self> {
        let mut display_adapter: DISPLAY_DEVICEW = unsafe { mem::zeroed() };
        display_adapter.cb = mem::size_of::<DISPLAY_DEVICEW>() as u32;

        let ok = match unsafe {
            EnumDisplayDevicesW(std::ptr::null(), n, &mut display_adapter, CDS_FULLSCREEN)
        } {
            0 => false,
            1 => true,
            n => panic!("Invalid bool: {}", n),
        };
        if !ok {
            return None;
        }

        let mut name = String::from_utf16(&display_adapter.DeviceName).unwrap();
        name.retain(|c| c != '\u{0}');
        let mut string = String::from_utf16(&display_adapter.DeviceString).unwrap();
        string.retain(|c| c != '\u{0}');
        let state = DisplayState::from_bits(display_adapter.StateFlags).unwrap();
        let mut id = String::from_utf16(&display_adapter.DeviceID).unwrap();
        id.retain(|c| c != '\u{0}');
        let mut key = String::from_utf16(&display_adapter.DeviceKey).unwrap();
        key.retain(|c| c != '\u{0}');

        Some(Self {
            name,
            string,
            state,
            id,
            key,
            raw: display_adapter,
        })
    }

    pub fn monitors(&self) -> Option<Monitors> {
        Monitors::new(self)
    }

    pub fn info(&self) -> DisplayDeviceInfo {
        DisplayDeviceInfo::new(self)
    }

    pub fn set_orientation(
        &self,
        orientation: DisplayOrientation,
    ) -> Result<(), SetDisplaySettingsError> {
        let mut devmode = DisplayDeviceInfo::get_raw(&self);
        devmode.dmFields = DmFields::DISPLAYORIENTATION.bits();
        unsafe { devmode.u1.s2_mut() }.dmDisplayOrientation = orientation.as_raw();

        // TODO: Parametrize the `dwFlags` argument
        let ret = unsafe { ChangeDisplaySettingsW(&mut devmode, 0) };

        match ret {
            DISP_CHANGE_SUCCESSFUL => Ok(()),
            n => Err(SetDisplaySettingsError::from_raw(n)),
        }
    }
}

// This is a slightly modified form of the derived Debug impl from before the `raw` field was added
impl std::fmt::Debug for DisplayAdapter {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            DisplayAdapter {
                name: ref __self_0_0,
                string: ref __self_0_1,
                state: ref __self_0_2,
                id: ref __self_0_3,
                key: ref __self_0_4,
                ..
            } => {
                let mut debug_trait_builder = f.debug_struct("DisplayAdapter");
                let _ = debug_trait_builder.field("name", &&(*__self_0_0));
                let _ = debug_trait_builder.field("string", &&(*__self_0_1));
                let _ = debug_trait_builder.field("state", &&(*__self_0_2));
                let _ = debug_trait_builder.field("id", &&(*__self_0_3));
                let _ = debug_trait_builder.field("key", &&(*__self_0_4));
                debug_trait_builder.finish()
            }
        }
    }
}

pub struct Monitors {
    monitors: Vec<Monitor>,
}

impl Monitors {
    fn new(adapter: &DisplayAdapter) -> Option<Self> {
        let mut monitors = Vec::new();

        let mut display_device: DISPLAY_DEVICEW = unsafe { mem::zeroed() };
        display_device.cb = mem::size_of::<DISPLAY_DEVICEW>() as u32;

        let mut i = 0;
        while match unsafe {
            EnumDisplayDevicesW(&adapter.raw.DeviceName[0], i, &mut display_device, 0)
        } {
            0 => false,
            1 => true,
            n => panic!("Invalid bool: {}", n),
        } {
            let mut name = String::from_utf16(&display_device.DeviceName).unwrap();
            name.retain(|c| c != '\u{0}');
            let mut string = String::from_utf16(&display_device.DeviceString).unwrap();
            string.retain(|c| c != '\u{0}');
            let mut id = String::from_utf16(&display_device.DeviceID).unwrap();
            id.retain(|c| c != '\u{0}');
            let mut key = String::from_utf16(&display_device.DeviceKey).unwrap();
            key.retain(|c| c != '\u{0}');

            let monitor = Monitor {
                name,
                string,
                id,
                key,
                raw: display_device,
            };
            monitors.push(monitor);

            i += 1;
        }

        if monitors.is_empty() {
            None
        } else {
            Some(Self { monitors })
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Monitor> {
        self.monitors.iter()
    }
}

pub struct Monitor {
    pub name: String,
    pub string: String,
    pub id: String,
    pub key: String,
    raw: DISPLAY_DEVICEW,
}

impl Monitor {}

// This is a slightly modified form of the derived Debug impl from before the `raw` field was added
impl std::fmt::Debug for Monitor {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            Monitor {
                name: ref __self_0_0,
                string: ref __self_0_1,
                id: ref __self_0_2,
                key: ref __self_0_3,
                ..
            } => {
                let mut debug_trait_builder = f.debug_struct("Monitor");
                let _ = debug_trait_builder.field("name", &&(*__self_0_0));
                let _ = debug_trait_builder.field("string", &&(*__self_0_1));
                let _ = debug_trait_builder.field("id", &&(*__self_0_2));
                let _ = debug_trait_builder.field("key", &&(*__self_0_3));
                debug_trait_builder.finish()
            }
        }
    }
}

bitflags::bitflags! {
    pub struct DisplayState: u32 {
        const ACTIVE = DISPLAY_DEVICE_ACTIVE;
        const MIRRORING_DRIVE = DISPLAY_DEVICE_MIRRORING_DRIVER;
        const MODESPRUNED = DISPLAY_DEVICE_MODESPRUNED;
        const PRIMARY_DEVICE = DISPLAY_DEVICE_PRIMARY_DEVICE;
        const REMOVABLE = DISPLAY_DEVICE_REMOVABLE;
        const VGA_COMPATIBLE = DISPLAY_DEVICE_VGA_COMPATIBLE;
    }
}

impl DisplayState {
    pub fn active(self) -> bool {
        self.contains(Self::ACTIVE)
    }

    pub fn primary_device(self) -> bool {
        self.contains(Self::PRIMARY_DEVICE)
    }
}

#[derive(Debug)]
pub struct DisplayDeviceInfo {
    pub name: String,
    pub driver_version: u16,

    pub position: Option<Point>,
    pub orientation: Option<DisplayOrientation>,
    pub bits_per_pel: Option<u32>,
    pub pels_width: Option<u32>,
    pub pels_height: Option<u32>,
    pub flags: Option<DisplayFlags>,
    pub frequency: Option<u32>,
}

impl DisplayDeviceInfo {
    fn new(adapter: &DisplayAdapter) -> Self {
        let devmode = Self::get_raw(adapter);

        let name = string_from_utf16_and_strip_null(&devmode.dmDeviceName);
        // TODO: Check spec_version
        let spec_version = devmode.dmSpecVersion;
        let driver_version = devmode.dmDriverVersion;
        let fields = DmFields::from_bits(devmode.dmFields).unwrap();

        let struct_2 = unsafe { devmode.u1.s2() };

        let position = if fields.contains(DmFields::POSITION) {
            Some(struct_2.dmPosition.into())
        } else {
            None
        };

        let orientation = if fields.contains(DmFields::DISPLAYORIENTATION) {
            DisplayOrientation::from_raw(struct_2.dmDisplayOrientation)
        } else {
            None
        };

        let bits_per_pel = if fields.contains(DmFields::BITSPERPEL) {
            Some(devmode.dmBitsPerPel)
        } else {
            None
        };

        let pels_width = if fields.contains(DmFields::PELSWIDTH) {
            Some(devmode.dmPelsWidth)
        } else {
            None
        };

        let pels_height = if fields.contains(DmFields::PELSHEIGHT) {
            Some(devmode.dmPelsHeight)
        } else {
            None
        };

        let flags = if fields.contains(DmFields::DISPLAYFLAGS) {
            DisplayFlags::from_bits(unsafe { *devmode.u2.dmDisplayFlags() })
        } else {
            None
        };

        let frequency = if fields.contains(DmFields::DISPLAYFREQUENCY) {
            Some(devmode.dmDisplayFrequency)
        } else {
            None
        };

        Self {
            name,
            driver_version,
            position,
            orientation,
            bits_per_pel,
            pels_width,
            pels_height,
            flags,
            frequency,
        }
    }

    fn get_raw(adapter: &DisplayAdapter) -> DEVMODEW {
        let mut devmode: DEVMODEW = unsafe { std::mem::zeroed() };
        devmode.dmSize = mem::size_of::<DEVMODEW>() as u16;

        unsafe {
            EnumDisplaySettingsW(
                &adapter.raw.DeviceName[0],
                ENUM_CURRENT_SETTINGS,
                &mut devmode,
            )
        };

        devmode
    }
}

bitflags::bitflags! {
    pub struct DmFields: u32 {
        const ORIENTATION = DM_ORIENTATION;
        const PAPERSIZE = DM_PAPERSIZE;
        const PAPERLENGTH = DM_PAPERLENGTH;
        const PAPERWIDTH = DM_PAPERWIDTH;
        const SCALE = DM_SCALE;
        const COPIES = DM_COPIES;
        const DEFAULTSOURCE = DM_DEFAULTSOURCE;
        const PRINTQUALITY = DM_PRINTQUALITY;
        const POSITION = DM_POSITION;
        const DISPLAYORIENTATION = DM_DISPLAYORIENTATION;
        const DISPLAYFIXEDOUTPUT = DM_DISPLAYFIXEDOUTPUT;
        const COLOR = DM_COLOR;
        const DUPLEX = DM_DUPLEX;
        const YRESOLUTION = DM_YRESOLUTION;
        const TTOPTION = DM_TTOPTION;
        const COLLATE = DM_COLLATE;
        const FORMNAME = DM_FORMNAME;
        const LOGPIXELS = DM_LOGPIXELS;
        const BITSPERPEL = DM_BITSPERPEL;
        const PELSWIDTH = DM_PELSWIDTH;
        const PELSHEIGHT = DM_PELSHEIGHT;
        const DISPLAYFLAGS = DM_DISPLAYFLAGS;
        const NUP = DM_NUP;
        const DISPLAYFREQUENCY = DM_DISPLAYFREQUENCY;
        const ICMMETHOD = DM_ICMMETHOD;
        const ICMINTENT = DM_ICMINTENT;
        const MEDIATYPE = DM_MEDIATYPE;
        const DITHERTYPE = DM_DITHERTYPE;
        const PANNINGWIDTH = DM_PANNINGWIDTH;
        const PANNINGHEIGHT = DM_PANNINGHEIGHT;
    }
}

#[derive(Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl From<POINTL> for Point {
    fn from(from: POINTL) -> Self {
        Self {
            x: from.x,
            y: from.y,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DisplayOrientation {
    Default,
    Rotate90,
    Rotate180,
    Rotate270,
}

impl DisplayOrientation {
    // TODO: Deal with invalid values in a better way
    pub fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            DMDO_DEFAULT => Some(Self::Default),
            DMDO_90 => Some(Self::Rotate90),
            DMDO_180 => Some(Self::Rotate180),
            DMDO_270 => Some(Self::Rotate270),
            _ => None,
        }
    }

    pub fn as_raw(self) -> u32 {
        match self {
            Self::Default => DMDO_DEFAULT,
            Self::Rotate90 => DMDO_90,
            Self::Rotate180 => DMDO_180,
            Self::Rotate270 => DMDO_270,
        }
    }
}

bitflags::bitflags! {
    pub struct DisplayFlags: u32 {
        // FIXME: winapi doesn't seem to define `DM_GRAYSCALE` anywhere
        // const GRAYSCALE = DM_GRAYSCALE;
        const INTERLACED = DM_INTERLACED;
    }
}

/// https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-changedisplaysettingsw#return-value
#[derive(Debug)]
pub enum SetDisplaySettingsError {
    BadDualView,
    BadFlags,
    BadMode,
    BadParam,
    Failed,
    NotUpdated,
    Restart,
}

impl SetDisplaySettingsError {
    fn from_raw(raw: i32) -> Self {
        match raw {
            DISP_CHANGE_BADDUALVIEW => Self::BadDualView,
            DISP_CHANGE_BADFLAGS => Self::BadFlags,
            DISP_CHANGE_BADMODE => Self::BadMode,
            DISP_CHANGE_BADPARAM => Self::BadParam,
            DISP_CHANGE_FAILED => Self::Failed,
            DISP_CHANGE_NOTUPDATED => Self::NotUpdated,
            DISP_CHANGE_RESTART => Self::Restart,
            n => panic!("Unexpected error code: {}", n),
        }
    }
}

fn string_from_utf16_and_strip_null(v: &[u16]) -> String {
    let mut string = String::from_utf16(v).unwrap();
    string.retain(|c| c != '\u{0}');
    string
}
