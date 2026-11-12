import path from "node:path";

export default async function loader() {
	const callback = this.async()
	const result = await this.importModule(path.resolve(import.meta.dirname, './execute-module.js'))
	callback(null, `export default ${result.default}`)
}
