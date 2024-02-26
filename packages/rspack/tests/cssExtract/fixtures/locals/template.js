import css from "./index.css";

let html = "\n";

for (const key in css) {
	html += `css.${key}: ${css[key]}\n`;
}

export default html;
