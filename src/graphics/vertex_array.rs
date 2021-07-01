//TODO: THIS IS ALL WRONG; ONE VERTEX ARRAY HOLDS A GROUP OF BUFFERS
use super::ArrayBuffer;
use super::ElementBuffer;
use super::vertex::{AttributePointerList, Vertex};

pub struct VertexArray {
    id: u32,
    vbos: Vec<ArrayBuffer>,
    ebos: Vec<ElementBuffer>,
    attributes: AttributePointerList,
}

//TODO: WHAT TO DO WHEN MULTIPLE VERTREX ARRAYS TRY TO BIND SAME LOCATION AT THE SAME TIME??
impl VertexArray {
    //TODO: validate inputs
    pub unsafe fn new() -> Result<VertexArray, String> {
        let attributes = Vertex::attribute_pointers();

        let mut vbos: Vec<ArrayBuffer> = Vec::new();
        let mut ebos: Vec<ElementBuffer> = Vec::new();

        let mut id = 0;
        gl::GenVertexArrays(1, &mut id);
        gl::BindVertexArray(id);
        for _ in 0..100 {
            let ebo = ElementBuffer::new().unwrap();
            let vbo = ArrayBuffer::new().unwrap();

            vbo.bind();
            for i in 0..attributes.len() {
                gl::EnableVertexAttribArray(attributes[i].get_index() as u32); // this is "layout (location = 0)" in vertex shader
                gl::VertexAttribPointer(
                    attributes[i].get_index() as u32, // index of the generic vertex attribute ("layout (location = 0)")
                    attributes[i].get_components() as i32, // the number of components per generic vertex attribute
                    attributes[i].get_type(),              // data type
                    if attributes[i].get_normalized() {
                        gl::TRUE
                    } else {
                        gl::FALSE
                    }, // normalized (int-to-float conversion)
                    std::mem::size_of::<Vertex>() as gl::types::GLint, // stride (byte offset between consecutive attributes)
                    attributes[i].get_offset() as *const std::ffi::c_void, // offset of the first component
                );
            }
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            vbos.push(vbo);
            ebos.push(ebo);
        }
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        Ok(VertexArray {
            id,
            vbos,
            ebos,
            attributes,
        })
    }

    ///
    /// Fills the vbo of the vertex array with the given data.
    pub unsafe fn fill_vbo(&mut self, target: usize, data: &[Vertex]) {
        self.vbos[target].fill(data);
    }

    ///
    /// Fills the ebo (element buffer object) of the vertex array with the given data.
    pub unsafe fn fill_ebo(&mut self, target: usize, data: &[u32]) {
        self.ebos[target].fill(data);
    }

    ///
    /// Clears the vbo of the vertex array.
    pub unsafe fn clear(&mut self, target: usize) {
        let vert: Vec<Vertex> = Vec::new();
        self.vbos[target].fill(&vert);
    }

    //TODO: MAKE THIS SAFE BY HANDLING MULTI BINDING OF SAME LAYOUT LOCATION
    pub unsafe fn bind(&self, target: usize) {
        gl::BindVertexArray(self.id);
        let attributes = &self.attributes;
        self.vbos[target].bind();
        for i in 0..attributes.len() {
            gl::EnableVertexAttribArray(attributes[i].get_index() as u32); // this is "layout (location = 0)" in vertex shader
            gl::VertexAttribPointer(
                attributes[i].get_index() as u32, // index of the generic vertex attribute ("layout (location = 0)")
                attributes[i].get_components() as i32, // the number of components per generic vertex attribute
                attributes[i].get_type(),              // data type
                if attributes[i].get_normalized() {
                    gl::TRUE
                } else {
                    gl::FALSE
                }, // normalized (int-to-float conversion)
                std::mem::size_of::<Vertex>() as gl::types::GLint, // stride (byte offset between consecutive attributes)
                attributes[i].get_offset() as *const std::ffi::c_void, // offset of the first component
            );
        }
        if self.ebos[target].get_size() > 0 {
            self.ebos[target].bind();
        }
    }

    pub unsafe fn draw(&self, target: usize) {
        self.bind(target);

        if self.ebos[target].get_size() > 0 {
            gl::DrawElements(
                gl::TRIANGLES,
                self.ebos[target].get_size() as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        } else {
            gl::DrawArrays(gl::POINTS, 0, self.vbos[target].get_size() as i32);
        }
    }

    pub fn get_size(&self, target: usize) -> usize {
        self.vbos[target].get_size()
    }

    pub fn get_vbo_count(&self) -> usize {
        self.vbos.len()
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        }
    }
}
