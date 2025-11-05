use nalgebra::{Matrix4, Vector2, Vector3, Vector4};
use plotters::{coord::types::RangedCoordf64, element::{Drawable, PointCollection}, prelude::*};
use serde::{Serialize, Deserialize};

#[derive(Copy,Clone,Debug, Serialize, Deserialize)]
pub enum Marker{
    None,
    Circle,
    Triangle,
    Cross,
}

#[derive(Copy,Clone,Debug, Serialize, Deserialize)]
pub struct Vertex{
    pub position:Vector3<f64>,
    pub size: f64,
    pub marker: Marker
}

impl Vertex {
    pub fn new(position: Vector3<f64>, size: f64, marker: Marker) -> Self {
        Self { position, size, marker }
    }

    pub fn as_drawn_vertex(&self, matrix:Matrix4<f64>, style:ShapeStyle)->Option<DrawnVertex>{
        // let expanded:Vector4<f64> = self.position.to_homogeneous();
        // let expanded = matrix*expanded;
        // if expanded.w<=0.0{
        //     None
        // }
        // else{
        //     Some(DrawnVertex::new(expanded.xy()/expanded.w, self.size, self.marker, style))
        // }
        self.get_position(matrix).map(|position|{
            DrawnVertex::new(position, self.size, self.marker, style)
        })
    }

    pub fn get_position(&self, matrix:Matrix4<f64>)->Option<Vector2<f64>>{
        let mut expanded:Vector4<f64> = self.position.to_homogeneous();
        expanded.w = 1.0;
        let expanded = matrix*expanded;
        if expanded.w<=0.0{
            None
        }
        else{
            let mut point = expanded.xy();
            point.x *= -1.0;
            Some(point/expanded.w)
        }
    }
}

#[derive(Copy,Clone,Debug)]
pub struct DrawnVertex{
    pub position:Vector2<f64>,
    pub size: f64,
    pub marker: Marker,
    pub style:ShapeStyle,
}

impl DrawnVertex {
    pub fn new(position: Vector2<f64>, size: f64, marker: Marker, style:ShapeStyle) -> Self {
        Self { position, size, marker, style }
    }
}

impl<DB: DrawingBackend> Drawable<DB> for DrawnVertex{
    fn draw<I: Iterator<Item = <plotters::element::BackendCoordOnly as plotters::element::CoordMapper>::Output>>(
        &self,
        pos: I,
        backend: &mut DB,
        parent_dim: (u32, u32),
    ) -> Result<(), plotters_iced::plotters_backend::DrawingErrorKind<<DB as DrawingBackend>::ErrorType>> {
        match self.marker {
            Marker::None=>Ok(()),
            Marker::Circle=>{
                Circle::new(self.position, self.size, self.style)
                    .draw(pos, backend, parent_dim)
            },
            Marker::Triangle=>{
                TriangleMarker::new(self.position, self.size, self.style)
                    .draw(pos, backend, parent_dim)
            },
            Marker::Cross=>{
                Cross::new(self.position, self.size, self.style)
                    .draw(pos, backend, parent_dim)
            },
        }
    }
}

impl<'a> PointCollection<'a, (f64,f64)> for &'a DrawnVertex
{
    type IntoIter = std::iter::Once<(f64,f64)>;
    type Point = (f64,f64);
    fn point_iter(self) -> std::iter::Once<(f64,f64)> {
        std::iter::once((self.position.x, self.position.y))
    }
}

#[derive(Clone,Debug, Serialize, Deserialize)]
pub struct Mesh{
    pub vertices:Vec<Vertex>,
    pub lines:Vec<Vec<usize>>,
}

impl Mesh{
    pub fn new()->Self{
        Self { vertices: Vec::new(), lines: Vec::new() }
    }

    pub fn add_vertex<T:Into<Vector3<f64>>>(&mut self, position:T, size:f64, marker:Marker){
        let vert = Vertex::new(position.into(), size, marker);
        self.vertices.push(vert);
    }

    pub fn regiester_path<T:Into<Vec<usize>>>(&mut self, ids:T){
        self.lines.push(ids.into());
    }

    pub fn draw<'a, DB:DrawingBackend>(&'a self, matrix:Matrix4<f64>, style:ShapeStyle, mut target:ChartContext<'a,DB,Cartesian2d<RangedCoordf64, RangedCoordf64>>){
        // Drawing markers
        target.draw_series(self.vertices.iter()
            .filter_map(|x| x.as_drawn_vertex(matrix, style))
        ).unwrap();

        // Drawing lines
        for line_indices in self.lines.iter(){
            target.draw_series(
                LineSeries::new(
                    line_indices.iter()
                        .filter_map(|i| self.vertices.get(*i))
                        .filter_map(|x| {
                            if let Some(pos) = x.get_position(matrix){
                                Some((pos.x, pos.y))
                            }
                            else{
                                None
                            }
                        }),
                    style
                )
            ).unwrap();
        }
    }
}
