import React from "react";
import { createRoot } from "react-dom/client";
import { App } from './App';
import { CountProvider } from "./CountProvider";

const container = createRoot(document.getElementById("root"));
container.render(
    <CountProvider>
        <App />
    </CountProvider>
);
