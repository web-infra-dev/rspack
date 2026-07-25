const selected = require(
	globalThis.__USE_EXTERNAL_OS__ ? "external-os" : "./local.cjs"
);
const mixed = require(
	globalThis.__USE_EXTERNAL_OS__
		? "external-os"
		: "./" + globalThis.__LOCAL_MODULE__
);

export const platform = selected.platform();
export const mixedPlatform = mixed.platform;
