import { useCallback, useEffect, useRef, useState } from "react";
import {
  ArrowDownToLine,
  ArrowLeftRight,
  ArrowUpRight,
  Box,
  Check,
  ChevronDown,
  CircleHelp,
  Clock3,
  Cpu,
  Expand,
  Folder,
  HardDrive,
  ImagePlus,
  Images,
  LoaderCircle,
  Paintbrush,
  PencilLine,
  Plus,
  RotateCcw,
  Shuffle,
  Sparkles,
  Square,
  WandSparkles,
  X,
} from "lucide-react";
import * as bridge from "./bridge";
import {
  bytes,
  dimensions,
  parsedSeed,
  progressPercent,
  RATIOS,
} from "./studio";
import type {
  Artwork,
  Mode,
  Operation,
  Reference,
  Snapshot,
  View,
} from "./types";

const example =
  "A sculptural glass orchid, soft morning light, editorial still life";
const tabs = [
  { id: "create", label: "Create", icon: Sparkles },
  { id: "transform", label: "Transform", icon: ArrowLeftRight },
  { id: "edit", label: "Edit", icon: Paintbrush },
  { id: "library", label: "Library", icon: Images },
  { id: "models", label: "Models", icon: Box },
] as const;
const stylePrompts = {
  Natural: "",
  Cinematic: ", cinematic lighting, rich atmospheric color, film still",
  Illustration: ", a beautifully composed editorial illustration",
};
const descriptions = {
  create: "Give your imagination a little room.",
  transform: "A new direction for an existing image.",
  edit: "Small changes. A whole new possibility.",
};
function errorText(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function Progress({
  operation,
  stop,
  dismiss,
}: {
  operation: Operation;
  stop: () => void;
  dismiss: () => void;
}) {
  const [time, setTime] = useState(Date.now());
  useEffect(() => {
    const timer = window.setInterval(() => setTime(Date.now()), 1000);
    return () => clearInterval(timer);
  }, []);
  const percent = progressPercent(operation.completed, operation.total);
  const active = operation.finishedAt === null;
  const seconds = Math.max(
    0,
    Math.floor(((operation.finishedAt ?? time) - operation.startedAt) / 1000),
  );
  return (
    <div
      className={`operation ${active ? "is-active" : ""} ${operation.phase === "failed" ? "is-failed" : ""}`}
      role="status"
      aria-live="polite"
    >
      <div className="operation-top">
        <span>
          {active ? (
            <LoaderCircle size={14} className="spin" />
          ) : operation.phase === "succeeded" ? (
            <Check size={14} />
          ) : (
            <CircleHelp size={14} />
          )}
          <strong>
            {operation.kind === "install"
              ? "Local model setup"
              : operation.kind === "check"
                ? "Checking your engine"
                : "Creating your image"}
          </strong>
        </span>
        <span className="elapsed">
          <Clock3 size={12} />
          {Math.floor(seconds / 60)}:{String(seconds % 60).padStart(2, "0")}
          {active && (
            <button
              className="icon-button"
              onClick={stop}
              aria-label="Stop operation"
            >
              <Square size={12} />
            </button>
          )}
          {!active && (
            <button
              className="icon-button"
              onClick={dismiss}
              aria-label="Dismiss operation"
            >
              <X size={12} />
            </button>
          )}
        </span>
      </div>
      <p>{operation.detail}</p>
      {active && (
        <div
          className={`progress-track ${percent === null ? "indeterminate" : ""}`}
        >
          <div style={{ width: percent === null ? "30%" : `${percent}%` }} />
        </div>
      )}
      {active && operation.total && operation.total > 10000 && (
        <div className="progress-caption">
          {bytes(operation.completed ?? 0)} of {bytes(operation.total)}{" "}
          <span>{percent}%</span>
        </div>
      )}
    </div>
  );
}

function ModelManager({
  data,
  refresh,
  notify,
  back,
}: {
  data: Snapshot;
  refresh: () => Promise<void>;
  notify: (message: string) => void;
  back: () => void;
}) {
  const [accepted, setAccepted] = useState(data.preferences.licenseAccepted);
  const [profile, setProfile] = useState(data.preferences.profile);
  const [submitting, setSubmitting] = useState(false);
  const active = data.operation?.finishedAt === null;
  useEffect(() => {
    setProfile(data.preferences.profile);
    setAccepted(data.preferences.licenseAccepted);
  }, [data.preferences.profile, data.preferences.licenseAccepted]);
  const attempt = async (action: () => Promise<unknown>) => {
    setSubmitting(true);
    try {
      await action();
      await refresh();
    } catch (error) {
      notify(errorText(error));
    } finally {
      setSubmitting(false);
    }
  };
  const save = async (directory: string | null = null) => {
    await bridge.preferences(profile, accepted, directory);
  };
  return (
    <main className="models-page">
      <div className="page-heading">
        <div>
          <span className="eyebrow">YOUR COMPUTER. YOUR CREATIVE SPACE.</span>
          <h1>A home for your imagination.</h1>
          <p>Set up your local engine once. Then create on your own terms.</p>
        </div>
        <button className="secondary" onClick={back}>
          <ArrowUpRight size={15} />
          Back to studio
        </button>
      </div>
      <div className="models-layout">
        <section className="model-card">
          <div className="model-card-top">
            <div className="model-symbol">
              <WandSparkles size={27} />
            </div>
            <span className="tag">OFFICIAL CHECKPOINT</span>
          </div>
          <h2>
            Qwen-Image<span>2.1 Turbo</span>
          </h2>
          <p className="model-description">
            One engine for creating, transforming, and editing images. Eight
            steps from an idea to something you can see.
          </p>
          <div className="capabilities">
            <span>
              <Check size={13} />
              Text to image
            </span>
            <span>
              <Check size={13} />
              Image editing
            </span>
            <span>
              <Check size={13} />
              Multiple references
            </span>
            <span>
              <Check size={13} />
              Transparent PNG
            </span>
          </div>
          <div className="model-meta">
            <span>
              <HardDrive size={16} />
              Model download<strong>{bytes(data.hardware.modelBytes)}</strong>
            </span>
            <span>
              <Box size={16} />
              Installation
              <strong>
                {data.ready
                  ? "Ready to create"
                  : data.modelDownloaded
                    ? "Downloaded · check needed"
                    : "Not installed"}
              </strong>
            </span>
          </div>
          <div className="license-note">
            <CircleHelp size={17} />
            <div>
              <strong>A note about the model license</strong>
              <p>
                Qwen's weights are licensed for research and evaluation.
                Commercial use needs a separate license.
              </p>
              <button
                className="text-button"
                onClick={() => void bridge.license()}
              >
                Read the full license <ArrowUpRight size={13} />
              </button>
            </div>
          </div>
        </section>
        <section className="setup-card">
          <h3>
            <Cpu size={18} />
            Your local setup
          </h3>
          <div className="hardware-grid">
            <div>
              <span>Graphics</span>
              <strong>
                {data.hardware.gpuName ??
                  (bridge.desktop
                    ? "No NVIDIA GPU detected"
                    : "Detected in the desktop app")}
              </strong>
            </div>
            <div>
              <span>GPU memory</span>
              <strong>
                {data.hardware.vramMb
                  ? `${(data.hardware.vramMb / 1024).toFixed(0)} GB`
                  : "—"}
              </strong>
            </div>
            <div>
              <span>System memory</span>
              <strong>
                {data.hardware.ramBytes ? bytes(data.hardware.ramBytes) : "—"}
              </strong>
            </div>
            <div>
              <span>Available storage</span>
              <strong>
                {data.hardware.freeBytes ? bytes(data.hardware.freeBytes) : "—"}
              </strong>
            </div>
          </div>
          <label className="field-label" htmlFor="profile">
            Memory profile
          </label>
          <div className="select-wrap">
            <select
              id="profile"
              value={profile}
              disabled={active}
              onChange={(event) =>
                setProfile(event.target.value as "nf4" | "bf16")
              }
            >
              <option value="nf4">Memory saver · 4-bit</option>
              <option value="bf16">Original precision · CPU offload</option>
            </select>
            <ChevronDown size={14} />
          </div>
          <p className="field-hint">
            {profile === "nf4"
              ? "Less GPU memory, with some variation in fine detail. The full official checkpoint is kept on disk."
              : "Preserves original precision. Needs substantial system memory and can be slower."}{" "}
            A real image check confirms whether this profile works here.
          </p>
          <label className="field-label">Model storage</label>
          <div className="storage-location">
            <Folder size={17} />
            <span title={data.preferences.modelDir}>
              {data.preferences.modelDir}
            </span>
            <button
              onClick={() =>
                void attempt(async () => {
                  const directory = await bridge.pickDirectory();
                  if (directory && typeof directory === "string")
                    await save(directory);
                })
              }
              disabled={active || submitting}
            >
              Change
            </button>
          </div>
          <p className="field-hint">
            Allow at least 43 GB for the model and runtime. Your images stay in
            the studio's own library.
          </p>
          <label className="license-checkbox">
            <input
              type="checkbox"
              checked={accepted}
              disabled={active}
              onChange={(event) => setAccepted(event.target.checked)}
            />
            <span>
              I've read and accept the Qwen Research License for this
              installation.
            </span>
          </label>
          <div className="setup-actions">
            <button
              className="primary"
              disabled={!accepted || active || submitting}
              onClick={() =>
                void attempt(async () => {
                  await save();
                  await bridge.install();
                })
              }
            >
              {submitting ? (
                <LoaderCircle size={16} className="spin" />
              ) : (
                <ArrowDownToLine size={17} />
              )}{" "}
              {data.modelDownloaded
                ? "Repair / verify installation"
                : data.operation?.phase === "cancelled" ||
                    data.operation?.phase === "interrupted" ||
                    data.operation?.phase === "failed"
                  ? "Resume setup"
                  : "Install local engine"}
            </button>
            {data.modelDownloaded && (
              <button
                className="secondary"
                disabled={active || submitting}
                onClick={() =>
                  void attempt(async () => {
                    await save();
                    await bridge.check();
                  })
                }
              >
                <Sparkles size={16} />
                Check engine
              </button>
            )}
          </div>
          <p className="privacy-note">
            <span className="status-dot" />
            Generation stays on this computer. Setup downloads trusted
            dependencies and weights.
          </p>
          {data.ready && (
            <div className="ready-note">
              <Check size={17} />
              Your engine passed its image check. You're ready to create.
            </div>
          )}
        </section>
      </div>
    </main>
  );
}

export default function App() {
  const [data, setData] = useState<Snapshot | null>(null);
  const [view, setView] = useState<View>("create");
  const [mode, setMode] = useState<Mode>("create");
  const [prompt, setPrompt] = useState(() => {
    try {
      return localStorage.getItem("iw-prompt") ?? example;
    } catch {
      return example;
    }
  });
  const [style, setStyle] = useState<keyof typeof stylePrompts>("Natural");
  const [references, setReferences] = useState<Reference[]>([]);
  const [ratio, setRatio] = useState(0);
  const [resolution, setResolution] = useState<"draft" | "standard" | "native">(
    "standard",
  );
  const [seed, setSeed] = useState("");
  const [transparent, setTransparent] = useState(false);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [compare, setCompare] = useState(false);
  const [search, setSearch] = useState("");
  const activeJob = useRef<string | null>(null);
  const refreshSequence = useRef(0);
  const [dismissedOperation, setDismissedOperation] = useState<string | null>(
    null,
  );
  const notify = useCallback((text: string) => setMessage(text), []);
  const refresh = useCallback(async () => {
    const sequence = ++refreshSequence.current;
    const result = await bridge.snapshot();
    if (sequence !== refreshSequence.current) return;
    setData(result);
    if (
      activeJob.current &&
      result.artworks.some((art) => art.id === activeJob.current)
    ) {
      setSelectedId(activeJob.current);
      activeJob.current = null;
    }
  }, []);
  useEffect(() => {
    let live = true;
    const fetch = () => {
      void refresh().catch((error) => {
        if (live) notify(errorText(error));
      });
    };
    fetch();
    const timer = window.setInterval(fetch, 1200);
    return () => {
      live = false;
      clearInterval(timer);
    };
  }, [refresh, notify]);
  useEffect(() => {
    try {
      localStorage.setItem("iw-prompt", prompt);
    } catch {
      /* Private browsing can disable storage. */
    }
  }, [prompt]);
  useEffect(() => {
    if (!message) return;
    const timer = window.setTimeout(() => setMessage(null), 9000);
    return () => clearTimeout(timer);
  }, [message]);
  const selected = data?.artworks.find((art) => art.id === selectedId) ?? null;
  const active = data?.operation?.finishedAt === null;
  const size = dimensions(ratio, resolution);
  const select = (art: Artwork) => {
    activeJob.current = null;
    setSelectedId(art.id);
    setCompare(false);
  };
  const changeView = (value: View) => {
    setView(value);
    if (value === "create" || value === "transform" || value === "edit") {
      setMode(value);
      setCompare(false);
    }
  };
  const attempt = async (action: () => Promise<unknown>) => {
    setSubmitting(true);
    try {
      await action();
      await refresh();
    } catch (error) {
      notify(errorText(error));
    } finally {
      setSubmitting(false);
    }
  };
  const addReferences = () =>
    void attempt(async () => {
      const added = await bridge.importImages();
      if (added.length + references.length > 10)
        throw new Error("Add at most ten reference images.");
      setReferences((old) => [...old, ...added]);
    });
  const reuse = (art: Artwork) => {
    setPrompt(art.prompt);
    setStyle("Natural");
    setSeed(String(art.seed));
    select(art);
    changeView(art.mode);
    setTransparent(art.transparencyRequested ?? false);
    setReferences(
      art.references.map((path, index) => ({
        id: `${art.id}-${index}`,
        path,
        name: `Reference ${index + 1}`,
        width: 0,
        height: 0,
      })),
    );
    const nearest = RATIOS.map((preset) =>
      Math.abs(preset.width / preset.height - art.width / art.height),
    );
    setRatio(nearest.indexOf(Math.min(...nearest)));
    setResolution(
      art.width <= 512 && art.height <= 512
        ? "draft"
        : art.width <= 1024 && art.height <= 1024
          ? "standard"
          : "native",
    );
  };
  const useAsReference = (newMode: Mode) => {
    if (!selected) return;
    setReferences([
      {
        id: selected.id,
        path: selected.path,
        name: "Source image",
        width: selected.width,
        height: selected.height,
      },
    ]);
    setPrompt("");
    setStyle("Natural");
    changeView(newMode);
  };
  const start = () =>
    void attempt(async () => {
      if (!data?.ready) {
        changeView("models");
        return;
      }
      const id = await bridge.generate({
        prompt: prompt.trim() + stylePrompts[style],
        mode,
        ...size,
        seed: parsedSeed(seed),
        transparent,
        references: references.map((reference) => reference.path),
        parentId:
          mode !== "create" &&
          selected &&
          references.some((reference) => reference.path === selected.path)
            ? selected.id
            : null,
      });
      activeJob.current = id;
    });
  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if (
        (event.ctrlKey || event.metaKey) &&
        event.key === "Enter" &&
        view !== "models" &&
        view !== "library" &&
        !active
      ) {
        event.preventDefault();
        start();
      }
      if (event.key === "Escape") setExpanded(false);
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  });
  const stop = () => void attempt(() => bridge.cancel());
  const canvasImage =
    compare && selected?.references[0]
      ? bridge.imageUrl(selected.references[0])
      : selected
        ? bridge.imageUrl(selected.path)
        : "/studio-sample.png";
  return (
    <div className="app-shell">
      <header className="app-header">
        <a
          className="brand"
          href="#"
          onClick={(event) => {
            event.preventDefault();
            changeView("create");
          }}
        >
          <img src="/logo.svg" alt="" />
          <span>Image Wizard</span>
        </a>
        <nav aria-label="Main navigation">
          {tabs.map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              className={view === id ? "nav-item active" : "nav-item"}
              onClick={() => changeView(id)}
            >
              <Icon size={17} />
              <span>{label}</span>
            </button>
          ))}
        </nav>
        <div className="header-end">
          <span className="edition">LOCAL STUDIO</span>
          <button
            className={`engine-chip ${data?.ready ? "ready" : ""}`}
            onClick={() => changeView("models")}
          >
            <span className="status-dot" />
            {data?.ready ? "Engine ready" : "Set up engine"}
            <ChevronDown size={12} />
          </button>
        </div>
      </header>
      {!data ? (
        <div className="loading-screen">
          <LoaderCircle className="spin" />
          <p>Opening your studio…</p>
          <button
            className="text-button"
            onClick={() =>
              void refresh().catch((error) => notify(errorText(error)))
            }
          >
            Retry
          </button>
        </div>
      ) : view === "models" ? (
        <ModelManager
          data={data}
          refresh={refresh}
          notify={notify}
          back={() => changeView(mode)}
        />
      ) : view === "library" ? (
        <main className="library-page">
          <div className="page-heading">
            <div>
              <span className="eyebrow">MADE HERE. KEPT HERE.</span>
              <h1>Your image library.</h1>
              <p>
                {data.artworks.length} image
                {data.artworks.length === 1 ? "" : "s"} · original prompts and
                settings included
              </p>
            </div>
            <input
              type="search"
              aria-label="Search library"
              placeholder="Find an image by its prompt…"
              value={search}
              onChange={(event) => setSearch(event.target.value)}
            />
          </div>
          {data.artworks.length ? (
            <div className="library-grid">
              {data.artworks
                .filter((art) =>
                  art.prompt.toLowerCase().includes(search.toLowerCase()),
                )
                .map((art) => (
                  <button
                    className="library-art"
                    key={art.id}
                    onClick={() => {
                      select(art);
                      changeView("create");
                    }}
                  >
                    <img
                      src={bridge.imageUrl(art.path)}
                      alt={art.prompt}
                      loading="lazy"
                    />
                    <span>{art.prompt}</span>
                    <small>
                      {art.width} × {art.height} · {art.mode}
                    </small>
                  </button>
                ))}
            </div>
          ) : (
            <div className="empty-library">
              <Images size={39} />
              <h2>A collection waiting to happen.</h2>
              <p>
                Your creations will live here, along with every prompt, seed,
                and edit.
              </p>
              <button
                className="primary"
                onClick={() => changeView(data.ready ? "create" : "models")}
              >
                <Sparkles size={16} />
                {data.ready
                  ? "Create your first image"
                  : "Set up your local engine"}
              </button>
            </div>
          )}
        </main>
      ) : (
        <main className="studio-layout">
          <aside className="compose-panel">
            <div className="section-heading">
              <div className="section-icon">
                {mode === "create" ? (
                  <Sparkles size={21} />
                ) : mode === "transform" ? (
                  <ArrowLeftRight size={21} />
                ) : (
                  <Paintbrush size={21} />
                )}
              </div>
              <div>
                <h1>
                  {mode === "create"
                    ? "Create"
                    : mode === "transform"
                      ? "Transform"
                      : "Edit"}
                </h1>
                <p>{descriptions[mode]}</p>
              </div>
            </div>
            <div className="label-row">
              <label htmlFor="prompt">
                {mode === "edit" ? "Describe your edit" : "Your prompt"}
              </label>
              <button
                className="text-button"
                onClick={() =>
                  setPrompt(
                    mode === "create"
                      ? example
                      : mode === "transform"
                        ? "Reimagine this image as a cinematic watercolor painting."
                        : "Replace the background with a softly lit studio. Preserve the subject.",
                  )
                }
              >
                <Shuffle size={12} />
                Try an idea
              </button>
            </div>
            <div className="prompt-wrap">
              <textarea
                id="prompt"
                placeholder={
                  mode === "edit"
                    ? "What would you like to change?"
                    : "Describe the image you have in mind…"
                }
                value={prompt}
                maxLength={24000}
                onChange={(event) => setPrompt(event.target.value)}
              />
              <div className="prompt-footer">
                <PencilLine size={12} />
                <span>{prompt.length.toLocaleString()} characters</span>
              </div>
            </div>
            <div className="style-options" aria-label="Prompt style">
              {(Object.keys(stylePrompts) as (keyof typeof stylePrompts)[]).map(
                (item) => (
                  <button
                    aria-pressed={style === item}
                    className={style === item ? "selected" : ""}
                    key={item}
                    onClick={() => setStyle(item)}
                  >
                    {item}
                  </button>
                ),
              )}
            </div>
            <div className="label-row reference-label">
              <span>
                {mode === "create" ? "Reference images" : "Source & references"}
              </span>
              <small>{references.length}/10</small>
            </div>
            {references.length > 0 && (
              <div className="reference-grid">
                {references.map((reference, index) => (
                  <div key={reference.id} className="reference">
                    <img
                      src={bridge.imageUrl(reference.path)}
                      alt={reference.name}
                    />
                    <span>{index + 1}</span>
                    <button
                      aria-label={`Remove ${reference.name}`}
                      onClick={() =>
                        setReferences((old) =>
                          old.filter((item) => item.id !== reference.id),
                        )
                      }
                    >
                      <X size={12} />
                    </button>
                  </div>
                ))}
              </div>
            )}
            <button
              className="upload-area"
              onClick={addReferences}
              disabled={references.length >= 10 || submitting}
            >
              <ImagePlus size={25} />
              <strong>
                {references.length
                  ? "Add another reference"
                  : "Add reference images"}
              </strong>
              <span>Choose PNG, JPEG, or WebP</span>
            </button>
            <p className="field-hint">
              {mode === "create"
                ? "Guide the composition, subject, or style with up to ten images."
                : "Describe the changes you want. Sources stay untouched; every edit becomes a new image."}
            </p>
            {mode === "edit" && (
              <div className="edit-tip">
                <CircleHelp size={15} />
                <span>
                  For a precise region, add your original and an annotated or
                  mask reference. Mention the marked area in your prompt.
                </span>
              </div>
            )}
            <div className="compose-bottom">
              <span className="mini-eyebrow">A LITTLE PROMPT CRAFT</span>
              <p>
                Be specific about your subject, light, materials, and mood. Put
                exact text in quotation marks.
              </p>
              <span className="shortcut">
                <kbd>Ctrl</kbd> + <kbd>Enter</kbd> to create
              </span>
            </div>
          </aside>
          <section className="canvas-panel">
            <div className="canvas-toolbar">
              <span>
                {selected ? "Your creation" : "The canvas"}
                <span className="toolbar-separator">/</span>
                <small>
                  {selected
                    ? `${selected.width} × ${selected.height}`
                    : "A little inspiration"}
                </small>
              </span>
              <div>
                {selected?.references.length ? (
                  <button
                    aria-pressed={compare}
                    className={`icon-button ${compare ? "selected" : ""}`}
                    onClick={() => setCompare((old) => !old)}
                    title="Compare with source"
                  >
                    <ArrowLeftRight size={16} />
                  </button>
                ) : null}
                <button
                  className="icon-button"
                  aria-label="Expand image"
                  onClick={() => setExpanded(true)}
                >
                  <Expand size={16} />
                </button>
              </div>
            </div>
            <div
              className={`art-canvas ${selected?.hasAlpha ? "checkerboard" : ""}`}
            >
              <img
                src={canvasImage}
                alt={
                  selected?.prompt ?? "Illustrative glass orchid studio sample"
                }
              />
              {!selected && (
                <span className="sample-badge">
                  <Sparkles size={12} />
                  Studio sample · illustrative artwork
                </span>
              )}
              {compare && <span className="sample-badge">Original source</span>}
              {active && data.operation?.kind !== "install" && (
                <div className="canvas-progress">
                  <LoaderCircle size={16} className="spin" />
                  <span>{data.operation?.detail}</span>
                </div>
              )}
            </div>
            <div className="canvas-caption">
              <div>
                <span className="eyebrow">
                  {selected
                    ? selected.mode.toUpperCase()
                    : "IMAGINE SOMETHING NEW"}
                </span>
                <p>
                  {selected?.prompt ??
                    "A sculptural glass orchid. Light, captured in glass."}
                </p>
              </div>
              {selected ? (
                <button className="text-button" onClick={() => reuse(selected)}>
                  <RotateCcw size={13} />
                  Reuse settings
                </button>
              ) : (
                <span className="sample-note">
                  Your images will appear here.
                </span>
              )}
            </div>
            <div className="filmstrip">
              <div className="filmstrip-label">
                <Images size={13} />
                <span>Recent</span>
              </div>
              {data.artworks.slice(0, 8).map((art) => (
                <button
                  className={`filmstrip-art ${selectedId === art.id ? "selected" : ""}`}
                  key={art.id}
                  title={art.prompt}
                  onClick={() => select(art)}
                >
                  <img src={bridge.imageUrl(art.path)} alt={art.prompt} />
                </button>
              ))}
              {!data.artworks.length && (
                <>
                  <button
                    className="filmstrip-art selected sample-thumb"
                    title="Studio sample"
                    onClick={() => setSelectedId(null)}
                  >
                    <img src="/studio-sample.png" alt="Glass orchid sample" />
                    <span>Sample</span>
                  </button>
                  <div className="filmstrip-empty">
                    <Plus size={18} />
                    <span>Your first creation</span>
                  </div>
                </>
              )}
              <button
                className="filmstrip-more"
                onClick={() => changeView("library")}
                title="Open library"
              >
                <ArrowUpRight size={16} />
              </button>
            </div>
          </section>
          <aside className="settings-panel">
            <div className="engine-status">
              <span className={`status-dot ${data.ready ? "ready" : ""}`} />
              <span>Local engine</span>
              <button
                className="text-button"
                onClick={() => changeView("models")}
              >
                Manage
                <ArrowUpRight size={12} />
              </button>
            </div>
            <div className="settings-section">
              <span className="mini-eyebrow">GENERATION</span>
              <div className="model-selector">
                <div className="qwen-symbol">
                  <WandSparkles size={22} />
                </div>
                <div>
                  <strong>Qwen-Image-2.1</strong>
                  <span>Turbo · 8 steps</span>
                </div>
                <span className="tiny-tag">LOCAL</span>
              </div>
              <label className="field-label" htmlFor="ratio">
                Aspect ratio
              </label>
              <div className="select-wrap">
                <Square size={16} />
                <select
                  id="ratio"
                  value={ratio}
                  onChange={(event) => setRatio(Number(event.target.value))}
                >
                  {RATIOS.map((preset, index) => (
                    <option value={index} key={preset.label}>
                      {preset.label} · {preset.name}
                    </option>
                  ))}
                </select>
                <ChevronDown size={14} />
              </div>
              <label className="field-label" htmlFor="resolution">
                Image size
              </label>
              <div className="select-wrap">
                <select
                  id="resolution"
                  value={resolution}
                  onChange={(event) =>
                    setResolution(event.target.value as typeof resolution)
                  }
                >
                  <option value="draft">Draft · 512px</option>
                  <option value="standard">Standard · 1024px</option>
                  <option value="native">Native · 2K</option>
                </select>
                <ChevronDown size={14} />
              </div>
              <span className="dimension-note">
                {size.width} × {size.height} pixels
              </span>
              <label className="field-label" htmlFor="seed">
                Seed{" "}
                <button
                  className="icon-button"
                  title="Use a random seed"
                  aria-label="Reset seed to random"
                  onClick={() => setSeed("")}
                >
                  <Shuffle size={12} />
                </button>
              </label>
              <input
                className="seed-input"
                id="seed"
                inputMode="numeric"
                placeholder="Random"
                value={seed}
                onChange={(event) => setSeed(event.target.value)}
              />
              <label className="toggle-row">
                <span>
                  Transparent background
                  <small>Request an alpha channel in PNG</small>
                </span>
                <input
                  type="checkbox"
                  checked={transparent}
                  onChange={(event) => setTransparent(event.target.checked)}
                />
                <span className="toggle" aria-hidden="true" />
              </label>
            </div>
            <div className="generation-actions">
              {!data.ready && (
                <div className="setup-nudge">
                  <Box size={17} />
                  <div>
                    <strong>Your studio, ready when you are.</strong>
                    <p>Install the local engine to start creating.</p>
                  </div>
                </div>
              )}
              <button
                className="primary generate"
                disabled={
                  active ||
                  submitting ||
                  (data.ready &&
                    (!prompt.trim() ||
                      (mode !== "create" && !references.length)))
                }
                onClick={start}
              >
                {submitting ? (
                  <LoaderCircle size={18} className="spin" />
                ) : (
                  <Sparkles size={18} />
                )}{" "}
                {data.ready
                  ? mode === "edit"
                    ? "Apply edit"
                    : "Generate image"
                  : "Set up local engine"}
              </button>
              <button
                className="secondary export"
                disabled={!selected}
                onClick={() =>
                  selected &&
                  void attempt(() => bridge.exportImage(selected.id))
                }
              >
                <ArrowDownToLine size={17} />
                Export PNG
              </button>
              {selected && (
                <div className="result-actions">
                  <button onClick={() => useAsReference("transform")}>
                    <ArrowLeftRight size={13} />
                    Transform
                  </button>
                  <button onClick={() => useAsReference("edit")}>
                    <Paintbrush size={13} />
                    Edit this
                  </button>
                </div>
              )}
            </div>
            <div className="settings-foot">
              <Cpu size={14} />
              <span>
                {data.hardware.gpuName ??
                  "Private by design. Local by default."}
              </span>
            </div>
          </aside>
        </main>
      )}
      {data?.operation &&
        (data.operation.finishedAt === null ||
          dismissedOperation !== data.operation.id) && (
          <div className="global-progress">
            <Progress
              operation={data.operation}
              stop={stop}
              dismiss={() => setDismissedOperation(data.operation!.id)}
            />
          </div>
        )}
      <footer className="app-footer">
        <span>
          <span className="status-dot" />{" "}
          {bridge.desktop
            ? "Local workspace"
            : "Browser preview · native actions require the desktop app"}
        </span>
        <span>
          Qwen-Image-2.1-Turbo<span className="footer-dot">·</span>Image Wizard
          0.2.0
        </span>
      </footer>
      {message && (
        <div className="toast" role="alert">
          <CircleHelp size={18} />
          <p>{message}</p>
          <button
            className="icon-button"
            onClick={() => setMessage(null)}
            aria-label="Dismiss message"
          >
            <X size={16} />
          </button>
        </div>
      )}
      {expanded && (
        <div
          className="lightbox"
          role="dialog"
          aria-modal="true"
          aria-label="Expanded image"
          onKeyDown={(event) => {
            if (event.key === "Tab") {
              event.preventDefault();
              event.currentTarget
                .querySelector<HTMLButtonElement>("button")
                ?.focus();
            }
          }}
          onClick={() => setExpanded(false)}
        >
          <button
            className="lightbox-close"
            aria-label="Close expanded image"
            onClick={() => setExpanded(false)}
            autoFocus
          >
            <X size={24} />
          </button>
          <img
            src={canvasImage}
            alt={selected?.prompt ?? "Studio sample"}
            onClick={(event) => event.stopPropagation()}
          />
        </div>
      )}
    </div>
  );
}
