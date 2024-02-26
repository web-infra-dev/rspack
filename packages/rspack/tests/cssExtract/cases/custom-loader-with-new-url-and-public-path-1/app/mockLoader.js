export default function loader() {
	const callback = this.async();

	callback(
		null,
		`export default [
      [module.id, ".foo {background: url(" + new URL("./img.png", import.meta.url) + ")}", ""],
      [module.id, ".bar {background: url(" + new URL("../outer-img.png", import.meta.url) + ")}", ""],
      [module.id, ".baz {background: url(" + new URL("./nested/nested-img.png", import.meta.url) + ")}", ""]
  ]`
	);
}
