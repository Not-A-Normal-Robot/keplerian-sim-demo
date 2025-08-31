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
        ::pastey::paste! {
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
                ::three_d::egui::Image::new(::pastey::paste! { [<$name _SOURCE>] })
            });
    };
}

use_img!(PAUSED_IMAGE, "pause.svg");
use_img!(PLAY_IMAGE, "play.svg");
use_img!(TIME_IMAGE, "time.svg");
use_img!(ELLIPSIS_IMAGE, "more.svg");
use_img!(TREE_LIST_IMAGE, "tree-list.svg");
use_img!(ADD_ORBIT_IMAGE, "add-orbit.svg");
use_img!(EDIT_ORBIT_IMAGE, "edit-orbit.svg");
use_img!(OPTIONS, "options.svg");
