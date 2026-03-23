use crate::storage;

use serde::Deserialize;
use std::path::Path;
use tauri::{AppHandle, Emitter};

const VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const RESOURCES_BASE_URL: &str = "https://resources.download.minecraft.net";

#[derive(Deserialize)]
struct VersionManifest {
    versions: Vec<VersionEntry>,
}

#[derive(Deserialize)]
struct VersionEntry {
    id: String,
    url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VersionJson {
    asset_index: AssetIndexRef,
    downloads: Downloads,
}

#[derive(Deserialize)]
struct AssetIndexRef {
    url: String,
}

#[derive(Deserialize)]
struct Downloads {
    client: DownloadEntry,
}

#[derive(Deserialize)]
struct DownloadEntry {
    url: String,
    sha1: String,
    size: u64,
}

#[derive(Deserialize)]
struct AssetIndexJson {
    objects: std::collections::HashMap<String, AssetObject>,
}

#[derive(Deserialize)]
struct AssetObject {
    hash: String,
    size: u64,
}

#[derive(serde::Serialize, Clone)]
pub struct DownloadProgress {
    pub downloaded: u32,
    pub total: u32,
    pub status: String,
}

pub fn needs_download(version: &str) -> bool {
    let no_index = !storage::indexes_dir()
        .join(format!("{version}.json"))
        .exists();
    let no_jar = !storage::assets_dir()
        .join("jar")
        .join(".extracted")
        .exists();
    no_index || no_jar
}

pub async fn download(app: &AppHandle, version: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let index_path = storage::indexes_dir().join(format!("{version}.json"));

    let (asset_index, version_json) = if index_path.exists() {
        let content = std::fs::read_to_string(&index_path).map_err(|e| e.to_string())?;
        let idx: AssetIndexJson = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        (idx, None)
    } else {
        emit_progress(app, 0, 1, "Fetching version manifest...");
        let manifest: VersionManifest = client
            .get(VERSION_MANIFEST_URL)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        let entry = manifest
            .versions
            .iter()
            .find(|v| v.id == version)
            .ok_or_else(|| format!("Version {version} not found"))?;

        let vj: VersionJson = client
            .get(&entry.url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        let index_content = client
            .get(&vj.asset_index.url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        std::fs::write(&index_path, &index_content).map_err(|e| e.to_string())?;
        let idx: AssetIndexJson =
            serde_json::from_str(&index_content).map_err(|e| e.to_string())?;
        (idx, Some(vj))
    };

    download_objects(app, &client, &asset_index).await?;
    download_jar(app, &client, version, version_json.as_ref()).await?;

    Ok(())
}

async fn download_objects(
    app: &AppHandle,
    client: &reqwest::Client,
    index: &AssetIndexJson,
) -> Result<(), String> {
    let total = index.objects.len() as u32;
    let mut downloaded = 0u32;
    let mut skipped = 0u32;
    let objects = storage::objects_dir();

    for (name, obj) in &index.objects {
        let prefix = &obj.hash[..2];
        let dir = objects.join(prefix);
        let path = dir.join(&obj.hash);

        if path.exists() && std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0) == obj.size {
            skipped += 1;
            continue;
        }

        let _ = std::fs::create_dir_all(&dir);
        let url = format!("{RESOURCES_BASE_URL}/{prefix}/{}", obj.hash);

        let bytes = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to download {name}: {e}"))?
            .bytes()
            .await
            .map_err(|e| e.to_string())?;

        std::fs::write(&path, &bytes).map_err(|e| e.to_string())?;
        downloaded += 1;

        if downloaded.is_multiple_of(50) {
            let need = total - skipped;
            emit_progress(
                app,
                downloaded,
                need,
                &format!("Downloading assets ({downloaded}/{need})"),
            );
        }
    }

    emit_progress(app, total, total, "Assets downloaded");
    Ok(())
}

async fn download_jar(
    app: &AppHandle,
    client: &reqwest::Client,
    version: &str,
    cached_vj: Option<&VersionJson>,
) -> Result<(), String> {
    let jar_assets = storage::assets_dir().join("jar");
    let marker = jar_assets.join(".extracted");
    if marker.exists() {
        return Ok(());
    }

    let fetched;
    let dl = if let Some(vj) = cached_vj {
        &vj.downloads.client
    } else {
        emit_progress(app, 0, 1, "Fetching version info...");
        let manifest: VersionManifest = client
            .get(VERSION_MANIFEST_URL)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;
        let entry = manifest
            .versions
            .iter()
            .find(|v| v.id == version)
            .ok_or_else(|| format!("Version {version} not found"))?;
        let vj: VersionJson = client
            .get(&entry.url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;
        fetched = vj.downloads.client;
        &fetched
    };

    let jar_path = storage::versions_dir().join(format!("{version}.jar"));
    if !jar_path.exists() || std::fs::metadata(&jar_path).map(|m| m.len()).unwrap_or(0) != dl.size {
        emit_progress(app, 0, 1, "Downloading client JAR...");
        let bytes = client
            .get(&dl.url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .bytes()
            .await
            .map_err(|e| e.to_string())?;

        let actual_hash = format!("{}", sha1_smol::Sha1::from(&bytes).digest());
        if actual_hash != dl.sha1 {
            log::warn!("JAR hash mismatch: expected {}, got {actual_hash}", dl.sha1);
        }

        std::fs::write(&jar_path, &bytes).map_err(|e| e.to_string())?;
    }

    emit_progress(app, 0, 1, "Extracting client JAR...");
    extract_jar(&jar_path, &jar_assets)?;
    std::fs::write(&marker, version).map_err(|e| e.to_string())?;

    Ok(())
}

fn extract_jar(jar_path: &Path, output_dir: &Path) -> Result<(), String> {
    let file = std::fs::File::open(jar_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();

        if !name.starts_with("assets/") || entry.is_dir() {
            continue;
        }

        let out_path = output_dir.join(&name);
        if let Some(parent) = out_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let mut out_file = std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut out_file).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn emit_progress(app: &AppHandle, downloaded: u32, total: u32, status: &str) {
    let _ = app.emit(
        "download-progress",
        DownloadProgress {
            downloaded,
            total,
            status: status.to_string(),
        },
    );
}
