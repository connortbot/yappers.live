import ReactDOM from "react-dom/client";
import { BrowserRouter, Routes, Route } from "react-router";
import { GameProvider } from "./context/GameContext";
import { AnimatedBackground } from "./components/AnimatedBackground";
import "./globals.css";
import Home from "./Home";
import Lobby from "./pages/Lobby";
import Admin from "./pages/Admin";
import TeamDraft from "./pages/TeamDraft";

const root = document.getElementById("root");
ReactDOM.createRoot(root!).render(
  <GameProvider>
    <AnimatedBackground />
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/lobby/:mode" element={<Lobby />} />
        <Route path="/admin" element={<Admin />} />
        <Route path="/team_draft" element={<TeamDraft />} />
      </Routes>
    </BrowserRouter>
  </GameProvider>
);
