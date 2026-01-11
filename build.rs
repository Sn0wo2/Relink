use std::env;
use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=app.manifest");

    if env::var("CARGO_CFG_WINDOWS").is_ok() {
        compile_windows_resources()?;
    }

    Ok(())
}

fn compile_windows_resources() -> Result<(), Box<dyn Error>> {
    let mut res = winres::WindowsResource::new();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let manifest_path = Path::new(&manifest_dir).join("app.manifest");
    
    if let Some(path) = manifest_path.to_str() {
        res.set_manifest_file(path);
    }

    res.compile()?;

    handle_gnu_toolchain()?;

    Ok(())
}

fn handle_gnu_toolchain() -> Result<(), Box<dyn Error>> {
    let target = env::var("TARGET").unwrap_or_default();
    
    // 针对 GNU 工具链 (MinGW)，强制链接 resource.o
    // 防止链接器因为未引用符号而丢弃资源文件
    if target.contains("gnu") {
        let out_dir = env::var("OUT_DIR")?;
        println!("cargo:rustc-link-arg={}/resource.o", out_dir);
    }
    
    Ok(())
}