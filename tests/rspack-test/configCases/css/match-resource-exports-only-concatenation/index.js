import * as appStyles from "./App.vue.module.css!=!./identity-loader.mjs!./App.vue";
import * as noIdStyles from "./AppNoId.vue.no-id.module.css!=!./identity-loader.mjs!./AppNoId.vue";
import * as plainStyles from "./plain.css";
import * as literalGlobalStyles from "./literal.global.css";
import * as scopedGlobalStyles from "./scoped.global.css";

it("should concatenate css/auto matchResource modules with exportsOnly and esModule", () => {
	expect(typeof appStyles.title).toBe("string");
	expect(appStyles.title).not.toBe("title");
	expect(appStyles.title.endsWith("-title")).toBe(true);
	expect(appStyles.title).not.toContain("__RSPACK_PLUGIN_CSS_MODULE_ID__");
	expect(appStyles.custom).toBe("value");

	expect(noIdStyles.card).toMatch(/^card-[a-zA-Z0-9_-]{6}$/);

	expect(Object.keys(plainStyles)).toEqual([]);
	expect(literalGlobalStyles.exportedColor).toBe("red");

	expect(typeof scopedGlobalStyles.badge).toBe("string");
	expect(scopedGlobalStyles.badge).not.toBe("badge");
	expect(scopedGlobalStyles.badge.endsWith("-badge")).toBe(true);
	expect(scopedGlobalStyles.badge).not.toContain("__RSPACK_PLUGIN_CSS_MODULE_ID__");
});

it("should concatenate all imported CSS modules into a single module", () => {
	const concatModules = __STATS__.modules.filter((m) => m.modules);
	expect(concatModules.length).toBe(1);
	const nestedNames = concatModules[0].modules.map((m) => m.name);
	expect(nestedNames.some((name) => name.includes("App.vue"))).toBe(true);
	expect(nestedNames.some((name) => name.includes("AppNoId.vue"))).toBe(true);
	expect(nestedNames.some((name) => name.includes("plain.css"))).toBe(true);
	expect(nestedNames.some((name) => name.includes("literal.global.css"))).toBe(true);
	expect(nestedNames.some((name) => name.includes("scoped.global.css"))).toBe(true);
});
