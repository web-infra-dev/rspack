import { createRoot } from "react-dom/client";
import Component from "./Component"
import { de } from "date-fns/locale";

const el = document.createElement("main");
const root = createRoot(el);
root.render(
	<div>
		<h1>Lib 1</h1>
		<Component locale={de} />
	</div>
);
document.body.appendChild(el);
