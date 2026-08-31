export function load(name) {
	return import(
		/* webpackMode: "async-weak", webpackChunkName: "unused-eager" */ `./eager/${name}.js`
	);
}
