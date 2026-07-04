use tokio::sync::broadcast;
use tracing::{error, info};

pub struct TrayManager {
    shutdown_tx: broadcast::Sender<()>,
}

impl TrayManager {
    pub fn new(shutdown_tx: broadcast::Sender<()>) -> Self {
        Self { shutdown_tx }
    }

    pub fn run(self) {
        run_tray(self.shutdown_tx);
    }
}

fn run_tray(shutdown_tx: broadcast::Sender<()>) {
    use tray_icon::menu::{Menu, MenuItem, MenuEvent};
    use tray_icon::{TrayIconBuilder, TrayIconEvent};

    let open_item = MenuItem::new("Open futou", true, None);
    let exit_item = MenuItem::new("Exit", true, None);

    let menu = Menu::with_items(&[&open_item, &exit_item])
        .expect("Failed to create tray menu");

    let _tray = match TrayIconBuilder::new()
        .with_tooltip("futou Environment Manager")
        .with_menu(Box::new(menu))
        .build()
    {
        Ok(tray) => tray,
        Err(e) => {
            error!("Failed to create tray icon: {}", e);
            return;
        }
    };

    info!("Tray icon created");

    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();

    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));

        if let Ok(event) = menu_channel.try_recv() {
            if event.id == exit_item.id() {
                info!("Exit requested from tray menu");
                let _ = shutdown_tx.send(());
                break;
            }
        }

        if let Ok(_event) = tray_channel.try_recv() {
            // Tray icon clicked - could open GUI
        }
    }

    info!("Tray manager stopped");
}
