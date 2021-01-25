use std::{error::Error, time::Duration};

use dbus::{
    arg,
    blocking::{stdintf::org_freedesktop_dbus::Properties, Connection, Proxy},
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
        Ok(StatusNotifierItem::new(items[item], self.conn)?)
    }
}

#[derive(Clone)]
pub struct StatusNotifierItem<'conn> {
    item: Proxy<'conn, &'conn Connection>,
    menu: Option<Proxy<'conn, &'conn Connection>>,
}

impl<'conn> StatusNotifierItem<'conn> {
    pub fn new(name: String, connection: &'conn Connection) -> Result<Self, Box<dyn Error>> {
        let path = name.split(':').collect::<Vec<&str>>()[1];

        Ok(StatusNotifierItem {
            item: connection.with_proxy("org.kde.StatusNotifierItem", path, TIMEOUT),
            menu: None,
        })
    }
}

impl fmt::Debug for StatusNotifierItem<'conn> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "StatusNotifierItem { menu: {:?} }", self.menu)
    }
}
