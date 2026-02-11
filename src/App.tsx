import { BrowserRouter, Routes, Route } from "react-router-dom";
import { Layout } from "./components/Layout";
import { Landing } from "./pages/Landing";
import { Simulation } from "./pages/Simulation";
import { Training } from "./pages/Training";
import { Settings } from "./pages/Settings";
import { GameProvider } from "./context/GameContext";

function App() {
  return (
    <BrowserRouter>
      <GameProvider>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Landing />} />
          <Route path="simulation" element={<Simulation />} />
          <Route path="training" element={<Training />} />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
      </GameProvider>
    </BrowserRouter>
  );
}

export default App;
