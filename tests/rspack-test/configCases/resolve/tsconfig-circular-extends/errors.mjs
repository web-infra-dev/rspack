export default [
	/Tsconfig extends configs circularly: .*self\.json -> .*self\.json/,
	/Tsconfig extends configs circularly: .*a\.json -> .*b\.json -> .*a\.json/,
	/Tsconfig extends configs circularly: .*array\.json -> .*a\.json -> .*b\.json -> .*a\.json/,
	/Tsconfig extends configs circularly: .*a\.json -> .*b\.json -> .*a\.json/
];
