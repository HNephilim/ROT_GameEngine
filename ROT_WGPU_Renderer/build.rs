use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use glob::glob;
use std::env;
use std::fs::{read_to_string, write};
use std::path::PathBuf;

struct ShaderData {
    src: String,
    src_path: PathBuf,
    spirv_path: PathBuf,
    kind: shaderc::ShaderKind,
}

impl ShaderData {
    pub fn load(src_path: PathBuf) -> Result<Self> {
        let extension = src_path
            .extension()
            .context("File has no extension")?
            .to_str()
            .context("Extension cannot be converted to &str")?;

        let kind = match extension {
            "vert" => shaderc::ShaderKind::Vertex,
            "frag" => shaderc::ShaderKind::Fragment,
            "comp" => shaderc::ShaderKind::Compute,
            _ => bail!("Unsuported shader: {}", src_path.display()),
        };

        let src = read_to_string(src_path.clone())?;

        let spirv_path = src_path.with_extension(format!("{}.spv", extension));

        Ok(Self {
            src,
            src_path,
            spirv_path,
            kind,
        })
    }
}

fn main() -> Result<()> {
    //collect all shaders recursively within /src/
    let mut shader_path = [
        glob("shaders/*.vert")?,
        glob("shaders/*.frag")?,
        glob("shaders/*.comp")?,
    ];

    //TODO This could be paralelized
    let shaders = shader_path
        .iter_mut()
        .flatten()
        .map(|glob_result| ShaderData::load(glob_result?))
        .collect::<Vec<Result<_>>>()
        .into_iter()
        .collect::<Result<Vec<_>>>()?;

    let mut compiler = shaderc::Compiler::new().context("Unable to create shader compiler")?;

    for shader in shaders {
        // This tells cargo to rerun this script if something in /src/ changes.
        println!(
            "cargo:rerun-if-changed={}",
            shader.src_path.as_os_str().to_str().unwrap()
        );

        let compiled = compiler.compile_into_spirv(
            &shader.src,
            shader.kind,
            &shader.src_path.to_str().unwrap(),
            "main",
            None,
        )?;
        write(shader.spirv_path, compiled.as_binary_u8())?;
    }

    Ok(())
}
