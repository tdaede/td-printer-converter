use gtk::prelude::*;

use gtk::{gdk, gdk_pixbuf, glib, Orientation};
use glib::{MainContext, PRIORITY_DEFAULT, clone};
use std::sync::{Arc, Mutex, mpsc::channel, atomic::AtomicU32, atomic::Ordering};
use std::time::Duration;
use std::thread;
use std::ops::Deref;
use serialport;
use image::RgbImage;
use image::imageops;

use crate::Args;
use crate::ipp_print;
use crate::printer::Printer;
use crate::printer::*;

pub(crate) fn gui_main() {
    let application = gtk::Application::new(
        Some("com.thomasdaede.td-printer-converter"),
        Default::default(),
    );
    application.connect_activate(build_ui);
    let cli_args = Vec::<String>::new();
    application.run_with_args(&cli_args);
}

struct printer_config {
    printer: String
}

#[derive(Default)]
struct PageInfo {
    covered_x: u32,
    covered_y: u32
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);
    window.set_title(Some("td-printer-converter"));
    window.set_default_size(500, 500);

    let picture = gtk::Picture::new();
    picture.set_halign(gtk::Align::Center);
    picture.set_can_shrink(false);
    picture.set_size_request(200, 200);

    let scrolledwindow = gtk::ScrolledWindow::new();
    scrolledwindow.set_hexpand(true);
    scrolledwindow.set_child(Some(&picture));

    let clear_button = gtk::Button::new();
    clear_button.set_label("Clear");

    let supported_printers = [
        "cz-8pc4",
        "cz-6pv1",
    ];

    let drop_down = gtk::DropDown::from_strings(&supported_printers);

    let save_button = gtk::Button::new();
    save_button.set_label("Save");

    let print_button = gtk::Button::new();
    print_button.set_label("Print");

    let button_box = gtk::Box::new(Orientation::Vertical, 0);
    button_box.append(&clear_button);
    button_box.append(&drop_down);
    button_box.append(&save_button);
    button_box.append(&print_button);

    let top_box = gtk::Box::new(Orientation::Horizontal, 0);
    top_box.append(&scrolledwindow);
    top_box.append(&button_box);

    window.set_child(Some(&top_box));
    window.present();

    let (tx_config, rx_config) = channel();

    let img = RgbImage::from_pixel(Cz8pc4::PAGE_WIDTH, Cz8pc4::PAGE_HEIGHT, image::Rgb([255,255,255]));
    let img_arc_mutex = Arc::new(Mutex::new(img));
    let page_info_arc_mutex = Arc::new(Mutex::new(PageInfo::default()));

    let img_arc_mutex_redraw = Arc::clone(&img_arc_mutex);

    let update_printer = clone!(@strong drop_down => move || {
        let config = printer_config {
            printer: supported_printers[drop_down.selected() as usize].to_string()
        };
        tx_config.send(config).unwrap();
    });

    save_button.connect_clicked(clone!(@strong img_arc_mutex, @strong page_info_arc_mutex => move |_| {
        let img = img_arc_mutex.lock().unwrap();
        let page_info = page_info_arc_mutex.lock().unwrap();
        let mut start_y: u32 = u32::MAX;
        for (y, row) in img.rows().enumerate() {
            for pixel in row {
                if pixel[0] != 255 && pixel[1] != 255 && pixel[2] != 255 {
                    start_y = y as u32;
                    break;
                }
            }
            if start_y != u32::MAX {
                break;
            }
        }
        let img_cropped = imageops::crop_imm(img.deref(), 0, start_y, page_info.covered_x, page_info.covered_y - start_y).to_image();
        img_cropped.save("print.png").unwrap();
    }));


    print_button.connect_clicked(clone!(@strong img_arc_mutex, @strong page_info_arc_mutex => move |_| {
        let img = img_arc_mutex.lock().unwrap();
        let page_info = page_info_arc_mutex.lock().unwrap();
        let mut start_y: u32 = u32::MAX;
        for (y, row) in img.rows().enumerate() {
            for pixel in row {
                if pixel[0] != 255 && pixel[1] != 255 && pixel[2] != 255 {
                    start_y = y as u32;
                    break;
                }
            }
            if start_y != u32::MAX {
                break;
            }
        }
        let img_cropped = imageops::crop_imm(img.deref(), 0, start_y, page_info.covered_x, page_info.covered_y - start_y).to_image();
        ipp_print(&"http://CP1500fb99b1.local:631".to_string(), img_cropped);
    }));

    clear_button.connect_clicked(clone!(@strong update_printer => move |_| {
        update_printer();
    }));
    drop_down.connect_notify_local(Some("selected"), move|_,_| {
        update_printer();
    });

    glib::timeout_add_local(Duration::from_millis(16), clone!(@strong picture => move || {
        let img = img_arc_mutex_redraw.lock().unwrap();
        let pixbuf = gdk_pixbuf::Pixbuf::from_bytes(&glib::Bytes::from(img.as_raw()),
                                                    gdk_pixbuf::Colorspace::Rgb, false, 8,
                                                    img.width() as i32, img.height() as i32,
                                                    img.sample_layout().height_stride as i32);
        let texture = gdk::Texture::for_pixbuf(&pixbuf);
        picture.set_paintable(Some(&texture));
        picture.queue_draw();
        glib::Continue(true)
    }));

    let img_arc_mutex_thread = Arc::clone(&img_arc_mutex);
    thread::spawn(move || {
        let mut printer: Box<dyn Printer> = Box::new(Cz8pc4::default());
        let mut serial_port = serialport::new("/dev/ttyACM0", 500000).open().expect("Failed to open port");
        serial_port.set_timeout(Duration::from_secs(1)).unwrap();
        loop {
            let (covered_x_decode, covered_y_decode) = printer.decode(&mut serial_port, &img_arc_mutex_thread);
            let mut page_info = page_info_arc_mutex.lock().unwrap();
            page_info.covered_x = covered_x_decode;
            page_info.covered_y = covered_y_decode;
            println!("Set page_info to {covered_x_decode}, {covered_y_decode}");
            if let Ok(config) = rx_config.try_recv() {
                printer = match config.printer.as_str() {
                    "cz-8pc4" => Box::new(Cz8pc4::default()),
                    "cz-6pv1" => Box::new(Cz6pv1::default()),
                    _ => unreachable!(),
                };
                let mut img = img_arc_mutex.lock().unwrap();
                img.fill(255);
            }
        }
    });
}
