it('preserves the public path and version escapes', () => {
	expect(__webpack_public_path__).toBe('/assets/nested/');
	expect(__rspack_version__).toBe('release\nbuild');
});
