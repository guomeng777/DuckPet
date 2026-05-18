import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type {
  PetFrame,
  PetInteractionResult,
  PetManifestSummary,
  PetRuntimeStatus,
} from "../types/pet";

interface ContextMenuState {
  x: number;
  y: number;
}

function PetWindow() {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const isDraggingRef = useRef(false);
  const latestFrameRef = useRef<PetFrame | null>(null);
  const dragFallbackTimerRef = useRef<number | undefined>(undefined);
  const [runtimeStatus, setRuntimeStatus] = useState<PetRuntimeStatus | null>(null);
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null);
  const petInstanceId =
    new URLSearchParams(window.location.search).get("petInstanceId") ?? "pet_1";
  const currentInstance = runtimeStatus?.instances.find(
    (instance) => instance.petInstanceId === petInstanceId,
  );

  useEffect(() => {
    document.documentElement.classList.add("pet-window-html");
    document.body.classList.add("pet-window-body");

    let isMounted = true;
    let timeoutId: number | undefined;
    let activePath = "";
    let activeManifest: PetManifestSummary | null = null;
    let activeImage: HTMLImageElement | null = null;
    const canvas = canvasRef.current;
    const context = canvas?.getContext("2d");

    if (canvas && context) {
      const drawingContext = context;
      const pixelRatio = window.devicePixelRatio || 1;
      const size = 160;

      canvas.width = size * pixelRatio;
      canvas.height = size * pixelRatio;
      canvas.style.width = `${size}px`;
      canvas.style.height = `${size}px`;
      drawingContext.scale(pixelRatio, pixelRatio);
      drawingContext.clearRect(0, 0, size, size);

      async function ensurePetAssets(xmlPath: string) {
        if (xmlPath === activePath && activeManifest && activeImage) {
          return { manifest: activeManifest, image: activeImage };
        }

        const manifest = await invoke<PetManifestSummary>("load_pet_manifest", {
          xmlPath,
        });
        const image = await loadImage(manifest.spriteSheet.dataUrl);

        activePath = xmlPath;
        activeManifest = manifest;
        activeImage = image;

        return { manifest, image };
      }

      async function drawNextFrame() {
        if (!isMounted) {
          return;
        }

        try {
          const status = await invoke<PetRuntimeStatus>("get_pet_runtime_status");
          if (isMounted) {
            setRuntimeStatus(status);
          }
          const instance = status.instances.find(
            (instance) => instance.petInstanceId === petInstanceId,
          );
          const xmlPath = instance?.xmlPath;

          if (!xmlPath) {
            latestFrameRef.current = null;
            drawWaitingState(drawingContext, size);
            timeoutId = window.setTimeout(drawNextFrame, 250);
            return;
          }

          const { manifest, image } = await ensurePetAssets(xmlPath);
          const frame = await invoke<PetFrame>("next_pet_frame_for_instance", {
            petInstanceId,
            viewportWidth: size,
            viewportHeight: size,
          });

          drawFrame(drawingContext, image, manifest, frame, size);
          latestFrameRef.current = frame;
          timeoutId = window.setTimeout(drawNextFrame, Math.max(frame.intervalMs, 16));
        } catch (error) {
          latestFrameRef.current = null;
          drawErrorState(
            drawingContext,
            size,
            error instanceof Error ? error.message : String(error),
          );
          timeoutId = window.setTimeout(drawNextFrame, 500);
        }
      }

      void drawNextFrame();
    }

    return () => {
      isMounted = false;
      if (timeoutId !== undefined) {
        window.clearTimeout(timeoutId);
      }
      if (dragFallbackTimerRef.current !== undefined) {
        window.clearTimeout(dragFallbackTimerRef.current);
      }
      document.body.classList.remove("pet-window-body");
      document.documentElement.classList.remove("pet-window-html");
    };
  }, [petInstanceId]);

  useEffect(() => {
    function closeMenu() {
      setContextMenu(null);
    }

    function closeMenuOnEscape(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setContextMenu(null);
      }
    }

    window.addEventListener("click", closeMenu);
    window.addEventListener("keydown", closeMenuOnEscape);

    return () => {
      window.removeEventListener("click", closeMenu);
      window.removeEventListener("keydown", closeMenuOnEscape);
    };
  }, []);

  useEffect(() => {
    function finishDrag() {
      if (!isDraggingRef.current) {
        return;
      }

      isDraggingRef.current = false;
      if (dragFallbackTimerRef.current !== undefined) {
        window.clearTimeout(dragFallbackTimerRef.current);
        dragFallbackTimerRef.current = undefined;
      }
      void invoke("end_pet_drag", { petInstanceId });
    }

    window.addEventListener("mouseup", finishDrag);
    window.addEventListener("pointerup", finishDrag);

    return () => {
      window.removeEventListener("mouseup", finishDrag);
      window.removeEventListener("pointerup", finishDrag);
    };
  }, [petInstanceId]);

  function showContextMenu(event: React.MouseEvent<HTMLElement>) {
    event.preventDefault();
    event.stopPropagation();
    if (!isPointerOnPet(event)) {
      setContextMenu(null);
      return;
    }

    setContextMenu({
      x: Math.min(event.clientX, 38),
      y: Math.min(event.clientY, 16),
    });
  }

  function startDrag(event: React.PointerEvent<HTMLElement>) {
    if (event.button !== 0 || event.detail > 1 || contextMenu) {
      return;
    }

    if (!isPointerOnPet(event)) {
      return;
    }

    isDraggingRef.current = true;
    setContextMenu(null);
    void invoke("begin_pet_drag", { petInstanceId });
    void getCurrentWindow().startDragging();

    if (dragFallbackTimerRef.current !== undefined) {
      window.clearTimeout(dragFallbackTimerRef.current);
    }
    dragFallbackTimerRef.current = window.setTimeout(() => {
      if (isDraggingRef.current) {
        isDraggingRef.current = false;
        void invoke("end_pet_drag", { petInstanceId });
      }
    }, 5000);
  }

  async function runMenuAction(action: string) {
    setContextMenu(null);

    if (action === "settings") {
      await invoke("open_settings_window");
    } else if (action === "add") {
      const xmlPath = currentInstance?.xmlPath ?? runtimeStatus?.selectedXmlPath;
      if (xmlPath) {
        await invoke("spawn_pet_runtime", { xmlPath });
      }
    } else if (action === "pause") {
      await invoke(currentInstance?.isPaused ? "resume_pet_runtime" : "pause_pet_runtime", {
        petInstanceId,
      });
    } else if (action === "close") {
      await invoke("close_pet_runtime", { petInstanceId });
    } else if (action === "exit") {
      await invoke("exit_app");
    }
  }

  async function handleDoubleClick(event: React.MouseEvent<HTMLElement>) {
    event.preventDefault();
    event.stopPropagation();
    if (!isPointerOnPet(event)) {
      return;
    }

    const result = await invoke<PetInteractionResult>("trigger_pet_kill_or_close", {
      petInstanceId,
    });
    setRuntimeStatus(result.status);
  }

  function isPointerOnPet(
    event: React.MouseEvent<HTMLElement> | React.PointerEvent<HTMLElement>,
  ) {
    const frame = latestFrameRef.current;
    if (!frame) {
      return false;
    }

    const x = event.clientX;
    const y = event.clientY;
    const isInsideFrame =
      x >= frame.x &&
      x < frame.x + frame.width &&
      y >= frame.y &&
      y < frame.y + frame.height;

    if (!isInsideFrame) {
      return false;
    }

    const canvas = canvasRef.current;
    const context = canvas?.getContext("2d");
    if (!canvas || !context) {
      return true;
    }

    const canvasBounds = canvas.getBoundingClientRect();
    const pixelRatio = window.devicePixelRatio || 1;
    const pixelX = Math.floor((x - canvasBounds.left) * pixelRatio);
    const pixelY = Math.floor((y - canvasBounds.top) * pixelRatio);

    if (
      pixelX < 0 ||
      pixelY < 0 ||
      pixelX >= canvas.width ||
      pixelY >= canvas.height
    ) {
      return false;
    }

    return context.getImageData(pixelX, pixelY, 1, 1).data[3] > 16;
  }

  return (
    <main
      className="pet-shell"
      onContextMenu={showContextMenu}
      onPointerDown={startDrag}
      onDoubleClick={handleDoubleClick}
    >
      <canvas ref={canvasRef} aria-label="Pet window canvas" />
      {contextMenu ? (
        <div
          className="pet-context-menu"
          style={{ left: contextMenu.x, top: contextMenu.y }}
          role="menu"
          onClick={(event) => event.stopPropagation()}
        >
          <button type="button" role="menuitem" onClick={() => void runMenuAction("settings")}>
            Open Settings
          </button>
          <button type="button" role="menuitem" onClick={() => void runMenuAction("add")}>
            Add Pet
          </button>
          <button type="button" role="menuitem" onClick={() => void runMenuAction("pause")}>
            {currentInstance?.isPaused ? "Resume" : "Pause"}
          </button>
          <button type="button" role="menuitem" onClick={() => void runMenuAction("close")}>
            Close This Pet
          </button>
          <button type="button" role="menuitem" onClick={() => void runMenuAction("exit")}>
            Exit
          </button>
        </div>
      ) : null}
    </main>
  );
}

function loadImage(source: string) {
  return new Promise<HTMLImageElement>((resolve, reject) => {
    const image = new Image();

    image.onload = () => resolve(image);
    image.onerror = () => reject(new Error("failed to load pet sprite sheet"));
    image.src = source;
  });
}

function drawFrame(
  context: CanvasRenderingContext2D,
  image: HTMLImageElement,
  manifest: PetManifestSummary,
  frame: PetFrame,
  canvasSize: number,
) {
  const tileWidth = manifest.spriteSheet.tileWidth;
  const tileHeight = manifest.spriteSheet.tileHeight;
  const sourceX = (frame.frameIndex % manifest.spriteSheet.tilesX) * tileWidth;
  const sourceY =
    Math.floor(frame.frameIndex / manifest.spriteSheet.tilesX) * tileHeight;
  const targetWidth = frame.width;
  const targetHeight = frame.height;
  const targetX = frame.x;
  const targetY = frame.y;

  context.clearRect(0, 0, canvasSize, canvasSize);
  context.globalAlpha = frame.opacity;
  context.imageSmoothingEnabled = false;

  if (frame.flipped) {
    context.save();
    context.scale(-1, 1);
    context.drawImage(
      image,
      sourceX,
      sourceY,
      tileWidth,
      tileHeight,
      -targetX - targetWidth,
      targetY,
      targetWidth,
      targetHeight,
    );
    context.restore();
  } else {
    context.drawImage(
      image,
      sourceX,
      sourceY,
      tileWidth,
      tileHeight,
      targetX,
      targetY,
      targetWidth,
      targetHeight,
    );
  }

  context.globalAlpha = 1;
}

function drawWaitingState(context: CanvasRenderingContext2D, size: number) {
  context.clearRect(0, 0, size, size);
  context.fillStyle = "#f8fafc";
  context.strokeStyle = "#64748b";
  context.lineWidth = 2;
  context.beginPath();
  context.roundRect(18, 46, 112, 38, 8);
  context.fill();
  context.stroke();
  context.fillStyle = "#334155";
  context.font = "12px sans-serif";
  context.textAlign = "center";
  context.fillText("Waiting for pet", size / 2, 69);
}

function drawErrorState(
  context: CanvasRenderingContext2D,
  size: number,
  message: string,
) {
  context.clearRect(0, 0, size, size);
  context.fillStyle = "#fff1f1";
  context.strokeStyle = "#7f1d1d";
  context.lineWidth = 2;
  context.beginPath();
  context.roundRect(18, 42, 92, 44, 8);
  context.fill();
  context.stroke();
  context.fillStyle = "#7f1d1d";
  context.font = "12px sans-serif";
  context.textAlign = "center";
  context.fillText("Pet load error", size / 2, 62);
  context.fillText(message.slice(0, 14), size / 2, 78);
}

export default PetWindow;
