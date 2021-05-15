# panorama-merger
Panorama-merger for creating hdri's for wasm

Compilation instructions:

Dependencies:
  - Rust (make sure latest version is installed with "rustup update")
  - Vtk (install from your repositiries, on arch-linux: `sudo pacman -S vtk` 

To compile:

  `cargo build`
  
To compile and run (for testing):

  `cargo run 127.0.0.1:8080`

usage:
run the server, open browser and go to:
http://localhost:8080/app/index.html
