use std::mem;

use winapi::um::{
    wingdi::{
        DISPLAY_DEVICEW, DISPLAY_DEVICE_ACTIVE, DISPLAY_DEVICE_MIRRORING_DRIVER,
        DISPLAY_DEVICE_MODESPRUNED, DISPLAY_DEVICE_PRIMARY_DEVICE, DISPLAY_DEVICE_REMOVABLE,
        DISPLAY_DEVICE_VGA_COMPATIBLE,
    },
    winuser::EnumDisplayDevicesW,
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

        let ok = match unsafe { EnumDisplayDevicesW(std::ptr::null(), n, &mut display_adapter, 0) }
        {
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
