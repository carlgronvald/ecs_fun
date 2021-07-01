use std::ops::Index;
/// Impl this if you have a location in 3d space given by an f32. Used by some algorthims / data structures.
pub trait Locatedf32 {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
    fn z(&self) -> f32;
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(packed)]
pub struct V2C3UV {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub u: f32,
    pub v: f32,
}

///
/// The vertex used basically everywhere in the program.
pub type Vertex = V2C3UV;


impl V2C3UV {
    pub fn attribute_pointers() -> AttributePointerList {
        AttributePointerList::new::<V2C3UV>(vec![
            AttributePointer::new(0, 2, gl::FLOAT, false, 0),
            AttributePointer::new(1, 3, gl::FLOAT, false, 8),
            AttributePointer::new(2, 2, gl::FLOAT, false, 20),
        ])
        .unwrap()
    }
}

pub struct AttributePointerList {
    attribute_pointers: Vec<AttributePointer>,
    stride: usize,
}

impl AttributePointerList {
    /// TODO: Supplying a vector of attribute pointers here is a little weird when there's so many
    /// requirements for it. Maybe there should be a simpler input list that is then converted to
    /// attribute pointers.
    pub fn new<T>(
        attribute_pointers: Vec<AttributePointer>,
    ) -> Result<AttributePointerList, String> {
        let mut offset: u32 = 0;
        for (i, pointer) in attribute_pointers.iter().enumerate() {
            if pointer.get_offset() != offset {
                return Err(String::from(
                    "Offsets of attribute pointers don't make sense",
                ));
            }
            offset += pointer.get_components() as u32
                * (match pointer.get_type() {
                    gl::FLOAT => 4,
                    _ => {
                        return Err(String::from(
                            "Unknown type in attribute pointer! Only knows float",
                        ))
                    }
                });
            if i != pointer.get_index() as usize {
                return Err(String::from("Indices must increase by one every time!"));
            }
        }
        if offset as usize != std::mem::size_of::<T>() {
            return Err(String::from(
                "Attribute pointers claim a different amount of memory in vertex!",
            ));
        }

        Ok(AttributePointerList {
            attribute_pointers,
            stride: offset as usize,
        })
    }

    pub fn len(&self) -> usize {
        self.attribute_pointers.len()
    }

    pub fn get_stride(&self) -> usize {
        self.stride
    }
}

impl Index<usize> for AttributePointerList {
    type Output = AttributePointer;

    fn index(&self, i: usize) -> &Self::Output {
        &self.attribute_pointers[i]
    }
}

///
/// One attribute pointer
pub struct AttributePointer {
    index: u32,
    /// How many components the attribute has, like a vec3 has 3
    components: usize,
    attrib_type: gl::types::GLenum,
    normalized: bool,

    /// How offset this attribute is in bytes. TODO: offset is really dangerous to fiddle with
    offset: u32,
}

impl AttributePointer {
    pub fn new(
        index: u32,
        components: usize,
        attrib_type: gl::types::GLenum,
        normalized: bool,
        offset: u32,
    ) -> AttributePointer {
        AttributePointer {
            index,
            components,
            attrib_type,
            normalized,
            offset,
        }
    }

    pub fn get_type(&self) -> gl::types::GLenum {
        self.attrib_type
    }

    pub fn get_components(&self) -> usize {
        self.components
    }

    pub fn get_index(&self) -> u32 {
        self.index
    }

    pub fn get_normalized(&self) -> bool {
        self.normalized
    }

    pub fn get_offset(&self) -> u32 {
        self.offset
    }
}
