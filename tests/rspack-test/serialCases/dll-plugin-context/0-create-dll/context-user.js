export function loadWidget(name) {
	return import(`./lazy/${name}.js`);
}
