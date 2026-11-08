fn main() {
    topcoat::icon::iconify::BuildConfig::new()
        .icon_set("lucide")
        .stage()
        .expect("stage Lucide icons");
}
