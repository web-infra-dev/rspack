import { createRoot } from "react-dom/client";
import Component from "./Component"
import RemoteComponent1 from "Rspack_MF_v1/Component"
import RemoteComponent2 from "Rspack_MF_v1_5/Component"

const el = document.createElement("main");
const root = createRoot(el);
root.render(
	<div>
		<h1>Host: Webpack MF</h1>
		<Component />
    <RemoteComponent1 />
    <RemoteComponent2 />
	</div>
);
document.body.appendChild(el);
