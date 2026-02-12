cargo test
cargo build --release
cargo build --target x86_64-pc-windows-gnu --release
mv target/x86_64-pc-windows-gnu/release/light_pdf.exe /mnt/c/Users/innav
/mnt/c/Users/innav/light_pdf.exe