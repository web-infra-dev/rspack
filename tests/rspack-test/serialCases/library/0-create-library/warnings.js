const warnings = [];
for (let i = 0; i < 12; i++) {
	warnings.push(/Module not found: Can't resolve 'external'/);
	warnings.push(/Module not found: Can't resolve 'external-named'/);
}
module.exports = warnings;
