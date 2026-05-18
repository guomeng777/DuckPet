import type { AudioStatus, PetInstanceStatus, PetRuntimeStatus } from "../types/pet";

type RuntimeAction = "start" | "pause" | "resume" | "close" | "closeAll" | "spawn";

interface RuntimeControlsProps {
  status: PetRuntimeStatus | null;
  audioStatus: AudioStatus | null;
  selectedInstanceId: string;
  isWorking: boolean;
  canStart: boolean;
  onAction: (action: RuntimeAction) => void;
  onSelectInstance: (petInstanceId: string) => void;
  onToggleMute: () => void;
  onSetWindowCollision: (enabled: boolean) => void;
  onSetClickThrough: (enabled: boolean) => void;
  copy: {
    title: string;
    start: string;
    pause: string;
    resume: string;
    spawnAnother: string;
    close: string;
    closeAll: string;
    mute: string;
    unmute: string;
    windowCollision: string;
    clickThrough: string;
    instancesLabel: string;
    noRunningPets: string;
    open: string;
    closed: string;
    pausedSuffix: string;
    statusClosed: string;
    statusPaused: string;
    statusRunning: string;
  };
}

function RuntimeControls({
  status,
  audioStatus,
  selectedInstanceId,
  isWorking,
  canStart,
  onAction,
  onSelectInstance,
  onToggleMute,
  onSetWindowCollision,
  onSetClickThrough,
  copy,
}: RuntimeControlsProps) {
  const selectedInstance = selectedRuntimeInstance(status, selectedInstanceId);
  const disabled = isWorking || !canStart;
  const instanceCount = status?.instances.length ?? 0;
  const maxInstances = status?.maxInstances ?? 4;

  return (
    <article className="panel runtime-panel">
      <div className="panel-heading">
        <h2>{copy.title}</h2>
        <span>
          {statusLabel(selectedInstance, copy)} {instanceCount}/{maxInstances}
        </span>
      </div>

      <div className="button-row control-row">
        <button type="button" onClick={() => onAction("start")} disabled={disabled}>
          {copy.start}
        </button>
        <button
          type="button"
          className="secondary"
          onClick={() => onAction("pause")}
          disabled={isWorking || !selectedInstance || selectedInstance.isPaused}
        >
          {copy.pause}
        </button>
        <button
          type="button"
          className="secondary"
          onClick={() => onAction("resume")}
          disabled={isWorking || !selectedInstance?.isPaused}
        >
          {copy.resume}
        </button>
        <button
          type="button"
          className="secondary"
          onClick={() => onAction("spawn")}
          disabled={disabled || instanceCount >= maxInstances}
        >
          {copy.spawnAnother}
        </button>
        <button
          type="button"
          className="danger"
          onClick={() => onAction("close")}
          disabled={isWorking || !selectedInstance}
        >
          {copy.close}
        </button>
        <button
          type="button"
          className="danger secondary-danger"
          onClick={() => onAction("closeAll")}
          disabled={isWorking || instanceCount === 0}
        >
          {copy.closeAll}
        </button>
        <button type="button" className="secondary" onClick={onToggleMute} disabled={isWorking}>
          {audioStatus?.muted ? copy.unmute : copy.mute}
        </button>
      </div>

      <label className="toggle-row">
        <input
          type="checkbox"
          checked={status?.windowCollisionEnabled ?? true}
          disabled={isWorking}
          onChange={(event) => onSetWindowCollision(event.currentTarget.checked)}
        />
        <span>{copy.windowCollision}</span>
      </label>

      <label className="toggle-row">
        <input
          type="checkbox"
          checked={status?.clickThroughEnabled ?? false}
          disabled={isWorking}
          onChange={(event) => onSetClickThrough(event.currentTarget.checked)}
        />
        <span>{copy.clickThrough}</span>
      </label>

      <div className="instance-list" aria-label={copy.instancesLabel}>
        {status?.instances.length ? (
          status.instances.map((instance) => (
            <button
              type="button"
              key={instance.petInstanceId}
              className={
                instance.petInstanceId === selectedInstance?.petInstanceId
                  ? "instance-row selected"
                  : "instance-row"
              }
              onClick={() => onSelectInstance(instance.petInstanceId)}
            >
              <strong>{instance.petInstanceId}</strong>
              <span>
                {instance.windowOpen ? copy.open : copy.closed}
                {instance.isPaused ? copy.pausedSuffix : ""}
              </span>
            </button>
          ))
        ) : (
          <div className="empty-instance-row">{copy.noRunningPets}</div>
        )}
      </div>
    </article>
  );
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

function statusLabel(instance: PetInstanceStatus | null, copy: RuntimeControlsProps["copy"]) {
  if (!instance?.windowOpen) {
    return copy.statusClosed;
  }

  if (instance.isPaused) {
    return copy.statusPaused;
  }

  return copy.statusRunning;
}

export default RuntimeControls;
