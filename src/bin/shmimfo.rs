use anyhow::Result;
use clap::Parser;
use glob::glob;
use rich_rust::{r#box::*, prelude::*};
use risio::{
    Accessor, RawImage,
    datatype::{ComplexFloat, DataType, IsioDataType},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the shared memory image(s)
    names: Vec<String>,
    /// write the output in "bare" mode, without pretty formatting.
    #[clap(long, short, action)]
    bare: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let files = match args.names.len() {
        0 => shmim_names(glob("/dev/shm/*.im.shm").unwrap()),
        _ => args.names,
    };
    let mut shmimfos = ShmImfoVec { v: vec![] };
    for name in files {
        if let Ok(im) = RawImage::<u8>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        if let Ok(im) = RawImage::<u16>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        if let Ok(im) = RawImage::<u32>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        if let Ok(im) = RawImage::<u64>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        if let Ok(im) = RawImage::<i8>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        if let Ok(im) = RawImage::<i16>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        if let Ok(im) = RawImage::<i32>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        if let Ok(im) = RawImage::<i64>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        // if let Ok(im) = RawImage::<f16>::open(&name) {
        //     shmimfos.v.push((&im).into());
        //     continue;
        // }
        if let Ok(im) = RawImage::<f32>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        if let Ok(im) = RawImage::<f64>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        if let Ok(im) = RawImage::<ComplexFloat>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        if let Ok(im) = RawImage::<ComplexFloat>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        // shmimfos.v.push(shmimfo);
    }
    if args.bare {
        println!("name dtype shape cnt1");
        for shmimfo in shmimfos.v {
            let ShmImfo {
                name,
                dtype,
                shape,
                cnt1,
            } = shmimfo;
            println!(
                "{name} {dtype} [{},{},{}] {cnt1}",
                shape[0], shape[1], shape[2]
            );
        }
    } else {
        let table: Table = shmimfos.into();
        let console = Console::new();
        console.print_renderable(&table);
    }
    Ok(())
}

fn shmim_names(paths: glob::Paths) -> Vec<String> {
    paths
        .into_iter()
        .map(|path| {
            path.unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .rsplit_once(".im.shm")
                .unwrap()
                .0
                .to_string()
        })
        .collect()
}

impl<'a, T: IsioDataType> From<&'a RawImage<'a, T>> for ShmImfo {
    fn from(image: &'a RawImage<'a, T>) -> Self {
        let x = unsafe { image.image().md.get().read() }.cnt0;
        ShmImfo {
            name: image.name().to_string(),
            dtype: match TryInto::<DataType>::try_into(
                unsafe { image._image.md.get().read() }.datatype,
            ) {
                Ok(dt) => format!("{:?}", dt).to_ascii_lowercase(),
                Err(e) => e.to_string(),
            },
            shape: unsafe { image._image.md.get().read().size },
            cnt1: x,
        }
    }
}

struct ShmImfo {
    name: String,
    dtype: String,
    shape: [u32; 3],
    cnt1: u64,
}

impl From<ShmImfo> for Row {
    fn from(value: ShmImfo) -> Self {
        Row::new(vec![
            Cell::new(value.name),
            Cell::new(value.dtype.to_lowercase()),
            Cell::new(format!(
                "[ {:^4} , {:^4} , {:^4} ]",
                value.shape[0], value.shape[1], value.shape[2]
            )),
            Cell::new(value.cnt1.to_string()),
        ])
    }
}

// TODO: Add a bare mode without table formatting

struct ShmImfoVec {
    v: Vec<ShmImfo>,
}

impl From<ShmImfoVec> for Table {
    fn from(val: ShmImfoVec) -> Table {
        let mut table = Table::new()
            .border_style(Style::new().color(Color::from_ansi(6)))
            .header_style(Style::new().color(Color::from_ansi(2)))
            .box_style(&SQUARE);
        table.add_columns(vec![
            Column::new("NAME"),
            Column::new("DTYPE").justify(JustifyMethod::Center),
            Column::new("SHAPE").justify(JustifyMethod::Center),
            Column::new("CNT1").justify(JustifyMethod::Center),
        ]);
        for x in val.v {
            table.add_row(x.into());
        }
        table
    }
}
