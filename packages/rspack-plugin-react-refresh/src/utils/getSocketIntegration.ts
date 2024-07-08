export type IntegrationType = "wds";

export default function getSocketIntegration(integrationType: IntegrationType) {
	let resolvedSocketIntegration;
	switch (integrationType) {
		case "wds": {
			resolvedSocketIntegration = require.resolve("../sockets/WDSSocket");
			break;
		}
		default: {
			resolvedSocketIntegration = require.resolve(integrationType);
			break;
		}
	}

	return resolvedSocketIntegration;
}
