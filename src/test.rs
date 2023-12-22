use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;
use web_sys::console;
use rand::Rng;

#[wasm_bindgen]
pub fn print_obj(js_obj: JsValue) {
    console::log_1(&js_obj);
}

#[wasm_bindgen]
pub fn log(s: &str) {
    let js_obj = JsValue::from_str(s);
    console::log_1(&js_obj);
}

#[wasm_bindgen]
pub fn test_draw(ctx: &CanvasRenderingContext2d){
    let block_pixel_size = 20;
    let pixel_width = 100;
    let pixel_height = 100;
    let grid_width = pixel_width / block_pixel_size;
    let grid_height = pixel_height / block_pixel_size;

    let num_positions = grid_width * grid_height;

    let mut rng = rand::thread_rng();
    (0..num_positions)
        .for_each(|item| {
            let row = item / grid_width;
            let col = item % grid_width;
            let color: JsValue = match rng.gen_range(0..=2) {
                0 => JsValue::from_str("green"),
                1 => JsValue::from_str("red"),
                _ => JsValue::from_str("blue"),
            };
            ctx.set_fill_style(&color);
            ctx.fill_rect((col * block_pixel_size) as f64, (row * block_pixel_size) as f64, block_pixel_size as f64, block_pixel_size as f64);
    });
    ctx.stroke();    
}