use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(
        &[
            "src/message/wrappers.proto",
            "src/message/system.proto",
            "src/actor/address.proto",
        ],
        &["src/"],
    )?;
    Ok(())
}
