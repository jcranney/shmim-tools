use anyhow::Result;
use clap::Parser;
use glob::glob;
use rich_rust::prelude::*;
use risio::{
    Accessor, RawImage,
    datatype::{DataType, IsioDataType},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the shared memory image
    names: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let files = match args.names.len() {
        0 => shmim_names(glob("/dev/shm/*.im.shm").unwrap()),
        _ => args.names,
    };
    let mut shmimfos = ShmImfoVec { v: vec![] };
    for name in files {
        let image: RawImage<f64> = RawImage::open(&name)?;
        shmimfos.v.push((&image).into());
    }
    let table: Table = shmimfos.into();
    let console = Console::new();
    console.print_renderable(&table);
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
                Ok(dt) => format!("{:?}", dt),
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
            Cell::new(value.dtype),
            Cell::new(format!(
                "[ {:^6} , {:^6} , {:^6} ]",
                value.shape[0], value.shape[1], value.shape[2]
            )),
            Cell::new(value.cnt1.to_string()),
        ])
    }
}

struct ShmImfoVec {
    v: Vec<ShmImfo>,
}

impl From<ShmImfoVec> for Table {
    fn from(val: ShmImfoVec) -> Table {
        let mut table = Table::new().title("ImageStreamIO IMAGEs");
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
