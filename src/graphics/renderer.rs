use super::{render_caller::RenderCaller, vertex::Vertex, vertex_pack::VertexPack, ShaderIdentifier};
use crate::logic::{Asset, Position};

pub struct Renderer {
    render_caller : RenderCaller
}

impl Renderer {
    pub unsafe fn new(screen_dimensions : (u32,u32)) -> Self {
        Renderer {
            render_caller : RenderCaller::new(screen_dimensions)
        }
    }

    pub unsafe fn render(&mut self, iter : &[(Asset, Position)]) {
        //println!("Rendering!"); 
        self.render_caller.clear_buffers(&true, &true);
        let vertices : Vec<Vertex> = iter.iter()
            .map(|x|
            
                Vertex {
                    x : x.1.x+0.5,
                    y : x.1.y+0.5,
                    r : 1.0,
                    g : 0.0,
                    b : 0.0,
                    u : 0.0,
                    v : 0.0
                }
            )
            .collect();
        //println!("Vertex count: {}", vertices.len());
        self.render_caller.pack(&0, &VertexPack {
            vertices,
            elements : vec![]
        });
        self.render_caller.choose_shader(ShaderIdentifier::Default);
        self.render_caller.render(&0);
    }
}