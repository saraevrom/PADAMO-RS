use std::{cmp::min, fs::File, io::Write, path::{Path, PathBuf}};

use hex_literal::hex;
use indicatif::ProgressBar;
use hex::{FromHex, ToHex};
use futures_util::StreamExt;
use sha2::{Digest, Sha256};

cfg_match::cfg_match! {
    #[cfg(target_os = "linux")]
    #[cfg(target_arch = "x86_64")]
    fn get_official_build()->Option<(&'static str, &'static [u8;32])>{
        Some((
            "https://github.com/microsoft/onnxruntime/releases/download/v1.23.2/onnxruntime-linux-x64-gpu-1.23.2.tgz",
            &hex!("2083e361072a79ce16a90dcd5f5cb3ab92574a82a3ce0ac01e5cfa3158176f53")
        ))
    }
    #[cfg(target_os = "windows")]
    #[cfg(target_arch = "x86_64")]
    fn get_official_build()->Option<(&'static str, &'static[u8;32])>{
        Some((
            "https://github.com/microsoft/onnxruntime/releases/download/v1.23.2/onnxruntime-win-x64-gpu-1.23.2.zip",
            &hex!("e77afdbbc2b8cb6da4e5a50d89841b48c44f3e47dce4fb87b15a2743786d0bb9")
        ))
    }

    #[cfg(_)]
    fn get_official_build()->Option<(&'static str, &'static[u8;32])>{
        None
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DownloadError{
    #[error("Network: {0}")]
    ClientError(#[from] reqwest::Error),
    #[error("IO: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Checksums are not matching. Expected {0}, found {1}")]
    ChecksumsMismatch(String, String),
    #[error("No file name")]
    NoFileName,
}


// Credit to https://gist.github.com/Tapanhaz/096e299bf060607b572d700e89a62529
pub async fn download_file_async( url: &str, path: &Path, sha256_checksum:Option<&[u8]>) -> Result<PathBuf, DownloadError> {
    let filename = Path::new(&url).file_name();
    if let Some(f) = filename{
        let path1 = path.join(f);
        if let Some(v) = sha256_checksum{
            if path1.is_file(){
                if check_integrity(&path1, v).is_ok(){
                    println!("File is downloaded and integrity is OK");
                    return Ok(path1);
                }
            }
        }
    }


    // Reqwest setup
    let client = reqwest::Client::new();
    let res = client.get(url)
        .send()
        .await?;
    let total_size = res.content_length(); //No context length means no progressbar

    // Indicatif setup
    let pb = if let Some(size) = total_size{
        ProgressBar::new(size)
    }
    else{
        ProgressBar::new_spinner()
    };

    let filename = filename.map(|x| x.to_str().map(String::from)).flatten().or_else(||{
            res
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name.to_string()) })
    }).ok_or(DownloadError::NoFileName)?;

    let path1 = path.join(filename);
    if let Some(v) = sha256_checksum{
        if path1.is_file(){
            if check_integrity(&path1, v).is_ok(){
                println!("File is downloaded and integrity is OK");
                return Ok(path1);
            }
        }
    }

    println!("Downloading file {:?}", path1);
    // download chunks
    let mut file = File::create(&path1)?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk)?;
        let mut new = downloaded + (chunk.len() as u64);
        if let Some(v) = total_size{
            new = min(new,v);
        }
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message("Done");

    if let Some(sha256_chk) = sha256_checksum{
        check_integrity(&path1, sha256_chk)?;
    }
    // let mut file = File::open(path)?;
    // let mut sha256 = Sha256::new();
    // io::copy(&mut file, &mut sha256)?;
    // let hash = sha256.result();

    return Ok(path1);
}

fn check_integrity(path: &Path, sha256_chk: &[u8]) -> Result<(), DownloadError> {
    let mut file = File::open(path)?;
    let mut sha256 = Sha256::new();
    std::io::copy(&mut file, &mut sha256)?;
    let hash = sha256.finalize();
    let target = sha256_chk.encode_hex();
    let got = hash.as_slice().encode_hex();
    println!("Target {}", target);
    println!("Got {}", got);

    Ok(if hash[..] != sha256_chk[..]{
        return Err(DownloadError::ChecksumsMismatch(target,got));
    })
}

fn download_file( url: &str, path: &Path, sha256_checksum:Option<&[u8]>) -> Result<PathBuf, DownloadError> {
    // smol::block_on(download_file_async(url, path, sha256_checksum))
    // let job = tokio::spawn(async move {
    //     download_file_async(url, path, sha256_checksum)
    // });
    // job.


    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(download_file_async(url, path, sha256_checksum))
}

pub fn download_onnx(target_dir:&Path) -> Result<PathBuf, DownloadError> {
    if let Some((official_url, official_sha256)) = get_official_build(){
        if super::setup_onnx::ask_yesno("Download from official source?", Some(true)){
            return download_file(official_url, target_dir, Some(official_sha256.as_slice()));
        }
    }

    let src = super::setup_onnx::ask("Enter the url of the distriution archive");
    let sha256 = super::setup_onnx::ask("Enter sha256 checksum (leave empty to skip)");
    let sum:Option<[u8;32]> = if sha256.is_empty(){
        None
    }
    else{
        FromHex::from_hex(sha256).ok()
    };
    download_file(&src, target_dir, (&sum).as_ref().map(|x| x.as_slice()))
}
