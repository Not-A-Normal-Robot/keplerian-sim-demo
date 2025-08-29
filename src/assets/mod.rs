use std::{borrow::Cow, sync::LazyLock};

use pastey::paste;
use three_d::egui::{Image, ImageSource, load::Bytes};

/// Arguments:
/// 1. The name of the `const LazyLock<Image<'static>>` to be available as.
///    That name, combined with `_SOURCE`, yields the `ImageSource<'static>` that you can use.
/// 2. A string containing the relative path to the image. A `./` is prepended automatically.
/// Usage:
/// ```
/// use_img!(PAUSED_IMAGE, "pause.svg");
/// ```
macro_rules! use_img {
    ($name:ident, $path:literal) => {
        paste! {
            pub(crate) const [<$name _SOURCE>]:
                ::three_d::egui::ImageSource<'static> = ::three_d::egui::ImageSource::Bytes {
                    uri: ::std::borrow::Cow::Borrowed(
                        concat!("bytes://", $path)
                    ),
                    bytes: ::three_d::egui::load::Bytes::Static(
                        include_bytes!(
                            concat!("./", $path)
                        )
                    )
                };
        }

        pub(crate) const $name: ::std::sync::LazyLock<::three_d::egui::Image<'static>> =
            ::std::sync::LazyLock::new(|| {
                ::three_d::egui::Image::new(paste! { [<$name _SOURCE>] })
            });
    };
}

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

use_img!(ELLIPSIS_IMAGE, "more.svg");
use_img!(TREE_LIST_IMAGE, "tree-list.svg");
use_img!(ADD_ORBIT_IMAGE, "add-orbit.svg");
use_img!(EDIT_ORBIT_IMAGE, "edit-orbit.svg");
