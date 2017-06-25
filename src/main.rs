#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate multipart;

use std::io;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;

use rocket::response::NamedFile;
use rocket::{Request, Data, Outcome};
use rocket::data::{self, FromData};
use rocket::response::content::Plain;

use multipart::server::Multipart;

const STATIC_DIR: &'static str = "public/";

#[get("/")]
fn index() -> io::Result<NamedFile> {
    NamedFile::open("public/index.html")
}

#[get("/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("public/").join(file)).ok()
}

#[derive(Debug)]
struct MultipartData {
    image: Vec<u8>,
}

impl FromData for MultipartData {
    type Error = ();

    fn from_data(request: &Request, data: Data) -> data::Outcome<Self, Self::Error> {
        let ct = request.headers().get_one("Content-Type").expect("no content-type");
        let idx = ct.find("boundary=").expect("no boundary");
        let boundary = &ct[(idx + "boundary=".len())..];

        let mut d = Vec::new();
        data.stream_to(&mut d).expect("Unable to read");

        let mut mp = Multipart::with_body(Cursor::new(d), boundary);

        let mut image = None;

        mp.foreach_entry(|mut entry| {
            match entry.name.as_str() {
                "image" => {
                    let mut d = Vec::new();
                    let f = entry.data.as_file().expect("not file");
                    f.read_to_end(&mut d).expect("cant read");
                    image = Some(d);
                },
                other => panic!("No known key {}", other),
            }
        }).expect("Unable to iterate");

        let v = MultipartData {
            image: image.expect("image not set"),
        };

        Outcome::Success(v)
    }
}

#[post("/upload", data = "<data>")]
fn upload(data: MultipartData) -> String {
    let mut image = File::create(STATIC_DIR.to_owned() + "image.jpg").expect("Unable to create image");
    match image.write_all(&data.image) {
        Ok(_) => format!("{:?}", data),
        Err(err) => format!("{:?}", err)
    }
}

fn main() {
    rocket::ignite().mount("/", routes![index, files, upload]).launch();
}
