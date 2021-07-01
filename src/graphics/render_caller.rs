use super::{
    ShaderIdentifier, ShaderManager, TextureManager,
    VertexArray,
};
use super::{uniform_data::UniformData, vertex_pack::VertexPack};

const VERBOSE: bool = false;

///
/// TODO
/// This struct is in charge of rendering one round. Also in terms of setting opengl settings.
///
/// It needs to know what things change and what things stay the same, so it doesn't change anything that makes
/// opengl slow. I actually basically need a function for changing opengl things that knows if they're actually
/// changed.
///
pub struct RenderCaller {
    vertex_array: VertexArray,
    pub shader_manager: ShaderManager,
    texture_manager: TextureManager,
    screen_dimensions: (u32, u32),
}

impl RenderCaller {
    ///
    /// Marked as unsafe because it calls GL code
    pub unsafe fn new(screen_dimensions: (u32, u32)) -> RenderCaller {
        let vertex_array = VertexArray::new().unwrap();

        let shader_manager = super::loader::load_shaders();
        let texture_manager = super::loader::load_textures(screen_dimensions);


        RenderCaller {
            vertex_array,
            shader_manager,
            texture_manager,
            screen_dimensions,
        }
    }

    pub unsafe fn update_screen_dimensions(&mut self, screen_dimensions: (u32, u32)) {
        self.screen_dimensions = screen_dimensions;
        self.texture_manager
            .update_screen_dimensions(screen_dimensions);
    }

    ///
    /// This is supposed to turn a packed render into something that can then be rendered directly. So
    /// this has access to OpenGL calls.
    /// TODO: Enforce requirements on RenderPack<T> to make this safe.
    pub unsafe fn pack(&mut self, buffer: &usize, pack: &VertexPack) {
        if *buffer >= self.vertex_array.get_vbo_count() {
            panic!(
                "Trying to clear a buffer with index {}, but there's only {} buffers ",
                buffer,
                self.vertex_array.get_vbo_count()
            );
        }
        if VERBOSE {
            println!("Packing buffer {}", buffer);
        }
        self.vertex_array.fill_vbo(*buffer, &pack.vertices);
        self.vertex_array.fill_ebo(*buffer, &pack.elements);
    }

    pub unsafe fn clear(&mut self, buffer: &usize) {
        if *buffer >= self.vertex_array.get_vbo_count() {
            panic!(
                "Trying to clear an array with index {}, but there's only {} arrays ",
                buffer,
                self.vertex_array.get_vbo_count()
            );
        }
        if VERBOSE {
            println!("Clearing buffer {}", buffer);
        }
        self.vertex_array.clear(*buffer);
    }

    pub unsafe fn choose_shader(&mut self, shader: ShaderIdentifier) {
        if let Err(s) = self.shader_manager.bind_shader(shader) {
            println!("{}", s) //TODO: Log instead
        }
        if VERBOSE {
            println!("Choosing shader {}", shader.name());
        }
    }

    pub unsafe fn uniforms(&mut self, uniforms: &UniformData) {
        if let Err(s) = self
            .shader_manager
            .uniforms(uniforms, &self.texture_manager)
        {
            println!("{}", s); //TOOD: Log instead
        }
        if VERBOSE {
            println!("Passing uniforms");
        }
    }

    pub unsafe fn render(&mut self, buffer: &usize) {
        debug_assert!(
            self.vertex_array.get_size(*buffer) > 0,
            "A render call was made on an empty vertex array!"
        );
        self.vertex_array.draw(*buffer);
        if VERBOSE {
            println!("Rendering buffer {}", buffer);
        }
    }

    pub unsafe fn clear_buffers(&mut self, color_buffer: &bool, depth_buffer: &bool) {
        debug_assert!(*color_buffer || *depth_buffer, "A clear buffer call should never be made when neither color nor depth buffer is cleared!");
        gl::Clear(
            (if *color_buffer {
                gl::COLOR_BUFFER_BIT
            } else {
                0
            }) | (if *depth_buffer {
                gl::DEPTH_BUFFER_BIT
            } else {
                0
            }),
        );
    }

    pub unsafe fn dispatch_compute(&mut self, output_texture: &str, dimensions: &(u32, u32, u32)) {
        let tex = self.texture_manager.get_texture(output_texture);
        gl::BindImageTexture(
            0,
            tex.get_id(),
            0,
            gl::FALSE,
            0,
            gl::WRITE_ONLY,
            tex.metadata.internal_format.to_gl(),
        );
        gl::DispatchCompute(dimensions.0, dimensions.1, dimensions.2);
        if VERBOSE {
            println!(
                "Dispatching compute shader generating texture {}, id {}",
                output_texture,
                tex.get_id()
            );
        }
    }

    pub fn get_vbo_count(&self) -> usize {
        self.vertex_array.get_vbo_count()
    }

    pub fn get_texture_manager(&self) -> &TextureManager {
        &self.texture_manager
    }

    pub fn get_shader_manager(&self) -> &ShaderManager {
        &self.shader_manager
    }
}
