use std::thread;
use std::time::Duration;
use std::time::Instant;

use anyhow::Result;
use clap::Parser;
use clap::ValueEnum;
use colored::Colorize;
use rand::RngExt;
use rand::distr::Distribution;
use rand::distr::StandardUniform;
use risio::datatype::ComplexDouble;
use risio::{
    Accessor, RawImage,
    datatype::{ComplexFloat, DataType, IsioDataType},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the shared memory image
    name: String,
    /// Datatype of image
    datatype: DataTypeWrap,
    /// Target framerate
    freq: f64,
    /// len of first axis
    dim0: usize,
    /// len of second axis
    dim1: Option<usize>,
    /// len of third axis
    dim2: Option<usize>,
}

#[derive(ValueEnum, Clone, Debug)]
enum DataTypeWrap {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    C64,
    C128,
}

enum Shape {
    OneDim(usize),
    TwoDim(usize, usize),
    ThreeDim(usize, usize, usize),
}

fn main() -> Result<()> {
    let args = Args::parse();
    let shape = match (args.dim0, args.dim1, args.dim2) {
        (x, None, None) => Shape::OneDim(x),
        (x, Some(y), None) => Shape::TwoDim(x, y),
        (x, Some(y), Some(z)) => Shape::ThreeDim(x, y, z),
        (_, None, Some(_)) => unreachable!("shouldn't be able to get here with serial arguments"),
    };
    match args.datatype {
        DataTypeWrap::U8 => make_noise::<u8>(&args.name, shape, args.freq)?,
        DataTypeWrap::U16 => make_noise::<u16>(&args.name, shape, args.freq)?,
        DataTypeWrap::U32 => make_noise::<u32>(&args.name, shape, args.freq)?,
        DataTypeWrap::U64 => make_noise::<u64>(&args.name, shape, args.freq)?,
        DataTypeWrap::I8 => make_noise::<i8>(&args.name, shape, args.freq)?,
        DataTypeWrap::I16 => make_noise::<i16>(&args.name, shape, args.freq)?,
        DataTypeWrap::I32 => make_noise::<i32>(&args.name, shape, args.freq)?,
        DataTypeWrap::I64 => make_noise::<i64>(&args.name, shape, args.freq)?,
        DataTypeWrap::F32 => make_noise::<f32>(&args.name, shape, args.freq)?,
        DataTypeWrap::F64 => make_noise::<f64>(&args.name, shape, args.freq)?,
        DataTypeWrap::C64 => make_noise::<ComplexFloatWrap>(&args.name, shape, args.freq)?,
        DataTypeWrap::C128 => make_noise::<ComplexDoubleWrap>(&args.name, shape, args.freq)?,
    }
    Ok(())
}

fn make_noise<T>(name: &str, shape: Shape, freq: f64) -> Result<()>
where
    T: IsioDataType,
    StandardUniform: Distribution<T>,
{
    // Try and create the image (noting that it might be one, two, or three
    // dimensions depending on the CLI arguments)
    println!(
        "Creating/opening a new {} image, named {}",
        format!("{:?}", T::to_datatype()).green().italic(),
        name.to_string().green().italic(),
    );
    let creation_result: Result<RawImage<T>, risio::error::Error> = match shape {
        Shape::OneDim(x) => RawImage::<T>::create_new(name, &[x]),
        Shape::TwoDim(x, y) => RawImage::<T>::create_new(name, &[x, y]),
        Shape::ThreeDim(x, y, z) => RawImage::<T>::create_new(name, &[x, y, z]),
    };

    // If the creation was successful, get the image. If it failed for an IO
    // reason, we assume the file might exist so we try opening it.
    let im = match creation_result {
        Ok(im) => im,
        Err(e) => match e {
            risio::error::Error::StdIoError(_) => {
                RawImage::<T>::open(name)?
            }
            e => return Err(e)?,
        },
    };
    let target_duration = Duration::from_secs_f64(1.0 / freq);
    println!("{} Making some noise!", "Success!".to_string().italic());
    println!("{}", "press Ctrl+C to quit".to_string().magenta().italic());
    loop {
        let start = Instant::now();
        (unsafe { im.modify(|(_, x)| *x = rand::random()) })?;
        unsafe { im._image.md.get().read().cnt0 += 1 };
        unsafe { im.sem_post_all() };
        let elapsed = start.elapsed();
        if elapsed < target_duration {
            thread::sleep(target_duration - start.elapsed());
        }
    }
}

struct ComplexFloatWrap {
    _val: ComplexFloat,
}

struct ComplexDoubleWrap {
    _val: ComplexDouble,
}

impl Distribution<ComplexFloatWrap> for StandardUniform {
    fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> ComplexFloatWrap {
        ComplexFloatWrap {
            _val: ComplexFloat {
                re: rng.random(),
                im: rng.random(),
            },
        }
    }
}

impl IsioDataType for ComplexFloatWrap {
    fn to_datatype() -> DataType {
        DataType::C64
    }
}

impl Distribution<ComplexDoubleWrap> for StandardUniform {
    fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> ComplexDoubleWrap {
        ComplexDoubleWrap {
            _val: ComplexDouble {
                re: rng.random(),
                im: rng.random(),
            },
        }
    }
}

impl IsioDataType for ComplexDoubleWrap {
    fn to_datatype() -> DataType {
        DataType::C64
    }
}
