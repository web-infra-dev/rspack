import { object as dynamicModules } from "./dynamic-module-layer"
import { object as direct, external1 as entryLayerExternal1, external2 as entryLayerExternal2, __loaderValue as entryLayerValue } from "./module";
import { direct as layerDirect, external1 as layerExternal1, external2 as layerExternal2, reexported as layerReexported, __loaderValue as layerValue } from "./module-layer-change";
import { direct as otherLayerDirect, reexported as otherLayerReexported, __loaderValue as otherLayerValue } from "./module-other-layer-change";
import { object as reexported } from "./reexport";

it("should allow to duplicate modules with layers", () => {
	expect(direct).toBe(reexported);
	expect(layerDirect).toBe(layerReexported);
	expect(otherLayerDirect).toBe(otherLayerReexported);

	expect(layerDirect).not.toBe(direct);
	expect(otherLayerDirect).not.toBe(direct);
	expect(otherLayerDirect).not.toBe(layerDirect);
});

it("apply rules based on layer", () => {
	expect(layerValue).toBe("ok");
	expect(otherLayerValue).toBe("other");
	expect(entryLayerValue).toBe("entry");
});

it("apply externals based on layer", () => {
	expect(entryLayerExternal1).toBe(42);
	expect(entryLayerExternal2).toBe(42);
	expect(layerExternal1).toBe(43);
	expect(layerExternal2).toBe(43);
});

it("apply layer for dynamic imports with dynamic resources", async () => {
	const mods = await Promise.all(dynamicModules.modules)
	expect(dynamicModules.layer).toBe('dynamic-layer')
	expect(mods[0]).toMatchObject({ layer: 'dynamic-layer', name: 'module1' })
	expect(mods[1]).toMatchObject({ layer: 'dynamic-layer', name: 'module2' })
})
