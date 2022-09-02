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

export function resolveDevConfig(devConfig: Dev = {}): ResolvedDev {
	return {
		port: devConfig.port ?? 8080,
		static: {
			directory: devConfig.static?.directory ?? path.resolve(__dirname, "dist") // TODO:
		}
	};
}
