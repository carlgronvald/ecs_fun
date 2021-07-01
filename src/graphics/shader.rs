use super::TextureManager;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fs;
use super::uniform_data::{UniformData, UniformValue};
use strum::{EnumCount, EnumIter};

#[derive(Clone)]
pub enum ProgramType {
    Graphics,
}

///TODO: Figure out a way to unimplement Send and Sync for Shader
pub struct Shader {
    program_id: u32,
    uniform_locations: HashMap<String, i32>,
    metadata: ShaderMetadata,
}

#[derive(Clone, Copy, Debug, EnumCount, EnumIter)]
pub enum ShaderIdentifier {
    Default,
}

impl ShaderIdentifier {
    pub fn name(&self) -> &'static str{
        match self {
            ShaderIdentifier::Default => "Default"
        }
    }
    pub fn extensionless_path(&self) -> &'static str{
        match self {
            ShaderIdentifier::Default => "shaders/default"
        }
    }

}
#[derive(Clone)]
pub struct ShaderMetadata {
    pub identifier: ShaderIdentifier,
    /// The uniforms that this shader needs filled out
    /// First string is the name of the uniform, second is the file and line at which it is found (for error reports)
    pub required_uniforms: Vec<(String, String)>,
    pub shader_type: ProgramType,
}

impl Shader {
    unsafe fn compile_shader(source: &CStr, shader_type: u32) -> Result<u32, String> {
        if shader_type != gl::VERTEX_SHADER
            && shader_type != gl::FRAGMENT_SHADER
            && shader_type != gl::COMPUTE_SHADER
        {
            return Err(String::from(
                "Invalid shader type! Only allowed types are vertex, fragment, and compute!",
            ));
        }

        let id = gl::CreateShader(shader_type);

        gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        gl::CompileShader(id);

        let mut success: gl::types::GLint = 1;
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            //Get the length of the error message
            let mut len: gl::types::GLint = 0;
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);

            let error = create_whitespace_cstring_with_len(len as usize);

            gl::GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar,
            );

            return Err(error.to_string_lossy().into_owned());
        }

        Ok(id)
    }

    unsafe fn load_shader(
        file: &str,
        shader_type: u32,
    ) -> Result<(u32, Vec<(String, String)>), String> {
        let shader_source = match fs::read_to_string(file) {
            Ok(v) => v,
            Err(e) => {
                return Err(format!("Couldn't open file {}. {:?}", file, e));
            }
        };
        let shader_source = shader_source + "\0";

        let id = match Self::compile_shader(
            &CStr::from_bytes_with_nul(shader_source.as_bytes()).unwrap(),
            shader_type,
        ) {
            Ok(vsid) => vsid,
            Err(s) => return Err(s),
        };

        Ok((id, Self::find_uniforms(&shader_source, file)))
    }

    pub unsafe fn new(identifier: ShaderIdentifier) -> Result<Shader, String> {
        Shader::new_graphical(identifier)
    }

    unsafe fn new_graphical(identifier: ShaderIdentifier) -> Result<Shader, String> {
        let name = identifier.name();
        let extensionless_path = crate::ASSETS_PATH.join(identifier.extensionless_path());
        let vertex_file = extensionless_path
            .with_extension("vert")
            .to_str()
            .unwrap()
            .to_owned();
        let fragment_file = extensionless_path
            .with_extension("frag")
            .to_str()
            .unwrap()
            .to_owned();
        println!("Loading shader {}", name);
        let (vsid, mut vsuniforms) = match Self::load_shader(&vertex_file, gl::VERTEX_SHADER) {
            Ok(id) => id,
            Err(s) => return Err(s),
        };
        let (fsid, mut fsuniforms) = match Self::load_shader(&fragment_file, gl::FRAGMENT_SHADER) {
            Ok(id) => id,
            Err(s) => return Err(s),
        };

        vsuniforms.append(&mut fsuniforms);
        let required_uniforms = vsuniforms;

        let program_id = gl::CreateProgram();

        gl::AttachShader(program_id, vsid);
        gl::AttachShader(program_id, fsid);
        gl::LinkProgram(program_id);

        let mut success: gl::types::GLint = 1;
        gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);

            let error = create_whitespace_cstring_with_len(len as usize);

            gl::GetProgramInfoLog(
                program_id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar,
            );

            return Err(error.to_string_lossy().into_owned());
        }

        //TODO: Why do I validate the program?
        gl::ValidateProgram(program_id);

        gl::DetachShader(program_id, vsid);
        gl::DetachShader(program_id, fsid);
        gl::DeleteShader(vsid);
        gl::DeleteShader(fsid);

        let uniform_locations = Shader::uniform_hashmap(&required_uniforms, program_id);

        gl::UseProgram(0);

        Ok(Shader {
            program_id,
            uniform_locations,
            metadata: ShaderMetadata {
                identifier,
                required_uniforms,
                shader_type: ProgramType::Graphics,
            },
        })
    }


    unsafe fn uniform_hashmap(
        required_uniforms: &[(String, String)],
        program_id: u32,
    ) -> HashMap<String, i32> {
        let mut uniform_locations: HashMap<String, i32> = HashMap::new();

        gl::UseProgram(program_id);
        for entry in required_uniforms {
            let ename = format!("{}{}", entry.0, "\0");
            let ename = ename.as_bytes();
            let ename = CStr::from_bytes_with_nul(ename).unwrap();
            let id =
                gl::GetUniformLocation(program_id, (ename.as_ptr()) as *const gl::types::GLchar);

            println!(
                "Creating a uniform location for uniform {:?} at {}",
                ename, id
            );
            uniform_locations.insert(String::from(&entry.0), id);
        }

        uniform_locations
    }

    ///TODO: This should work for any valid notation.
    fn find_uniforms(source: &str, filename: &str) -> Vec<(String, String)> {
        let mut uniforms: Vec<(String, String)> = Vec::new();
        let mut counter = 0;
        for line in source.lines() {
            counter += 1;
            if line.starts_with("uniform") {
                let next = &line[8..];
                let re = regex::Regex::new(r"\w+").unwrap();
                let mut ms = re.captures_iter(next);
                ms.next();
                if let Some(type_name) = ms.next() {
                    uniforms.push((
                        String::from(&type_name[0]),
                        format!("{}:{}", filename, counter),
                    ));
                }
            }
        }

        uniforms
    }

    pub unsafe fn bind(&self) {
        gl::UseProgram(self.program_id);
    }

    pub unsafe fn unbind(&self) {
        gl::UseProgram(0);
    }

    pub fn get_metadata(&self) -> &ShaderMetadata {
        &self.metadata
    }
}

/// Tests don't have a GL Context, so they can't drop the shader.
/// From outside the shader::test module, shaders can only be instantiated by the unsafe function Shader::new,
/// which requires a GL context.
#[cfg(not(test))]
impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program_id);
        }
    }
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    // allocate buffer of correct size
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    // fill it with len spaces
    buffer.extend([b' '].iter().cycle().take(len));
    // convert buffer to CString
    unsafe { CString::from_vec_unchecked(buffer) }
}

pub struct ShaderManager {
    /// Shaders indexed by their count in the ShaderIdentifier enum
    shaders: Vec<Shader>,
    // Name of chosen shader.
    bound_shader: Option<ShaderIdentifier>,
}

impl<'a> ShaderManager {
    pub fn new(shaders: Vec<Shader>) -> ShaderManager {
        let mut count = 0;
        for shader in &shaders {
            if shader.metadata.identifier as usize != count {
                panic!("Loading shaders into the shader manager out of order!")
            }
            count += 1;
        }
        if count < ShaderIdentifier::COUNT {
            panic!("Not enough shaders were supplied for the shader manager. Likely some loading failed; look further up for compilation errors.")
        }
        ShaderManager {
            shaders,
            bound_shader: None,
        }
    }

    //pub unsafe fn add_shader(&mut self, shader: Shader) {
    //    self.shaders.push(shader);
    //}

    pub unsafe fn bind_shader(&mut self, shader: ShaderIdentifier) -> Result<String, String> {
        if let Some(s) = self.bound_shader.take() {
            self.shaders[s as usize].unbind();
        }
        self.shaders[shader as usize].bind(); //TODO: MAKE NO SHADER MESSAGE
        self.bound_shader = Some(shader);

        Ok(format!(""))
    }

    pub unsafe fn uniforms(
        &mut self,
        uniforms: &UniformData,
        texture_manager: &TextureManager,
    ) -> Result<(), String> {
        //TODO: THERE COULD BE NO CURRENT SHADER
        if self.bound_shader.is_none() {
            return Err(String::from(
                "Uniforms were sent, but there's no bound shader!",
            ));
        }

        let s = &self.shaders[self.bound_shader.unwrap() as usize];
        let mut texture_slot: i32 = 0;

        for uniform in uniforms.get_uniforms() {
            let loc = s.uniform_locations.get(uniform.location);
            match uniform.value {
                UniformValue::float(value) => gl::Uniform1f(*loc.unwrap(), *value),
                UniformValue::int(value) => gl::Uniform1i(*loc.unwrap(), *value),
                UniformValue::uint(value) => gl::Uniform1ui(*loc.unwrap(), *value),
                UniformValue::mat2(value) => gl::UniformMatrix2fv(
                    *loc.unwrap(),
                    1,
                    gl::FALSE,
                    (value.as_ptr()) as *const f32,
                ),
                UniformValue::mat3(value) => gl::UniformMatrix3fv(
                    *loc.unwrap(),
                    1,
                    gl::FALSE,
                    (value.as_ptr()) as *const f32,
                ),
                UniformValue::mat4(value) => gl::UniformMatrix4fv(
                    *loc.unwrap(),
                    1,
                    gl::FALSE,
                    (value.as_ptr()) as *const f32,
                ),
                UniformValue::vec2(value) => {
                    gl::Uniform2fv(*loc.unwrap(), 1, (value.as_ptr()) as *const f32)
                }

                UniformValue::vec3(value) => {
                    gl::Uniform3fv(*loc.unwrap(), 1, (value.as_ptr()) as *const f32)
                }

                UniformValue::vec4(value) => {
                    gl::Uniform4fv(*loc.unwrap(), 1, (value.as_ptr()) as *const f32)
                }

                UniformValue::ivec4(value) => {
                    gl::Uniform4iv(*loc.unwrap(), 1, (value.as_ptr()) as *const i32)
                }

                UniformValue::ivec3(value) => {
                    gl::Uniform3iv(*loc.unwrap(), 1, (value.as_ptr()) as *const i32)
                }

                UniformValue::ivec2(value) => {
                    gl::Uniform2iv(*loc.unwrap(), 1, (value.as_ptr()) as *const i32)
                }

                UniformValue::uvec4(value) => {
                    gl::Uniform4uiv(*loc.unwrap(), 1, (value.as_ptr()) as *const u32)
                }

                UniformValue::uvec3(value) => {
                    gl::Uniform3uiv(*loc.unwrap(), 1, (value.as_ptr()) as *const u32)
                }

                UniformValue::uvec2(value) => {
                    gl::Uniform2uiv(*loc.unwrap(), 1, (value.as_ptr()) as *const u32)
                }

                UniformValue::texture(value) => {
                    let tex = texture_manager.get_texture(value);
                    gl::ActiveTexture(gl::TEXTURE0 + texture_slot as u32);
                    tex.bind();
                    gl::Uniform1i(*loc.unwrap(), texture_slot);
                    texture_slot += 1;
                }
            }
        }

        Ok(())
    }

    pub fn get_active_shader_name(&self) -> Option<String> {
        self.bound_shader.map(|index| String::from(index.name()))
    }

    /// Returns the metadata of every Shader indexed by its ShaderIdentifier.
    pub fn get_shader_metadata(&self) -> Vec<ShaderMetadata> {
        let mut res = Vec::new();
        for shader in &self.shaders {
            res.push(shader.metadata.clone());
        }
        res
    }
}