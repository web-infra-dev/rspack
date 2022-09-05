import path from "node:path";

export interface Dev {
	port?: number;
	static?: {
		directory?: string;
	};
}

export interface ResolvedDev {
	port: number;
	static: {
		directory: string;
	};
}

interface ResolveDevConfigContext {
	context: string;
}

export function resolveDevOptions(
	devConfig: Dev = {},
	context: ResolveDevConfigContext
): ResolvedDev {
	return {
		port: devConfig.port ?? 8080,
		static: {
			directory:
				devConfig.static?.directory ?? path.resolve(context.context, "dist")
		}
	};
}
