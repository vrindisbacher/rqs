fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .out_dir("src/message/") // Specify your desired output directory here
        .compile(&["proto/message.proto"], &["proto"])?;    

    tonic_build::configure()
        .out_dir("src/queue/") // Specify your desired output directory here
        .compile(&["proto/queue.proto"], &["proto"])?;    
    Ok(())
}
