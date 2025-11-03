#[derive(Clone, Debug)]
pub struct Mesh{
    pub vectices: Vec<nalgebra::Vector3<f64>>,
    pub triangles: Vec<usize>,
}

impl Mesh {
    pub fn new(vectices: Vec<nalgebra::Vector3<f64>>, triangles: Vec<usize>) -> Self {
        Self { vectices, triangles }
    }

    pub fn iter(&self)-> MeshVertexIterator<'_>{
        MeshVertexIterator::new(self)
    }
}

pub struct MeshVertexIterator<'a>{
    mesh: &'a Mesh,
    current_index: usize,
}

impl<'a> MeshVertexIterator<'a> {
    pub fn new(mesh: &'a Mesh) -> Self {
        Self { mesh, current_index:0 }
    }
}


impl<'a> Iterator for MeshVertexIterator<'a>{
    type Item = nalgebra::Vector3<f64>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(id) = self.mesh.triangles.get(self.current_index){
            self.current_index += 1;
            self.mesh.vectices.get(*id).map(|x| *x)
        }
        else{
            None
        }
    }
}
