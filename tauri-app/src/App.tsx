import PetWindow from "./windows/PetWindow";
import PetManagerWindow from "./windows/PetManagerWindow";
import SettingsWindow from "./windows/SettingsWindow";

function App() {
  const windowType = new URLSearchParams(window.location.search).get("window");

  if (windowType === "pet") {
    return <PetWindow />;
  }

  if (windowType === "pet-manager") {
    return <PetManagerWindow />;
  }

  return <SettingsWindow />;
}

export default App;
