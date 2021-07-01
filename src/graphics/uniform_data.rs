
macro_rules! create_uniform_data {
    ( $($name:ident ($type:ty), )* ) => {
        #[derive(Debug)]
        pub struct UniformData {
            $(pub $name : Vec<($type, String)>,)*
        }
        impl UniformData {
            /*pub fn new(
                $($name : Vec<($type, String)>,)*
            ) -> UniformData {
                UniformData {
                    $($name,)*
                }
            }*/

            pub fn new() -> UniformData {
                UniformData {
                    $($name : Vec::new(),)*
                }
            }

            pub fn get_uniform_locations(&self) -> Vec<&String> {
                let mut res = Vec::new();

                $(for entry in &self.$name {
                    res.push(&entry.1);
                })*

                res
            }

            $(pub fn $name<T : Into<String>>(&mut self, value : $type, location : T) {
                self.$name.push((value, location.into()));
            })*

            pub fn get_uniforms<'a>(&'a self) -> Vec<Uniform<'a>> {
                let mut res = Vec::new();

                $(for entry in &self.$name {
                    res.push(Uniform { value : UniformValue::$name(&entry.0), location : &entry.1});
                })*

                res
            }

        }

        pub struct Uniform<'a> {
            pub value : UniformValue<'a>,
            pub location : &'a String
        }

        #[allow(non_camel_case_types)]
        pub enum UniformValue<'a> {
            $($name (&'a $type),)*
        }
    }
}

//This generates the UniformData struct
create_uniform_data! {
    float(f32), int(i32), uint(u32),
    vec4(glm::Vec4), vec3 (glm::Vec3), vec2(glm::Vec2),
    ivec4(glm::IVec4), ivec3(glm::IVec3), ivec2(glm::IVec2),
    uvec4(glm::UVec4), uvec3(glm::UVec3), uvec2(glm::UVec2),
    mat4 (glm::Mat4), mat3(glm::Mat3), mat2(glm::Mat2),
    texture (String),

}