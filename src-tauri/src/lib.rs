mod model;
mod process_job;
mod runtime;
mod store;
use model::{GenerateRequest, Hardware, Reference, Snapshot};
use std::{
    path::{Path, PathBuf},
    sync::{atomic::Ordering, Arc},
};
use store::Store;
use tauri::{Manager, State};

fn ensure_idle(store: &Store) -> Result<(), String> {
    if store.busy.load(Ordering::SeqCst) {
        return Err("Stop the active operation before changing engine settings.".into());
    }
    Ok(())
}

fn hardware(store: &Store, model_dir: &str) -> Hardware {
    let mut cached = store.hardware.lock().unwrap_or_else(|e| e.into_inner());
    let (name, vram, ram) = cached
        .get_or_insert_with(|| {
            let mut system = sysinfo::System::new();
            system.refresh_memory();
            let result = runtime::command("nvidia-smi")
                .args([
                    "--query-gpu=name,memory.total",
                    "--format=csv,noheader,nounits",
                ])
                .as_std_mut()
                .output()
                .ok();
            let output = result
                .map(|result| String::from_utf8_lossy(&result.stdout).into_owned())
                .unwrap_or_default();
            let fields: Vec<_> = output
                .lines()
                .next()
                .unwrap_or_default()
                .split(',')
                .map(str::trim)
                .collect();
            (
                fields
                    .first()
                    .filter(|name| !name.is_empty())
                    .map(|name| (*name).into()),
                fields.get(1).and_then(|value| value.parse::<u64>().ok()),
                system.total_memory(),
            )
        })
        .clone();
    let target = Path::new(model_dir);
    let free = sysinfo::Disks::new_with_refreshed_list()
        .iter()
        .filter(|disk| target.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().as_os_str().len())
        .map(|disk| disk.available_space())
        .unwrap_or(0);
    Hardware {
        gpu_name: name,
        vram_mb: vram,
        ram_bytes: ram,
        free_bytes: free,
        model_bytes: 32_493_327_348,
    }
}

#[tauri::command]
async fn get_snapshot(store: State<'_, Arc<Store>>) -> Result<Snapshot, String> {
    let store = store.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let preferences = store.preferences()?;
        let model_downloaded = runtime::unpacked_marker(Path::new(&preferences.model_dir));
        let runtime_installed = runtime::python(&store).is_file();
        let ready = model_downloaded
            && runtime_installed
            && preferences.checked_profile.as_deref() == Some(&preferences.profile);
        Ok(Snapshot {
            hardware: hardware(&store, &preferences.model_dir),
            preferences,
            model_downloaded,
            runtime_installed,
            ready,
            operation: store.get("operation")?,
            artworks: store
                .artworks()?
                .into_iter()
                .filter(|art| art.provenance["verification"] != "text-and-image")
                .collect(),
            data_dir: store.root.to_string_lossy().into(),
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn save_preferences(
    store: State<'_, Arc<Store>>,
    profile: String,
    directory: Option<String>,
    license_accepted: bool,
) -> Result<(), String> {
    ensure_idle(&store)?;
    if !["nf4", "bf16"].contains(&profile.as_str()) {
        return Err("Unsupported memory profile.".into());
    }
    let mut preferences = store.preferences()?;
    if let Some(directory) = directory {
        let parent = PathBuf::from(directory)
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if !parent.is_dir() {
            return Err("Choose a storage folder.".into());
        }
        let new_path = parent
            .join("Image Wizard Models/qwen-image-2.1-turbo")
            .to_string_lossy()
            .into_owned();
        if preferences.model_dir != new_path {
            preferences.checked_profile = None;
            preferences.model_dir = new_path;
        }
    }
    if preferences.profile != profile {
        preferences.checked_profile = None;
    }
    preferences.profile = profile;
    preferences.license_accepted = license_accepted;
    store.put("preferences", &preferences)
}

#[tauri::command]
fn start_install(store: State<'_, Arc<Store>>) -> Result<String, String> {
    let preferences = store.preferences()?;
    if !preferences.license_accepted {
        return Err("Read and accept the model's research license before installing.".into());
    }
    let device = hardware(&store, &preferences.model_dir);
    if device.gpu_name.is_none() {
        return Err("This release requires an NVIDIA CUDA GPU.".into());
    }
    if device.free_bytes
        < runtime::remaining_download_bytes(Path::new(&preferences.model_dir)) + 10_000_000_000
        && !runtime::unpacked_marker(Path::new(&preferences.model_dir))
    {
        return Err("This drive needs room for the remaining model download plus 10 GB of working space. Choose a larger drive.".into());
    }
    let operation = store.begin("install")?;
    let id = operation.id.clone();
    tauri::async_runtime::spawn(runtime::install(store.inner().clone(), operation));
    Ok(id)
}

#[tauri::command]
fn start_generation(
    store: State<'_, Arc<Store>>,
    mut request: GenerateRequest,
) -> Result<String, String> {
    request.validate()?;
    let preferences = store.preferences()?;
    if preferences.checked_profile.as_deref() != Some(&preferences.profile)
        || !runtime::unpacked_marker(Path::new(&preferences.model_dir))
    {
        return Err("Set up and check your local engine first.".into());
    }
    request.references = request
        .references
        .iter()
        .map(|path| {
            store
                .owned_image(path)
                .map(|path| path.to_string_lossy().into())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let operation = store.begin("generate")?;
    let id = operation.id.clone();
    tauri::async_runtime::spawn(runtime::generate(
        store.inner().clone(),
        operation,
        request,
        false,
    ));
    Ok(id)
}

#[tauri::command]
fn check_engine(store: State<'_, Arc<Store>>) -> Result<String, String> {
    let preferences = store.preferences()?;
    if !runtime::unpacked_marker(Path::new(&preferences.model_dir))
        || !runtime::python(&store).is_file()
    {
        return Err("Download the model and runtime first.".into());
    }
    let operation = store.begin("check")?;
    let id = operation.id.clone();
    let request = GenerateRequest {
        prompt: "A small orange ceramic sphere on a charcoal background, studio light".into(),
        mode: "create".into(),
        width: 512,
        height: 512,
        seed: Some(42),
        transparent: false,
        references: vec![],
        parent_id: None,
    };
    tauri::async_runtime::spawn(runtime::generate(
        store.inner().clone(),
        operation,
        request,
        true,
    ));
    Ok(id)
}

#[tauri::command]
fn cancel_operation(store: State<'_, Arc<Store>>) {
    store.cancelled.store(true, Ordering::SeqCst);
}

#[tauri::command]
async fn import_reference(store: State<'_, Arc<Store>>, path: String) -> Result<Reference, String> {
    let store = store.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let path = PathBuf::from(path);
        let metadata = path.metadata().map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.len() > 32 * 1024 * 1024 {
            return Err("Choose a PNG, JPEG or WebP image smaller than 32 MB.".into());
        }
        let reader = image::ImageReader::open(&path)
            .map_err(|e| e.to_string())?
            .with_guessed_format()
            .map_err(|e| e.to_string())?;
        let (width, height) = reader
            .into_dimensions()
            .map_err(|e| format!("Could not read this image: {e}"))?;
        if u64::from(width) * u64::from(height) > 40_000_000 {
            return Err("Use an image smaller than 40 megapixels.".into());
        }
        let decoded = image::ImageReader::open(&path)
            .map_err(|e| e.to_string())?
            .with_guessed_format()
            .map_err(|e| e.to_string())?
            .decode()
            .map_err(|e| e.to_string())?;
        let id = uuid::Uuid::new_v4().to_string();
        let target = store.root.join("inputs").join(format!("{id}.png"));
        decoded
            .save_with_format(&target, image::ImageFormat::Png)
            .map_err(|e| e.to_string())?;
        Ok(Reference {
            id,
            path: target.to_string_lossy().into(),
            name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into(),
            width: decoded.width(),
            height: decoded.height(),
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn export_image(
    store: State<'_, Arc<Store>>,
    id: String,
    destination: String,
) -> Result<(), String> {
    let artwork = store
        .artworks()?
        .into_iter()
        .find(|art| art.id == id)
        .ok_or("Image is not in the library")?;
    let source = store.owned_image(&artwork.path)?;
    let target = Path::new(&destination);
    if target
        .extension()
        .and_then(|extension| extension.to_str())
        .is_none_or(|extension| !extension.eq_ignore_ascii_case("png"))
    {
        return Err("Export with a .png extension to preserve the image.".into());
    }
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)
        .map_err(|e| format!("Choose a new export filename: {e}"))?;
    let mut input = std::fs::File::open(source).map_err(|e| e.to_string())?;
    std::io::copy(&mut input, &mut output).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let store = Arc::new(
                Store::new(app.path().home_dir()?.join(".image-wizard"))
                    .map_err(std::io::Error::other)?,
            );
            std::fs::write(
                store.root.join("runtime/worker.py"),
                include_bytes!("../../engine/worker.py"),
            )?;
            std::fs::write(
                store.root.join("runtime/download-manifest.json"),
                include_bytes!("../../engine/download-manifest.json"),
            )?;
            let webview_dir = store.root.join("webview");
            app.manage(store);
            tauri::WebviewWindowBuilder::from_config(app, &app.config().app.windows[0])?
                .data_directory(webview_dir)
                .build()?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            save_preferences,
            start_install,
            start_generation,
            check_engine,
            cancel_operation,
            import_reference,
            export_image
        ])
        .run(tauri::generate_context!())
        .expect("Unable to start Image Wizard");
}
