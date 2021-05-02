use anyhow::{bail, Context, Result};
use glob::glob;
use std::fs::{read_to_string, write};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug)]
struct Shader {
    src: String,
    src_path: PathBuf,
    kind: shaderc::ShaderKind,
}

impl Shader {
    pub fn new(src_path: PathBuf) -> Result<Self> {
        let ext = src_path
            .extension()
            .context("No extension")?
            .to_str()
            .context("Invalid path utf-8")?;

        let kind = match ext {
            "vert" => shaderc::ShaderKind::Vertex,
            "frag" => shaderc::ShaderKind::Fragment,
            "comp" => shaderc::ShaderKind::Compute,
            _ => bail!("Unknown shader kind: {}", src_path.display()),
        };

        let src = read_to_string(&src_path)?;

        Ok(Shader {
            src,
            src_path,
            kind,
        })
    }

    pub fn compile(&self, compiler: &mut shaderc::Compiler, opts: &shaderc::CompileOptions) -> Result<Vec<u8>> {
        let artifact: shaderc::CompilationArtifact = compiler.compile_into_spirv(
            &self.src,
            self.kind,
            self.src_path.to_str().context("Invalid path utf-8")?,
            "main",
            Some(opts),
        )?;
        Ok(artifact.as_binary_u8().to_vec())
    }

    pub fn dst_path(&self) -> Result<PathBuf> {
        let mut dst = self.src_path.clone();
        dst.pop();
        dst.push("compiled");
        let mut name = self.src_path.file_name().unwrap().to_owned();
        name.push(std::ffi::OsString::from_str(".spv").unwrap());
        dst.push(name);

        Ok(dst)
    }
}

fn compile(shader: &Shader, compiler: &mut shaderc::Compiler, opts: &shaderc::CompileOptions) -> Result<()> {
    let path = shader
        .src_path
        .as_os_str()
        .to_str()
        .context(format!("Invalid path: {}", shader.src_path.display()))?;
    println!("cargo:rerun-if-changed={}", path);

    let out = shader.compile(compiler, opts)?;

    // TODO: Auto mkdir!

    write(shader.dst_path()?, out)?;

    Ok(())
}

fn include_file(path: &str, ty: shaderc::IncludeType, requester: &str, inc_depth: usize) -> shaderc::IncludeCallbackResult {
    if inc_depth > 100 {
        Err("Include overflow".to_string())
    } else {
        let mut inc_path = match ty {
            shaderc::IncludeType::Relative => {
                let mut p = PathBuf::from(requester);
                p.pop();
                p
            }
            shaderc::IncludeType::Standard => {
                PathBuf::from("glsl-lib")
            }
        };
        inc_path.push(path);

        if inc_path.is_file() {
            let cont = read_to_string(&inc_path).map_err(|e| format!("{}", e))?;
            Ok(shaderc::ResolvedInclude {
                resolved_name: format!("{}", inc_path.display()),
                content: cont,
            })
        } else {
            Err(format!("{:?} not found", path))
        }

    }
}

fn main() -> Result<()> {
    let mut shader_paths = [
        glob("./src/**/*.vert")?,
        glob("./src/**/*.frag")?,
        glob("./src/**/*.comp")?,
    ];
    let mut opts = shaderc::CompileOptions::new().context("Couln't create compile options")?;

    let mut compiler = shaderc::Compiler::new().context("Couldn't create compiler")?;

    opts.set_include_callback(include_file);

    let shaders = shader_paths
        .iter_mut()
        .flatten()
        .map(|glob_res| Shader::new(glob_res?))
        .collect::<Result<Vec<_>>>()?;

    for shader in shaders {
        compile(&shader, &mut compiler, &opts).context(format!("Compiling {}", shader.src_path.display()))?;
    }

    Ok(())
}
