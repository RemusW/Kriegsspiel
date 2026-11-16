import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { Menu } from "./interface/menu";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Menu />
    <App />
  </React.StrictMode>,
);
