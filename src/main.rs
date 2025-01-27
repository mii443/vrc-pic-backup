mod args;
mod compression;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use anyhow::Result;
use args::Cli;
use clap::Parser;
use compression::compress_and_save;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

///
/// Lists all the PNG files in a directory
/// 
fn list_files(dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir)
        .unwrap()
        .par_bridge()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                Some(list_files(&path))
            } else {
                // Only return PNG files
                if path.extension().map_or(false, |ext| ext == "png") {
                    Some(vec![path])
                } else {
                    None
                }
            }
        })
        .flatten()
        .collect()
}

fn main() -> Result<()> {
    // Parse the command line arguments
    let cli = Cli::parse();
    let base = cli.src;
    let to = cli.dst;
    let base_path = Path::new(&base);
    let to_path = Path::new(&to);

    if let Some(threads) = cli.threads {
        rayon::ThreadPoolBuilder::new().num_threads(threads).build_global()?;
    }

    println!("Listing files...");
    let files: Vec<PathBuf> = list_files(base_path);

    // Create a progress bar
    let bar = ProgressBar::new(files.len() as u64);
    bar.set_style(ProgressStyle::with_template("[{elapsed_precise} ({per_sec})] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}").unwrap().progress_chars("##-"));
    let bar = Arc::new(bar);

    let errors = Arc::new(std::sync::Mutex::new(Vec::new()));

    println!("Creating directories...");
    let dirs: Vec<_> = files
        .par_iter()
        .map(|path| {
            let rel_path = path.strip_prefix(base_path).unwrap();
            let mut dest_path = rel_path.to_path_buf();
            dest_path.set_extension("jpg");
            to_path.join(dest_path).parent().unwrap().to_path_buf()
        })
        .collect();

    dirs.par_iter()
        .for_each(|dir| {
            if !dir.exists() {
                let _ = std::fs::create_dir_all(dir);
            }
        });

    println!("Compressing images...");
    files.par_iter().for_each(|path| {
        // Calculate the destination path
        let rel_path = path.strip_prefix(base_path).unwrap();
        let mut dest_path = rel_path.to_path_buf();
        dest_path.set_extension("jpg");
        let full_dest = to_path.join(&dest_path);

        // Check if the file already exists
        if full_dest.exists() {
            bar.inc(1);
            return;
        }

        match compress_and_save(path, &full_dest, cli.quality) {
            Ok(_) => bar.inc(1),
            Err(e) => {
                // If an error occurs, add it to the errors list and remove the file
                errors.lock().unwrap().push((path.clone(), e));
                let _ = std::fs::remove_file(&full_dest);
            }
        }
    });

    // Print the errors
    let errors = errors.lock().unwrap();
    errors.par_iter().for_each(|(path, e)| {
        eprintln!("Error: {:?} {:?}", path, e);
    });

    bar.finish();
    println!("Done");

    Ok(())
}
