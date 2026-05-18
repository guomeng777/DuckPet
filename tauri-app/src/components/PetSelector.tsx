import type { AvailablePet, PetManifestSummary } from "../types/pet";

interface PetSelectorProps {
  pets: AvailablePet[];
  selectedXmlPath: string;
  manifest: PetManifestSummary | null;
  isLoading: boolean;
  error: string | null;
  onSelect: (xmlPath: string) => void;
  copy: {
    title: string;
    found: string;
    pet: string;
    loadingPets: string;
    noPetSelected: string;
    chooseManifest: string;
    xmlPath: string;
    version: string;
    author: string;
    animations: string;
    sounds: string;
    pending: string;
  };
}

function PetSelector({
  pets,
  selectedXmlPath,
  manifest,
  isLoading,
  error,
  onSelect,
  copy,
}: PetSelectorProps) {
  return (
    <article className="panel pet-selector-panel">
      <div className="panel-heading">
        <h2>{copy.title}</h2>
        <span>{pets.length} {copy.found}</span>
      </div>

      <label className="field-label" htmlFor="pet-select">
        {copy.pet}
      </label>
      <select
        id="pet-select"
        className="pet-select"
        value={selectedXmlPath}
        onChange={(event) => onSelect(event.currentTarget.value)}
        disabled={isLoading || pets.length === 0}
      >
        {pets.map((pet) => (
          <option key={pet.xmlPath} value={pet.xmlPath}>
            {pet.header.petname || pet.id}
          </option>
        ))}
      </select>

      {manifest ? (
        <div className="pet-row selected-pet-row">
          <img className="pet-icon-image" src={manifest.spriteSheet.dataUrl} alt="" />
          <div>
            <strong>{manifest.header.petname}</strong>
            <span>{manifest.header.title}</span>
          </div>
        </div>
      ) : (
        <div className="pet-row selected-pet-row">
          <div className="pet-icon" aria-hidden="true" />
          <div>
            <strong>{isLoading ? copy.loadingPets : copy.noPetSelected}</strong>
            <span>{error ?? copy.chooseManifest}</span>
          </div>
        </div>
      )}

      <dl className="runtime-list metadata-list">
        <div>
          <dt>{copy.xmlPath}</dt>
          <dd>{manifest?.sourcePath ?? (selectedXmlPath || copy.pending)}</dd>
        </div>
        <div>
          <dt>{copy.version}</dt>
          <dd>{manifest?.header.version ?? copy.pending}</dd>
        </div>
        <div>
          <dt>{copy.author}</dt>
          <dd>{manifest?.header.author ?? copy.pending}</dd>
        </div>
        <div>
          <dt>{copy.animations}</dt>
          <dd>{manifest?.animationCount ?? copy.pending}</dd>
        </div>
        <div>
          <dt>{copy.sounds}</dt>
          <dd>{manifest?.soundCount ?? copy.pending}</dd>
        </div>
      </dl>

      {error ? <p className="error-text">{error}</p> : null}
    </article>
  );
}

export default PetSelector;
