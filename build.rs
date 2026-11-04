fn main() {
    let target_s = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_s == "windows" {
        embed_resource::compile("icon.rc", embed_resource::NONE);
    }
}