use glam::Vec3;
use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex as VulkanoVertex;

#[derive(BufferContents, VulkanoVertex)]
#[repr(C)]
pub struct Polygon {
    #[format(R32G32_SFLOAT)]
    positions: [[f32; 3]; 3],
    #[format(R32G32_SFLOAT)]
    normal: [f32; 3]
}

impl Polygon {
    pub fn new(&self, positions: [[f32; 3]; 3]) -> Self {
        let normal = Self::normal(positions);

        Self {
            positions,
            normal
        }
    }

    fn normal(positions: [[f32; 3]; 3]) -> [f32; 3] {
        let p1 = Vec3::from_array(positions[0]);
        let p2 = Vec3::from_array(positions[1]);
        let p3 = Vec3::from_array(positions[2]);

        let u = p2 - p1;
        let v = p3 - p1;

        let normal = Vec3::new(
            u.y * v.z - u.z - v.y,
            u.z * v.x - u.x - v.z,
            u.x * v.y - u.y - v.x
        );

        normal.normalize().to_array()
    }
}

pub struct Object {
    polygons: Vec<Polygon>
}