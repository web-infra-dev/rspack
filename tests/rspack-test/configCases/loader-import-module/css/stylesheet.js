import { red } from "./colors.js";
export default `body { background: url("${
	new URL("./file.png", import.meta.url).href
}"); color: ${red}; }`;
