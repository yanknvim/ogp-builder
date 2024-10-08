use anyhow::Result;
use axum::extract::{State, Query};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize};

use ab_glyph::{point, Font, FontRef, Glyph, PxScale};
use image::{ImageFormat, Rgba};
use imageproc::drawing::draw_text_mut;
use std::io::Cursor;

use shuttle_persist::PersistInstance;

#[derive(Serialize, Deserialize)]
struct Payload {
    title: String,
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_persist::Persist] persist: PersistInstance
) -> shuttle_axum::ShuttleAxum {
    let route = Router::new().route("/image", get(image_handler)).with_state(persist);

    Ok(route.into())
}

#[axum_macros::debug_handler]
async fn image_handler(
    State(persist): State<PersistInstance>,
    Query(params): Query<Payload>,
) -> Result<impl IntoResponse, StatusCode> {
    if let Ok(bytes) = persist.load::<Vec<u8>>(&params.title) {
        let response = ([(header::CONTENT_TYPE, "image/png")], bytes);
        return Ok(response);
    }
    
    let mut img = image::open("./backgrounds/wave-haikei.png")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let font = FontRef::try_from_slice(include_bytes!("../fonts/LINESeedJP_A_TTF_Rg.ttf")).unwrap();

    let scale = PxScale { x: 50.0, y: 50.0 };
    draw_text_mut(
        &mut img,
        Rgba([255, 255, 255, 255]),
        10,
        5,
        scale,
        &font,
        "uneu.net",
    );

    let scale = PxScale { x: 80.0, y: 80.0 };
    let width = params
        .title
        .chars()
        .map(|c| {
            let glyph = font.glyph_id(c).with_scale(scale);
            let rect = font.glyph_bounds(&glyph);
            rect.width()
        })
        .sum::<f32>() as i32;
    let x = (img.width() as i32 - width) / 2 as i32;
    let y = ((img.height() - scale.y as u32) / 2) as i32;

    draw_text_mut(
        &mut img,
        Rgba([255, 255, 255, 255]),
        x,
        y,
        scale,
        &font,
        &params.title,
    );

    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    persist.save(&params.title, bytes.clone()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let response = ([(header::CONTENT_TYPE, "image/png")], bytes);
    Ok(response)
}
