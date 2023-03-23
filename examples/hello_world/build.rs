use std::io::Result;

fn main() -> Result<()> {
    let mut config = prost_build::Config::new();
    config
        .type_attribute(".", "#[derive(::busan::Message)]")
        .compile_protos(&["src/hello_world.proto"], &["src/"])?;
    Ok(())
}
