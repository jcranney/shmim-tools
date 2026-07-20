use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use glob::glob;
use rich_rust::{r#box::*, prelude::*};
use risio::{
    Accessor, ShmImage,
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
    /// estimate the frequency of the shmimage by timing the counts.
    /// Note that this requires blocking the thread to estimate the duration,
    /// so shouldn't be used in high-performance pipelines.
    #[clap(long, short)]
    freq: bool,
    /// Period (in seconds) over which to estimate the frequency - longer period means a
    /// more precise estimate.
    #[clap(long, short, default_value = "0.01")]
    period: f64,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let files = match args.names.len() {
        0 => shmim_names(glob("/dev/shm/*.im.shm").unwrap()),
        _ => args.names,
    };
    let mut shmimfos = extract_shmimfo(&files);
    if args.freq {
        std::thread::sleep(Duration::from_secs_f64(args.period));
        for (shmimfo_old, shmimfo_new) in shmimfos.v.iter_mut().zip(extract_shmimfo(&files).v) {
            let cnt_diff = u64::saturating_sub(shmimfo_new.cnt1, shmimfo_old.cnt1);
            match cnt_diff {
                0 => {
                    shmimfo_old.freq_est = FreqEst::TooSlow {
                        period: shmimfo_new
                            .time_accessed
                            .duration_since(shmimfo_old.time_accessed)
                            .as_secs_f64(),
                    };
                }
                c => {
                    shmimfo_old.freq_est = FreqEst::Freq {
                        cnt_diff: c,
                        period: shmimfo_new
                            .time_accessed
                            .duration_since(shmimfo_old.time_accessed)
                            .as_secs_f64(),
                    }
                }
            }
            shmimfo_old.cnt1 = shmimfo_new.cnt1;
        }
        shmimfos.freq = true;
    }
    if args.bare {
        match args.freq {
            false => println!("name dtype shape cnt1"),
            true => println!("name dtype shape cnt1 freq"),
        }
        for shmimfo in shmimfos.v {
            let ShmImfo {
                name,
                dtype,
                shape,
                cnt1,
                freq_est,
                ..
            } = shmimfo;
            match args.freq {
                false => println!(
                    "{name} {dtype} [{},{},{}] {cnt1}",
                    shape[0], shape[1], shape[2]
                ),
                true => println!(
                    "{name} {dtype} [{},{},{}] {cnt1} {freq_est}",
                    shape[0], shape[1], shape[2],
                ),
            }
        }
    } else {
        let table: Table = shmimfos.into();
        let console = Console::new();
        console.print_renderable(&table);
    }
    Ok(())
}

fn extract_shmimfo(files: &[String]) -> ShmImfoVec {
    let mut shmimfos = ShmImfoVec {
        v: vec![],
        freq: false,
    };
    for name in files {
        if let Ok(im) = ShmImage::<u8>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        if let Ok(im) = ShmImage::<u16>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        if let Ok(im) = ShmImage::<u32>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        if let Ok(im) = ShmImage::<u64>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        if let Ok(im) = ShmImage::<i8>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        if let Ok(im) = ShmImage::<i16>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        if let Ok(im) = ShmImage::<i32>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        if let Ok(im) = ShmImage::<i64>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        // if let Ok(im) = ShmImage::<f16>::open(&name) {
        //     shmimfos.v.push((&im).into());
        //     continue;
        // }
        if let Ok(im) = ShmImage::<f32>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        if let Ok(im) = ShmImage::<f64>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        if let Ok(im) = ShmImage::<ComplexFloat>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
        if let Ok(im) = ShmImage::<ComplexFloat>::open(&name) {
            shmimfos.v.push((&im).into());
            continue;
        }
    }
    shmimfos
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

impl<'a, T: IsioDataType> From<&'a ShmImage<'a, T>> for ShmImfo {
    fn from(image: &'a ShmImage<'a, T>) -> Self {
        let x = image.metadata().cnt0;
        ShmImfo {
            name: unsafe { image.name().to_string() },
            dtype: match TryInto::<DataType>::try_into(image.metadata().datatype) {
                Ok(dt) => format!("{:?}", dt).to_ascii_lowercase(),
                Err(e) => e.to_string(),
            },
            shape: image.metadata().size,
            cnt1: x,
            freq_est: FreqEst::None,
            time_accessed: Instant::now(),
        }
    }
}

enum FreqEst {
    None,
    TooSlow { period: f64 },
    Freq { cnt_diff: u64, period: f64 },
}

impl std::fmt::Display for FreqEst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FreqEst::None => "N/A".to_string(),
                FreqEst::TooSlow { period } => format!("<{:0.1}", 1.0 / period),
                FreqEst::Freq { cnt_diff, period } => format!("~{:0.1}", *cnt_diff as f64 / period),
            }
        )
    }
}

struct ShmImfo {
    name: String,
    dtype: String,
    shape: [u32; 3],
    cnt1: u64,
    freq_est: FreqEst,
    time_accessed: Instant,
}

impl From<ShmImfo> for Row {
    fn from(value: ShmImfo) -> Self {
        let mut row = vec![
            Cell::new(value.name),
            Cell::new(value.dtype.to_lowercase()),
            Cell::new(format!(
                "[ {:^4} , {:^4} , {:^4} ]",
                value.shape[0], value.shape[1], value.shape[2]
            )),
            Cell::new(value.cnt1.to_string()),
        ];
        match value.freq_est {
            FreqEst::None => (),
            x => row.push(Cell::new(format!("{}", x))),
        }
        Row::new(row)
    }
}

// TODO: Add a bare mode without table formatting
struct ShmImfoVec {
    v: Vec<ShmImfo>,
    freq: bool,
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
        if val.freq {
            table.add_column(Column::new("FREQ (Hz)").justify(JustifyMethod::Center));
        }
        for x in val.v {
            table.add_row(x.into());
        }
        table
    }
}
