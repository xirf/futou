use tokio::sync::broadcast;
use tracing::{error, info};

pub fn run_tray(shutdown_tx: broadcast::Sender<()>) {
    use tray_icon::menu::{Menu, MenuEvent, MenuItem};
    use tray_icon::{TrayIconBuilder, TrayIconEvent};

    let open_item = MenuItem::new("Open futou", true, None);
    let exit_item = MenuItem::new("Exit", true, None);

    let menu = Menu::with_items(&[&open_item, &exit_item]).expect("Failed to create tray menu");

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
        if let Ok(event) = menu_channel.recv() {
            if event.id == exit_item.id() {
                info!("Exit requested from tray menu");
                let _ = shutdown_tx.send(());
                break;
            }
        }

        if let Ok(_event) = tray_channel.try_recv() {
            // ponytail: busy-poll tray icon clicks, crossbeam doesn't support multi-channel select
        }
    }

    info!("Tray manager stopped");
}
