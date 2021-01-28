use std::{error::Error, fmt, time::Duration};

use dbus::{
    arg::{AppendAll, Get, ReadAll, RefArg},
    blocking::{stdintf::org_freedesktop_dbus::Properties, Connection, MakeSignal, Proxy},
    channel::Token,
    message::SignalArgs,
    Message,
};

use crate::interfaces::{
    status_notifier_item::{
        OrgKdeStatusNotifierItemNewAttentionIcon, OrgKdeStatusNotifierItemNewIcon,
        OrgKdeStatusNotifierItemNewStatus, OrgKdeStatusNotifierItemNewTitle,
    },
    status_notifier_watcher::{
        OrgKdeStatusNotifierWatcherStatusNotifierItemRegistered,
        OrgKdeStatusNotifierWatcherStatusNotifierItemUnregistered,
    },
};

const TIMEOUT: Duration = Duration::from_millis(50);

#[derive(Clone)]
pub struct StatusNotifierHost<'conn> {
    conn: &'conn Connection,
    watcher: Proxy<'conn, &'conn Connection>,
}

impl<'conn> StatusNotifierHost<'conn> {
    pub fn new(conn: &'conn Connection) -> Result<StatusNotifierHost, Box<dyn Error>> {
        let watcher = conn.with_proxy(
            "org.kde.StatusNotifierWatcher",
            "/StatusNotifierWatcher",
            TIMEOUT,
        );

        Ok(StatusNotifierHost { conn, watcher })
    }

    pub fn get_protocol_version(&self) -> Result<u8, Box<dyn Error>> {
        let (version,): (u8,) = self
            .watcher
            .get("org.kde.StatusNotifierWatcher", "ProtocolVersion")?;

        Ok(version)
    }

    pub fn registered_status_notifier_items(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let items: Vec<String> = self.watcher.get(
            "org.kde.StatusNotifierWatcher",
            "RegisteredStatusNotifierItems",
        )?;

        Ok(items)
    }

    pub fn get_item(&self, item: usize) -> Result<StatusNotifierItem, Box<dyn Error>> {
        let items = self.registered_status_notifier_items()?;
        Ok(StatusNotifierItem::new(items[item].clone(), self.conn)?)
    }

    // UNIMPLEMENTED: Method RegusterStatusNotifierHost
    // UNIMPLEMENTED: Method RegusterStatusNotifierItem

    pub fn signal<S: ReadAll + SignalArgs, F: 'static + SignalFunction<S>>(
        &self,
        f: F,
    ) -> Result<Token, Box<dyn Error>> {
        Ok(self.watcher.match_signal(f)?)
    }

    // UNIMPLEMENTED: Signal StatusNotifierHostRegistered
    // UNIMPLEMENTED: Signal StatusNotifierHostUnregistered

    pub fn status_notifier_item_registered<
        F: 'static + SignalFunction<OrgKdeStatusNotifierWatcherStatusNotifierItemRegistered>,
    >(
        &self,
        f: F,
    ) -> Result<Token, Box<dyn Error>> {
        self.signal(f)
    }

    pub fn status_notifier_item_unregistered<
        F: 'static + SignalFunction<OrgKdeStatusNotifierWatcherStatusNotifierItemUnregistered>,
    >(
        &self,
        f: F,
    ) -> Result<Token, Box<dyn Error>> {
        self.signal(f)
    }
}

pub trait SignalFunction<S: ReadAll + SignalArgs>:
    Fn(S, &Connection, &Message) -> bool + Send
{
}

/**
 * An alias to a status notifier item, for pulling data, calling methods and registering signals in an ergonomic way
 */
#[derive(Clone)]
pub struct StatusNotifierItem<'conn> {
    item: Proxy<'conn, &'conn Connection>,
    menu: Option<Proxy<'conn, &'conn Connection>>,
}

impl<'conn> StatusNotifierItem<'conn> {
    pub fn new(name: String, connection: &'conn Connection) -> Result<Self, Box<dyn Error>> {
        let mut name = name.split('/');
        let id = name.next().unwrap().to_string();
        let path = name.collect::<Vec<&str>>();
        let path = format!("/{}", path.join("/"));

        let item = connection.with_proxy(id, path, TIMEOUT);

        Ok(StatusNotifierItem { item, menu: None })
    }

    pub fn get<R0: for<'b> Get<'b> + 'static>(
        &self,
        property_name: &str,
    ) -> Result<R0, Box<dyn Error>> {
        Ok(self.item.get("org.kde.StatusNotifierItem", property_name)?)
    }

    pub fn get_category(&self) -> Result<String, Box<dyn Error>> {
        self.get("Category")
    }

    pub fn get_id(&self) -> Result<String, Box<dyn Error>> {
        self.get("Id")
    }

    pub fn get_title(&self) -> Result<String, Box<dyn Error>> {
        self.get("Title")
    }

    pub fn get_status(&self) -> Result<String, Box<dyn Error>> {
        self.get("Status")
    }

    pub fn get_window_id(&self) -> Result<u32, Box<dyn Error>> {
        self.get("WindowId")
    }

    pub fn get_icon_name(&self) -> Result<String, Box<dyn Error>> {
        self.get("IconName")
    }

    // UNIMPLEMENTED: Gettter for IconPixmap

    pub fn get_overlay_icon_name(&self) -> Result<String, Box<dyn Error>> {
        self.get("OverlayIconName")
    }

    // UNIMPLEMENTED: Getter for OverlayIconPixmap

    pub fn get_attention_icon_name(&self) -> Result<String, Box<dyn Error>> {
        self.get("AttentionIconName")
    }

    // UNIMPLEMENTED: Getter for AttentionIconPixmap

    pub fn get_attention_movie_name(&self) -> Result<String, Box<dyn Error>> {
        self.get("AttentionMovieName")
    }

    // UNIMPLEMENTED: Getter for Tooltip

    pub fn get_is_menu(&self) -> Result<bool, Box<dyn Error>> {
        self.get("IsMenu")
    }

    pub fn get_menu(&self) -> Result<Box<dyn RefArg>, Box<dyn Error>> {
        self.get("Menu")
    }

    pub fn call<A: AppendAll, R: ReadAll>(
        &self,
        method_name: &str,
        args: A,
    ) -> Result<R, Box<dyn Error>> {
        Ok(self
            .item
            .method_call("org.kde.StatusNotifierItem", method_name, args)?)
    }

    pub fn context_menu(&self, x: i64, y: i64) -> Result<(), Box<dyn Error>> {
        self.call("ContextMenu", (x, y))?;
        Ok(())
    }

    pub fn activate(&self, x: i64, y: i64) -> Result<(), Box<dyn Error>> {
        self.call("Activate", (x, y))?;
        Ok(())
    }

    pub fn secondary_activate(&self, x: i64, y: i64) -> Result<(), Box<dyn Error>> {
        self.call("SecondaryActivate", (x, y))?;
        Ok(())
    }

    pub fn scroll(&self, delta: i64, orientation: &str) -> Result<(), Box<dyn Error>> {
        self.call("Scroll", (delta, orientation))?;
        Ok(())
    }

    pub fn signal<S: ReadAll + SignalArgs, F: 'static + SignalFunction<S>>(
        &self,
        f: F,
    ) -> Result<Token, Box<dyn Error>> {
        Ok(self.item.match_signal(f)?)
    }

    pub fn new_title<F: 'static + SignalFunction<OrgKdeStatusNotifierItemNewTitle>>(
        &self,
        f: F,
    ) -> Result<Token, Box<dyn Error>> {
        self.signal(f)
    }

    pub fn new_icon<F: 'static + SignalFunction<OrgKdeStatusNotifierItemNewIcon>>(
        &self,
        f: F,
    ) -> Result<Token, Box<dyn Error>> {
        self.signal(f)
    }

    pub fn new_attention_icon<
        F: 'static + SignalFunction<OrgKdeStatusNotifierItemNewAttentionIcon>,
    >(
        &self,
        f: F,
    ) -> Result<Token, Box<dyn Error>> {
        self.signal(f)
    }

    // UNIMPLEMENTED: NewOverlayIcon
    // UNIMPLEMENTED: NewToolTip

    pub fn new_status<F: 'static + SignalFunction<OrgKdeStatusNotifierItemNewStatus>>(
        &self,
        f: F,
    ) -> Result<Token, Box<dyn Error>> {
        self.signal(f)
    }
}

impl<'conn> fmt::Debug for StatusNotifierItem<'conn> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let menu = match &self.menu {
            Some(menu) => format!("{} {}", menu.destination, menu.path),
            None => String::from("None"),
        };

        let item = format!("{} {}", self.item.destination, self.item.path);

        write!(
            fmt,
            "StatusNotifierItem {{ item: {}, menu: {} }}",
            item, menu
        )
    }
}
