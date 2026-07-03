use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use risio::{Accessor, DataType, RawImage};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the shared memory image
    name: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let image: RawImage<u8> = RawImage::open(&args.name)?;

    // print shmim name
    let name = image
        ._image
        .name
        .clone()
        .iter()
        .filter_map(|x| match x {
            0 => None,
            x => Some(*x as u8 as char),
        })
        .collect::<String>();
    println!("name:  {}", name.green());

    // print datatype
    // let datatype: DataType = unsafe { image._image.md.read() }.datatype.try_into()?;
    match TryInto::<DataType>::try_into(unsafe { image._image.md.read() }.datatype) {
        Ok(dt) => println!("type:  {}", format!("{:?}", dt).green()),
        Err(e) => println!("type:  {}", format!("{}", e.to_string()).red()),
    }

    // print dimensions
    let size = unsafe { image._image.md.read().size };
    let size_str = match unsafe { image._image.md.read().naxis } {
        1 => format!("[{}]", size[0]),
        2 => format!("[{},{}]", size[0], size[1]),
        3 => format!("[{},{},{}]", size[0], size[1], size[2]),
        x => format!("invalid SHM image, naxis={x}, must be 1, 2, or 3."),
    };
    println!("size:  {}", size_str.green());

    // // print some stats:
    // let datatype = unsafe { image._image.md.read() }.datatype
    // let sum = image.array().iter().sum();
    // let std = image.array().iter().map(|x| x as )

    Ok(())
}
