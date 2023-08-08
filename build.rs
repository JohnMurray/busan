use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(
        &[
            "src/message/wrappers.proto",
            "src/actor/address.proto",
            "src/patterns/load_balancer.proto",
        ],
        &["src/"],
    )?;
    Ok(())
}
