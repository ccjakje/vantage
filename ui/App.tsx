import { Routes, Route } from "react-router-dom";
import Titlebar from "./components/Titlebar";
import Home from "./pages/Home";
import MatchView from "./pages/MatchView";
import Settings from "./pages/Settings";

export default function App() {
  return (
    <div className="flex flex-col h-screen bg-[var(--bg-primary)] rounded-lg overflow-hidden border border-[var(--border)]">
      <Titlebar />
      <main className="flex-1 overflow-y-auto">
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/match" element={<MatchView />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </main>
    </div>
  );
}
