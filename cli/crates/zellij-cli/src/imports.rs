//! Prepare imports completely before writing; compose delivery without rebuilding.
use crate::args::{ImportArgs, ImportLayout, ShipArgs};
use camino::{Utf8Path, Utf8PathBuf};
use zellij_convert::{ConvertOptions, Unmapped};
use zellij_core::{
    Error, Result,
    delivery::{DeliveryPlan, ImageKind, ImagePublication},
    files,
    site::Site,
    yaml,
};

const SITE_CONFIG: &str = "version: 1\nrenderer: hosted\n";

struct Prepared {
    unmapped: Vec<Unmapped>,
    notes: Vec<String>,
    asset_count: usize,
    files: Vec<(Utf8PathBuf, Vec<u8>)>,
}
impl Prepared {
    fn read(
        source: &str,
        out: &str,
        from: Option<&str>,
        layout: ImportLayout,
        config: &str,
    ) -> Result<Self> {
        let source = Utf8Path::new(source);
        let out = Utf8Path::new(out);
        files::disjoint(source, out)?;
        files::empty_directory(out)?;
        let converter = match from {
            Some(id) => zellij_convert::all()
                .find(|converter| converter.id() == id)
                .ok_or_else(|| {
                    Error::InvalidArgument("unknown converter; see `zel formats`".into())
                })?,
            None => {
                let found = zellij_convert::detect(source);
                let (converter, confidence) =
                    found.first().ok_or_else(|| Error::UnknownFormat {
                        path: source.to_owned(),
                    })?;
                if found.iter().skip(1).any(|(_, other)| other == confidence) {
                    let mut ids: Vec<_> = found
                        .iter()
                        .filter(|(_, other)| other == confidence)
                        .map(|(converter, _)| converter.id())
                        .collect();
                    ids.sort_unstable();
                    return Err(Error::InvalidArgument(format!(
                        "ambiguous source ({}) — select one with --from",
                        ids.join(", ")
                    )));
                }
                *converter
            }
        };
        zellij_core::statusln!("Converting {source} as {}", converter.label());
        let conversion = converter.convert(source, &ConvertOptions::default())?;
        let asset_count = conversion.assets.len();
        let mut files: Vec<_> = conversion
            .files
            .into_iter()
            .map(|file| (file.path, file.body.into_bytes()))
            .chain(
                conversion
                    .assets
                    .into_iter()
                    .map(|asset| (asset.path, asset.body)),
            )
            .collect();
        // Validate converter paths before prefixing; otherwise an absolute
        // path could replace content/ when joined.
        for (path, _) in &files {
            files::relative_path(path)?;
        }
        if layout == ImportLayout::Site {
            for (path, _) in &mut files {
                // Generated relative paths use '/' on every OS; Path::join
                // inserts '\\' on Windows, which the portable plan rejects.
                *path = format!("content/{path}").into();
            }
            files.push(("zel.yaml".into(), config.as_bytes().to_vec()));
        }
        files::preflight(
            out,
            &files.iter().map(|(p, _)| p.as_path()).collect::<Vec<_>>(),
            false,
        )?;
        Ok(Self {
            unmapped: conversion.unmapped,
            notes: conversion.notes,
            asset_count,
            files,
        })
    }

    fn report(&self) {
        if !self.unmapped.is_empty() {
            zellij_core::reportln!(
                "\n{} thing(s) could not be carried over:",
                self.unmapped.len()
            );
            for entry in &self.unmapped {
                zellij_core::reportln!("  {}: {}", entry.source, entry.reason);
            }
        }
        for note in &self.notes {
            zellij_core::reportln!("\nNote: {note}");
        }
    }

    fn write(&self, out: &str, dry_run: bool) -> Result<()> {
        if dry_run {
            zellij_core::reportln!("\nWould write {} files into {out}:", self.files.len());
            for (path, _) in &self.files {
                zellij_core::reportln!("  {path}");
            }
        } else {
            // Recheck emptiness immediately before committing the prepared files.
            files::empty_directory(Utf8Path::new(out))?;
            let files: Vec<_> = self
                .files
                .iter()
                .map(|(p, b)| (p.as_path(), b.as_slice()))
                .collect();
            files::write_files(Utf8Path::new(out), &files, false)?;
            zellij_core::statusln!(
                "\nWrote {} files into {out} ({} assets)",
                files.len(),
                self.asset_count
            );
        }
        Ok(())
    }
}

pub(crate) fn convert(options: &ImportArgs, validate: bool) -> Result<()> {
    let import = Prepared::read(
        &options.source,
        &options.out,
        options.from.as_deref(),
        options.layout,
        SITE_CONFIG,
    )?;
    import.report();
    import.write(&options.out, options.dry_run)?;
    if options.dry_run {
        zellij_core::reportln!("\nPlan only; no files written or content validation performed.");
    } else if validate {
        crate::validation::check(
            Some(&zellij_core::site::SiteInput::from(options.out.as_str())),
            None,
            Default::default(),
            false,
        )
        .inspect_err(|_| retained(&options.out))?;
    } else {
        zellij_core::statusln!(
            "\nValidate it with `zel check <destination>` using {0} as the destination.",
            options.out
        );
    }
    Ok(())
}

fn plan(options: &ShipArgs) -> Result<DeliveryPlan> {
    if let Some(kind) = options.image {
        if options.provider.is_some() || options.site_id.is_some() || options.prod {
            return Err(Error::InvalidArgument(
                "image publication cannot also select a deployment provider, site or --prod".into(),
            ));
        }
        let kind = match kind {
            crate::args::ImageKind::Static => ImageKind::Static,
            crate::args::ImageKind::Next => ImageKind::Next,
        };
        return ImagePublication::new(
            kind,
            options
                .tag
                .as_deref()
                .ok_or_else(|| Error::InvalidArgument("--image requires --tag".into()))?,
        )
        .map(DeliveryPlan::Push)
        .and_then(|plan| plan.with_platform(options.platform));
    }
    if options.provider.as_deref() != Some("netlify") || options.tag.is_some() {
        return Err(Error::InvalidArgument(
            "ship requires --provider netlify or --image static|next with --tag".into(),
        ));
    }
    let environment_site = std::env::var("NETLIFY_SITE_ID").ok();
    let id = options
        .site_id
        .as_deref()
        .or(environment_site.as_deref())
        .ok_or_else(|| {
            Error::InvalidArgument(
                "Netlify requires an existing site: pass --site-id or set NETLIFY_SITE_ID".into(),
            )
        })?;
    DeliveryPlan::netlify(id, options.prod)
}

fn configuration(plan: &DeliveryPlan) -> String {
    match plan {
        DeliveryPlan::Push(image) => format!(
            "{SITE_CONFIG}image:\n  name: {}\n{}publish:\n  action: push\n  image: {}\n",
            yaml::string(&image.tag),
            image
                .platform
                .map(|platform| format!("  platform: {platform}\n"))
                .unwrap_or_default(),
            image.kind.as_str()
        ),
        DeliveryPlan::Netlify { site, .. } => format!(
            "{SITE_CONFIG}publish:\n  action: deploy\n  provider: netlify\n  site: {}\n",
            yaml::string(site)
        ),
    }
}

fn retained(out: &str) {
    zellij_core::diagnosticln!(
        "Import completed. Converted files remain at {out}; the source is unchanged. Review the conversion report and correct the failing stage before retrying from this site."
    );
}

pub(crate) fn ship(options: &ShipArgs) -> Result<()> {
    let plan = plan(options)?;
    let import = Prepared::read(
        &options.source,
        &options.out,
        options.from.as_deref(),
        ImportLayout::Site,
        &configuration(&plan),
    )?;
    import.report();
    if options.dry_run || !zellij_core::reporting::options().quiet {
        crate::delivery::describe_destination(&plan);
    }
    import.write(&options.out, options.dry_run)?;
    if options.dry_run {
        zellij_core::reportln!(
            "\nPlan only; no files written, content validation, tools, credentials, build or remote requests performed."
        );
        return Ok(());
    }
    let execute = || {
        let site = Site::load(Utf8Path::new(&options.out))?;
        crate::delivery::execute_plan(&site, &site.content, &plan)
    };
    execute().inspect_err(|_| retained(&options.out))?;
    zellij_core::statusln!(
        "\nSaved delivery settings in {}/zel.yaml; future changes can use `zel publish <destination>` (Netlify defaults to a draft; --prod is never saved).",
        options.out
    );
    Ok(())
}
