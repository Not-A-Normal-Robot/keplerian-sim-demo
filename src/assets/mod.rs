use std::{borrow::Cow, sync::LazyLock};

use three_d::egui::{Image, ImageSource, load::Bytes};

pub(crate) const PAUSED_IMAGE_SOURCE: ImageSource<'static> = ImageSource::Bytes {
    uri: Cow::Borrowed("bytes://pause.svg"),
    bytes: Bytes::Static(include_bytes!("./pause.svg")),
};
pub(crate) const PLAY_IMAGE_SOURCE: ImageSource<'static> = ImageSource::Bytes {
    uri: Cow::Borrowed("bytes://play.svg"),
    bytes: Bytes::Static(include_bytes!("./play.svg")),
};
pub(crate) const TIME_IMAGE_SOURCE: ImageSource<'static> = ImageSource::Bytes {
    uri: Cow::Borrowed("bytes://time.svg"),
    bytes: Bytes::Static(include_bytes!("./time.svg")),
};

pub(crate) static PAUSED_IMAGE: LazyLock<Image<'static>> =
    LazyLock::new(|| Image::new(PAUSED_IMAGE_SOURCE));
pub(crate) static PLAY_IMAGE: LazyLock<Image<'static>> =
    LazyLock::new(|| Image::new(PLAY_IMAGE_SOURCE));
pub(crate) static TIME_IMAGE: LazyLock<Image<'static>> =
    LazyLock::new(|| Image::new(TIME_IMAGE_SOURCE));
