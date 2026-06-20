use std::{
    fs::{File, create_dir_all},
    io::{BufWriter, Write},
    path::Path,
};

use crate::utils::{
    header::Header, patch_dir::PatchDir, patch_krdir::KrPatchDir, patch_sf::PatchSF,
    patch_single::PatchSingle, structs::DataReferenceInfo,
};

mod utils;

// RIP tests

pub fn patch_hdiff(
    source_path: &Path,
    diff_path: &Path,
    dest_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut diff_file = File::open(diff_path)?;

    let mut header_info = Default::default();
    let mut reference_info: DataReferenceInfo = Default::default();
    let is_dir_patch = Header::try_parse_header_info(
        &mut diff_file,
        diff_path,
        &mut header_info,
        &mut reference_info,
    )?;

    if is_dir_patch && header_info.is_input_dir && header_info.is_output_dir {
        let mut patcher = PatchDir::new(header_info, reference_info, diff_path);
        patcher.patch(source_path, dest_path, None)?;
        return Ok(());
    }

    let mut old_file = File::open(source_path)?;
    let old_len = old_file.metadata()?.len() as i64;
    if old_len != header_info.old_data_size {
        return Err(format!(
            "[HDiff] Input file size mismatch: expected {} bytes, got {} bytes",
            header_info.old_data_size, old_len
        )
        .into());
    }

    #[cfg(debug_assertions)]
    println!(
        "[HDiff] Old size: {} ✓ | New size: {}",
        old_len, header_info.new_data_size
    );

    let out_file = File::create(dest_path)?;
    let mut out_writer = BufWriter::new(out_file);
    if header_info.is_single_compressed_diff {
        PatchSF::new(header_info).patch(&mut old_file, &mut out_writer, diff_path, None)?;
    } else {
        PatchSingle::new(header_info).patch(&mut old_file, &mut out_writer, diff_path, None)?;
    }
    out_writer.flush()?;
    Ok(())
}

/// "extremely cursed"
pub fn patch_krdiff(
    source_path: &Path,
    diff_path: &Path,
    dest_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if !source_path.exists() || !source_path.is_dir() {
        return Err(format!(
            "[KrDiff] Source path {} does not exist or is not a directory",
            source_path.display()
        )
        .into());
    }
    if !diff_path.exists() || !diff_path.is_file() {
        return Err(format!("[KrDiff] Diff file {} does not exist", diff_path.display()).into());
    }
    if !dest_path.exists() {
        create_dir_all(&dest_path)?;
    }

    let patcher = KrPatchDir::new(diff_path);
    patcher.patch(source_path, dest_path, None)?;
    Ok(())
}
