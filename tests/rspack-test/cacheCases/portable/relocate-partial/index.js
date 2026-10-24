// index.js runs through the harness loader, which lives outside the project
// root, so it and the modules it issues are bound. The data modules they
// import are issued by data modules and stay unbound, except the one that
// inlines WRITER_ROOT.
import kept from 'data:text/javascript,export { default } from "data:text/javascript,export default 1"';
import load from 'data:text/javascript,export { default } from "data:text/javascript,export default () => import(`data:text/javascript,export default WRITER_ROOT`)"';

it("should keep unbound modules and rebuild owners of dropped targets after relocation", async () => {
	expect(kept).toBe(1);
	expect(typeof (await load()).default).toBe("string");
	if (COMPILER_INDEX === 0) {
		const stats = await NEXT_START(false);
		expect(stats.modules.filter(module => module.built)).toHaveLength(0);
	}
	if (COMPILER_INDEX === 1) {
		const stats = await NEXT_MOVE_DIR_START();
		const built = name => stats.modules.find(module => module.name.endsWith(name)).built;
		expect(built(",export default 1")).toBe(false);
		expect(built(",export default WRITER_ROOT")).toBe(true);
		expect(built("export default WRITER_ROOT`)")).toBe(true);
		expect(built("/index.js")).toBe(true);
	}
});
