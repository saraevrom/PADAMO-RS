use std::fmt::Debug;

use super::ArrayND;

use nalgebra::ArrayStorage;
use nalgebra::Matrix;

// impl<T, R, C, S> Into<Matrix<T, R, C, S>> for ArrayND<T>
// where
//     T:Clone+abi_stable::StableAbi,
//
// {
//     fn into(self) -> Matrix<T, R, C, S> {
//         todo!()
//     }
// }

impl<T, R, C, const R1:usize, const C1:usize> From<Matrix<T, R, C, ArrayStorage<T, R1, C1>>> for ArrayND<T>
where
    T:Clone+abi_stable::StableAbi,
{
    fn from(value: Matrix<T, R, C, ArrayStorage<T, R1, C1>>) -> Self {
        let shape = vec![R1, C1];
        let mut flat_data = Vec::with_capacity(R1*C1);
        for offset in 0..R1*C1{
            let column = offset % C1;
            let row = offset / C1;
            flat_data.push(value.data.0[column][row].clone());
        }
        Self { flat_data: flat_data.into(), shape: shape.into() }
    }
}

impl<T, R, C, const R1:usize, const C1:usize> TryInto<Matrix<T, R, C, ArrayStorage<T, R1, C1>>> for ArrayND<T>
where
    T:Clone+Debug+abi_stable::StableAbi+Default+std::cmp::PartialEq,
    R:nalgebra::Dim,
    C:nalgebra::Dim,
    Matrix<T, R, C, ArrayStorage<T, R1, C1>>:Default
{
    type Error = ();
    fn try_into(self) -> Result<Matrix<T, R, C, ArrayStorage<T, R1, C1>>, Self::Error> {
        if self.shape.len()!=2{
            return Err(());
        }
        if self.shape[0] != R1 || self.shape[1] != C1{
            return Err(());
        }
        let mut res = Matrix::default();
        let mut index = vec![0,0];
        for row in 0..R1{
            for column in 0..C1{
                index[0] = row;
                index[1] = column;
                if let Some(r) = self.try_get(&index){
                    res.data.0[column][row] = r.clone();
                }
            }
        }
        Ok(res)
    }
}

#[cfg(test)]
mod test_conversions{
    use nalgebra::Matrix3x4;

    use crate::ArrayND;

    #[test]
    fn test_into_and_back(){
        let matrix1:Matrix3x4<f64> = Matrix3x4::new(1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.);
        let matrix1_conv:ArrayND<f64> = matrix1.into();
        let conv_back = matrix1_conv.try_into();
        assert_eq!(conv_back, Ok(matrix1))
    }

    #[test]
    fn test_shape_preservation(){
        let matrix1:Matrix3x4<f64> = Matrix3x4::new(1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.);
        let matrix1_conv:ArrayND<f64> = matrix1.into();
        assert_eq!(matrix1_conv.shape.to_vec(), vec![3,4]);
    }
}
