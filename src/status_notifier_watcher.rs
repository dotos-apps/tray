use std::{error::Error, sync::Mutex};

use dbus::blocking::{Connection, Proxy};
use dbus_crossroads::Crossroads;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Category {
    ApplicationStatus,
    Communications,
    SystemServices,
    Hardware,
    Unknown,
}

impl Category {
    pub fn from_str(s: &str) -> Self {
        match s {
            "ApplicationStatus" => Category::ApplicationStatus,
            "Communications" => Category::Communications,
            "SystemServices" => Category::SystemServices,
            "Hardware" => Category::Hardware,
            _ => Category::Unknown,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Category::ApplicationStatus => "ApplicationStatus",
            Category::Communications => "Communications",
            Category::SystemServices => "SystemServices",
            Category::Hardware => "Hardware",
            Category::Unknown => "Unknown",
        }
    }

    pub fn to_string(&self) -> String {
        self.to_str().to_string()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Status {
    Passive,
    Active,
    NeedsAttention,
}

impl Status {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Passive" => Status::Passive,
            "Active" => Status::Active,
            "NeedsAttention" => Status::NeedsAttention,
            _ => Status::Passive,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Status::Passive => "Passive",
            Status::Active => "Active",
            Status::NeedsAttention => "NeedsAttention",
        }
    }

    pub fn to_string(&self) -> String {
        self.to_str().to_string()
    }
}

#[derive(Clone)]
pub struct StatusNotifierItem {
    pub service: String, //
    pub sender: String,  //
                         // proxy: Proxy<&Connection>,                  //
                         // pub category: Option<Category>,           //
                         // pub title: Option<String>,                //
                         // pub id: Option<String>,                   //
                         // pub status: Option<Status>,               //
                         // pub window_id: Option<u32>,               //
                         // pub icon_name: Option<String>,            //
                         // pub icon_piximap: Option<()>,             // TODO
                         // pub overlay_icon_name: Option<String>,    //
                         // pub overlay_icon_piximap: Option<()>,     // TODO
                         // pub attention_icon_name: Option<String>,  //
                         // pub attention_icon_piximap: Option<()>,   // TODO
                         // pub attention_movie_name: Option<String>, // Not implementing
                         // pub tool_tip: Option<()>,                 // TODO
                         // pub item_is_menu: Option<bool>,           // Not implementing
                         // pub menu: Option<String>,                 // Not implementing
}

impl StatusNotifierItem {
    pub fn new(service: String, sender: String) -> Self {
        StatusNotifierItem {
            service,
            sender,
            // proxy,
            // category: None,
            // title: None,
            // id: None,
            // status: None,
            // window_id: None,
            // icon_name: None,
            // icon_piximap: None,
            // overlay_icon_piximap: None,
            // overlay_icon_name: None,
            // attention_icon_name: None,
            // attention_icon_piximap: None,
            // attention_movie_name: None,
            // tool_tip: None,
            // item_is_menu: None,
            // menu: None,
        }
    }

    // pub fn get_title(&self) -> String {}

    pub fn to_register_string(&self) -> String {
        format!("{}{}", self.sender, self.service)
    }
}

pub struct StatusNotifierWatcher {
    pub services: Vec<StatusNotifierItem>,
}

impl StatusNotifierWatcher {
    fn new() -> Result<Self, Box<dyn Error>> {
        Ok(StatusNotifierWatcher {
            services: Vec::new(),
        })
    }

    pub fn services_to_register_string(&self) -> Vec<String> {
        (&self.services)
            .into_iter()
            .map(|sni| sni.to_register_string())
            .collect()
    }
}

pub fn run(inited_mutex: &Mutex<bool>) -> Result<(), Box<dyn Error>> {
    // Lock mutex to stop stuff from happening on other threads
    let mut inited = inited_mutex.lock().unwrap();

    // Create connection
    let connection = Connection::new_session()?;

    // Request the dbus name that is required for AppIndicators
    connection.request_name("org.kde.StatusNotifierWatcher", false, true, false)?;

    // Create a crossroads
    let mut cr = Crossroads::new();

    let status_notifier_watcher = cr.register("org.kde.StatusNotifierWatcher", |b| {
        // Methods
        // -------
        // Register status notifier host
        b.method(
            "RegisterStatusNotifierHost",
            ("service",),
            (),
            |_, _, (service,): (String,)| {
                println!("RegisterStatusNotifierHost service={}", service);
                Ok(())
            },
        );

        // Register status notifier item
        b.method(
            "RegisterStatusNotifierItem",
            ("service",),
            (),
            |context, data: &mut StatusNotifierWatcher, (service,): (String,)| {
                // Log register information to the console
                println!("RegisterStatusNotifierItem service={}", service);

                let path = context.message().sender().unwrap().to_string();

                // Add the service to the data store
                data.services
                    .push(StatusNotifierItem::new(service.clone(), path));

                // Create and send the StatusNotifierItemRegistered signal
                let signal_msg = context.make_signal("StatusNotifierItemRegistered", ("/",));
                context.push_msg(signal_msg);

                // Return
                Ok(())
            },
        );

        // Signals
        // -------
        // On host register
        b.signal::<(), &'static str>("StatusNotifierHostRegistered", ());
        // On host unregister
        b.signal::<(), &'static str>("StatusNotifierHostUnregistered", ());
        // Status notifier item registered
        b.signal::<(&'static str,), &'static str>("StatusNotifierItemRegistered", ("String",));
        // Status notifier item unregistered
        b.signal::<(&'static str,), &'static str>("StatusNotifierItemUnregistered", ("String",));

        // Properties
        // ----------
        // Note: You use `get` and `set` for setting and getting the values
        b.property::<Vec<String>, &str>("RegisteredStatusNotifierItems")
            .get(|_, data| Ok(data.services_to_register_string()));
        b.property::<bool, &str>("IsStatusNotifierHostRegistered")
            .get(|_, _| Ok(true));
        b.property::<u8, &str>("ProtocolVersion").get(|_, _| Ok(0));
    });

    // Insert the functionality into our watcher
    cr.insert(
        "/StatusNotifierWatcher",
        &[status_notifier_watcher],
        StatusNotifierWatcher::new()?,
    );

    // Set innitied to true and drop mutex
    *inited = true;
    drop(inited);

    // Add to the connection
    cr.serve(&connection)?;

    unreachable!()
}
