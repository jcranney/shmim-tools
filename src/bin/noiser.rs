use std::fmt::Display;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use anyhow::{Result, anyhow};
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
    /// Name of the shared memory image.
    name: String,
    /// Datatype of image. If unspecified, the image must already exist.
    datatype: Option<DataTypeWrap>,
    /// len of first axis. If unspecified, the image must already exist.
    dim0: Option<usize>,
    /// len of second axis.
    dim1: Option<usize>,
    /// len of third axis.
    dim2: Option<usize>,
    /// Target framerate.
    #[clap(short, long, default_value = "100")]
    freq: f64,
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

#[derive(Copy, Clone)]
enum Shape {
    NotSpecified,
    OneDim(usize),
    TwoDim(usize, usize),
    ThreeDim(usize, usize, usize),
}

fn main() -> Result<()> {
    let args = Args::parse();
    let shape = match (args.dim0, args.dim1, args.dim2) {
        (None, _, _) => Shape::NotSpecified,
        (Some(x), None, None) => Shape::OneDim(x),
        (Some(x), Some(y), None) => Shape::TwoDim(x, y),
        (Some(x), Some(y), Some(z)) => Shape::ThreeDim(x, y, z),
        (_, None, Some(_)) => unreachable!("shouldn't be able to get here with serial arguments"),
    };
    match args.datatype {
        Some(x) => match x {
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
        },
        None => {
            // We have to try all the options and one will be the right datatype
            // Any errors will be propagated in the expected way.
            make_noise::<u8>(&args.name, shape, args.freq)
                .or_else(|_| make_noise::<u16>(&args.name, shape, args.freq))
                .or_else(|_| make_noise::<u32>(&args.name, shape, args.freq))
                .or_else(|_| make_noise::<u64>(&args.name, shape, args.freq))
                .or_else(|_| make_noise::<i8>(&args.name, shape, args.freq))
                .or_else(|_| make_noise::<i16>(&args.name, shape, args.freq))
                .or_else(|_| make_noise::<i32>(&args.name, shape, args.freq))
                .or_else(|_| make_noise::<i64>(&args.name, shape, args.freq))
                .or_else(|_| make_noise::<f32>(&args.name, shape, args.freq))
                .or_else(|_| make_noise::<f64>(&args.name, shape, args.freq))
                .or_else(|_| make_noise::<ComplexFloatWrap>(&args.name, shape, args.freq))
                .or_else(|_| make_noise::<ComplexDoubleWrap>(&args.name, shape, args.freq))
                .inspect_err(|_| {
                    eprintln!("Couldn't open the image using any valid datatypes!")
                })?
        }
    };
    Ok(())
}

fn make_noise<T>(name: &str, shape: Shape, freq: f64) -> Result<()>
where
    T: IsioDataType,
    StandardUniform: Distribution<T>,
{
    // Try and create the image (noting that it might be one, two, or three
    // dimensions depending on the CLI arguments)
    let creation_result: Result<RawImage<T>, risio::error::Error> = match shape {
        Shape::OneDim(x) => RawImage::<T>::create_new(name, &[x]),
        Shape::TwoDim(x, y) => RawImage::<T>::create_new(name, &[x, y]),
        Shape::ThreeDim(x, y, z) => RawImage::<T>::create_new(name, &[x, y, z]),
        Shape::NotSpecified => RawImage::<T>::open(name),
    };

    // If the creation was successful, get the image. If it failed for an IO
    // reason, we assume the file might exist so we try opening it.
    let im = match creation_result {
        Ok(im) => im,
        Err(e) => match e {
            risio::error::Error::StdIoError(_) => RawImage::<T>::open(name)?,
            e => return Err(e)?,
        },
    };

    // It should be an error to load an image with a shape different to the one
    // you requested
    let im_shape = match im.metadata().size.map(|x| x as usize) {
        [0, 0, 0] => Shape::NotSpecified,
        [x, 0, 0] => Shape::OneDim(x),
        [x, y, 0] => Shape::TwoDim(x, y),
        [x, y, z] => Shape::ThreeDim(x, y, z),
    };
    // precompute error message incase it's needed
    let err = anyhow!(
        "mismatched dimensions. Expected {}, found {} in {}.im.shm",
        shape,
        im_shape,
        name
    );
    match (im_shape, shape) {
        (_, Shape::NotSpecified) => (), // don't care!
        (Shape::OneDim(x0), Shape::OneDim(x1)) => {
            if x0 != x1 {
                return Err(err);
            }
        }
        (Shape::TwoDim(x0, y0), Shape::TwoDim(x1, y1)) => {
            if x0 != x1 || y0 != y1 {
                return Err(err);
            }
        }
        (Shape::ThreeDim(x0, y0, z0), Shape::ThreeDim(x1, y1, z1)) => {
            if x0 != x1 || y0 != y1 || z0 != z1 {
                return Err(err);
            }
        }
        (_, _) => return Err(err),
    };
    println!(
        "Image name: {}, datatype: {}, shape: {}",
        name.to_string().green().italic(),
        format!("{:?}", T::to_datatype())
            .to_ascii_lowercase()
            .green()
            .italic(),
        format!(
            "[{},{},{}]",
            im.metadata().size[0].to_string().green().italic(),
            im.metadata().size[1].to_string().green().italic(),
            im.metadata().size[2].to_string().green().italic()
        )
    );
    let target_duration = Duration::from_secs_f64(1.0 / freq);
    println!(
        "{} Making some noise at {} Hz!",
        "Success!".to_string().italic(),
        freq.to_string().green()
    );
    println!(
        "{}",
        "press Ctrl+C to cleanly quit."
            .to_string()
            .magenta()
            .italic()
    );
    let (tx, rx) = channel();

    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    loop {
        let start = Instant::now();
        (unsafe { im.modify(|(_, x)| *x = rand::random()) })?;
        unsafe { im._image.md.get().read().cnt0 += 1 };
        unsafe { im.sem_post_all() };
        match rx.try_recv() {
            Ok(()) => break,
            Err(e) => match e {
                std::sync::mpsc::TryRecvError::Empty => (),
                e => return Err(e)?,
            },
        }
        let elapsed = start.elapsed();
        if elapsed < target_duration {
            thread::sleep(target_duration - start.elapsed());
        }
    }
    Ok(())
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

impl Display for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Shape::NotSpecified => write!(f, "[?,?,?]"),
            Shape::OneDim(x) => write!(f, "[{x},0,0]"),
            Shape::TwoDim(x, y) => write!(f, "[{x},{y},0]"),
            Shape::ThreeDim(x, y, z) => write!(f, "[{x},{y},{z}]"),
        }
    }
}
