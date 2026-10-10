use crate::{
    model::{Artwork, GenerateRequest, Operation},
    store::{now, Store},
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    io::{Cursor, Read},
    path::{Path, PathBuf},
    process::Stdio,
    sync::{atomic::Ordering, Arc},
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

const UV_URL: &str =
    "https://github.com/astral-sh/uv/releases/download/0.11.32/uv-x86_64-pc-windows-msvc.zip";
const UV_HASH: &str = "acfde570451cfdb8689fa159a138ee805ba4e241c466432750302c86254b0984";
const DIFFUSERS: &str =
    "https://github.com/huggingface/diffusers/archive/1d5d056ecff3a8c1d887a7dd1aa66ca4a0323458.zip";

pub fn command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut command = Command::new(program);
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    command
}
pub fn python(store: &Store) -> PathBuf {
    store
        .root
        .join("runtime")
        .join("venv")
        .join("Scripts")
        .join("python.exe")
}

fn progress(store: &Store, op: &mut Operation, phase: &str, detail: &str) -> Result<(), String> {
    op.phase = phase.into();
    op.detail = detail.into();
    op.completed = None;
    op.total = None;
    store.put("operation", op)
}

async fn execute(
    store: &Store,
    op: &mut Operation,
    cmd: &mut Command,
    protocol: bool,
) -> Result<Option<Value>, String> {
    let log_path = store.root.join("logs").join(format!("{}.log", op.id));
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| e.to_string())?;
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::from(log))
        .kill_on_drop(true);
    let mut child = cmd
        .spawn()
        .map_err(|e| {
            if e.raw_os_error() == Some(1455) {
                "Windows virtual memory is exhausted. Free disk space for the paging file or close memory-heavy apps, then retry.".into()
            } else {
                format!("Could not start the local runtime: {e}")
            }
        })?;
    #[cfg(windows)]
    let _job = crate::process_job::ProcessJob::attach(
        child.id().ok_or("Worker did not have a process ID")?,
    )?;
    let mut lines = BufReader::new(child.stdout.take().ok_or("Worker output unavailable")?).lines();
    let mut result = None;
    let mut worker_error = None;
    loop {
        if store.cancelled.load(Ordering::SeqCst) {
            #[cfg(windows)]
            if let Some(pid) = child.id() {
                let _ = command("taskkill")
                    .args(["/PID", &pid.to_string(), "/T", "/F"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .await;
            }
            let _ = child.kill().await;
            let _ = child.wait().await;
            return Err("Stopped. Partial model downloads are kept for resume.".into());
        }
        tokio::select! {
            line = lines.next_line() => {
                match line.map_err(|e| e.to_string())? {
                    None => break,
                    Some(line) if protocol => {
                        if let Ok(record) = serde_json::from_str::<Value>(&line) {
                            if let Some(phase) = record["phase"].as_str() { op.phase = phase.into(); }
                            if let Some(detail) = record["detail"].as_str() { op.detail = detail.into(); }
                            op.completed = record["completed"].as_u64(); op.total = record["total"].as_u64();
                            if op.phase == "failed" { worker_error = Some(op.detail.clone()); }
                            if let Some(value) = record.get("result") { result = Some(value.clone()); }
                            store.put("operation", op)?;
                        }
                    }
                    Some(_) => {}
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(200)) => {}
        }
    }
    let status = child.wait().await.map_err(|e| e.to_string())?;
    if !status.success() {
        let log = std::fs::read_to_string(&log_path).unwrap_or_default();
        let tail = log
            .split(['\n', '\r'])
            .filter(|line| !line.trim().is_empty())
            .rev()
            .take(7)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        return Err(worker_error.unwrap_or_else(|| {
            format!(
                "Local runtime failed ({status}). {}\nDetails: {}",
                tail.chars()
                    .rev()
                    .take(1800)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<String>(),
                log_path.display()
            )
        }));
    }
    Ok(result)
}

async fn uv(store: &Store, op: &mut Operation) -> Result<PathBuf, String> {
    let destination = store.root.join("runtime/uv.exe");
    let marker = store.root.join("runtime/uv.sha256");
    if destination.exists() {
        let expected = std::fs::read_to_string(&marker).unwrap_or_default();
        let actual = format!(
            "{:x}",
            Sha256::digest(std::fs::read(&destination).map_err(|e| e.to_string())?)
        );
        if expected == actual && !expected.is_empty() {
            return Ok(destination);
        }
    }
    progress(
        store,
        op,
        "installing_runtime",
        "Downloading the verified Python environment manager",
    )?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;
    let bytes = client
        .get(UV_URL)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;
    if format!("{:x}", Sha256::digest(&bytes)) != UV_HASH {
        return Err("Runtime manager download failed its checksum. Retry installation.".into());
    }
    if store.cancelled.load(Ordering::SeqCst) {
        return Err("Stopped before runtime installation.".into());
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let mut file = archive.by_name("uv.exe").map_err(|e| e.to_string())?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).map_err(|e| e.to_string())?;
    std::fs::write(&destination, &contents).map_err(|e| e.to_string())?;
    std::fs::write(marker, format!("{:x}", Sha256::digest(contents))).map_err(|e| e.to_string())?;
    Ok(destination)
}

async fn prepare_runtime(store: &Store, op: &mut Operation) -> Result<(), String> {
    let manager = uv(store, op).await?;
    let environment = store.root.join("runtime").join("venv");
    let setup = |args: Vec<String>| {
        let mut cmd = command(&manager);
        cmd.args(args)
            .env(
                "UV_PYTHON_INSTALL_DIR",
                store.root.join("runtime").join("python"),
            )
            .env("UV_CACHE_DIR", store.root.join("runtime").join("cache"));
        cmd
    };
    progress(
        store,
        op,
        "installing_runtime",
        "Setting up isolated Python 3.12.12",
    )?;
    let interpreter = store
        .root
        .join("runtime")
        .join("python")
        .join("cpython-3.12.12-windows-x86_64-none")
        .join("python.exe");
    if !interpreter.is_file() {
        let installed = execute(
            store,
            op,
            &mut setup(vec!["python".into(), "install".into(), "3.12.12".into()]),
            false,
        )
        .await;
        // uv can fail creating its minor-version junction after successfully extracting Python.
        // Verify the exact interpreter below instead of depending on that junction.
        if !interpreter.is_file() {
            installed?;
        }
    }
    if store.cancelled.load(Ordering::SeqCst) {
        return Err("Stopped before creating the environment.".into());
    }
    execute(
        store,
        op,
        command(&interpreter).args([
            "-c",
            "import sys, ssl, sqlite3; assert sys.version_info[:3] == (3, 12, 12)",
        ]),
        false,
    )
    .await?;
    if !python(store).exists() {
        execute(
            store,
            op,
            &mut setup(vec![
                "venv".into(),
                "--python".into(),
                interpreter.to_string_lossy().into(),
                environment.to_string_lossy().into(),
            ]),
            false,
        )
        .await?;
    }
    progress(
        store,
        op,
        "installing_runtime",
        "Installing CUDA PyTorch (this may take several minutes)",
    )?;
    execute(
        store,
        op,
        &mut setup(vec![
            "pip".into(),
            "install".into(),
            "--python".into(),
            python(store).to_string_lossy().into(),
            "torch==2.9.1".into(),
            "torchvision==0.24.1".into(),
            "--index-url".into(),
            "https://download.pytorch.org/whl/cu128".into(),
        ]),
        false,
    )
    .await?;
    progress(
        store,
        op,
        "installing_runtime",
        "Installing the pinned Qwen inference libraries",
    )?;
    execute(
        store,
        op,
        &mut setup(vec![
            "pip".into(),
            "install".into(),
            "--python".into(),
            python(store).to_string_lossy().into(),
            DIFFUSERS.into(),
            "transformers==5.19.0".into(),
            "accelerate==1.15.0".into(),
            "bitsandbytes==0.50.2".into(),
            "safetensors==0.8.0".into(),
            "pillow==12.3.0".into(),
        ]),
        false,
    )
    .await?;
    worker(store, op, json!({"action":"probe"})).await?;
    Ok(())
}

pub async fn worker(
    store: &Store,
    op: &mut Operation,
    mut request: Value,
) -> Result<Option<Value>, String> {
    let path = store
        .root
        .join("operations")
        .join(format!("{}.json", op.id));
    request["cancel_path"] = json!(store
        .root
        .join("operations")
        .join(format!("{}.cancel", op.id)));
    std::fs::write(
        &path,
        serde_json::to_vec(&request).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let mut cmd = command(python(store));
    cmd.args(["-u"])
        .arg(store.root.join("runtime/worker.py"))
        .arg(path)
        .env("HF_HUB_OFFLINE", "1")
        .env("PYTHONUTF8", "1")
        .env("PYTHONUNBUFFERED", "1");
    execute(store, op, &mut cmd, true).await
}

pub async fn install(store: Arc<Store>, mut op: Operation) {
    let outcome = async {
        let mut preferences = store.preferences()?;
        preferences.checked_profile = None; store.put("preferences", &preferences)?;
        prepare_runtime(&store, &mut op).await?;
        worker(&store, &mut op, json!({"action":"install", "model_dir":preferences.model_dir, "manifest_path":store.root.join("runtime/download-manifest.json")})).await?;
        op.detail = "Model downloaded and verified. Run Check engine before generating.".into();
        Ok(())
    }.await;
    store.finish(op, outcome);
}

pub async fn generate(store: Arc<Store>, mut op: Operation, request: GenerateRequest, check: bool) {
    let outcome = async {
        let mut preferences = store.preferences()?;
        let bytes = uuid::Uuid::new_v4();
        let seed = request.seed.unwrap_or_else(|| u32::from_le_bytes(bytes.as_bytes()[..4].try_into().unwrap_or_default()) & 0x7fffffff);
        let path = store.root.join("outputs").join(format!("{}.png", op.id));
        let metadata = Artwork { id: op.id.clone(), path: path.to_string_lossy().into(), prompt: request.prompt.clone(), effective_prompt: String::new(), mode: request.mode.clone(),
            width: request.width, height: request.height, seed, has_alpha: false, transparency_requested: request.transparent, profile: preferences.profile.clone(), created_at: now(), references: request.references.clone(), parent_id: request.parent_id.clone(), provenance: Value::Null };
        let result = worker(&store, &mut op, json!({"action":"generate", "model_dir":preferences.model_dir, "profile":preferences.profile,
            "prompt":request.prompt, "mode":request.mode, "width":request.width, "height":request.height, "seed":seed,
            "transparent":request.transparent, "references":request.references, "output_path":path, "test_edit":check, "studioMetadata":metadata})).await?.ok_or("Worker did not return an image")?;
        if !path.is_file() { return Err("Generated image was not saved.".into()); }
        let artwork: Artwork = serde_json::from_value(result["artwork"].clone()).map_err(|e| e.to_string())?;
        store.add_artwork(&artwork)?;
        if check {
            preferences.checked_profile = Some(preferences.profile.clone()); store.put("preferences", &preferences)?;
            op.detail = "Engine check passed. Your local studio is ready.".into();
        }
        Ok(())
    }.await;
    store.finish(op, outcome);
}

pub fn unpacked_marker(model: &Path) -> bool {
    let Ok(marker) = std::fs::read(model.join("verified.json")) else {
        return false;
    };
    let Ok(value) = serde_json::from_slice::<Value>(&marker) else {
        return false;
    };
    if value["revision"] != "d65dbc9a7e8f6b5479e33dee6030eaab2a906509" {
        return false;
    }
    let Ok(manifest) =
        serde_json::from_slice::<Value>(include_bytes!("../../engine/download-manifest.json"))
    else {
        return false;
    };
    let expected: Vec<_> = manifest["files"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|entry| {
            entry["path"]
                .as_str()
                .is_some_and(|path| !path.starts_with("assets/") && path != ".gitattributes")
        })
        .collect();
    value["files"].as_array().is_some_and(|files| {
        files.len() == expected.len()
            && expected.iter().all(|entry| {
                files
                    .iter()
                    .any(|file| file["path"] == entry["path"] && file["size"] == entry["bytes"])
            })
            && files.iter().all(|file| {
                let Some(name) = file["path"].as_str() else {
                    return false;
                };
                if name.contains("..")
                    || name.contains('\\')
                    || name.contains(':')
                    || Path::new(name).is_absolute()
                {
                    return false;
                }
                model
                    .join(name)
                    .metadata()
                    .ok()
                    .is_some_and(|metadata| Some(metadata.len()) == file["size"].as_u64())
            })
    })
}

pub fn remaining_download_bytes(model: &Path) -> u64 {
    let manifest: Value =
        serde_json::from_slice(include_bytes!("../../engine/download-manifest.json"))
            .unwrap_or_default();
    manifest["files"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|entry| {
            entry["path"]
                .as_str()
                .is_some_and(|path| !path.starts_with("assets/") && path != ".gitattributes")
        })
        .map(|entry| {
            let name = entry["path"].as_str().unwrap_or_default();
            let size = entry["bytes"].as_u64().unwrap_or_default();
            let full = model
                .join(name)
                .metadata()
                .map(|m| m.len())
                .unwrap_or_default();
            let partial = model
                .join(format!("{name}.partial"))
                .metadata()
                .map(|m| m.len())
                .unwrap_or_default();
            size.saturating_sub(full.max(partial).min(size))
        })
        .sum()
}
