export default function getSource(name, stats) {
	const { modules } = stats.toJson({ source: true });

	for (let i = 0; i < modules.length; i++) {
		const module = modules[i];

		if (module.modules && module.modules.length > 0) {
			for (let j = 0; j < module.modules.length; j++) {
				if (module.modules[j].name === name) {
					return module.modules[j].source;
				}
			}
		} else if (module.name === name) {
			return module.source;
		}
	}

	// eslint-disable-next-line no-undefined
	return undefined;
}
