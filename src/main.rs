use cursive::event::Key;
use cursive::traits::{Nameable, Resizable};
use cursive::views::{Button, Dialog, EditView, LinearLayout, ListView, TextView};
use cursive::{Cursive, CursiveExt};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Merch {
    size: String,
    merch_type: String,
    price: f32,
    amount: usize,
    total_price: f32,
    design: String,
}

const FILE_PATH: &str = "inventory.json";

fn save_to_file(merch_items: &Vec<Merch>) -> io::Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(FILE_PATH)?;
    serde_json::to_writer(file, merch_items)?;
    Ok(())
}

fn load_from_file() -> Vec<Merch> {
    if let Ok(mut file) = File::open(FILE_PATH) {
        let mut input = String::new();
        if file.read_to_string(&mut input).is_ok() {
            if let Ok(merch_items) = serde_json::from_str::<Vec<Merch>>(&input) {
                return merch_items;
            }
        }
    }
    Vec::new()
}

fn quit_check(win: &mut Cursive) {
    win.add_layer(
        Dialog::around(TextView::new(
            "Quit program? All unsaved changes will be lost.",
        ))
        .title("Are you sure you want to quit?")
        .dismiss_button("Cancel")
        .button("Quit", |w| w.quit()),
    );
}

fn main() {
    let mut win = cursive::default();
    let merch_items = Arc::new(Mutex::new(load_from_file()));
    win.add_global_callback(Key::Esc, quit_check);
    let edit_fields = ListView::new()
        .child("Merch Type:", EditView::new().with_name("merch_type"))
        .child("Size:", EditView::new().with_name("size"))
        .child("Price:", EditView::new().with_name("price"))
        .child("Design:", EditView::new().with_name("design"))
        .child("Amount Available:", EditView::new().with_name("amount"));
    let buttons = LinearLayout::horizontal()
        .child(Button::new("Quit", quit_check))
        .child(Button::new("Save", |w| w.quit()))
        .child(Button::new("Show all", |w| w.quit()))
        .child(Button::new("Delete by ID", |w| w.quit()));
    win.add_layer(
        Dialog::around(LinearLayout::vertical().child(edit_fields).child(buttons))
            .title("Merch Inventory Management"),
    );
    win.run();
}
