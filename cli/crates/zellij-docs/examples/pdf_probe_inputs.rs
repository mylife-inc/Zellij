//! Maintainer-only export of generated print inputs for toolchain comparisons.
//! This is not a CLI command and does not run TeX or publish a PDF.
use camino::Utf8PathBuf;
use zellij_core::{files, site::SiteInput};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        return Err("usage: pdf_probe_inputs SITE NEW_OUTPUT_DIRECTORY".into());
    }
    let source = SiteInput::from(args[0].as_str());
    let output = Utf8PathBuf::from(&args[1]);
    let prepared = zellij_docs::pdf::prepare(&source, &output, Default::default(), false)?;
    // Refuse to overwrite even a previously owned PDF output.
    std::fs::create_dir(&output)?;
    files::write_files(
        &output,
        &prepared
            .source
            .files
            .iter()
            .map(|(path, bytes)| (path.as_path(), bytes.as_slice()))
            .collect::<Vec<_>>(),
        false,
    )?;
    Ok(())
}
