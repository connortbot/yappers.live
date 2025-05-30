import ReactDOM from "react-dom/client";
import { BrowserRouter, Routes, Route } from "react-router";
import { GameProvider } from "./context/GameContext";
import "./globals.css";
import Home from "./Home";
import Lobby from "./pages/Lobby";

const root = document.getElementById("root");
ReactDOM.createRoot(root!).render(
  <GameProvider>
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/lobby/:mode" element={<Lobby />} />
      </Routes>
    </BrowserRouter>
  </GameProvider>
);
