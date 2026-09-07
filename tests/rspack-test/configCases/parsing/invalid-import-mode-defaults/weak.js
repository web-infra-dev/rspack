import existing from "./weak/a.js";

export { existing };

export function load(name) {
	return import(
		/* webpackMode: true, webpackChunkName: "unused-weak" */ `./weak/${name}.js`
	);
}
