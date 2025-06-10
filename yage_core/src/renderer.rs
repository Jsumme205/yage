pub trait Renderer {
    type Pixel: Copy + Clone;
    type Error;

    fn width(&self) -> u32;
    fn height(&self) -> u32;

    fn data(&self) -> &[Self::Pixel];
    fn data_mut(&mut self) -> &mut [Self::Pixel];

    fn sync(&mut self) -> Result<bool, Self::Error>;

    fn pixel(&mut self, x: u32, y: u32, pix: Self::Pixel) -> Result<(), Self::Error>;

    fn arc(&mut self, x: u32, y: u32, parts: u8, pix: Self::Pixel) -> Result<(), Self::Error>;

    fn circle(&mut self, x: u32, y: u32, radius: i32, pix: Self::Pixel) -> Result<(), Self::Error>;

    fn rect(&mut self, x: u32, y: u32, width: u32, height: u32) -> Result<(), Self::Error>;

    fn overwrite_with(
        &mut self,
        x: u32,
        y: u32,
        buffer_width: u32,
        buffer_height: u32,
        buffer: &[Self::Pixel],
    ) -> Result<(), Self::Error>;
}

pub trait MakeRenderer: Renderer {
    
    fn new(width: u32, height: u32) -> Result<Self, Self::Error>
    where
        Self: Sized;
}
