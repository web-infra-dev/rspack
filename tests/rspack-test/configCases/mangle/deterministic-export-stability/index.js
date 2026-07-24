import {
	moduleId as fourteenModuleId,
	stableExport00 as fourteen00,
	stableExport01 as fourteen01,
	stableExport02 as fourteen02,
	stableExport03 as fourteen03,
	stableExport04 as fourteen04,
	stableExport05 as fourteen05,
	stableExport06 as fourteen06,
	stableExport07 as fourteen07,
	stableExport08 as fourteen08,
	stableExport09 as fourteen09,
	stableExport10 as fourteen10,
	stableExport11 as fourteen11,
	stableExport12 as fourteen12,
	stableExport13 as fourteen13
} from "./fourteen";
import {
	moduleId as fifteenModuleId,
	stableExport00 as fifteen00,
	stableExport01 as fifteen01,
	stableExport02 as fifteen02,
	stableExport03 as fifteen03,
	stableExport04 as fifteen04,
	stableExport05 as fifteen05,
	stableExport06 as fifteen06,
	stableExport07 as fifteen07,
	stableExport08 as fifteen08,
	stableExport09 as fifteen09,
	stableExport10 as fifteen10,
	stableExport11 as fifteen11,
	stableExport12 as fifteen12,
	stableExport13 as fifteen13,
	zzAddedExport
} from "./fifteen";

const fourteenValues = [
	fourteen00,
	fourteen01,
	fourteen02,
	fourteen03,
	fourteen04,
	fourteen05,
	fourteen06,
	fourteen07,
	fourteen08,
	fourteen09,
	fourteen10,
	fourteen11,
	fourteen12,
	fourteen13
];
const fifteenValues = [
	fifteen00,
	fifteen01,
	fifteen02,
	fifteen03,
	fifteen04,
	fifteen05,
	fifteen06,
	fifteen07,
	fifteen08,
	fifteen09,
	fifteen10,
	fifteen11,
	fifteen12,
	fifteen13,
	zzAddedExport
];

const getMangledNames = (moduleId, originalValues) =>
	Object.fromEntries(
		Object.entries(require.cache[moduleId].exports)
			.filter(([, value]) => originalValues.includes(value))
			.map(([mangledName, originalName]) => [originalName, mangledName])
	);

it("should preserve deterministic export names when adding an export", () => {
	const fourteenNames = getMangledNames(fourteenModuleId, fourteenValues);
	const fifteenNames = getMangledNames(fifteenModuleId, fifteenValues);

	for (const originalName of Object.keys(fourteenNames)) {
		expect(fifteenNames[originalName]).toBe(fourteenNames[originalName]);
	}
});
