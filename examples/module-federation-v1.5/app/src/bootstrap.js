import { createRoot } from "react-dom/client"; // <- this is a shared module, but used as usual
import App from "./App";

// load app
const el = document.createElement("main");
const root = createRoot(el);
root.render(<App />);
document.body.appendChild(el);
