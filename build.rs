use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
  let glslc = Path::new(core::env!("VULKAN_SDK"))
  .join("Bin")
  .join("glslc.exe");
  println!("Using glslc from {:?}", glslc);
  
  let current_dir = std::env::current_dir().unwrap();
  let shader_dir = Path::new("src").join("gfx").join("shaders");  
  let read_dir = fs::read_dir(shader_dir.as_os_str()).unwrap();
  let mut shader_paths = Vec::new();
  for dir in read_dir {
    if let Ok(entry) = dir {
      let path = entry.path();
      if let Some(ext) = path.extension() {
        if ext == "vert" || ext == "frag" {
          shader_paths.push(current_dir.join(path).display().to_string());
        }
      }
    }
  }
  
  for path in shader_paths {
    let in_path = format!("{}", path);
    let spv_path = format!("{}.spv", path);
    println!("Compiling {}...", in_path);
    Command::new(glslc.as_os_str())
    .arg(in_path)
    .arg("-o")
    .arg(spv_path)
    .output()
    .expect("Failed to execute process");
  }

  println!("cargo:rerun-if-changed=C:/Users/tkchanat/flux/src/gfx/shaders/");
}
