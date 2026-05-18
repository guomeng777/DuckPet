import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import PetSelector from "../components/PetSelector";
import RuntimeControls from "../components/RuntimeControls";
import PetManagerWindow from "./PetManagerWindow";
import type {
  AudioStatus,
  AvailablePet,
  PetManifestSummary,
  PetRuntimeStatus,
} from "../types/pet";

type CommandState = "idle" | "working" | "ready" | "error";
type RuntimeAction = "start" | "pause" | "resume" | "close" | "closeAll" | "spawn";
type Locale = "zh" | "en";
type MessageKey =
  | "petWindowClosed"
  | "noPetSelected"
  | "petRuntimeStarted"
  | "petRuntimePaused"
  | "petRuntimeResumed"
  | "allPetWindowsClosed"
  | "anotherPetRuntimeSpawned"
  | "audioMuted"
  | "audioUnmuted"
  | "windowCollisionEnabled"
  | "windowCollisionDisabled"
  | "petClickThroughEnabled"
  | "petClickThroughDisabled";


const COPY = {
  zh: {
    languageButton: "English",
    petManagerButton: "素材管理",
    currentTime: "当前时间",
    brand: "DuckPet 岛屿管理所",
    platformTag: "Windows 11 宠物控制台",
    title: "好萌鸭",
    subtitle: "把桌面宠物搬进你的小岛❤",
    statusSummary: (count: number, max: number, collisionEnabled: boolean) =>
      `当前已有 ${count} / ${max} 个宠物实例在桌面活动，窗口碰撞已${collisionEnabled ? "开启" : "关闭"}。`,
    statusLabels: {
      idle: "待命",
      working: "处理中",
      ready: "运行中",
      error: "异常",
    },
    currentPet: "当前居民",
    pending: "待载入",
    yourPet: "桌面宠物",
    heroIdle: "选择一个居民，然后召唤到桌面。",
    heroActive: (petName: string, animationName: string) =>
      `${petName}正在${animationName}。`,
    ready: "就绪",
    summon: "召唤到桌面",
    spawnAnother: "再生成一个",
    closeAll: "全部关闭",
    sprite: {
      title: "精灵表",
      tiles: "图块",
      columnsRows: (columns: number, rows: number, width: number, height: number) =>
        `${columns} 列 × ${rows} 行，${width}×${height}px`,
      transparency: "透明色",
      spawns: "生成点",
      sounds: "声音",
    },
    instance: {
      title: "当前实例",
      none: "无",
      currentAnimation: "当前动画",
      frameIndex: "帧序号",
      position: "位置",
      audio: "音频",
      noFrame: "暂无帧",
      muted: "已静音",
      unmuted: "未静音",
      noSoundErrors: "没有声音错误",
    },
    messages: {
      petWindowClosed: "宠物窗口已关闭。",
      noPetSelected: "还没有选择宠物。",
      petRuntimeStarted: "宠物运行实例已启动。",
      petRuntimePaused: "宠物运行实例已暂停。",
      petRuntimeResumed: "宠物运行实例已继续。",
      allPetWindowsClosed: "所有宠物窗口已关闭。",
      anotherPetRuntimeSpawned: "已生成另一个宠物运行实例。",
      audioMuted: "声音已静音。",
      audioUnmuted: "声音已取消静音。",
      windowCollisionEnabled: "窗口碰撞已开启。",
      windowCollisionDisabled: "窗口碰撞已关闭。",
      petClickThroughEnabled: "宠物点击穿透已开启。",
      petClickThroughDisabled: "宠物点击穿透已关闭。",
    },
    petSelector: {
      title: "选择居民",
      found: "个可用",
      pet: "宠物",
      loadingPets: "正在加载宠物",
      noPetSelected: "未选择宠物",
      chooseManifest: "从 Pets/ 选择一个宠物清单。",
      xmlPath: "XML 路径",
      version: "版本",
      author: "作者",
      animations: "动画",
      sounds: "声音",
      pending: "待载入",
    },
    runtime: {
      title: "运行控制",
      start: "启动",
      pause: "暂停",
      resume: "继续",
      spawnAnother: "再生成一个",
      close: "关闭",
      closeAll: "全部关闭",
      mute: "静音",
      unmute: "取消静音",
      windowCollision: "窗口碰撞",
      clickThrough: "宠物点击穿透",
      instancesLabel: "运行中的宠物实例",
      noRunningPets: "没有运行中的宠物",
      open: "已打开",
      closed: "已关闭",
      pausedSuffix: " · 已暂停",
      statusClosed: "关闭",
      statusPaused: "暂停",
      statusRunning: "运行中",
    },
  },
  en: {
    languageButton: "中文",
    petManagerButton: "Asset Manager",
    currentTime: "Current time",
    brand: "DuckPet Island Desk",
    platformTag: "Windows 11 Pet Control",
    title: "DuckPet",
    subtitle: "Bring your desktop pets onto your own little island.",
    statusSummary: (count: number, max: number, collisionEnabled: boolean) =>
      `${count} / ${max} active pet instances, window collision ${collisionEnabled ? "enabled" : "disabled"}.`,
    statusLabels: {
      idle: "idle",
      working: "working",
      ready: "ready",
      error: "error",
    },
    currentPet: "Current pet",
    pending: "Pending",
    yourPet: "Your pet",
    heroIdle: "Choose a pet and summon it to your desktop.",
    heroActive: (petName: string, animationName: string) =>
      `${petName} is ${animationName}.`,
    ready: "ready",
    summon: "Summon to Desktop",
    spawnAnother: "Spawn Another",
    closeAll: "Close All",
    sprite: {
      title: "Sprite Sheet",
      tiles: "Tiles",
      columnsRows: (columns: number, rows: number, width: number, height: number) =>
        `${columns} columns × ${rows} rows, ${width}×${height}px`,
      transparency: "Transparency",
      spawns: "Spawns",
      sounds: "Sounds",
    },
    instance: {
      title: "Current Instance",
      none: "None",
      currentAnimation: "Current animation",
      frameIndex: "Frame index",
      position: "Position",
      audio: "Audio",
      noFrame: "No frame yet",
      muted: "muted",
      unmuted: "unmuted",
      noSoundErrors: "No sound errors",
    },
    messages: {
      petWindowClosed: "Pet window is closed.",
      noPetSelected: "No pet selected.",
      petRuntimeStarted: "Pet runtime started.",
      petRuntimePaused: "Pet runtime paused.",
      petRuntimeResumed: "Pet runtime resumed.",
      allPetWindowsClosed: "All pet windows are closed.",
      anotherPetRuntimeSpawned: "Another pet runtime spawned.",
      audioMuted: "Audio muted.",
      audioUnmuted: "Audio unmuted.",
      windowCollisionEnabled: "Window collision enabled.",
      windowCollisionDisabled: "Window collision disabled.",
      petClickThroughEnabled: "Pet click-through enabled.",
      petClickThroughDisabled: "Pet click-through disabled.",
    },
    petSelector: {
      title: "Pet Selection",
      found: "found",
      pet: "Pet",
      loadingPets: "Loading pets",
      noPetSelected: "No pet selected",
      chooseManifest: "Choose a pet manifest from Pets/.",
      xmlPath: "XML path",
      version: "Version",
      author: "Author",
      animations: "Animations",
      sounds: "Sounds",
      pending: "Pending",
    },
    runtime: {
      title: "Runtime",
      start: "Start",
      pause: "Pause",
      resume: "Resume",
      spawnAnother: "Spawn Another",
      close: "Close",
      closeAll: "Close All",
      mute: "Mute",
      unmute: "Unmute",
      windowCollision: "Enable window collision",
      clickThrough: "Enable pet click-through",
      instancesLabel: "Running pet instances",
      noRunningPets: "No running pets",
      open: "open",
      closed: "closed",
      pausedSuffix: " paused",
      statusClosed: "closed",
      statusPaused: "paused",
      statusRunning: "running",
    },
  },
} as const;

function SettingsWindow() {
  const [locale, setLocale] = useState<Locale>("zh");
  const [status, setStatus] = useState<CommandState>("idle");
  const [messageKey, setMessageKey] = useState<MessageKey>("petWindowClosed");
  const [messageOverride, setMessageOverride] = useState<string | null>(null);
  const [pets, setPets] = useState<AvailablePet[]>([]);
  const [selectedXmlPath, setSelectedXmlPath] = useState("");
  const [petListError, setPetListError] = useState<string | null>(null);
  const [manifest, setManifest] = useState<PetManifestSummary | null>(null);
  const [manifestError, setManifestError] = useState<string | null>(null);
  const [runtimeStatus, setRuntimeStatus] = useState<PetRuntimeStatus | null>(null);
  const [selectedInstanceId, setSelectedInstanceId] = useState("");
  const [audioStatus, setAudioStatus] = useState<AudioStatus | null>(null);
  const [currentPage, setCurrentPage] = useState<"settings" | "petManager">("settings");
  const copy = COPY[locale];
  const message = messageOverride ?? copy.messages[messageKey];

  function setLocalizedMessage(nextMessageKey: MessageKey) {
    setMessageKey(nextMessageKey);
    setMessageOverride(null);
  }

  function setErrorMessage(error: unknown) {
    setStatus("error");
    setMessageOverride(error instanceof Error ? error.message : String(error));
  }

  useEffect(() => {
    let isMounted = true;
    let timerId: number | undefined;

    async function loadPets() {
      try {
        const loadedPets = await invoke<AvailablePet[]>("list_available_pets");
        const preferredPet =
          loadedPets.find((pet) => pet.id.toLowerCase() === "esheep64") ??
          loadedPets[0];

        if (isMounted) {
          setPets(loadedPets);
          setSelectedXmlPath((currentPath) => currentPath || preferredPet?.xmlPath || "");
          setPetListError(null);
        }
      } catch (error) {
        if (isMounted) {
          setPets([]);
          setPetListError(error instanceof Error ? error.message : String(error));
        }
      }
    }

    timerId = window.setTimeout(() => {
      void loadPets();
    }, 100);

    return () => {
      isMounted = false;
      if (timerId !== undefined) {
        window.clearTimeout(timerId);
      }
    };
  }, []);

  useEffect(() => {
    let isMounted = true;
    let timerId: number | undefined;

    async function refreshAudioStatus() {
      try {
        const loadedStatus = await invoke<AudioStatus>("get_audio_status");

        if (isMounted) {
          setAudioStatus(loadedStatus);
        }
      } catch {
        if (isMounted) {
          setAudioStatus(null);
        }
      } finally {
        if (isMounted) {
          timerId = window.setTimeout(refreshAudioStatus, 1000);
        }
      }
    }

    void refreshAudioStatus();

    return () => {
      isMounted = false;
      if (timerId !== undefined) {
        window.clearTimeout(timerId);
      }
    };
  }, []);

  useEffect(() => {
    if (!selectedXmlPath) {
      setManifest(null);
      return;
    }

    let isMounted = true;

    async function loadSelectedManifest() {
      try {
        const loadedManifest = await invoke<PetManifestSummary>("load_pet_manifest", {
          xmlPath: selectedXmlPath,
        });

        if (isMounted) {
          setManifest(loadedManifest);
          setManifestError(null);
        }
      } catch (error) {
        if (isMounted) {
          setManifest(null);
          setManifestError(error instanceof Error ? error.message : String(error));
        }
      }
    }

    void loadSelectedManifest();

    return () => {
      isMounted = false;
    };
  }, [selectedXmlPath]);

  useEffect(() => {
    let isMounted = true;
    let timerId: number | undefined;

    async function refreshRuntimeStatus() {
      try {
        const loadedStatus = await invoke<PetRuntimeStatus>("get_pet_runtime_status");

        if (isMounted) {
          setRuntimeStatus(loadedStatus);
          setSelectedInstanceId((currentId) =>
            chooseSelectedInstanceId(currentId, loadedStatus),
          );
        }
      } catch {
        if (isMounted) {
          setRuntimeStatus(null);
        }
      } finally {
        if (isMounted) {
          timerId = window.setTimeout(refreshRuntimeStatus, 500);
        }
      }
    }

    void refreshRuntimeStatus();

    return () => {
      isMounted = false;
      if (timerId !== undefined) {
        window.clearTimeout(timerId);
      }
    };
  }, []);

  async function runRuntimeAction(action: RuntimeAction) {
    if (!selectedXmlPath && action !== "pause" && action !== "resume" && action !== "close" && action !== "closeAll") {
      setStatus("error");
      setLocalizedMessage("noPetSelected");
      return;
    }

    setStatus("working");

    try {
      const command = commandForAction(action);
      const args = argsForAction(action, selectedXmlPath, selectedInstanceId);
      const nextStatus = await invokeWithTimeout<PetRuntimeStatus>(
        command,
        args,
        8000,
      );
      setRuntimeStatus(nextStatus);
      setSelectedInstanceId((currentId) =>
        chooseSelectedInstanceId(currentId, nextStatus),
      );

      if (action === "start" || action === "spawn" || action === "resume") {
        await ensurePetWindows(nextStatus);
      }

      setStatus("ready");
      setLocalizedMessage(messageKeyForAction(action));
    } catch (error) {
      setErrorMessage(error);
    }
  }

  async function toggleMute() {
    setStatus("working");

    try {
      const nextStatus = await invoke<AudioStatus>("set_audio_muted", {
        muted: !audioStatus?.muted,
      });
      setAudioStatus(nextStatus);
      setStatus("ready");
      setLocalizedMessage(nextStatus.muted ? "audioMuted" : "audioUnmuted");
    } catch (error) {
      setErrorMessage(error);
    }
  }

  async function setWindowCollision(enabled: boolean) {
    setStatus("working");

    try {
      const nextStatus = await invoke<PetRuntimeStatus>("set_window_collision_enabled", {
        enabled,
      });
      setRuntimeStatus(nextStatus);
      setStatus("ready");
      setLocalizedMessage(
        enabled ? "windowCollisionEnabled" : "windowCollisionDisabled",
      );
    } catch (error) {
      setErrorMessage(error);
    }
  }

  async function setClickThrough(enabled: boolean) {
    setStatus("working");

    try {
      const nextStatus = await invoke<PetRuntimeStatus>(
        "set_pet_click_through_enabled",
        { enabled },
      );
      setRuntimeStatus(nextStatus);
      setStatus("ready");
      setLocalizedMessage(
        enabled ? "petClickThroughEnabled" : "petClickThroughDisabled",
      );
    } catch (error) {
      setErrorMessage(error);
    }
  }

  const combinedPetError = petListError ?? manifestError;
  const selectedInstance = selectedRuntimeInstance(runtimeStatus, selectedInstanceId);
  const frame = selectedInstance?.latestFrame ?? runtimeStatus?.latestFrame ?? null;
  const instanceCount = runtimeStatus?.instances.length ?? 0;
  const maxInstances = runtimeStatus?.maxInstances ?? 4;

  if (currentPage === "petManager") {
    return <PetManagerWindow locale={locale} onBack={() => setCurrentPage("settings")} />;
  }

  return (
    <main className="settings-shell">
      <section className="settings-header">
        <div>
          <div className="brand-line">
            <span className="eyebrow">{copy.brand}</span>
            <span className="mini-tag">{copy.platformTag}</span>
            <button
              type="button"
              className="language-toggle"
              onClick={() => setLocale((currentLocale) => (currentLocale === "zh" ? "en" : "zh"))}
            >
              {copy.languageButton}
            </button>
            <button
              type="button"
              className="language-toggle"
              onClick={() => setCurrentPage("petManager")}
            >
              {copy.petManagerButton}
            </button>
          </div>
          <h1>{copy.title}</h1>
          <p className="settings-subtitle">{copy.subtitle}</p>
        </div>
        <aside className="clock-card" aria-label={copy.currentTime}>
          <span>{formatHeaderDate(new Date(), locale)}</span>
          <strong>{formatHeaderTime(new Date())}</strong>
        </aside>
      </section>

      <section className={`notice status-${status}`} aria-live="polite">
        <div className="notice-icon" aria-hidden="true">
          {status === "error" ? "!" : "✓"}
        </div>
        <div>
          <strong>{message}</strong>
          <span>{copy.statusSummary(instanceCount, maxInstances, runtimeStatus?.windowCollisionEnabled ?? true)}</span>
        </div>
        <span className="mini-tag">{copy.statusLabels[status]}</span>
      </section>

      <section className="settings-layout">
        <aside className="left-rail">
          <PetSelector
            pets={pets}
            selectedXmlPath={selectedXmlPath}
            manifest={manifest}
            isLoading={status === "working" && pets.length === 0}
            error={combinedPetError}
            onSelect={setSelectedXmlPath}
            copy={copy.petSelector}
          />

          <RuntimeControls
            status={runtimeStatus}
            audioStatus={audioStatus}
            selectedInstanceId={selectedInstanceId}
            isWorking={status === "working"}
            canStart={Boolean(selectedXmlPath)}
            onAction={runRuntimeAction}
            onSelectInstance={setSelectedInstanceId}
            onToggleMute={toggleMute}
            onSetWindowCollision={setWindowCollision}
            onSetClickThrough={setClickThrough}
            copy={copy.runtime}
          />
        </aside>

        <section className="main-stack">
          <article className="panel hero-card">
            <div className="hero-copy">
              <span className="mini-tag">
                {copy.currentPet} · {manifest?.header.petname ?? copy.pending}
              </span>
              <h2>
                {frame
                  ? copy.heroActive(manifest?.header.petname ?? copy.yourPet, frame.animationName)
                  : copy.heroIdle}
              </h2>
              <div className="hero-actions">
                <button
                  type="button"
                  onClick={() => runRuntimeAction("start")}
                  disabled={status === "working" || !selectedXmlPath}
                >
                  {copy.summon}
                </button>
                <button
                  type="button"
                  className="secondary"
                  onClick={() => runRuntimeAction("spawn")}
                  disabled={
                    status === "working" ||
                    !selectedXmlPath ||
                    instanceCount >= maxInstances
                  }
                >
                  {copy.spawnAnother}
                </button>
                <button
                  type="button"
                  className="danger"
                  onClick={() => runRuntimeAction("closeAll")}
                  disabled={status === "working" || instanceCount === 0}
                >
                  {copy.closeAll}
                </button>
              </div>
            </div>
            <div className="pet-stage" aria-hidden="true">
              <div className="pet-bubble">
                {manifest ? (
                  <img className="hero-pet-image" src={manifest.spriteSheet.dataUrl} alt="" />
                ) : (
                  <div className="pet-icon" />
                )}
                <span className="bubble-label">
                  {frame ? `${frame.animationName} #${frame.frameIndex}` : copy.ready}
                </span>
              </div>
            </div>
          </article>

          <section className="two-col">
            <article className="panel">
              <div className="panel-heading">
                <h2>{copy.sprite.title}</h2>
                <span>
                  {manifest
                    ? `${manifest.spriteSheet.tilesX} × ${manifest.spriteSheet.tilesY}`
                    : copy.pending}
                </span>
              </div>
              <dl className="runtime-list">
                <div>
                  <dt>{copy.sprite.tiles}</dt>
                  <dd>
                    {manifest
                      ? copy.sprite.columnsRows(
                          manifest.spriteSheet.tilesX,
                          manifest.spriteSheet.tilesY,
                          manifest.spriteSheet.tileWidth,
                          manifest.spriteSheet.tileHeight,
                        )
                      : copy.pending}
                  </dd>
                </div>
                <div>
                  <dt>{copy.sprite.transparency}</dt>
                  <dd>{manifest?.spriteSheet.transparency ?? copy.pending}</dd>
                </div>
                <div>
                  <dt>{copy.sprite.spawns}</dt>
                  <dd>{manifest?.spawnCount ?? copy.pending}</dd>
                </div>
                <div>
                  <dt>{copy.sprite.sounds}</dt>
                  <dd>{manifest?.soundCount ?? copy.pending}</dd>
                </div>
              </dl>
              <div className="sprite-preview" aria-hidden="true">
                {manifest ? <img src={manifest.spriteSheet.dataUrl} alt="" /> : null}
              </div>
            </article>

            <article className="panel current-instance-panel">
              <div className="panel-heading">
                <h2>{copy.instance.title}</h2>
                <span>{selectedInstance?.petInstanceId ?? copy.instance.none}</span>
              </div>
              <dl className="runtime-list">
                <div>
                  <dt>{copy.instance.currentAnimation}</dt>
                  <dd>
                    {frame
                      ? `${frame.animationId} / ${frame.animationName}`
                      : copy.instance.noFrame}
                  </dd>
                </div>
                <div>
                  <dt>{copy.instance.frameIndex}</dt>
                  <dd>
                    {frame
                      ? `${frame.frameIndex} (${frame.sequenceStep + 1}/${frame.totalSteps})`
                      : copy.pending}
                  </dd>
                </div>
                <div>
                  <dt>{copy.instance.position}</dt>
                  <dd>{frame ? `${frame.x}, ${frame.y}` : copy.pending}</dd>
                </div>
                <div>
                  <dt>{copy.instance.audio}</dt>
                  <dd>
                    {audioStatus?.muted ? copy.instance.muted : copy.instance.unmuted} ·{" "}
                    {audioStatus?.lastError ?? copy.instance.noSoundErrors}
                  </dd>
                </div>
              </dl>
            </article>
          </section>

        </section>
      </section>
    </main>
  );
}

function commandForAction(action: RuntimeAction) {
  switch (action) {
    case "start":
      return "start_pet_runtime";
    case "pause":
      return "pause_pet_runtime";
    case "resume":
      return "resume_pet_runtime";
    case "close":
      return "close_pet_runtime";
    case "closeAll":
      return "close_all_pet_runtimes";
    case "spawn":
      return "spawn_pet_runtime";
  }
}

function argsForAction(
  action: RuntimeAction,
  selectedXmlPath: string,
  selectedInstanceId: string,
) {
  switch (action) {
    case "start":
    case "spawn":
      return { xmlPath: selectedXmlPath };
    case "pause":
    case "resume":
    case "close":
      return { petInstanceId: selectedInstanceId || null };
    case "closeAll":
      return undefined;
  }
}

function messageKeyForAction(action: RuntimeAction): MessageKey {
  switch (action) {
    case "start":
      return "petRuntimeStarted";
    case "pause":
      return "petRuntimePaused";
    case "resume":
      return "petRuntimeResumed";
    case "close":
      return "petWindowClosed";
    case "closeAll":
      return "allPetWindowsClosed";
    case "spawn":
      return "anotherPetRuntimeSpawned";
  }
}

function chooseSelectedInstanceId(currentId: string, status: PetRuntimeStatus) {
  if (status.instances.some((instance) => instance.petInstanceId === currentId)) {
    return currentId;
  }

  return status.activeInstanceId ?? status.instances[0]?.petInstanceId ?? "";
}

function selectedRuntimeInstance(
  status: PetRuntimeStatus | null,
  selectedInstanceId: string,
) {
  if (!status?.instances.length) {
    return null;
  }

  return (
    status.instances.find((instance) => instance.petInstanceId === selectedInstanceId) ??
    status.instances.find((instance) => instance.petInstanceId === status.activeInstanceId) ??
    status.instances[0]
  );
}

function formatHeaderDate(date: Date, locale: Locale) {
  return new Intl.DateTimeFormat(locale === "zh" ? "zh-CN" : "en-US", {
    weekday: "short",
    month: "short",
    day: "numeric",
  }).format(date);
}

function formatHeaderTime(date: Date) {
  return new Intl.DateTimeFormat("en-US", {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(date);
}

export default SettingsWindow;

async function ensurePetWindows(status: PetRuntimeStatus) {
  const targetInstances = status.instances.filter((instance) => !instance.isPaused);

  for (const instance of targetInstances) {
    await ensurePetWindow(instance.petInstanceId, status.clickThroughEnabled);
  }
}

async function ensurePetWindow(petInstanceId: string, clickThrough: boolean) {
  const existing = await WebviewWindow.getByLabel(petInstanceId);

  if (existing) {
    await existing.show();
    await existing.setAlwaysOnTop(true);
    await existing.setIgnoreCursorEvents(clickThrough);
    return;
  }

  const petWindow = new WebviewWindow(petInstanceId, {
    url: `index.html?window=pet&petInstanceId=${encodeURIComponent(petInstanceId)}`,
    title: `DuckPet ${petInstanceId}`,
    width: 160,
    height: 160,
    resizable: false,
    decorations: false,
    transparent: true,
    alwaysOnTop: true,
    skipTaskbar: true,
    shadow: false,
    focusable: false,
  });

  await waitForPetWindowCreated(petWindow);
  await petWindow.setIgnoreCursorEvents(clickThrough);
}

function waitForPetWindowCreated(petWindow: WebviewWindow) {
  return new Promise<void>((resolve, reject) => {
    let finished = false;
    const timeoutId = window.setTimeout(() => {
      if (!finished) {
        finished = true;
        reject(new Error(`pet window ${petWindow.label} creation timed out`));
      }
    }, 5000);

    void petWindow.once("tauri://created", () => {
      if (!finished) {
        finished = true;
        window.clearTimeout(timeoutId);
        resolve();
      }
    });

    void petWindow.once<string>("tauri://error", (event) => {
      if (!finished) {
        finished = true;
        window.clearTimeout(timeoutId);
        reject(new Error(`pet window ${petWindow.label} failed: ${event.payload}`));
      }
    });
  });
}

function invokeWithTimeout<T>(
  command: string,
  args: Record<string, unknown> | undefined,
  timeoutMs: number,
) {
  return new Promise<T>((resolve, reject) => {
    const timeoutId = window.setTimeout(() => {
      reject(new Error(`${command} timed out after ${timeoutMs}ms`));
    }, timeoutMs);

    invoke<T>(command, args)
      .then(resolve)
      .catch(reject)
      .finally(() => window.clearTimeout(timeoutId));
  });
}
