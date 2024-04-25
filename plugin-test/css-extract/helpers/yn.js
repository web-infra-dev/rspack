function yn(value, defaultValue = false) {
	if (/^(?:y|yes|true|1|on)$/i.test(value)) {
		return true;
	}

	if (/^(?:n|no|false|0|off)$/i.test(value)) {
		return false;
	}

	return defaultValue;
}

module.exports = yn;
