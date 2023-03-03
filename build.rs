use std::io::Result;
fn main() -> Result<()> {
    println!("running");
    prost_build::compile_protos(
        &[
            "examples/hello_world/hello_world.proto",
            "src/message/wrappers.proto",
        ],
        &["examples/hello_world/", "examples/ping_pong/", "src/"],
    )?;
    Ok(())
}
