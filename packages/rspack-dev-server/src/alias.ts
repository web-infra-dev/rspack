const RESOLVER_MAP: Record<string, typeof require.resolve> = {};
export const addResolveAlias = (
	name: string,
	aliasMap: Record<string, string>
) => {
	if (RESOLVER_MAP[name]) {
		throw new Error(`Should not add resolve alias to ${name} again.`);
	}
	const m = require.cache[require.resolve(name)];
	if (!m) {
		throw new Error("Failed to resolve webpack-dev-server.");
	}
	RESOLVER_MAP[name] = m.require.resolve;
	m.require.resolve = ((id: string, options?: any) =>
		aliasMap[id] ||
		RESOLVER_MAP[name]!.apply(m.require, [
			id,
			options
		])) as typeof require.resolve;
};

export const removeResolveAlias = (name: string) => {
	if (!RESOLVER_MAP[name]) {
		throw new Error(`Should add resolve alias to ${name} before removing.`);
	}
	const m = require.cache[require.resolve(name)];
	if (!m) {
		throw new Error("Failed to resolve webpack-dev-server");
	}
	if (RESOLVER_MAP[name]) {
		m.require.resolve = RESOLVER_MAP[name]!;
		delete RESOLVER_MAP[name];
	}
};
