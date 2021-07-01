use super::vertex::Vertex;

///
/// Has size*components members.
pub struct ArrayBuffer {
    id: u32,
    size: usize,
    stride: usize,
}

///TODO: MAKE TYPE SAFE
impl ArrayBuffer {
    pub unsafe fn new() -> Result<ArrayBuffer, String> {
        let mut vbo = 0;
        gl::GenBuffers(1, &mut vbo);

        Ok(ArrayBuffer {
            id: vbo,
            size: 0,
            stride: std::mem::size_of::<Vertex>(),
        })
    }

    pub unsafe fn bind(&self) {
        gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
    }

    pub unsafe fn fill(&mut self, data: &[Vertex]) {
        self.bind();
        //TODO: Maybe shouldn't be static draw
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (self.stride * data.len()) as isize,
            data.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW,
        );
        self.size = data.len();
    }

    pub fn get_stride(&self) -> usize {
        self.stride
    }

    pub fn get_size(&self) -> usize {
        self.size
    }
}

impl Drop for ArrayBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

pub struct ElementBuffer {
    id: u32,
    size: usize,
}

impl ElementBuffer {
    pub unsafe fn new() -> Result<ElementBuffer, String> {
        let mut ebo = 0;
        gl::GenBuffers(1, &mut ebo);

        Ok(ElementBuffer { id: ebo, size: 0 })
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub unsafe fn bind(&self) {
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id);
    }

    pub unsafe fn fill(&mut self, data: &[u32]) {
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (std::mem::size_of::<u32>() * data.len()) as isize,
            data.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW,
        );
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        self.size = data.len();
    }
}
