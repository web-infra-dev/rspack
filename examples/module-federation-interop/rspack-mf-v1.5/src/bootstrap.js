import { createRoot } from "react-dom/client";
import Component from "./Component"
import RemoteComponent1 from "Webpack_MF/Component"
import RemoteComponent2 from "Rspack_MF_v1/Component"

const el = document.createElement("main");
const root = createRoot(el);
root.render(
	<div>
		<h1>Host: Rspack MF v1.5</h1>
		<Component />
    <RemoteComponent1 />
    <RemoteComponent2 />
	</div>
);
document.body.appendChild(el);
