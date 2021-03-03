use anyhow::*;
use wgpu::util::DeviceExt;
use crate::texture;
use rand::Rng;
use std::collections::HashMap;
use cgmath::Vector3;
use cgmath::Vector2;

pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ModelVertex {
    position: cgmath::Vector3<f32>,
    //tex_coords: cgmath::Vector2<f32>,
    //normal: cgmath::Vector3<f32>,
    //tangent: cgmath::Vector3<f32>,
    //bitangent: cgmath::Vector3<f32>,
}

unsafe impl bytemuck::Zeroable for ModelVertex {}
unsafe impl bytemuck::Pod for ModelVertex {}

impl Vertex for ModelVertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
                /*
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float3,
                },
                // Tangent and bitangent
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float3,
                },
                */
            ],
        }
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub blocktype: BlockType,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indexes: u32,
    pub instances: Vec<Instance>,
    pub instances_buffer: wgpu::Buffer,
    pub num_instances: u32,
}

#[derive(Debug)]
pub struct World{
    pub chunks: HashMap<[i64;3], Chunk>,
}

impl World {
    pub fn GetBlockFromGlobalAddress(&self, x : f64, y: f64, z: f64) -> Option<&Block>
    {
        //Find chunk
        let chunk_x : i64 = ((x / (CHUNKSIZE as f64)) as f64).floor() as i64;
        let chunk_y : i64 = ((y / (CHUNKSIZE as f64)) as f64).floor() as i64;
        let chunk_z : i64 = ((z / (CHUNKSIZE as f64)) as f64).floor() as i64;
        println!("chunk_x {:?}", chunk_x);
        println!("chunk_y {:?}", chunk_y);
        println!("chunk_z {:?}", chunk_z);

        let chunk = self.chunks.get(&[chunk_x, chunk_y, chunk_z]);
        if chunk.is_none(){
            println!("chunk not found ");
            return None
        }

        //find block
        let block_x : u8 = (x.floor().abs() as u64 - (chunk_x.abs() as u64 * CHUNKSIZE as u64) as u64) as u8;
        let block_y : u8 = (y.floor().abs() as u64 - (chunk_y.abs() as u64 * CHUNKSIZE as u64) as u64) as u8;
        let block_z : u8 = (z.floor().abs() as u64 - (chunk_z.abs() as u64 * CHUNKSIZE as u64) as u64) as u8;

        println!("block_x {:?}", block_x);
        println!("block_y {:?}", block_y);
        println!("block_z {:?}", block_z);

        let block = chunk.unwrap().blocks.get(&[block_x, block_y, block_z]);

        block
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum BlockType {
    NORMAL,
}

#[derive(Debug)]
pub struct Block {
    pub blocktype : BlockType,
    pub color: [f32; 3]
}

#[derive(Debug)]
pub struct Chunk {
    pub blocks: HashMap<[u8;3], Block>,
}

const CHUNKSIZE: u8 = 3;

const CUBE_INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0, // top
    4, 5, 6, 6, 7, 4, // bottom
    8, 9, 10, 10, 11, 8, // right
    12, 13, 14, 14, 15, 12, // left
    16, 17, 18, 18, 19, 16, // front
    20, 21, 22, 22, 23, 20, // back
];

#[derive(Debug)]
pub struct Instance {
    position: cgmath::Vector3<f32>,
    color: cgmath::Vector3<f32>,
    //rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: cgmath::Matrix4::from_translation(self.position).into(),
            color: [self.color[0], self.color[1], self.color[2]]
                //* cgmath::Matrix4::from(self.rotation),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    color: [f32; 3]
}

impl InstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We don't have to do this in code though.
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float4,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float4,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float4,
                },
                //color
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float3,
                },                
            ],
        }
    }
}

#[derive(Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub world : World,

}

impl Model {
    fn build_random_chunk(&self)->Chunk
    {
        //Generate random chunk
        let mut chunk = Chunk{ blocks : HashMap::new(),};
        //chunk.blocks.insert( [1, 1, 1], Block{blocktype:BlockType::GRASS});
        
        let mut rng = rand::thread_rng();
        for k in 0..CHUNKSIZE {
            for l in 0..CHUNKSIZE {
                for m in 0..CHUNKSIZE {
                    //Add block
                    chunk.blocks.insert( [k, l, m], Block{blocktype:BlockType::NORMAL, color: [ 0.0, 1.0, 0.0]});
                }
            }
        }
        chunk
    }


    fn create_vertices(&self, blocktype:BlockType) -> Vec<ModelVertex>{
        //Build ModelVertex. Have to lookup u and v wich is dependent on QuadType. (this decides where to find in correct bitmap in blockatlas.jpg)
        //TODO: move umap and vmap outside function and convert to closure
        fn build_vertex(position:[i8;3])->ModelVertex
        {
            let pos = Vector3::new(position[0] as f32, position[1] as f32, position[2] as f32);
            ModelVertex{position:pos}
        }
            
        let mut vertex_data: Vec<ModelVertex>= Vec::new();
        
        // top (0, 0, 1)    
        vertex_data.push(build_vertex([0, 0, 1]));
        vertex_data.push(build_vertex([1, 0, 1]));
        vertex_data.push(build_vertex([1, 1, 1]));
        vertex_data.push(build_vertex([0, 1, 1]));
    
        // bottom (0, 0, -1)     
        vertex_data.push(build_vertex([0, 1, 0]));
        vertex_data.push(build_vertex([1, 1, 0]));
        vertex_data.push(build_vertex([1, 0, 0]));
        vertex_data.push(build_vertex([0, 0, 0]));
    
        // right (1, 0, 0)
        vertex_data.push(build_vertex([1, 0, 0]));
        vertex_data.push(build_vertex([1, 1, 0]));
        vertex_data.push(build_vertex([1, 1, 1]));
        vertex_data.push(build_vertex([1, 0, 1]));
    
        // left (-1, 0, 0)    
        vertex_data.push(build_vertex([0, 0, 1]));
        vertex_data.push(build_vertex([0, 1, 1]));
        vertex_data.push(build_vertex([0, 1, 0]));
        vertex_data.push(build_vertex([0, 0, 0]));
    
        // front (0, 1, 0)    
        vertex_data.push(build_vertex([1, 1, 0]));
        vertex_data.push(build_vertex([0, 1, 0]));
        vertex_data.push(build_vertex([0, 1, 1]));
        vertex_data.push(build_vertex([1, 1, 1]));
    
        // back (0, -1, 0)
        vertex_data.push(build_vertex([1, 0, 1]));
        vertex_data.push(build_vertex([0, 0, 1]));
        vertex_data.push(build_vertex([0, 0, 0]));
        vertex_data.push(build_vertex([1, 0, 0]));
    
        vertex_data
    }
    
    pub fn new()-> Result<Self>{
        Ok(Self { meshes: Vec::new(), world: World{chunks:HashMap::new()} })
    }

    pub fn load(
        &mut self,
        device: &wgpu::Device,
    ){        
        //build world
        //First chunk,
        //trenger flere sef
        self.world.chunks.insert( [0, 0, 0], self.build_random_chunk());
        //self.world.chunks.insert( [1, 0, 0], self.build_random_chunk());
        //self.world.chunks.insert( [1, 0, 1], self.build_random_chunk());
        //self.world.chunks.insert( [0, 0, 1], self.build_random_chunk());
        //self.world.chunks.insert( [0, 1, 1], self.build_random_chunk());
        //self.world.chunks.insert( [1, 1, 1], self.build_random_chunk());

        //Go through world and build meshes. One mesh for each blocktype
        let mut create_mesh_and_addto_model = |blocktype| {
            let create_instance = |x, y, z, r, g, b| {
                let position = cgmath::Vector3 {
                    x: x as f32,
                    y: y as f32,
                    z: z as f32,
                };
                let color = cgmath::Vector3 {
                    x: r as f32,
                    y: g as f32,
                    z: b as f32,
                };
                Instance { position, color }
            };

            let mut instances=Vec::new();
            for (chunkkey, chunk) in &self.world.chunks {
                for (blockkey, block) in &chunk.blocks {
                    if block.blocktype == blocktype
                    {
                        //transler til rett plass. Må ta hensyn til flere chunks.
                        let x = (chunkkey[0] * CHUNKSIZE as i64) + blockkey[0] as i64;
                        let y = (chunkkey[1] * CHUNKSIZE as i64) + blockkey[1] as i64;
                        let z = (chunkkey[2] * CHUNKSIZE as i64) + blockkey[2] as i64;

                        let r = block.color[0];
                        let g = block.color[1];
                        let b = block.color[2];

                        instances.push(create_instance(x as f32, y as f32, z as f32, r as f32, g as f32, b as f32));
                    }
                }
            }
            //println!("gvtest instances: {:?}", instances);
            let num_instances = instances.len() as u32;
            if num_instances > 0
            {
                let vertices = self.create_vertices(blocktype);
                let  vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsage::VERTEX,
                });
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(CUBE_INDICES),
                    usage: wgpu::BufferUsage::INDEX,
                });
        
                let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
                let instances_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&instance_data),
                    usage: wgpu::BufferUsage::VERTEX,
                });
                
                self.meshes.push(Mesh{
                    blocktype: blocktype, 
                    vertex_buffer: vertex_buffer,
                    index_buffer: index_buffer,
                    num_indexes: CUBE_INDICES.len() as u32,
                    instances: instances,
                    instances_buffer: instances_buffer,
                    //uniform_bind_group_instances: uniform_bind_group_instances,
                    num_instances: num_instances,
                });
            }
        };

        create_mesh_and_addto_model(BlockType::NORMAL);
        //create_mesh_and_addto_model(BlockType::DIRT);
        //create_mesh_and_addto_model(BlockType::STONE);
    }
}

pub trait DrawModel<'a, 'b>
where
    'b: 'a,
{
    fn draw_mesh(
        &mut self,
        mesh: &'b Mesh,
        uniforms: &'b wgpu::BindGroup,
        //light: &'b wgpu::BindGroup,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        //instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        //light: &'b wgpu::BindGroup,
    );

    fn draw_model(
        &mut self,
        model: &'b Model,
        uniforms: &'b wgpu::BindGroup,
        //light: &'b wgpu::BindGroup,
    );
    fn draw_model_instanced(
        &mut self,
        model: &'b Model,
        //instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        //light: &'b wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawModel<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(
        &mut self,
        mesh: &'b Mesh,
        uniforms: &'b wgpu::BindGroup,
        //light: &'b wgpu::BindGroup,
    ) {
        self.draw_mesh_instanced(mesh, /*0..1,*/ uniforms/*, light*/);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        //instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        //light: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_vertex_buffer(1, mesh.instances_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..));
        //self.set_bind_group(0, &material.bind_group, &[]);
        //self.set_bind_group(1, &uniforms, &[]);
        self.set_bind_group(0, &uniforms, &[]);

        //self.set_bind_group(2, &light, &[]);
        //self.draw_indexed(0..mesh.num_elements, 0, instances);
        self.draw_indexed(0..mesh.num_indexes, 0, 0..mesh.num_instances);        
    }

    fn draw_model(
        &mut self,
        model: &'b Model,
        uniforms: &'b wgpu::BindGroup,
        //light: &'b wgpu::BindGroup,
    ) {
        self.draw_model_instanced(model, /*0..1,*/ uniforms/*, light*/);
    }

    fn draw_model_instanced(
        &mut self,
        model: &'b Model,
        //instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        //light: &'b wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            //let material = &model.materials[mesh.material];
            
            self.draw_mesh_instanced(mesh, /*, instances.clone()*/uniforms/*, light*/);
        }
    }
}
