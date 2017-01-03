extern crate ffmpeg;
use ffmpeg::util::hwaccel::HWAccel;

fn main() {
    ffmpeg::init().expect("ffmpeg init");

    for hwaccel in HWAccel::registred() {
        println!("{:#?}", hwaccel);
    }
}
