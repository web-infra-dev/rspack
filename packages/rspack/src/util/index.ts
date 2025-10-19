import type { LoaderObject } from "../loader-runner";

export function isNil(value: unknown): value is null | undefined {
	return value === null || value === undefined;
}

export const toBuffer = (bufLike: string | Buffer | Uint8Array): Buffer => {
	if (Buffer.isBuffer(bufLike)) {
		return bufLike;
	}
	if (typeof bufLike === "string") {
		return Buffer.from(bufLike);
	}
	if (bufLike instanceof Uint8Array) {
		return Buffer.from(bufLike.buffer);
	}

	throw new Error("Buffer, Uint8Array or string expected");
};

export const toObject = (input: string | Buffer | object): object => {
	let s: string;
	if (Buffer.isBuffer(input)) {
		s = input.toString("utf8");
	} else if (input && typeof input === "object") {
		return input;
	} else if (typeof input === "string") {
		s = input;
	} else {
		throw new Error("Buffer or string or object expected");
	}

	return JSON.parse(s);
};

export function serializeObject(
	map: string | object | undefined | null
): Buffer | undefined {
	if (isNil(map)) {
		return undefined;
	}

	if (typeof map === "string") {
		if (map) {
			return toBuffer(map);
		}
		return undefined;
	}

	return toBuffer(JSON.stringify(map));
}

export function indent(str: string, prefix: string) {
	const rem = str.replace(/\n([^\n])/g, `\n${prefix}$1`);
	return prefix + rem;
}

export function stringifyLoaderObject(o: LoaderObject): string {
	return o.path + o.query + o.fragment;
}

export const unsupported = (name: string, issue?: string) => {
	let s = `${name} is not supported by rspack.`;
	if (issue) {
		s += ` Please refer to issue ${issue} for more information.`;
	}
	throw new Error(s);
};
