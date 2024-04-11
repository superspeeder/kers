use std::ffi::{c_char, CStr, CString};
use std::rc::Rc;
use ash::{vk};
use log::info;
use winit::event_loop::EventLoop;
use winit::raw_window_handle::{DisplayHandle, HasDisplayHandle, RawDisplayHandle};

const ENGINE_NAME: &str = "kers";
const ENGINE_VERSION: (u32, u32, u32) = (0, 1, 0);

pub struct Engine {
    vulkan: Vulkan,
    event_loop: Rc<EventLoop<()>>,
}

impl Engine {
    pub fn init() -> anyhow::Result<Self> {
        Ok(Self {
            vulkan: Vulkan::load()?,
            event_loop: Rc::new(EventLoop::new()?),
        })
    }


    pub fn vulkan(&self) -> &Vulkan {
        &self.vulkan
    }
    pub fn event_loop(&self) -> &Rc<EventLoop<()>> {
        &self.event_loop
    }
}

pub struct Vulkan {
    entry: ash::Entry,
}

impl Vulkan {
    pub fn load() -> anyhow::Result<Self> {
        let entry = unsafe { ash::Entry::load() }?;

        let api_version = unsafe { entry.try_enumerate_instance_version() }?.unwrap_or(vk::API_VERSION_1_0);
        info!("Loaded Vulkan Version {:?}.{:?}.{:?}", vk::api_version_major(api_version), vk::api_version_minor(api_version), vk::api_version_patch(api_version));

        Ok(Self {
            entry
        })
    }


    pub fn entry(&self) -> &ash::Entry {
        &self.entry
    }
}

pub struct AppContext {
    instance: ash::Instance,
}

fn platform_extension(display_handle: DisplayHandle) -> &'static CStr {
    match display_handle.as_raw() {
        RawDisplayHandle::Xlib(_) => ash::khr::xlib_surface::NAME,
        RawDisplayHandle::Xcb(_) => ash::khr::xcb_surface::NAME,
        RawDisplayHandle::Wayland(_) => ash::khr::wayland_surface::NAME,
        RawDisplayHandle::Windows(_) => ash::khr::win32_surface::NAME,
        _ => panic!("Unsupported System"),
    }
}

impl AppContext {
    pub fn new(engine: &Engine, settings: &AppSettings) -> anyhow::Result<Self> {
        let instance_extensions: Vec<*const c_char> = vec![
            ash::khr::surface::NAME.as_ptr(),
            platform_extension(engine.event_loop.display_handle()?).as_ptr(),
        ];

        let engine_name = CString::new(ENGINE_NAME)?;
        let app_name = CString::new(settings.name)?;

        const ENGINE_VERSION: u32 = vk::make_api_version(0, crate::ENGINE_VERSION.0, crate::ENGINE_VERSION.1, crate::ENGINE_VERSION.2);

        let app_version = vk::make_api_version(0, settings.version.0, settings.version.1, settings.version.2);

        let app_info = vk::ApplicationInfo::default()
            .api_version(vk::API_VERSION_1_3)
            .engine_name(engine_name.as_c_str())
            .application_name(app_name.as_c_str())
            .engine_version(ENGINE_VERSION)
            .application_version(app_version);

        let instance_create_info = vk::InstanceCreateInfo::default()
            .enabled_extension_names(instance_extensions.as_slice())
            .application_info(&app_info);

        let instance = unsafe { engine.vulkan().entry().create_instance(&instance_create_info, None) }?;

        info!("Initialized Vulkan Instance");

        Ok(Self {
            instance
        })
    }
}

pub struct AppSettings {
    pub name: &'static str,
    pub version: (u32, u32, u32),
}

impl Default for AppSettings {
    fn default() -> Self {
        Self { name: "App", version: (1, 0, 0) }
    }
}

pub struct App {
    engine: Engine,
    settings: AppSettings,
    app_context: Option<AppContext>,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            engine: Engine::init()?,
            settings: Default::default(),
            app_context: None,
        })
    }

    // intentionally moves self into the function scope.
    pub fn run(mut self) -> anyhow::Result<()> {
        self.app_context = Some(AppContext::new(&self.engine, &self.settings)?);

        Ok(())
    }

    pub fn settings(mut self, settings: AppSettings) -> Self {
        self.settings = settings;
        self
    }
}

