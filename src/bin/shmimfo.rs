use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use glob::glob;
use risio::{Accessor, RawImage, datatype::DataType};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the shared memory image
    names: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let files = match args.names.len() {
        0 => {
            let mut files: Vec<String> = vec![];
            for entry in glob("/dev/shm/*.im.shm").unwrap() {
                match entry {
                    Ok(path) => {
                        let shmimname = path
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .rsplit_once(".im.shm")
                            .unwrap()
                            .0
                            .to_string();
                        files.push(shmimname);
                    }
                    Err(e) => eprintln!("{e}"),
                };
            }
            files
        }
        _ => args.names,
    };

    println!(
        "{}",
        "---------------------------------------------------------"
            .to_string()
            .blue()
    );
    for name in files {
        let image: RawImage<f64> = RawImage::open(&name)?;

        // print shmim name
        let name = image.name().to_string();
        println!("name:  {}", name.green());

        // print datatype
        // let datatype: DataType = unsafe { image._image.md.read() }.datatype.try_into()?;
        match TryInto::<DataType>::try_into(unsafe { image._image.md.get().read() }.datatype) {
            Ok(dt) => println!("type:  {}", format!("{:?}", dt).green()),
            Err(e) => println!("type:  {}", format!("{}", e.to_string()).red()),
        }

        // print dimensions
        let size = unsafe { image._image.md.get().read().size };
        let size_str = match unsafe { image._image.md.get().read().naxis } {
            1 => format!("[{}]", size[0]),
            2 => format!("[{},{}]", size[0], size[1]),
            3 => format!("[{},{},{}]", size[0], size[1], size[2]),
            x => format!("invalid SHM image, naxis={x}, must be 1, 2, or 3."),
        };
        println!("size:  {}", size_str.green());

        // print some stats:
        let sum = unsafe { image.array().iter().sum::<f64>() };
        let mean: f64 = sum / unsafe { image.array().len() } as f64;
        let std = unsafe { image
            .array()
            .iter()
            .map(|x| (*x - mean).powf(2.0))
            .sum::<f64>() }
            / unsafe { image.array().len() } as f64;
        println!("sum:   {}", format!("{:0.6e}", sum).green());
        println!(
            "mean:  {} (std: {})",
            format!("{:0.6e}", mean).green(),
            format!("{:0.6e}", std).green()
        );
        println!(
            "{}",
            "---------------------------------------------------------"
                .to_string()
                .blue()
        );
    }
    Ok(())
}
