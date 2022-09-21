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

interface ResolveDevOptionContext {
	context: string;
}

export function resolveDevOptions(
	devConfig: Dev = {},
	context: ResolveDevOptionContext
): ResolvedDev {
	return {
		port: devConfig.port ?? 8080,
		static: {
			directory:
				devConfig.static?.directory ?? path.resolve(context.context, "dist")
		}
	};
}
