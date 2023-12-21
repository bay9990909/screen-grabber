use std::collections::HashSet;
use std::path::PathBuf;
use egui::Image;
use egui::epaint::TextShape;
use egui::{menu, Button, Color32};
use egui::emath::RectTransform;
use image::DynamicImage;
use egui::*;
use image::ImageBuffer;
use image::ImageOutputFormat;
use image::RgbaImage;
use image::GenericImageView;
//use eframe::egui;
//use eframe::egui::TextureHandle;
use crate::draws_functions::Draws;
use crate::draws_functions::Text;
use crate::{Schermata, edit, EditType};
use crate::screen;
use crate::MyGlobalHotKeyManager;
use global_hotkey::hotkey::{HotKey, Code, Modifiers};
use egui::{Grid, Stroke, Ui, Visuals, Label};
use image::{save_buffer, ImageFormat, ColorType};
use rfd::FileDialog;
use chrono::prelude::*;
use std::io::{Cursor, Write};
use image::io::Reader as ImageReader;
use std::ptr;
use std::thread::sleep;
use std::time::Duration;
use arboard::{Clipboard, ImageData};
use std::io::stdout;
use crate::icons::*;

pub fn home(ctx: &egui::Context, schermata: &mut Schermata, image: &mut RgbaImage, texture : &mut Option<TextureHandle>, hotkeys_list: &mut Vec<(Modifiers, Code, String)>, file_format: &mut String, save_path: &mut PathBuf, name_convention: &mut String){
    egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            menu::bar(ui, |ui| {

                ui.menu_button("Settings", |ui| {
                    if ui.button("Custom Hotkey").on_hover_text("Customize your Hotkeys").clicked() {
                        *schermata = Schermata::Setting_Hotkey;
                    }

                    if ui.button("Saving settings").on_hover_text("Customize default saving options").clicked() {
                        *schermata = Schermata::Setting_Saving;
                    }
                }).response.on_hover_text("Change your Settings");; //.on_hover_text("Take a Screenshot");



                if ui.button("Screenshots").on_hover_text("Take a Screenshot").clicked() {
                    *image = screen::screenshot().unwrap();
                    let flat_image = image.as_flat_samples();
                    let color_image2 = egui::ColorImage::from_rgba_unmultiplied([image.width() as usize, image.height() as usize],flat_image.samples);
                    let image_data = egui::ImageData::from(color_image2);
                    *texture = Some(ui.ctx().load_texture("screen", image_data, Default::default()));
                    *schermata = Schermata::Edit;
                }
                    
            });

            ui.centered_and_justified(|ui| {
            //mostro le hotkeys registrate
                Grid::new("some_unique_id").show(ui, |ui| {
                    ui.label("REGISTERED KEYS");
                    ui.end_row();
        
                    for curr_hotkey in hotkeys_list.iter(){
                        //ui.label(hotkey_to_String(curr_hotkey.0, curr_hotkey.1)); 
                        ui.label(hotkey_to_String(curr_hotkey.0, curr_hotkey.1));
                        ui.label(curr_hotkey.2.clone());
                        ui.end_row();
                    }

                    ui.add_space(20.0);

                    ui.end_row();                
                    ui.label("CUSTOM SAVING");

                    ui.end_row();
                    ui.label("File Format: ");
                    ui.label(file_format.clone());

                    ui.end_row();
                    ui.label("Default Path :");

                    if *save_path == PathBuf::default(){
                        ui.label("Go to settings...");
                    }
                    else {
                        ui.label(save_path.clone().into_os_string().into_string().unwrap());
                    }

                    ui.end_row();
                    ui.label("File name:");
                    ui.label(name_convention.clone());
                    

                });
            });
    });    
}

pub fn edit(ctx: &egui::Context, draws: &mut Vec<Draws>, texture : &mut Option<TextureHandle>, frame: &mut eframe::Frame, stroke: &mut Stroke, schermata: &mut Schermata, rgba_image: &mut RgbaImage, file_format: &mut String, save_path: &mut PathBuf, name_convention: &mut String, last_index: &mut Option<usize>, mode: &mut EditType){
    //sleep(Duration::from_millis(200));
    egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {  
        menu::bar(ui, |ui| {
            add_edits_buttons(ui, stroke, mode,last_index,draws);
            if ui.button("Discard").clicked() {
                *schermata = Schermata::Home;
                //elimina anche gli edit
                *texture = None; //e setta a null la textureHandle
            }

            if ui.button("Save").clicked(){
                let now = Utc::now();
                let ts = now.timestamp(); //add timestamp in the name convention, in order to have unique files

                // Save the DynamicImage to a file
                let dynamic_image = DynamicImage::ImageRgba8(rgba_image.clone());                
                if(*save_path != PathBuf::default()) {
                    let output_path = format!("{}\\{}_{}{}", save_path.clone().into_os_string().into_string().unwrap(), name_convention, ts, file_format);
                    dynamic_image.save_with_format(output_path, ImageFormat::Jpeg).expect("Failed to save image");
                }
                else {
                    let p = FileDialog::new().set_directory("/").pick_folder();
                    if(p.is_none()) { }
                    else{
                        let mut path_tmp = p.unwrap();
                        let output_path = format!("{}\\{}_{}{}", path_tmp.clone().into_os_string().into_string().unwrap(), name_convention, ts, file_format);
                        dynamic_image.save_with_format(output_path, ImageFormat::Jpeg).expect("Failed to save image");
                    }   
                }
            }

            if ui.button("Copy").on_hover_text("Copy the Screenshot to Clipboard").clicked() {
                // Copy the image to the clipboard
                let mut ctx_clip = Clipboard::new().unwrap();
                let clipboard_image = DynamicImage::ImageRgba8(rgba_image.clone());
                let image_bytes = clipboard_image.into_bytes();
                #[rustfmt::skip]
                let img_data = ImageData { width: rgba_image.width() as usize, height: rgba_image.height() as usize, bytes: image_bytes.into() };
                ctx_clip.set_image(img_data).unwrap();
            }
        });

        ui.add_space(30.0);
        if !(texture.is_none()) { 
            ui.vertical_centered(|ui| {
                let mut padding = ui.max_rect();
                let x = texture.clone().unwrap().aspect_ratio();
                let y: f32 = padding.aspect_ratio();
                if x > y {
                    padding.set_bottom(ui.max_rect().top()+(ui.max_rect().height()/2.0 - (ui.max_rect().width() / x)/2.0 ));
                    ui.advance_cursor_after_rect(padding);
                }                
                let mut edited_image = Image::new(texture.as_ref().unwrap()).max_size(ui.available_size()).maintain_aspect_ratio(true).shrink_to_fit().ui(ui);
                    let texture_rect = egui::Rect::from_min_size(Pos2::ZERO, texture.clone().unwrap().size_vec2()); //rettangolo della dimensione dell'immagine
                    let screen_rect = eframe::emath::RectTransform::from_to(texture_rect,edited_image.rect);
                    let painter = Painter::new(ctx.clone(),edited_image.layer_id,edited_image.rect);
                    match mode {
                        EditType::Circle => {
                            edit::write_circles(draws, ui,screen_rect.inverse(),stroke);
                        }
                        EditType::Rectangle => {
                            edit::write_rects(draws, ui, screen_rect.inverse(),stroke);
                        }
                        EditType::Free => {
                            edit::write_lines( draws, ui,screen_rect.inverse(),stroke);
                        }
                        EditType::Text => {
                            edit::write_text(&painter, draws, ui, screen_rect.inverse(),last_index,stroke);
                            if last_index.is_some()  {
                                edit::read_keyboard_input(ui, draws[last_index.unwrap()].to_text().unwrap(),last_index);
                            }
                        }
                        EditType::Segment => {
                            edit::write_segments(draws, ui,screen_rect.inverse(),stroke);
                        }
                        EditType::Eraser => {
                            edit::erase_edit(draws, ui, screen_rect.inverse(),&painter);
                        }
                        _ => {

                        }
                    }
                    print_draws3(&painter, draws, screen_rect,last_index);
            });
        }
    });
}


fn add_edits_buttons(ui: &mut Ui, stroke: &mut Stroke, mode: &mut EditType,last_index: &mut Option<usize>, draws: &mut Vec<Draws>) {
    color_picker_and_width(ui, stroke);
    if edit_single_button(ui,&CURSOR,mode,&EditType::Cursor).clicked(){
        *mode = EditType::Cursor;
        *last_index = None;
    }
    if edit_single_button(ui,&ERASER,mode,&EditType::Eraser).clicked(){
        *mode = EditType::Eraser;
        *last_index = None;

    }
    if edit_single_button(ui,&CIRCLE,mode,&EditType::Circle).clicked(){
        *mode = EditType::Circle;
        *last_index = None;
    }
    if edit_single_button(ui,&RECTANGLE,mode,&EditType::Rectangle).clicked(){
        *mode = EditType::Rectangle;
        *last_index = None;
    }
    if edit_single_button(ui,&SEGMENT,mode,&EditType::Segment).clicked(){
        *mode = EditType::Segment;
        *last_index = None;
    }
    if edit_single_button(ui,&FREE,mode,&EditType::Free).clicked(){
        *mode = EditType::Free;
        *last_index = None;
    }
    if edit_single_button(ui,&TEXT,mode,&EditType::Text).clicked(){
        *mode = EditType::Text;
    }
    if edit_single_button(ui,&SCISSOR,mode,&EditType::Crop).clicked(){
        *mode = EditType::Crop;
        *last_index = None;
    }
    if edit_single_button(ui,&BACK,mode,&EditType::Back).clicked(){
        *last_index = None;
        if draws.len() > 0 {
            draws.pop();
        }
    }

}



fn color_picker_and_width(ui: &mut Ui, stroke: &mut Stroke) {
    let size_points = egui::Vec2::new(128.0,32.0);
    let (id, rect) = ui.allocate_space(size_points);
    ui.allocate_ui_at_rect(rect, |ui| {
        ui.color_edit_button_srgba(&mut stroke.color);
    });
    let (id, rect2) = ui.allocate_space(size_points);
    ui.allocate_ui_at_rect(rect2, |ui| {
        ui.add(egui::Slider::new(&mut stroke.width, 1.0..=8.0).integer());   
    });
    
}



fn edit_single_button(ui: &mut Ui,image: &Image<'_>, mode: &EditType,current_mode: &EditType) -> Response {
    let size_points = egui::Vec2::splat(32.0);
    let (id, rect) = ui.allocate_space(size_points);
    let response = ui.interact(rect, id, Sense::click());
    if response.hovered() || mode == current_mode  {
        ui.painter().rect_filled(
            rect,
            Rounding::same(4.0),
            Color32::from_rgb(83,83,83)
        );
        //ui.visuals().widgets.active.fg_stroke.color
    }
    let image = image
    .clone()
    .maintain_aspect_ratio(true)
    //.tint(tint)
    .fit_to_exact_size(size_points);
//ui.add(Button::image(image));
    image.paint_at(ui, rect);
    response
}

// pub fn print_draws(painter: &Painter, draws: &Vec<Draws>,screen_rect: RectTransform) {
//                     println!("Testo {:?}",draws);
//                     //print_text(painter);
//                     let shapes = 
//                     draws
//                     .iter()
//                     .map(|draw| {
                        
//                         match draw {
//                             Draws::Line(single_line) => {
//                                 let points: Vec<Pos2> = single_line.points.iter().map(|p| screen_rect.transform_pos_clamped(*p)).collect();
//                                 egui::Shape::line(points, Stroke::new(5.0,Color32::RED))
//                             }
//                             Draws::Circle(circle) => {
//                                 // Gestisci il caso Circle
//                                 let center = screen_rect.transform_pos_clamped(circle.center);
//                                 let modify = screen_rect.from().width() / screen_rect.to().width();
//                                 let radius = circle.radius / modify;
//                                 egui::Shape::circle_stroke(center, radius,Stroke::new(5.0,Color32::RED))
//                             }
//                             Draws::Rect(rectange) => {
//                                 // Gestisci il caso Circle
//                                 let min = screen_rect.transform_pos_clamped(rectange.rect.min);
//                                 let max = screen_rect.transform_pos_clamped(rectange.rect.max);
//                                 egui::Shape::rect_stroke(Rect::from_min_max(min, max), epaint::Rounding::ZERO, Stroke::new(5.0,Color32::RED))
//                             }
//                             Draws::Text(text) => {
//                                 // Gestisci il caso Circle
//                                 println!("Testo {:?}",text);
//                                 //let point = screen_rect.transform_pos_clamped(text.point);
//                                 //println!("Punto: {:?}",point);
//                                 println!("prima");
//                                 //let point_1 = screen_rect.transform_pos_clamped(text.points[0]);
//                                 //let point_2 = screen_rect.transform_pos_clamped(text.points[1]);
//                                 //print_text(painter);
//                                 //let galley = painter.layout_no_wrap(text.letters.clone(), FontId::monospace(32.0), Color32::RED);
//                                 //stdout().flush();
//                                 let galley = painter.fonts(|f|f.layout("Ciao bella\n".into(), FontId::proportional(1.0), Color32::RED, f32::INFINITY));
//                                 println!("dopo");
//                                 //egui::Shape::Text(TextShape::new(point, galley))
//                                 //egui::Shape::line_segment([point_1,point_2],Stroke::new(5.0,Color32::RED))
//                                 //text.render(painter, screen_rect)
//                                 egui::Shape::Noop
//                             }
//                             Draws::Segment(segment) => {
//                                 // Gestisci il caso Circle
//                                 let point_1 = screen_rect.transform_pos_clamped(segment.points[0]);
//                                 let point_2 = screen_rect.transform_pos_clamped(segment.points[1]);
//                                 egui::Shape::line_segment([point_1,point_2],Stroke::new(5.0,Color32::RED))
//                             }
//                             // Utilizza l'underscore per trattare tutti gli altri casi
//                             _ => {
//                                 egui::Shape::Noop
//                             }
//                         }
//                     });
//                     painter.extend(shapes);
// }

// pub fn print_draws2(painter: &Painter, draws: &mut Vec<Draws>,screen_rect: RectTransform) {
//     println!("Testo {:?}",draws);
//     println!("prima2");
//     let shapes = draws.iter().for_each(|dr| {
//         println!("Testo2 {:?}",dr);
//         // print_text(painter);
//         match dr {
//             Draws::Line(single_line) => {
//                 let points: Vec<Pos2> = single_line.points.iter().map(|p| screen_rect.transform_pos_clamped(*p)).collect();
//                 //egui::Shape::line(points, Stroke::new(5.0,Color32::RED))
//             }
//             Draws::Circle(circle) => {
//                 // Gestisci il caso Circle
//                 let center = screen_rect.transform_pos_clamped(circle.center);
//                 let modify = screen_rect.from().width() / screen_rect.to().width();
//                 let radius = circle.radius / modify;
//                 //egui::Shape::circle_stroke(center, radius,Stroke::new(5.0,Color32::RED))
//             }
//             Draws::Rect(rectange) => {
//                 // Gestisci il caso Circle
//                 let min = screen_rect.transform_pos_clamped(rectange.rect.min);
//                 let max = screen_rect.transform_pos_clamped(rectange.rect.max);
//                 //egui::Shape::rect_stroke(Rect::from_min_max(min, max), epaint::Rounding::ZERO, Stroke::new(5.0,Color32::RED))
//             }
//             Draws::Text(text) => {
//                 println!("dentro text================================");
//                 print_text(painter,text.letters.clone());
//                 //egui::Shape::Noop
//             }
//             _ => {
//                 //print_text(painter);
//                 println!("tutto");
//                 //egui::Shape::Noop
//             }
//         }
        
//     });
//     println!("dopo2");
// }


pub fn print_draws3(painter: &Painter, draws: &Vec<Draws>,screen_rect: RectTransform,last_index: &mut Option<usize>) {
    let mut shape: Vec<Shape> = Vec::new();
    //println!("Testo {:?}",draws);
    //print_text(painter);
    //let shapes = 
    draws
    .iter().enumerate()
    .for_each(|(index,draw)| {
        match draw {
            Draws::Line(single_line) => {
                let points: Vec<Pos2> = single_line.points.iter().map(|p| screen_rect.transform_pos_clamped(*p)).collect();
                shape.push(egui::Shape::line(points, single_line.stroke));
            }
            Draws::Circle(circle) => {
                let center = screen_rect.transform_pos_clamped(circle.center);
                let modify = screen_rect.from().width() / screen_rect.to().width();
                let radius = circle.radius / modify;
                shape.push(egui::Shape::circle_stroke(center, radius,circle.stroke));
            }
            Draws::Rect(rectangle) => {
                let min = screen_rect.transform_pos_clamped(rectangle.rect.min);
                let max = screen_rect.transform_pos_clamped(rectangle.rect.max);
                shape.push(egui::Shape::rect_stroke(Rect::from_min_max(min, max), epaint::Rounding::ZERO, rectangle.stroke));
            }
            Draws::Text(text) => {
                let galley = painter.layout_no_wrap(text.letters.clone(), FontId::monospace(32.0), text.stroke.color);
                let point = screen_rect.transform_pos_clamped(text.point);
                let rect = Align2::CENTER_CENTER.anchor_rect(Rect::from_min_size(point, galley.size()));
                if last_index.is_some() && last_index.unwrap() == index {
                let path = [Pos2::new(rect.left(),rect.top()),
                            Pos2::new(rect.right(),rect.top()),
                            Pos2::new(rect.right(),rect.bottom()),
                            Pos2::new(rect.left(),rect.bottom()),
                            Pos2::new(rect.left(),rect.top())];
                let dotted_line = Shape::dotted_line(&path, Color32::GRAY, 12.0, 4.0);
                shape.extend(dotted_line);
                }
                shape.push(Shape::galley(rect.min, galley));
            }
            Draws::Segment(segment) => {
                let point_1 = screen_rect.transform_pos_clamped(segment.points[0]);
                let point_2 = screen_rect.transform_pos_clamped(segment.points[1]);
                shape.push(egui::Shape::line_segment([point_1,point_2],segment.stroke));
            }
            // Utilizza l'underscore per trattare tutti gli altri casi
            _ => {
                shape.push(egui::Shape::Noop);
            }
        }
    });
    
    painter.extend(shape);
}



pub fn setting_hotkey(ctx: &egui::Context, schermata: &mut Schermata, manager: &mut MyGlobalHotKeyManager, modifier_copy: &mut Modifiers, key_copy: &mut Code, modifier_screen: &mut Modifiers, key_screen: &mut Code, modifier_save: &mut Modifiers, key_save: &mut Code, hotkeys_list: &mut Vec<(Modifiers, Code, String)>, modifier_copy_tmp: &mut Modifiers, key_copy_tmp: &mut Code, modifier_screen_tmp: &mut Modifiers, key_screen_tmp: &mut Code, modifier_save_tmp: &mut Modifiers, key_save_tmp: &mut Code, update_file: &mut bool){
    let window_size = egui::vec2(0.0, 0.0);

    egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
        menu::bar(ui, |ui| {
            ui.menu_button("Settings", |ui| {
                if ui.button("Custom Hotkey").clicked() {
                    *schermata = Schermata::Setting_Hotkey;
                }
                if ui.button("Saving settings").clicked() {
                    *schermata = Schermata::Setting_Saving;
                }
            });
        });

        ui.add_space(20.0);

            Grid::new("miao").show(ui, |ui| {
                ui.label("COPY ");

                egui::ComboBox::from_id_source("Choose modifier copy")
                .selected_text(format!("{:?}", modifier_copy_tmp))
                .show_ui(ui, |ui| {
                    ui.selectable_value(modifier_copy_tmp, Modifiers::CONTROL, "Ctrl");
                    ui.selectable_value(modifier_copy_tmp, Modifiers::SHIFT, "Shift");
                    ui.selectable_value(modifier_copy_tmp, Modifiers::ALT, "Alt");
                });

                egui::ComboBox::from_id_source("Choose Key copy")
                .selected_text(format!("{:?}", key_copy_tmp))
                .show_ui(ui, |ui| {
                    ui.selectable_value(key_copy_tmp, Code::KeyA, "KeyA");
                    ui.selectable_value(key_copy_tmp, Code::KeyB, "KeyB");
                    ui.selectable_value(key_copy_tmp, Code::KeyC, "KeyC");
                    ui.selectable_value(key_copy_tmp, Code::KeyD, "KeyD");
                    ui.selectable_value(key_copy_tmp, Code::KeyE, "KeyE");
                    ui.selectable_value(key_copy_tmp, Code::KeyF, "KeyF");
                    ui.selectable_value(key_copy_tmp, Code::KeyG, "KeyG");
                    ui.selectable_value(key_copy_tmp, Code::KeyH, "KeyH");
                    ui.selectable_value(key_copy_tmp, Code::KeyI, "KeyI");
                    ui.selectable_value(key_copy_tmp, Code::KeyJ, "KeyJ");
                    ui.selectable_value(key_copy_tmp, Code::KeyK, "KeyK");
                    ui.selectable_value(key_copy_tmp, Code::KeyL, "KeyL");
                    ui.selectable_value(key_copy_tmp, Code::KeyM, "KeyM");
                    ui.selectable_value(key_copy_tmp, Code::KeyN, "KeyN");
                    ui.selectable_value(key_copy_tmp, Code::KeyO, "KeyO");
                    ui.selectable_value(key_copy_tmp, Code::KeyP, "KeyP");
                    ui.selectable_value(key_copy_tmp, Code::KeyQ, "KeyQ");
                    ui.selectable_value(key_copy_tmp, Code::KeyR, "KeyR");
                    ui.selectable_value(key_copy_tmp, Code::KeyS, "KeyS");
                    ui.selectable_value(key_copy_tmp, Code::KeyT, "KeyT");
                    ui.selectable_value(key_copy_tmp, Code::KeyU, "KeyU");
                    ui.selectable_value(key_copy_tmp, Code::KeyV, "KeyV");
                    ui.selectable_value(key_copy_tmp, Code::KeyW, "KeyW");
                    ui.selectable_value(key_copy_tmp, Code::KeyX, "KeyX");
                    ui.selectable_value(key_copy_tmp, Code::KeyY, "KeyY");
                    ui.selectable_value(key_copy_tmp, Code::KeyZ, "KeyZ");
                    ui.selectable_value(key_copy_tmp, Code::F1, "F1");
                    ui.selectable_value(key_copy_tmp, Code::F2, "F2");
                    ui.selectable_value(key_copy_tmp, Code::F3, "F3");
                    ui.selectable_value(key_copy_tmp, Code::F5, "F5");
                    ui.selectable_value(key_copy_tmp, Code::F6, "F6");
                    ui.selectable_value(key_copy_tmp, Code::F7, "F7");
                    ui.selectable_value(key_copy_tmp, Code::F8, "F8");
                    ui.selectable_value(key_copy_tmp, Code::F9, "F9");
                    ui.selectable_value(key_copy_tmp, Code::F10, "F10");
                    ui.selectable_value(key_copy_tmp, Code::F11, "F11");
                    ui.selectable_value(key_copy_tmp, Code::F12, "F12");
                    //... aggiungere altre keys nel caso sia necessario ...
                });

                ui.end_row();

                ui.label("SCREEN ");

                egui::ComboBox::from_id_source("Choose modifier screen")
                .selected_text(format!("{:?}", modifier_screen_tmp))
                .show_ui(ui, |ui| {
                    ui.selectable_value(modifier_screen_tmp, Modifiers::CONTROL, "Ctrl");
                    ui.selectable_value(modifier_screen_tmp, Modifiers::SHIFT, "Shift");
                    ui.selectable_value(modifier_screen_tmp, Modifiers::ALT, "Alt");
                });

                egui::ComboBox::from_id_source("Choose Key screen")
                .selected_text(format!("{:?}", key_screen_tmp))
                .show_ui(ui, |ui| {
                    ui.selectable_value(key_screen_tmp, Code::KeyA, "KeyA");
                    ui.selectable_value(key_screen_tmp, Code::KeyB, "KeyB");
                    ui.selectable_value(key_screen_tmp, Code::KeyC, "KeyC");
                    ui.selectable_value(key_screen_tmp, Code::KeyD, "KeyD");
                    ui.selectable_value(key_screen_tmp, Code::KeyE, "KeyE");
                    ui.selectable_value(key_screen_tmp, Code::KeyF, "KeyF");
                    ui.selectable_value(key_screen_tmp, Code::KeyG, "KeyG");
                    ui.selectable_value(key_screen_tmp, Code::KeyH, "KeyH");
                    ui.selectable_value(key_screen_tmp, Code::KeyI, "KeyI");
                    ui.selectable_value(key_screen_tmp, Code::KeyJ, "KeyJ");
                    ui.selectable_value(key_screen_tmp, Code::KeyK, "KeyK");
                    ui.selectable_value(key_screen_tmp, Code::KeyL, "KeyL");
                    ui.selectable_value(key_screen_tmp, Code::KeyM, "KeyM");
                    ui.selectable_value(key_screen_tmp, Code::KeyN, "KeyN");
                    ui.selectable_value(key_screen_tmp, Code::KeyO, "KeyO");
                    ui.selectable_value(key_screen_tmp, Code::KeyP, "KeyP");
                    ui.selectable_value(key_screen_tmp, Code::KeyQ, "KeyQ");
                    ui.selectable_value(key_screen_tmp, Code::KeyR, "KeyR");
                    ui.selectable_value(key_screen_tmp, Code::KeyS, "KeyS");
                    ui.selectable_value(key_screen_tmp, Code::KeyT, "KeyT");
                    ui.selectable_value(key_screen_tmp, Code::KeyU, "KeyU");
                    ui.selectable_value(key_screen_tmp, Code::KeyV, "KeyV");
                    ui.selectable_value(key_screen_tmp, Code::KeyW, "KeyW");
                    ui.selectable_value(key_screen_tmp, Code::KeyX, "KeyX");
                    ui.selectable_value(key_screen_tmp, Code::KeyY, "KeyY");
                    ui.selectable_value(key_screen_tmp, Code::KeyZ, "KeyZ");
                    ui.selectable_value(key_screen_tmp, Code::F1, "F1");
                    ui.selectable_value(key_screen_tmp, Code::F2, "F2");
                    ui.selectable_value(key_screen_tmp, Code::F3, "F3");
                    ui.selectable_value(key_screen_tmp, Code::F5, "F5");
                    ui.selectable_value(key_screen_tmp, Code::F6, "F6");
                    ui.selectable_value(key_screen_tmp, Code::F7, "F7");
                    ui.selectable_value(key_screen_tmp, Code::F8, "F8");
                    ui.selectable_value(key_screen_tmp, Code::F9, "F9");
                    ui.selectable_value(key_screen_tmp, Code::F10, "F10");
                    ui.selectable_value(key_screen_tmp, Code::F11, "F11");
                    ui.selectable_value(key_screen_tmp, Code::F12, "F12");
                    //... aggiungere altre keys nel caso sia necessario ...
                });

                ui.end_row();

                ui.label("SAVE ");

                egui::ComboBox::from_id_source("Choose modifier save")
                .selected_text(format!("{:?}", modifier_save_tmp))
                .show_ui(ui, |ui| {
                    ui.selectable_value(modifier_save_tmp, Modifiers::CONTROL, "Ctrl");
                    ui.selectable_value(modifier_save_tmp, Modifiers::SHIFT, "Shift");
                    ui.selectable_value(modifier_save_tmp, Modifiers::ALT, "Alt");
                });

                egui::ComboBox::from_id_source("Choose Key save")
                .selected_text(format!("{:?}", key_save_tmp))
                .show_ui(ui, |ui| {
                    ui.selectable_value(key_save_tmp, Code::KeyA, "KeyA");
                    ui.selectable_value(key_save_tmp, Code::KeyB, "KeyB");
                    ui.selectable_value(key_save_tmp, Code::KeyC, "KeyC");
                    ui.selectable_value(key_save_tmp, Code::KeyD, "KeyD");
                    ui.selectable_value(key_save_tmp, Code::KeyE, "KeyE");
                    ui.selectable_value(key_save_tmp, Code::KeyF, "KeyF");
                    ui.selectable_value(key_save_tmp, Code::KeyG, "KeyG");
                    ui.selectable_value(key_save_tmp, Code::KeyH, "KeyH");
                    ui.selectable_value(key_save_tmp, Code::KeyI, "KeyI");
                    ui.selectable_value(key_save_tmp, Code::KeyJ, "KeyJ");
                    ui.selectable_value(key_save_tmp, Code::KeyK, "KeyK");
                    ui.selectable_value(key_save_tmp, Code::KeyL, "KeyL");
                    ui.selectable_value(key_save_tmp, Code::KeyM, "KeyM");
                    ui.selectable_value(key_save_tmp, Code::KeyN, "KeyN");
                    ui.selectable_value(key_save_tmp, Code::KeyO, "KeyO");
                    ui.selectable_value(key_save_tmp, Code::KeyP, "KeyP");
                    ui.selectable_value(key_save_tmp, Code::KeyQ, "KeyQ");
                    ui.selectable_value(key_save_tmp, Code::KeyR, "KeyR");
                    ui.selectable_value(key_save_tmp, Code::KeyS, "KeyS");
                    ui.selectable_value(key_save_tmp, Code::KeyT, "KeyT");
                    ui.selectable_value(key_save_tmp, Code::KeyU, "KeyU");
                    ui.selectable_value(key_save_tmp, Code::KeyV, "KeyV");
                    ui.selectable_value(key_save_tmp, Code::KeyW, "KeyW");
                    ui.selectable_value(key_save_tmp, Code::KeyX, "KeyX");
                    ui.selectable_value(key_save_tmp, Code::KeyY, "KeyY");
                    ui.selectable_value(key_save_tmp, Code::KeyZ, "KeyZ");
                    ui.selectable_value(key_save_tmp, Code::F1, "F1");
                    ui.selectable_value(key_save_tmp, Code::F2, "F2");
                    ui.selectable_value(key_save_tmp, Code::F3, "F3");
                    ui.selectable_value(key_save_tmp, Code::F5, "F5");
                    ui.selectable_value(key_save_tmp, Code::F6, "F6");
                    ui.selectable_value(key_save_tmp, Code::F7, "F7");
                    ui.selectable_value(key_save_tmp, Code::F8, "F8");
                    ui.selectable_value(key_save_tmp, Code::F9, "F9");
                    ui.selectable_value(key_save_tmp, Code::F10, "F10");
                    ui.selectable_value(key_save_tmp, Code::F11, "F11");
                    ui.selectable_value(key_save_tmp, Code::F12, "F12");
                    //... aggiungere altre keys nel caso sia necessario ...
                });

                ui.end_row();
            });


            ui.add_space(30.0);

            if ui.button("Chiudi").clicked(){

                for el in hotkeys_list.iter(){
                    //non lascio che un utente modifichi i valori delle caselle e poi lasci il casino...
                    //RIMETTO A POSTO...
                    if el.2 == "Copy".to_string(){
                        *modifier_copy_tmp = el.0.clone();
                        *key_copy_tmp = el.1.clone();
                    }
                    else if el.2 == "Screen".to_string(){
                        *modifier_screen_tmp = el.0.clone();
                        *key_screen_tmp = el.1.clone();
                    }
                    else{ // el.2 == "Save".to_string()
                        *modifier_save_tmp = el.0.clone();
                        *key_save_tmp = el.1.clone();
                    }
                }
                *schermata = Schermata::Home; 
            }

           //fai un check per verificare che tutte le hotkeys siano diverse
           let mut set = HashSet::<(Modifiers, Code)>::new();
           let curr_hotkey_list = vec![(*modifier_copy_tmp, *key_copy_tmp, "Copy"), (*modifier_screen_tmp, *key_screen_tmp, "Screen"), (*modifier_save_tmp, *key_save_tmp, "Save")];
           let all_distinct = curr_hotkey_list.iter().all(|x| set.insert((x.0,x.1)));
           
           let mut hotkeys_to_save = Vec::<HotKey>::new();
           let mut hotkeys_to_delete = Vec::<HotKey>::new();

           ui.set_enabled(all_distinct && ((*modifier_copy != *modifier_copy_tmp) || (*modifier_screen != *modifier_screen_tmp) || (*modifier_save != *modifier_save_tmp) || (*key_copy != *key_copy_tmp) || (*key_screen != *key_screen_tmp) || (*key_save != *key_save_tmp)));
            
           if ui.button("Salva modifiche").clicked() {
            *modifier_copy = *modifier_copy_tmp;
            *modifier_save = *modifier_save_tmp;
            *modifier_screen = *modifier_screen_tmp;
            *key_copy = *key_copy_tmp;
            *key_screen = *key_screen_tmp;
            *key_save = *key_save_tmp;

            //genera la hotkey modificata
            if all_distinct {
                for el in hotkeys_list.iter_mut(){
                    if el.2 == "Copy".to_string(){
                        if el.0 != *modifier_copy || el.1 != *key_copy{
                            let mut hotkey_copy = HotKey::new(Some(*modifier_copy), *key_copy);
                            let mut hotkey_to_delete = HotKey::new(Some(el.0), el.1);

                            hotkeys_to_save.push(hotkey_copy);
                            hotkeys_to_delete.push(hotkey_to_delete);

                            el.0 = *modifier_copy;
                            el.1 = *key_copy;
                        }
                    }
                    else if el.2 == "Screen".to_string(){
                        if el.0 != *modifier_screen || el.1 != *key_screen{
                            let mut hotkey_screen = HotKey::new(Some(*modifier_screen), *key_screen);
                            let mut hotkey_to_delete = HotKey::new(Some(el.0), el.1);

                            hotkeys_to_save.push(hotkey_screen);
                            hotkeys_to_delete.push(hotkey_to_delete);

                            el.0 = *modifier_screen;
                            el.1 = *key_screen;
                        }
                    }
                    else { //if el.2 == "Save".to_string()
                        if el.0 != *modifier_save || el.1 != *key_save{
                            let mut hotkey_save = HotKey::new(Some(*modifier_save), *key_save);
                            let mut hotkey_to_delete = HotKey::new(Some(el.0), el.1);

                            hotkeys_to_save.push(hotkey_save);
                            hotkeys_to_delete.push(hotkey_to_delete);

                            el.0 = *modifier_save;
                            el.1 = *key_save;
                        }
                    }
                }

                ((*manager).0).unregister_all(&hotkeys_to_delete).unwrap();
                ((*manager).0).register_all(&hotkeys_to_save).unwrap(); //ho fatto in questo modo perchè GlobalHotKeyManager non aveva il tratto Default
                
                *update_file = true;
                *schermata = Schermata::Home; 
            }
        }
        });
}

pub fn setting_saving(ctx: &egui::Context, schermata: &mut Schermata, file_format: &mut String, save_path: &mut PathBuf, file_format_tmp: &mut String, save_path_tmp: &mut PathBuf, name_convention: &mut String, name_convention_tmp: &mut String, update_file: &mut bool){
    let window_size = egui::vec2(0.0, 0.0);

    egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {

        menu::bar(ui, |ui| {
            ui.menu_button("Settings", |ui| {
                if ui.button("Custom Hotkey").clicked() {
                    *schermata = Schermata::Setting_Hotkey;
                }

                if ui.button("Saving settings").clicked() {
                    *schermata = Schermata::Setting_Saving;
                }
            });
        });

        ui.add_space(20.0);
        
            egui::ComboBox::from_label("Choose format")
                .selected_text(format!("{}", file_format_tmp))
                .show_ui(ui, |ui| {
                    ui.selectable_value(file_format_tmp, ".png".to_string(), "PNG");
                    ui.selectable_value(file_format_tmp, ".jpeg".to_string(), "JPEG");
                    ui.selectable_value(file_format_tmp, ".gif".to_string(), "GIF");
                    ui.selectable_value(file_format_tmp, ".webp".to_string(), "WEBP");
                    ui.selectable_value(file_format_tmp, ".pnm".to_string(), "PNM");
                    ui.selectable_value(file_format_tmp, ".tiff".to_string(), "TIFF");
                    ui.selectable_value(file_format_tmp, ".tga".to_string(), "TGA");
                    ui.selectable_value(file_format_tmp, ".dds".to_string(), "DDS");
                    ui.selectable_value(file_format_tmp, ".bmp".to_string(), "BMP");
                    ui.selectable_value(file_format_tmp, ".ico".to_string(), "ICO");
                    ui.selectable_value(file_format_tmp, ".hdr".to_string(), "HDR");
                    ui.selectable_value(file_format_tmp, ".openexr".to_string(), "OPENEXR");
                    ui.selectable_value(file_format_tmp, ".farbfeld".to_string(), "FARBFELD");
                    ui.selectable_value(file_format_tmp, ".avif".to_string(), "AVIF");
                    ui.selectable_value(file_format_tmp, ".qoi".to_string(), "QOI");
                });

                ui.add_space(10.0);

                Grid::new("123").show(ui, |ui| {
                    ui.label("DEFAULT PATH");
                    ui.end_row();

                    let button_text1 = "Choose default path";
                    let button_text2 = "Change default path";
                    //let button_text1 = "Choose file name";
                    //let button_text2 = "Change file name";
                    let button_text = if *save_path_tmp == PathBuf::default() {button_text1} else {button_text2};

                    if ui.button(button_text).clicked(){
                        let p = FileDialog::new().set_directory("/").pick_folder();
                        if(p.is_none()) { }
                        else{
                            *save_path_tmp=p.unwrap();
                        }                         
                    }

                    ui.end_row();
                    ui.end_row();

                    ui.label("CHOOSE FILE NAME");
                    ui.end_row();
                    ui.add(egui::TextEdit::singleline(name_convention_tmp));
                    //aggiungere la parte relativa alle convenzioni sul nome del file da salvare (con auto incremento)
                });

                ui.add_space(30.0);

                if ui.button("Chiudi").clicked(){
                    *save_path_tmp = save_path.clone();
                    *file_format_tmp = file_format.clone();
                    *name_convention_tmp = name_convention.clone();
                    *schermata = Schermata::Home;
                }

                ui.set_enabled((*save_path != save_path_tmp.clone()) || (*file_format != file_format_tmp.clone()) || (*name_convention != *name_convention_tmp));

                if ui.button("Salva modifiche").clicked(){
                    *save_path = save_path_tmp.clone();
                    *file_format = file_format_tmp.clone(); 
                    *name_convention = name_convention_tmp.clone();

                    *update_file = true; //in order to update the default initial settings
                    *schermata = Schermata::Home; 
                }
            });
}


fn set_image_gui_visible (window_size :egui::Vec2, prop :f32) -> egui::Vec2 {
    let mut  size = egui::Vec2::new(0.0, 0.0);
    size.x = window_size.x * 0.8;
    size.y = size.x / prop;
    if size.y >= window_size.y * 0.8 {
        size.y = window_size.y * 0.8;
        size.x = size.y * prop;
    }
    size
}

pub fn hotkey_to_String(modifier: Modifiers, key: Code) -> String{
    let mut mystr = String::from("");

    match modifier {
        Modifiers::ALT => mystr.push_str("ALT + "),
        Modifiers::CONTROL => mystr.push_str("CONTROL + "), 
        Modifiers::SHIFT => mystr.push_str("SHIFT + "),
        _ => mystr.push_str(""),
    }

    match key {
        Code::KeyA => mystr.push_str("A"),
        Code::KeyB => mystr.push_str("B"),
        Code::KeyC => mystr.push_str("C"),
        Code::KeyD => mystr.push_str("D"),
        Code::KeyE => mystr.push_str("E"),
        Code::KeyF => mystr.push_str("F"),
        Code::KeyG => mystr.push_str("G"),
        Code::KeyH => mystr.push_str("H"),
        Code::KeyI => mystr.push_str("I"),
        Code::KeyJ => mystr.push_str("J"),
        Code::KeyK => mystr.push_str("K"),
        Code::KeyL => mystr.push_str("L"),
        Code::KeyM => mystr.push_str("M"),
        Code::KeyN => mystr.push_str("N"),
        Code::KeyO => mystr.push_str("O"),
        Code::KeyP => mystr.push_str("P"),
        Code::KeyQ => mystr.push_str("Q"),
        Code::KeyR => mystr.push_str("R"),
        Code::KeyS => mystr.push_str("S"),
        Code::KeyT => mystr.push_str("T"),
        Code::KeyU => mystr.push_str("U"),
        Code::KeyV => mystr.push_str("V"),
        Code::KeyW => mystr.push_str("W"),
        Code::KeyX => mystr.push_str("X"),
        Code::KeyY => mystr.push_str("Y"),
        Code::KeyZ => mystr.push_str("Z"),
        Code::F1 => mystr.push_str("F1"),
        Code::F2 => mystr.push_str("F2"),
        Code::F3 => mystr.push_str("F3"),
        Code::F5 => mystr.push_str("F5"),
        Code::F6 => mystr.push_str("F6"),
        Code::F7 => mystr.push_str("F7"),
        Code::F8 => mystr.push_str("F8"),
        Code::F9 => mystr.push_str("F9"),
        Code::F10 => mystr.push_str("F10"),
        Code::F11 => mystr.push_str("F11"),
        Code::F12 => mystr.push_str("F12"),
        _ => mystr.push_str(""),
    }

    return mystr;
}

pub fn String_to_hotkey(my_string: String) -> (Modifiers, Code){
    let mod_and_key: Vec<&str> = my_string.split("+").collect();
    let mut result : (Modifiers, Code) = (Modifiers::default(), Code::default());

    match mod_and_key[0].trim() {
        "ALT" => result.0 = Modifiers::ALT,
        "CONTROL" => result.0 = Modifiers::CONTROL, 
        "SHIFT" => result.0 = Modifiers::SHIFT,
        _ => panic!("miao"),
    }

    match mod_and_key[1].trim() {
        "A" => result.1 = Code::KeyA,
        "B" => result.1 = Code::KeyB,
        "C" => result.1 = Code::KeyC,
        "D" => result.1 = Code::KeyD,
        "E" => result.1 = Code::KeyE,
        "F" => result.1 = Code::KeyF,
        "G" => result.1 = Code::KeyG,
        "H" => result.1 = Code::KeyH,
        "I" => result.1 = Code::KeyI,
        "J" => result.1 = Code::KeyJ,
        "K" => result.1 = Code::KeyK,
        "L" => result.1 = Code::KeyL,
        "M" => result.1 = Code::KeyM,
        "N" => result.1 = Code::KeyN,
        "O" => result.1 = Code::KeyO,
        "P" => result.1 = Code::KeyP,
        "Q" => result.1 = Code::KeyQ,
        "R" => result.1 = Code::KeyR,
        "S" => result.1 = Code::KeyS,
        "T" => result.1 = Code::KeyT,
        "U" => result.1 = Code::KeyU,
        "V" => result.1 = Code::KeyV,
        "W" => result.1 = Code::KeyW,
        "X" => result.1 = Code::KeyX,
        "Y" => result.1 = Code::KeyY,
        "Z" => result.1 = Code::KeyZ,
        "F1" => result.1 = Code::KeyA,
        "F2" => result.1 = Code::KeyB,
        "F3" => result.1 = Code::KeyC,
        "F5" => result.1 = Code::KeyE,
        "F6" => result.1 = Code::KeyF,
        "F7" => result.1 = Code::KeyG,
        "F8" => result.1 = Code::KeyH,
        "F9" => result.1 = Code::KeyI,
        "F10" => result.1 = Code::KeyA,
        "F11" => result.1 = Code::KeyB,
        "F12" => result.1 = Code::KeyC,
        _ => panic!("miao"),
    }  

    return result;
}

