fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource.set_language(0x0409);
        resource
            .compile()
            .expect("failed to embed Windows version information");
    }
}
