import { BrowserRouter, Routes, Route } from "react-router-dom";
import App from "./App";
import Console from "./components/Console";
import { AppStateProvider } from "./lib/state";

export default function Router() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/*" element={
          <AppStateProvider>
            <App />
          </AppStateProvider>
        } />
        <Route path="/console/*" element={
          <Console />
        } />
      </Routes>
    </BrowserRouter>
  );
}
