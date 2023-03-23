use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["src/hello_world.proto"], &["src/"])?;
    Ok(())
}
