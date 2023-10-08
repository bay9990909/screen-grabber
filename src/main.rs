
mod gui;
mod screen;

use eframe::egui;
use eframe::egui::TextureHandle;
use image::RgbaImage;
fn main() {
    
    //let mut ctx = egui::Context::default();
    let native_options = eframe::NativeOptions {
        initial_window_size: Some([640.0, 360.0].into()),
        min_window_size: Some([400.0, 320.0].into()),
        resizable: true,
        
        ..Default::default()
    };
    let options = eframe::NativeOptions {
        always_on_top: false,
        maximized: false,
        decorated: true,
        drag_and_drop_support: true,
        icon_data: None,
        initial_window_pos: None,
        initial_window_size: Some([640.0, 360.0].into()),
        min_window_size: None,
        max_window_size: None,
        resizable: true,
        transparent: true,
        vsync: true,
        multisampling: 0,
        depth_buffer: 0,
        stencil_buffer: 0,
        fullscreen: false,
        ..Default::default()
    };
    eframe::run_native("Cattura", native_options, Box::new(|cc| Box::new(Windows::new(cc))));
    
}

#[derive(Default)]
struct Windows {
    schermata: Schermata,
    image : RgbaImage,
    texture : Option<TextureHandle>,
}

#[derive(Default,Debug)]
pub enum Schermata {
    #[default]
    Home,
    Edit,
}

impl Windows {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
         //cc.egui_ctx.set_pixels_per_point(1.0);
        //println!("{:?}",cc.egui_ctx.pixels_per_point());
        
        Self::default()
    }
}

impl eframe::App for Windows {
   fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
    eframe::egui::Context::set_pixels_per_point(ctx, 1.0);
    match self.schermata {
        Schermata::Home => gui::home(ctx,&mut self.schermata, &mut self.image, &mut self.texture, frame),
        Schermata::Edit => gui::edit(ctx),
    }
        //println!("{:?}",frame.info().window_info.size);
        println!("proporzione: {:?}",egui::Context::pixels_per_point(ctx));
   }
}






    
