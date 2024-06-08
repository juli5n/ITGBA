use derive_more::{Deref, DerefMut};
use image::{DynamicImage, Rgb};

#[derive(Deref, DerefMut)]
pub struct ColorPalette([Rgb<u8>; 4]);

#[derive(Clone, PartialEq, Eq, Hash, Deref, DerefMut)]
pub struct Tiledata(Array2d<u8,8,8>);

#[derive(Deref, DerefMut)]
pub struct TileIndexArray(Array2d<u8,32,32>);


#[derive(Deref, DerefMut)]
pub struct AttributeByteArray(Array2d<u8, 32,32>);


#[derive(Deref, DerefMut, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Array2d<T,const rows: usize, const colummns: usize>([[T; colummns]; rows]) where T: std::default::Default + std::marker::Copy;

impl<T, const rows: usize, const colummns: usize> Array2d<T,rows,colummns> where T:std::default::Default + std::marker::Copy{
    pub fn new() -> Self {
        Array2d([[T::default(); colummns]; rows])
    }
    pub fn assign<I>(&mut self, colummn_index : I, row_index: I, value: T) where I: TryInto<usize> {
        let Ok(x)= colummn_index.try_into() else {panic!()};
        let Ok(y)= row_index.try_into() else {panic!()};

        Self::boundary_check(x,y);

        self[y][x] = value;
    }
    pub fn get<I>(&self, colummn_index : I, row_index: I) -> T where I: TryInto<usize> {
        let Ok(x)= colummn_index.try_into() else {panic!()};
        let Ok(y)= row_index.try_into() else {panic!()};

        Self::boundary_check(x,y);

        self[y][x]
    }
    pub fn boundary_check(colummn_index: usize, row_index: usize) {
        if (!(0..colummns).contains(&colummn_index)) || (!(0..rows).contains(&row_index)) {
            panic!();
        }

    }
}