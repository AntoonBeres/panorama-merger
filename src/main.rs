use actix_web::{App, Error, HttpResponse, HttpServer, web, error, get};
use actix_web::http::{StatusCode};
use actix_multipart::Multipart;

use futures::{StreamExt, TryStreamExt};
use std::io::Write;
use std::io::Cursor;
use std::sync::Mutex;
use std::collections::HashMap;

use opencv::prelude::*;
use opencv::{imgcodecs, stitching};
use opencv::core::*;

use serde::Deserialize;

const IMREAD_TYPE: imgcodecs::ImreadModes = imgcodecs::ImreadModes::IMREAD_COLOR;


fn encode_to_buff<'a>(
    ext: &str, 
    img: &dyn ToInputArray, 
    params: &Vector<i32> ) -> Result<Vector<u8>, &'a str>  {
    
        let mut buf: Vector<u8> = Vector::new();
        imgcodecs::imencode(ext, img, &mut buf, params).expect("Houston, we have a problem");
        Ok(buf)
}


async fn upload_pic(mut payload: Multipart, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    

    let mut images: Vector<Mat> = Vector::new();
    let mut panorama = Mat::default();
    let mut stitcher = stitching::Stitcher::create(stitching::Stitcher_Mode::PANORAMA).unwrap();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let f: Result<Cursor<Vec<u8>>, error::BlockingError<&str>> = web::block(|| Ok(Cursor::new(Vec::new())))
            .await;
        let mut f = f.unwrap();

        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&data).map(|_| f)).await?;
        }

        println!("GOT PICTURE");
        let f = f.into_inner();
        let picture: Vector<u8> = Vector::from_iter(f);
        let picture: Mat = imgcodecs::imdecode(&picture, IMREAD_TYPE as i32).unwrap();
        images.push(picture);
    }
    let check = stitcher.stitch(&images, &mut panorama);
    let _check2 = match check {
        Ok(file) => file,
        Err(error) => panic!("kutprobleem!: {:?}", error),
    };
    let mut flags: Vector<i32> = Vector::new();
    flags.push(imgcodecs::ImwriteFlags::IMWRITE_JPEG_QUALITY as i32);

    let mut counter = data.counter.lock().unwrap();
    *counter += 1;
    let image_id: u32 = *counter;
    drop(counter);

    let mut images_hashmap = data.image_buffers.lock().unwrap();


    let buffer_succes =  web::block( move || encode_to_buff(".jpg", &panorama, &flags)).await.unwrap();


    images_hashmap.insert(image_id, Box::new(buffer_succes));
    drop(images_hashmap);
    
    
    let file_id = format!("image/{}", image_id);
    
    let html_body = format!("
    <!DOCTYPE html>
    <html lang=\"en\">
      <head>
        <meta charset=\"UTF-8\" />
        <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />
        <meta http-equiv=\"X-UA-Compatible\" content=\"ie=edge\" />
    
        <title>Image-Result</title>
      </head>
      <body>
        <img src=\"{}\">
      </body>
    </html>", &file_id);

    Ok(
        HttpResponse::build(StatusCode::OK)
            .body(html_body)
    )
}
async fn get_image(info: web::Path<u32>, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let image_index: u32 = info.into_inner();
    let pathname = format!("../tmp/panorama{}.jpg", image_index);
    //let byte_array: &[u8] = panorama_merger::load_bytes!(&pathname[..]);


    let images_hashmap = data.image_buffers.lock().unwrap();
    let byte_array = images_hashmap[&image_index].to_vec();
    //let byte_array2 = include_bytes!(&pathname[..]);
    Ok(
        HttpResponse::build(StatusCode::OK)
            .content_type("image/jpeg")
            .body(byte_array)
    )
}

async fn home() -> Result<HttpResponse, Error> {
    Ok(
        HttpResponse::build(StatusCode::OK)
            .content_type("text/html; charset=utf-8")
            .body(include_str!("../templates/index.html"))
    )

}

#[derive(Deserialize)]
struct Info {
    name: String,
}

struct AppState{
    counter: Mutex<u32>,
    image_buffers: Mutex<HashMap<u32, Box<Vector<u8> > > >
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let counter = web::Data::new(AppState {
        counter: Mutex::new(0),
        image_buffers: Mutex::new(HashMap::new())
    });
    HttpServer::new(move || {
        App::new()
        .app_data(counter.clone())
        .service(
            // prefixes all resources and routes attached to it...
            web::scope("/app")
                // ...so this handles requests for `GET /app/index.html`
                .route("/index.html", web::get().to(home))
                .route("/image/{number}", web::get().to(get_image))
                .route("/index.html", web::post().to(upload_pic))
        )
        .route("/tmp/{image_file}", web::get().to(get_image))
        //.route("/app/image", web::post().to(upload_pic))
    })
    .bind("127.0.0.1:8080")?
    //.bind("192.168.0.148:80")?
    .run()
    .await
}
