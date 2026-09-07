//! GTK4 layer-shell overlay for tier browser and match status.

use bgc_catalog::Catalog;
use bgc_state::GameState;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, Label, ListBox, ListBoxRow,
    Orientation, PolicyType, ScrolledWindow,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::time::Duration;

/// UI-facing snapshot pushed from the app loop.
#[derive(Debug, Clone)]
pub struct OverlaySnapshot {
    pub status: String,
    pub state: GameState,
    pub selected_tier: u8,
}

/// Messages from UI → app (tier clicks).
#[derive(Debug, Clone)]
pub enum UiMsg {
    SelectTier(u8),
    Quit,
}

struct Widgets {
    status: Label,
    races: Label,
    cards: ListBox,
    opponent: Label,
}

fn fill_cards(list: &ListBox, catalog: &Catalog, tier: u8, races: &[String]) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    for name in catalog.names_for_tier(tier, races) {
        let row = ListBoxRow::new();
        let label = Label::new(Some(&name));
        label.set_halign(gtk4::Align::Start);
        label.set_margin_start(8);
        label.set_margin_end(8);
        label.set_margin_top(2);
        label.set_margin_bottom(2);
        row.set_child(Some(&label));
        list.append(&row);
    }
}

fn opponent_text(state: &GameState) -> String {
    match state.last_opponent_board() {
        None => "Last opponent board: (none)".into(),
        Some(b) => {
            let mins: Vec<String> = b
                .minions
                .iter()
                .map(|m| format!("{} {}/{}", m.name, m.atk, m.health))
                .collect();
            format!(
                "Last opponent ({}) {}: {}",
                b.player_id,
                b.hero,
                if mins.is_empty() {
                    "(empty)".into()
                } else {
                    mins.join(", ")
                }
            )
        }
    }
}

fn apply_snapshot(w: &Widgets, catalog: &Catalog, snap: &OverlaySnapshot) {
    w.status.set_text(&format!(
        "{} | phase={} | {}",
        snap.status,
        snap.state.phase_label(),
        catalog.source
    ));
    w.races.set_text(&snap.state.races_label());
    fill_cards(
        &w.cards,
        catalog,
        snap.selected_tier,
        &snap.state.available_races,
    );
    w.opponent.set_text(&opponent_text(&snap.state));
}

/// Build and run the overlay; blocks on GTK main loop.
pub fn run(
    catalog: Arc<Catalog>,
    ui_tx: Sender<UiMsg>,
    snap_rx: Receiver<OverlaySnapshot>,
    initial: OverlaySnapshot,
) {
    let app = Application::builder()
        .application_id("dev.battlegrounds.companion")
        .build();

    // activate is Fn (may fire twice); take setup once.
    let setup = Rc::new(RefCell::new(Some((catalog, ui_tx.clone(), snap_rx, initial))));

    app.connect_activate(move |app| {
        let Some((catalog, ui_tx_activate, snap_rx, initial)) = setup.borrow_mut().take() else {
            return;
        };

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Battlegrounds Companion")
            .default_width(420)
            .default_height(640)
            .build();

        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Left, true);
        window.set_margin(Edge::Top, 48);
        window.set_margin(Edge::Left, 12);
        window.set_keyboard_mode(KeyboardMode::OnDemand);

        let root = GtkBox::new(Orientation::Vertical, 8);
        root.set_margin_top(8);
        root.set_margin_bottom(8);
        root.set_margin_start(8);
        root.set_margin_end(8);

        let status = Label::new(Some(&initial.status));
        status.set_halign(gtk4::Align::Start);
        status.set_wrap(true);
        let races = Label::new(Some(&initial.state.races_label()));
        races.set_halign(gtk4::Align::Start);

        let tier_row = GtkBox::new(Orientation::Horizontal, 4);
        for t in 1u8..=6 {
            let btn = Button::with_label(&format!("T{t}"));
            let tx = ui_tx_activate.clone();
            btn.connect_clicked(move |_| {
                let _ = tx.send(UiMsg::SelectTier(t));
            });
            tier_row.append(&btn);
        }

        let cards = ListBox::new();
        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Automatic)
            .vscrollbar_policy(PolicyType::Automatic)
            .vexpand(true)
            .min_content_height(320)
            .child(&cards)
            .build();

        let opponent = Label::new(Some(&opponent_text(&initial.state)));
        opponent.set_halign(gtk4::Align::Start);
        opponent.set_wrap(true);

        root.append(&status);
        root.append(&races);
        root.append(&tier_row);
        root.append(&scroll);
        root.append(&opponent);
        window.set_child(Some(&root));

        let widgets = Rc::new(Widgets {
            status,
            races,
            cards,
            opponent,
        });
        apply_snapshot(&widgets, &catalog, &initial);

        let widgets_p = widgets.clone();
        let catalog_p = catalog.clone();
        let snap_rx = Rc::new(RefCell::new(snap_rx));
        glib::timeout_add_local(Duration::from_millis(100), move || {
            let rx = snap_rx.borrow_mut();
            loop {
                match rx.try_recv() {
                    Ok(snap) => apply_snapshot(&widgets_p, &catalog_p, &snap),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return glib::ControlFlow::Break,
                }
            }
            glib::ControlFlow::Continue
        });

        window.present();
    });

    app.run_with_args(&[] as &[&str]);
    let _ = ui_tx.send(UiMsg::Quit);
}
