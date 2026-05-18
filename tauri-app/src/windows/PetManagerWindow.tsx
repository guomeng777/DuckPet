import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  ExportedPetArchive,
  ManageablePet,
  PetArchiveValidation,
} from "../types/pet";

type Locale = "zh" | "en";
type ManagerStatus = "idle" | "working" | "ready" | "error";

const COPY = {
  zh: {
    back: "返回主界面",
    refresh: "刷新列表",
    brand: "DuckPet 素材管理",
    platformTag: "ZIP 素材工作台",
    title: "居民素材库",
    subtitle: "查看当前可用 PET，准备上传你制作的 ZIP 素材，并下载已有素材作为参考。",
    listTitle: "当前 PET",
    builtin: "内置",
    user: "用户上传",
    loading: "正在加载素材",
    empty: "还没有可用 PET。",
    animations: "动画",
    spawns: "生成点",
    sounds: "声音",
    download: "下载参考 ZIP",
    delete: "删除",
    cannotDeleteBuiltin: "内置素材不可删除",
    uploadTitle: "上传新素材",
    petName: "素材名称",
    petNamePlaceholder: "例如 my_duck_pet",
    uploadFile: "ZIP 文件",
    chooseFile: "选择 ZIP 文件",
    noFile: "尚未选择文件",
    selectedFile: "已选择",
    validate: "检验素材",
    save: "保存素材",
    validating: "正在检验素材",
    saving: "正在保存素材",
    deleting: "正在删除素材",
    downloading: "正在准备下载",
    loadReady: "素材列表已载入。",
    validateReady: "素材检验通过，可以保存。",
    validateFailed: "素材检验未通过。",
    saveReady: "素材已保存。",
    deleteReady: "素材已删除。",
    downloadReady: "ZIP 下载成功。完整动作精灵表在 sprite-sheet.png，icon.png 只是图标。",
    chooseNameAndFile: "请先填写素材名称并选择 ZIP 文件。",
    validationStale: "素材名称或文件已变化，请重新检验。",
    confirmDelete: (name: string) => `确定删除“${name}”吗？此操作不可撤销。`,
    unsavedTitle: "素材还没有保存",
    unsavedBody: "当前页面里有未保存的素材内容。返回主界面前，你可以先保存，也可以放弃这次上传。",
    saveAndBack: "保存并返回",
    discardAndBack: "不保存返回",
    cancelBack: "取消",
    summaryTitle: "检验结果",
    normalizedName: "保存 ID",
    warnings: "警告",
    errors: "错误",
    requirementsTitle: "素材要求",
    requirements: [
      "上传格式必须是 ZIP 包。",
      "ZIP 根目录或单层目录内必须包含 animations.xml。",
      "animations.xml 必须包含 header、image、spawns、animations。",
      "image/png 字段必须是 base64 PNG 精灵表。",
      "tilesx 和 tilesy 必须大于 0，并且能整除 PNG 宽高。",
      "至少包含 1 个 spawn 和 1 个 animation。",
      "推荐包含 icon.png 和 README.md，便于预览和说明来源。",
      "文件名不能使用绝对路径、../ 路径穿越或不安全字符。",
    ],
    zipGuideTitle: "ZIP 包里的文件",
    zipGuide: [
      {
        name: "animations.xml",
        detail: "必需。宠物清单文件，定义名称、作者、精灵表、生成位置、动画序列和声音。",
      },
      {
        name: "sprite-sheet.png",
        detail: "参考文件。下载示例 ZIP 时自动导出，展示完整动作精灵表；上传时不强制要求，因为真正运行使用 XML 内的 base64 图片。",
      },
      {
        name: "icon.png",
        detail: "推荐。只用于列表、文件夹或预览的小图标，不是完整动作素材。",
      },
      {
        name: "README.md",
        detail: "推荐。写素材说明、作者、版权来源、制作备注和版本记录。",
      },
    ],
    imageGuideTitle: "自己制作图片的要求",
    imageGuide: [
      "把所有动作帧排在同一张 PNG 精灵表里，所有格子的宽高必须一致。",
      "PNG 宽度必须能被 tilesx 整除，高度必须能被 tilesy 整除。",
      "每一帧按从左到右、从上到下编号：第一帧是 0，下一帧是 1。",
      "建议使用透明背景；如果沿用旧格式色键透明，transparency 通常写 Magenta。",
      "不要只上传 icon.png。icon.png 只能放一个小图标，完整动作要放在 XML 的 image/png base64 中。",
      "图片不要过大，参考 1000×500px 左右；过大的精灵表会增加加载和保存成本。",
    ],
    xmlGuideTitle: "animations.xml 怎么写",
    xmlGuide: [
      "header：填写 author、title、petname、version、info、application 和 icon。",
      "image：填写 tilesx、tilesy、png、transparency；png 是完整精灵表转成的 base64。",
      "spawns：至少一个 spawn，定义宠物出现时的位置和进入的第一个动画。",
      "animations：至少一个 animation。每个 animation 需要 id、name、start、sequence。",
      "start/end：定义 x、y、interval、offsety、opacity 等运动参数；end 不写时默认沿用 start。",
      "sequence：用 frame 指定播放哪些帧；frame 数字对应精灵表编号。",
      "next：定义动画结束后跳到哪个 animation，可用 probability 控制概率。",
      "sounds：可选。给指定 animationid 配声音 base64、播放概率和循环次数。",
    ],
    notice: "可以下载已有 PET 作为示例 ZIP。完整动作精灵表会导出为 sprite-sheet.png。",
    errorPrefix: "加载失败",
  },
  en: {
    back: "Back",
    refresh: "Refresh",
    brand: "DuckPet Asset Manager",
    platformTag: "ZIP Asset Desk",
    title: "Pet Asset Library",
    subtitle: "Review available pets, stage your ZIP asset, and download existing pets as references.",
    listTitle: "Current PETs",
    builtin: "Built-in",
    user: "Uploaded",
    loading: "Loading assets",
    empty: "No PET assets available.",
    animations: "Animations",
    spawns: "Spawns",
    sounds: "Sounds",
    download: "Download ZIP",
    delete: "Delete",
    cannotDeleteBuiltin: "Built-in assets cannot be deleted",
    uploadTitle: "Upload Asset",
    petName: "Asset name",
    petNamePlaceholder: "e.g. my_duck_pet",
    uploadFile: "ZIP file",
    chooseFile: "Choose ZIP",
    noFile: "No file selected",
    selectedFile: "Selected",
    validate: "Validate",
    save: "Save",
    validating: "Validating asset",
    saving: "Saving asset",
    deleting: "Deleting asset",
    downloading: "Preparing download",
    loadReady: "Asset list loaded.",
    validateReady: "Asset validation passed. It can be saved.",
    validateFailed: "Asset validation failed.",
    saveReady: "Asset saved.",
    deleteReady: "Asset deleted.",
    downloadReady: "ZIP downloaded. The full animation sprite sheet is sprite-sheet.png; icon.png is only the icon.",
    chooseNameAndFile: "Enter an asset name and choose a ZIP file first.",
    validationStale: "The asset name or file changed. Validate again before saving.",
    confirmDelete: (name: string) => `Delete "${name}"? This cannot be undone.`,
    unsavedTitle: "Asset is not saved",
    unsavedBody: "There is unsaved asset work on this page. Save it before returning, or discard this upload.",
    saveAndBack: "Save and Back",
    discardAndBack: "Discard and Back",
    cancelBack: "Cancel",
    summaryTitle: "Validation Result",
    normalizedName: "Saved ID",
    warnings: "Warnings",
    errors: "Errors",
    requirementsTitle: "Asset Requirements",
    requirements: [
      "Upload format must be a ZIP archive.",
      "animations.xml must be at the ZIP root or inside one top-level folder.",
      "animations.xml must include header, image, spawns, and animations.",
      "The image/png field must contain a base64 PNG sprite sheet.",
      "tilesx and tilesy must be greater than 0 and divide the PNG dimensions.",
      "At least 1 spawn and 1 animation are required.",
      "icon.png and README.md are recommended for previews and credits.",
      "File names must not use absolute paths, ../ traversal, or unsafe characters.",
    ],
    zipGuideTitle: "Files in the ZIP",
    zipGuide: [
      {
        name: "animations.xml",
        detail: "Required. The pet manifest defining name, author, sprite sheet, spawn points, animation sequences, and sounds.",
      },
      {
        name: "sprite-sheet.png",
        detail: "Reference file. Downloaded sample ZIPs include it so the full animation sheet is visible; uploads do not require it because runtime uses the base64 image inside XML.",
      },
      {
        name: "icon.png",
        detail: "Recommended. A small list/folder/preview icon only, not the full animation asset.",
      },
      {
        name: "README.md",
        detail: "Recommended. Use it for description, author, credits, copyright source, notes, and changelog.",
      },
    ],
    imageGuideTitle: "Image Requirements",
    imageGuide: [
      "Place every animation frame in one PNG sprite sheet with equal-sized cells.",
      "PNG width must divide by tilesx, and PNG height must divide by tilesy.",
      "Frames are numbered left to right, top to bottom: first frame is 0, next is 1.",
      "Transparent background is recommended; legacy color-key transparency usually uses Magenta.",
      "Do not upload icon.png as the asset. It is only a small icon; full motion belongs in image/png base64 inside XML.",
      "Keep the sheet reasonably small, around 1000×500px as a reference, to avoid heavy load and save costs.",
    ],
    xmlGuideTitle: "How to Write animations.xml",
    xmlGuide: [
      "header: fill author, title, petname, version, info, application, and icon.",
      "image: fill tilesx, tilesy, png, and transparency; png is the full sprite sheet encoded as base64.",
      "spawns: include at least one spawn to define where the pet appears and which animation starts first.",
      "animations: include at least one animation with id, name, start, and sequence.",
      "start/end: define x, y, interval, offsety, opacity, and movement behavior; end defaults to start when omitted.",
      "sequence: add frame values to choose sprite cells; frame numbers map to sprite-sheet cell indexes.",
      "next: define which animation follows, optionally using probability.",
      "sounds: optional. Attach base64 audio, probability, and loop count to an animationid.",
    ],
    notice: "Download an existing PET as a sample ZIP. The full animation sprite sheet is exported as sprite-sheet.png.",
    errorPrefix: "Load failed",
  },
} as const;

interface PetManagerWindowProps {
  locale?: Locale;
  onBack?: () => void;
}

function PetManagerWindow({ locale = "zh", onBack }: PetManagerWindowProps) {
  const copy = COPY[locale];
  const [pets, setPets] = useState<ManageablePet[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [status, setStatus] = useState<ManagerStatus>("idle");
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string>(copy.notice);
  const [assetName, setAssetName] = useState("");
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [validation, setValidation] = useState<PetArchiveValidation | null>(null);
  const [validatedSignature, setValidatedSignature] = useState("");
  const [showUnsavedDialog, setShowUnsavedDialog] = useState(false);

  const currentSignature = assetSignature(assetName, selectedFile);
  const hasUploadInput = Boolean(assetName.trim() && selectedFile);
  const hasUnsavedWork = Boolean(assetName.trim() || selectedFile || validation);
  const isValidationCurrent =
    Boolean(validation?.valid) && validatedSignature === currentSignature;
  const isWorking = status === "working" || isLoading;

  async function loadPets() {
    setIsLoading(true);
    setError(null);

    try {
      const loadedPets = await invoke<ManageablePet[]>("list_manageable_pets");
      setPets(loadedPets);
      setStatus("ready");
      setMessage(copy.loadReady);
    } catch (loadError) {
      setPets([]);
      setError(errorToMessage(loadError));
      setStatus("error");
      setMessage(copy.errorPrefix);
    } finally {
      setIsLoading(false);
    }
  }

  useEffect(() => {
    void loadPets();
  }, []);

  function handleBack() {
    if (hasUnsavedWork) {
      setShowUnsavedDialog(true);
      return;
    }

    goBack();
  }

  function goBack() {
    if (onBack) {
      onBack();
      return;
    }

    window.history.back();
  }

  function discardAndBack() {
    setAssetName("");
    setSelectedFile(null);
    setValidation(null);
    setValidatedSignature("");
    setShowUnsavedDialog(false);
    goBack();
  }

  function updateAssetName(nextName: string) {
    setAssetName(nextName);
    clearValidation();
  }

  function updateSelectedFile(nextFile: File | null) {
    setSelectedFile(nextFile);
    clearValidation();
  }

  function clearValidation() {
    setValidation(null);
    setValidatedSignature("");
    setError(null);
    if (status !== "working") {
      setStatus("idle");
      setMessage(copy.notice);
    }
  }

  async function validateSelectedAsset() {
    if (!assetName.trim() || !selectedFile) {
      setStatus("error");
      setError(copy.chooseNameAndFile);
      setMessage(copy.chooseNameAndFile);
      return;
    }

    setStatus("working");
    setError(null);
    setMessage(copy.validating);

    try {
      const archiveBase64 = await fileToBase64(selectedFile);
      const result = await invoke<PetArchiveValidation>("validate_pet_archive", {
        name: assetName,
        archiveBase64,
      });

      setValidation(result);
      setValidatedSignature(currentSignature);

      if (result.valid) {
        setStatus("ready");
        setMessage(copy.validateReady);
      } else {
        setStatus("error");
        setError(result.errors.join("\n") || copy.validateFailed);
        setMessage(copy.validateFailed);
      }
    } catch (validateError) {
      setStatus("error");
      setError(errorToMessage(validateError));
      setMessage(copy.validateFailed);
    }
  }

  async function saveSelectedAsset(options: { returnAfterSave?: boolean } = {}) {
    if (!assetName.trim() || !selectedFile) {
      setStatus("error");
      setError(copy.chooseNameAndFile);
      setMessage(copy.chooseNameAndFile);
      return;
    }

    if (!isValidationCurrent) {
      setStatus("error");
      setError(copy.validationStale);
      setMessage(copy.validationStale);
      return;
    }

    setStatus("working");
    setError(null);
    setMessage(copy.saving);

    try {
      const archiveBase64 = await fileToBase64(selectedFile);
      await invoke<ManageablePet>("save_uploaded_pet", {
        name: assetName,
        archiveBase64,
      });

      setAssetName("");
      setSelectedFile(null);
      setValidation(null);
      setValidatedSignature("");
      setStatus("ready");
      setMessage(copy.saveReady);
      await loadPets();
      setMessage(copy.saveReady);
      if (options.returnAfterSave) {
        setShowUnsavedDialog(false);
        goBack();
      }
      return true;
    } catch (saveError) {
      setStatus("error");
      setError(errorToMessage(saveError));
      setMessage(copy.errorPrefix);
      return false;
    }
  }

  async function saveAndBack() {
    if (!isValidationCurrent) {
      setStatus("error");
      setError(copy.validationStale);
      setMessage(copy.validationStale);
      setShowUnsavedDialog(false);
      return;
    }

    await saveSelectedAsset({ returnAfterSave: true });
  }

  async function deletePet(pet: ManageablePet) {
    if (!pet.canDelete) {
      return;
    }

    if (!window.confirm(copy.confirmDelete(pet.displayName || pet.id))) {
      return;
    }

    setStatus("working");
    setError(null);
    setMessage(copy.deleting);

    try {
      await invoke<void>("delete_uploaded_pet", { petId: pet.id });
      setStatus("ready");
      setMessage(copy.deleteReady);
      await loadPets();
      setMessage(copy.deleteReady);
    } catch (deleteError) {
      setStatus("error");
      setError(errorToMessage(deleteError));
      setMessage(copy.errorPrefix);
    }
  }

  async function downloadPetArchive(pet: ManageablePet) {
    setStatus("working");
    setError(null);
    setMessage(copy.downloading);

    try {
      const archive = await invoke<ExportedPetArchive>("export_pet_archive", {
        petId: pet.id,
      });

      downloadBase64File(
        archive.archiveBase64,
        archive.fileName || `${pet.id}.zip`,
        "application/zip",
      );
      setStatus("ready");
      setMessage(copy.downloadReady);
    } catch (downloadError) {
      setStatus("error");
      setError(errorToMessage(downloadError));
      setMessage(copy.errorPrefix);
    }
  }

  return (
    <main className="settings-shell pet-manager-shell">
      {showUnsavedDialog ? (
        <div className="modal-backdrop" role="presentation">
          <section
            className="modal-card"
            role="dialog"
            aria-modal="true"
            aria-labelledby="unsaved-asset-title"
          >
            <h2 id="unsaved-asset-title">{copy.unsavedTitle}</h2>
            <p>{copy.unsavedBody}</p>
            {!isValidationCurrent ? (
              <p className="modal-hint">{copy.validationStale}</p>
            ) : null}
            <div className="modal-actions">
              <button
                type="button"
                onClick={() => void saveAndBack()}
                disabled={isWorking || !isValidationCurrent}
              >
                {copy.saveAndBack}
              </button>
              <button
                type="button"
                className="secondary-danger"
                onClick={discardAndBack}
                disabled={isWorking}
              >
                {copy.discardAndBack}
              </button>
              <button
                type="button"
                className="secondary"
                onClick={() => setShowUnsavedDialog(false)}
                disabled={isWorking}
              >
                {copy.cancelBack}
              </button>
            </div>
          </section>
        </div>
      ) : null}

      <section className="settings-header pet-manager-header">
        <div>
          <div className="brand-line">
            <span className="eyebrow">{copy.brand}</span>
            <span className="mini-tag">{copy.platformTag}</span>
          </div>
          <h1>{copy.title}</h1>
          <p className="settings-subtitle">{copy.subtitle}</p>
        </div>
        <div className="manager-header-actions">
          <button type="button" className="secondary" onClick={handleBack}>
            {copy.back}
          </button>
          <button
            type="button"
            className={isLoading ? "is-loading" : ""}
            onClick={() => void loadPets()}
            disabled={isLoading}
          >
            {copy.refresh}
          </button>
        </div>
      </section>

      <section className={`notice status-${status}`} aria-live="polite">
        <div className="notice-icon" aria-hidden="true">
          {status === "error" ? "!" : "✓"}
        </div>
        <div>
          <strong>{message}</strong>
          <span>{error ?? copy.notice}</span>
        </div>
        <span className="mini-tag">{pets.length} PET</span>
      </section>

      <section className="pet-manager-layout">
        <article className="panel asset-list-panel">
          <div className="panel-heading">
            <h2>{copy.listTitle}</h2>
            <span>{isLoading ? copy.loading : `${pets.length} PET`}</span>
          </div>

          <div className="asset-list">
            {pets.length ? (
              pets.map((pet) => (
                <article className="asset-row" key={`${pet.source}-${pet.id}`}>
                  <div className="pet-icon" aria-hidden="true" />
                  <div className="asset-row-main">
                    <div className="asset-row-title">
                      <strong>{pet.displayName || pet.id}</strong>
                      <span className={pet.source === "builtin" ? "asset-source" : "asset-source user"}>
                        {pet.source === "builtin" ? copy.builtin : copy.user}
                      </span>
                    </div>
                    <p>{pet.author || pet.id}</p>
                    <dl className="asset-metrics">
                      <div>
                        <dt>{copy.animations}</dt>
                        <dd>{pet.animationCount}</dd>
                      </div>
                      <div>
                        <dt>{copy.spawns}</dt>
                        <dd>{pet.spawnCount}</dd>
                      </div>
                      <div>
                        <dt>{copy.sounds}</dt>
                        <dd>{pet.soundCount}</dd>
                      </div>
                    </dl>
                  </div>
                  <div className="asset-actions">
                    <button
                      type="button"
                      className="secondary"
                      onClick={() => void downloadPetArchive(pet)}
                      disabled={isWorking}
                    >
                      {copy.download}
                    </button>
                    <button
                      type="button"
                      className="danger secondary-danger"
                      onClick={() => void deletePet(pet)}
                      disabled={isWorking || !pet.canDelete}
                      title={pet.canDelete ? copy.delete : copy.cannotDeleteBuiltin}
                    >
                      {copy.delete}
                    </button>
                  </div>
                </article>
              ))
            ) : (
              <div className="empty-instance-row">{isLoading ? copy.loading : copy.empty}</div>
            )}
          </div>
        </article>

        <aside className="asset-side-stack">
          <article className="panel upload-panel">
            <div className="panel-heading">
              <h2>{copy.uploadTitle}</h2>
              <span>ZIP</span>
            </div>

            <label className="field-label" htmlFor="pet-asset-name">
              {copy.petName}
            </label>
            <input
              id="pet-asset-name"
              className="text-input"
              type="text"
              value={assetName}
              placeholder={copy.petNamePlaceholder}
              onChange={(event) => updateAssetName(event.currentTarget.value)}
              disabled={isWorking}
            />

            <label className="field-label upload-label" htmlFor="pet-asset-file">
              {copy.uploadFile}
            </label>
            <label className="file-drop" htmlFor="pet-asset-file">
              <span>{copy.chooseFile}</span>
              <strong>
                {selectedFile
                  ? `${copy.selectedFile}: ${selectedFile.name}`
                  : copy.noFile}
              </strong>
            </label>
            <input
              id="pet-asset-file"
              className="visually-hidden"
              type="file"
              accept=".zip,application/zip,application/x-zip-compressed"
              onChange={(event) => updateSelectedFile(event.currentTarget.files?.[0] ?? null)}
              disabled={isWorking}
            />

            <div className="button-row">
              <button
                type="button"
                className={status === "working" ? "secondary is-loading" : "secondary"}
                onClick={() => void validateSelectedAsset()}
                disabled={isWorking || !hasUploadInput}
              >
                {copy.validate}
              </button>
              <button
                type="button"
                onClick={() => void saveSelectedAsset()}
                disabled={isWorking || !hasUploadInput || !isValidationCurrent}
              >
                {copy.save}
              </button>
            </div>

            {validation ? (
              <section
                className={`validation-box ${validation.valid ? "valid" : "invalid"}`}
                aria-live="polite"
              >
                <h3>{copy.summaryTitle}</h3>
                <dl className="runtime-list">
                  <div>
                    <dt>{copy.normalizedName}</dt>
                    <dd>{validation.normalizedId || "-"}</dd>
                  </div>
                  {validation.summary ? (
                    <>
                      <div>
                        <dt>{copy.animations}</dt>
                        <dd>{validation.summary.animationCount}</dd>
                      </div>
                      <div>
                        <dt>{copy.spawns}</dt>
                        <dd>{validation.summary.spawnCount}</dd>
                      </div>
                      <div>
                        <dt>{copy.sounds}</dt>
                        <dd>{validation.summary.soundCount}</dd>
                      </div>
                    </>
                  ) : null}
                </dl>
                {validation.warnings.length ? (
                  <div className="validation-list">
                    <strong>{copy.warnings}</strong>
                    <ul>
                      {validation.warnings.map((warning) => (
                        <li key={warning}>{warning}</li>
                      ))}
                    </ul>
                  </div>
                ) : null}
                {validation.errors.length ? (
                  <div className="validation-list">
                    <strong>{copy.errors}</strong>
                    <ul>
                      {validation.errors.map((validationError) => (
                        <li key={validationError}>{validationError}</li>
                      ))}
                    </ul>
                  </div>
                ) : null}
              </section>
            ) : null}
          </article>

          <article className="panel requirements-panel">
            <div className="panel-heading">
              <h2>{copy.requirementsTitle}</h2>
              <span>XML + PNG</span>
            </div>
            <section className="guide-section">
              <h3>{copy.zipGuideTitle}</h3>
              <dl className="file-guide-list">
                {copy.zipGuide.map((item) => (
                  <div key={item.name}>
                    <dt>{item.name}</dt>
                    <dd>{item.detail}</dd>
                  </div>
                ))}
              </dl>
            </section>
            <section className="guide-section">
              <h3>{copy.imageGuideTitle}</h3>
              <ul className="requirements-list">
                {copy.imageGuide.map((requirement) => (
                  <li key={requirement}>{requirement}</li>
                ))}
              </ul>
            </section>
            <section className="guide-section">
              <h3>{copy.xmlGuideTitle}</h3>
              <ul className="requirements-list">
                {copy.xmlGuide.map((requirement) => (
                  <li key={requirement}>{requirement}</li>
                ))}
              </ul>
            </section>
            <section className="guide-section compact">
              <h3>{copy.requirementsTitle}</h3>
              <ul className="requirements-list">
                {copy.requirements.map((requirement) => (
                  <li key={requirement}>{requirement}</li>
                ))}
              </ul>
            </section>
          </article>
        </aside>
      </section>
    </main>
  );
}

export default PetManagerWindow;

function assetSignature(assetName: string, selectedFile: File | null) {
  if (!selectedFile) {
    return `${assetName.trim()}|`;
  }

  return [
    assetName.trim(),
    selectedFile.name,
    selectedFile.size,
    selectedFile.lastModified,
  ].join("|");
}

async function fileToBase64(file: File) {
  const buffer = await file.arrayBuffer();
  const bytes = new Uint8Array(buffer);
  let binary = "";
  const chunkSize = 0x8000;

  for (let index = 0; index < bytes.length; index += chunkSize) {
    const chunk = bytes.subarray(index, index + chunkSize);
    binary += String.fromCharCode(...chunk);
  }

  return window.btoa(binary);
}

function downloadBase64File(base64: string, fileName: string, mimeType: string) {
  const binary = window.atob(base64);
  const bytes = new Uint8Array(binary.length);

  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }

  const url = URL.createObjectURL(new Blob([bytes], { type: mimeType }));
  const link = document.createElement("a");
  link.href = url;
  link.download = fileName;
  document.body.appendChild(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
}

function errorToMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
