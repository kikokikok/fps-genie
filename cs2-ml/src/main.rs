use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod data;
mod model;
mod server;
mod player;

#[derive(Parser)]
#[command(name = "cs2-ml")]
#[command(about = "CS2 behavior-cloning ML pipeline")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert demos → Parquet
    Prepare {
        demo_glob: String,
        output_dir: PathBuf,
    },
    /// Train the policy network
    Train {
        parquet: PathBuf,
        model_out: PathBuf,
        #[arg(long, default_value = "1000")]
        epochs: i64,
    },
    /// Serve the trained policy
    Serve {
        model: PathBuf,
        #[arg(long, default_value = "8123")]
        port: u16,
    },
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Prepare { demo_glob, output_dir } => {
            std::fs::create_dir_all(&output_dir)?;
            for entry in glob::glob(&demo_glob)? {
                let demo = entry?;
                let vecs = data::vectors_from_demo(&demo)?;
                let out = output_dir.join(demo.file_stem().unwrap()).with_extension("parquet");
                data::write_parquet(&vecs, &out)?;
                println!("Wrote {}", out.display());
            }
        }
        Commands::Train { parquet, model_out, epochs } => {
            use parquet::file::reader::SerializedFileReader;
            use parquet::record::reader::RowIter;
            use parquet::record::RowAccessor;
            let file = std::fs::File::open(parquet)?;
            let reader = SerializedFileReader::new(file)?;
            let row_iter = RowIter::from_file(None, &reader)?;
            let mut dataset = Vec::new();
            for row_result in row_iter {
                let row = row_result?;
                let vec: Vec<f32> = (0..14)
                    .map(|i| row.get_double(i).unwrap() as f32)
                    .collect();
                let label = vec![
                    row.get_double(14).unwrap() as f32,
                    row.get_double(15).unwrap() as f32
                ];
                dataset.push((vec, label));
            }
            let vs = tch::nn::VarStore::new(tch::Device::Cpu);
            model::BehaviorNet::train(&vs.root(), dataset, epochs, 0.001)?;
            vs.save(&model_out)?;
            println!("Model saved to {}", model_out.display());
        }
        Commands::Serve { model, port } => {
            server::serve(model.to_str().unwrap(), port)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_help() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
