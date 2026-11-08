fn main() {
    topcoat::icon::iconify::BuildConfig::new().icon_set("lucide").stage().expect("stage preview icons");
    println!("cargo:rerun-if-changed=styles.css");
    println!("cargo:rerun-if-changed=../src");
    topcoat::tailwind::BuildConfig::new().input("styles.css").render().expect("compile preview stylesheet");
}
