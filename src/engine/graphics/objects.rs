use std::sync::Arc;
use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex as VulkanoVertex;
use glam::{DAffine3, DQuat, DVec3, Vec3};


#[derive(BufferContents, VulkanoVertex)]
#[repr(C)]
#[derive(Clone)]
pub struct Polygon {
    #[format(R32G32_SFLOAT)]
    positions: [[f32; 3]; 3],
    #[format(R32G32_SFLOAT)]
    normal: [f32; 3]
}

impl Polygon {
    pub fn new(positions: [[f32; 3]; 3]) -> Self {
        let normal = Self::normal(
            [
                Vec3::from_array(positions[0]),
                Vec3::from_array(positions[1]),
                Vec3::from_array(positions[2])
            ]
        );

        Self {
            positions,
            normal
        }
    }

    fn normal(positions: [Vec3; 3]) -> [f32; 3] {
        let mut norm = Vec3::ZERO;

        let u = positions[1] - positions[0];
        let v = positions[2] - positions[0];

        norm.x = (u.y * v.z) - (u.z * v.y);
        norm.y = (u.z * v.x) - (u.x * v.z);
        norm.z = (u.x * v.y) - (u.y * v.x);
        println!("Normal: {}", norm);
        norm.normalize().to_array()
    }
}


#[derive(Clone)]
pub struct RenderData {
    pub(super) polygons: Vec<Polygon>
}

impl RenderData {
    pub(super) fn new(polygons: Vec<Polygon>) -> Self {
        Self {polygons}
    }

}

#[derive(Clone)]
pub enum Mesh {
    Triangle,
    //Cube
}

impl Mesh {
    pub(super) fn polygons(self) -> Vec<Polygon> {
        match self {
            Mesh::Triangle => {
                vec![
                    Polygon::new(
                        [[-0.5, 0.5, 0.0_f32],
                            [-0.5, -0.5, 0.0_f32],
                            [0.5, 0.5, 0.0_f32]]
                    ),
                ]
            }
        }
    }
}

#[derive(Clone)]
pub struct Object {
    render_data: RenderData,
    transform: DAffine3
}

impl Object {
    pub fn create(mesh: Mesh, translation: [f64; 3]) -> Self {
        let transform = DAffine3::from_scale_rotation_translation(
            [1.0, 1.0, 1.0].into(),
            DQuat::default(),
            translation.into()
        );

        let render_data = RenderData::new(mesh.polygons());
        Self {render_data, transform}
    }
}