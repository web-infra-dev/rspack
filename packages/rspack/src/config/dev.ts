import path from "node:path";

export interface Dev {
	port?: number;
	static?: {
		directory?: string;
	};
	hmr?: boolean;
	open?: boolean;
}

export interface ResolvedDev {
	port: number;
	static: {
		directory: string;
	};
	hmr: boolean;
	open: boolean;
}

interface ResolveDevConfigContext {
	context: string;
}

export function resolveDevOptions(
	devConfig: Dev = {},
	context: ResolveDevConfigContext
): ResolvedDev {
	const port = devConfig.port ?? 8080;
	const hmr = devConfig.hmr ?? true;
	const open = devConfig.open ?? true;
	return {
		port,
		hmr,
		open,
		static: {
			directory:
				devConfig.static?.directory ?? path.resolve(context.context, "dist")
		}
	};
}
