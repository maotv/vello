// Copyright 2023 The Vello authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[path = "src/compile/mod.rs"]
mod compile;
#[path = "src/types.rs"]
mod types;

use std::env;
use std::fmt::Write;
use std::path::Path;

use compile::ShaderInfo;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("shaders.rs");
    let mut shaders = compile::ShaderInfo::from_dir("../../shader");
    // Drop the HashMap and sort by name so that we get deterministic order.
    let mut shaders = shaders.drain().collect::<Vec<_>>();
    shaders.sort_by(|x, y| x.0.cmp(&y.0));
    let mut buf = String::default();
    write_types(&mut buf, &shaders).unwrap();
    write_shaders(&mut buf, &shaders).unwrap();
    std::fs::write(&dest_path, &buf).unwrap();
    println!("cargo:rerun-if-changed=../shader");
}

fn write_types(buf: &mut String, shaders: &[(String, ShaderInfo)]) -> Result<(), std::fmt::Error> {
    writeln!(buf, "pub struct Shaders<'a> {{")?;
    for (name, _) in shaders {
        writeln!(buf, "    pub {name}: ComputeShader<'a>,")?;
    }
    writeln!(buf, "}}")?;
    writeln!(buf, "pub struct Pipelines<T> {{")?;
    for (name, _) in shaders {
        writeln!(buf, "    pub {name}: T,")?;
    }
    writeln!(buf, "}}")?;
    writeln!(buf, "impl<T> Pipelines<T> {{")?;
    writeln!(buf, "    pub fn from_shaders<H: PipelineHost<ComputePipeline = T>>(shaders: &Shaders, device: &H::Device, host: &mut H) -> Result<Self, H::Error> {{")?;
    writeln!(buf, "        Ok(Self {{")?;
    for (name, _) in shaders {
        writeln!(
            buf,
            "            {name}: host.new_compute_pipeline(device, &shaders.{name})?,"
        )?;
    }
    writeln!(buf, "        }})")?;
    writeln!(buf, "    }}")?;
    writeln!(buf, "}}")?;
    Ok(())
}

fn write_shaders(
    buf: &mut String,
    shaders: &[(String, ShaderInfo)],
) -> Result<(), std::fmt::Error> {
    writeln!(buf, "mod gen {{")?;
    writeln!(buf, "    use super::*;")?;
    writeln!(buf, "    use BindType::*;")?;
    writeln!(buf, "    pub const SHADERS: Shaders<'static> = Shaders {{")?;
    for (name, info) in shaders {
        let bind_tys = info
            .bindings
            .iter()
            .map(|binding| binding.ty)
            .collect::<Vec<_>>();
        let wg_bufs = &info.workgroup_buffers;
        writeln!(buf, "        {name}: ComputeShader {{")?;
        writeln!(buf, "            name: Cow::Borrowed({:?}),", name)?;
        writeln!(
            buf,
            "            workgroup_size: {:?},",
            info.workgroup_size
        )?;
        writeln!(buf, "            bindings: Cow::Borrowed(&{:?}),", bind_tys)?;
        writeln!(
            buf,
            "            workgroup_buffers: Cow::Borrowed(&{:?}),",
            wg_bufs
        )?;
        if cfg!(feature = "wgsl") {
            writeln!(buf, "            wgsl: Cow::Borrowed(&{:?}),", info.source)?;
        }
        if cfg!(feature = "msl") {
            writeln!(
                buf,
                "            msl: Cow::Borrowed(&{:?}),",
                compile::msl::translate(info).unwrap()
            )?;
        }
        writeln!(buf, "        }},")?;
    }
    writeln!(buf, "    }};")?;
    writeln!(buf, "}}")?;
    Ok(())
}
