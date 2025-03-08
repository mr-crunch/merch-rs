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
    avail_amount: usize,
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
        .child(Button::new("Save", {
            let merch_clone = Arc::clone(&merch_items);
            move |w| {
                let merch_type = w
                    .call_on_name("merch_type", |view: &mut EditView| view.get_content())
                    .unwrap()
                    .to_string();
                let size = w
                    .call_on_name("size", |view: &mut EditView| view.get_content())
                    .unwrap()
                    .to_string();
                let price = w
                    .call_on_name("price", |view: &mut EditView| view.get_content())
                    .unwrap()
                    .parse::<f32>()
                    .unwrap_or(0.0);
                let design = w
                    .call_on_name("design", |view: &mut EditView| view.get_content())
                    .unwrap()
                    .to_string();
                let avail_amount = w
                    .call_on_name("amount", |view: &mut EditView| view.get_content())
                    .unwrap()
                    .parse::<usize>()
                    .unwrap_or(0);
                if merch_type.is_empty() {
                    w.add_layer(Dialog::info("Please enter a product type."));
                    return;
                }
                if size.is_empty() {
                    w.add_layer(Dialog::info("Please enter a size"));
                    return;
                }
                if price <= 0.0 {
                    w.add_layer(Dialog::info("Please enter a valid price"));
                    return;
                }
                if design.is_empty() {
                    w.add_layer(Dialog::info("Please enter a design ID"));
                    return;
                }
                if avail_amount == 0 {
                    w.add_layer(Dialog::info("Please enter a valid amount"));
                    return;
                }
                let total_price = price * avail_amount as f32;
                let merch = Merch {
                    size,
                    merch_type,
                    avail_amount,
                    price,
                    design,
                    total_price,
                };
                let mut stored_merch = merch_clone.lock().unwrap();
                stored_merch.push(merch.clone());
                if let Err(error) = save_to_file(&stored_merch) {
                    w.add_layer(Dialog::info(format!("Error saving entry: {:?}", error)));
                } else {
                    w.add_layer(Dialog::info("Entry saved successfully"));
                }
            }
        }))
        .child(Button::new("Show all", {
            let merch_clone = Arc::clone(&merch_items);
            move |w| {
                let stored_merch = merch_clone.lock().unwrap();
                let mut output = String::new();
                for (idx, merch_item) in stored_merch.iter().enumerate() {
                    output.push_str(&format!(
                    "{}. Type: {}, Design: {}, Size: {},\n    Available Amount: {}, Price: ${}, Total Price: ${}\n",
                    idx +1,
                    merch_item.merch_type,
                    merch_item.design,
                    merch_item.size,
                    merch_item.avail_amount,
                    merch_item.price,
                    merch_item.total_price,));
                }
                if output.is_empty() {
                    output = String::from("No products to display")
                }
                w.add_layer(Dialog::info(output));
            }
        }))
        .child(Button::new("Delete by ID", {
            let merch_clone = Arc::clone(&merch_items);
            move |w| {
                let id_input = EditView::new().with_name("delete_id").min_width(10);
                w.add_layer(Dialog::new().title("Delete entry").content(ListView::new().child("Enter entry ID to delete", id_input)).button("Confirm", {
                    let merch_clone = Arc::clone(&merch_clone);
                    move |w| {
                        let id_str = w.call_on_name("delete_id", |view: &mut EditView| {
                            view.get_content()
                        })
                        .unwrap()
                        .to_string();
                        if let Ok(id) = id_str.parse::<usize>() {
                            let mut stored_merch = merch_clone.lock().unwrap();
                            if id > 0 && id <= stored_merch.len() {
                                stored_merch.remove(id - 1);
                                if let Err(error) = save_to_file(&stored_merch) {
                                    w.add_layer(Dialog::info(format!("Error deleting entry: {}", error)));
                                } else {
                                    w.add_layer(Dialog::info("Entry deleted successfully"));
                                }
                            } else {
                                w.add_layer(Dialog::info("Could not find index. Please enter a valid ID"));
                            }
                        } else {
                            w.add_layer(Dialog::info("ID must be a valid number"));
                        }
                    }
                }).button("Cancel", |w| {
                        w.pop_layer();
                    })
                );
            }
        }));
    win.add_layer(
        Dialog::around(LinearLayout::vertical().child(edit_fields).child(buttons))
            .title("Merch Inventory Management"),
    );
    win.run();
}
