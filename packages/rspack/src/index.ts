export * from "./build";

import * as binding from "@rspack/binding";
import type {
	ExternalObject,
	RspackInternal,
	RawModuleRuleUse,
	RawModuleRule
} from "@rspack/binding";

import assert from "node:assert";
import * as Config from "./config";
import type { RspackOptions } from "./config";

interface ModuleRule {
	test?: RawModuleRule["test"];
	resource?: RawModuleRule["resource"];
	resourceQuery?: RawModuleRule["resourceQuery"];
	uses?: ModuleRuleUse[];
	type?: RawModuleRule["type"];
}

type ModuleRuleUse =
	| {
			builtinLoader: BuiltinLoader;
			options?: unknown;
	  }
	| {
			loader: JsLoader;
			options?: unknown;
	  };

interface JsLoader {
  (
    this: LoaderContext,
    loaderContext: LoaderContext
  ): Promise<LoaderResult | void> | LoaderResult | void
  displayName?: string
}

type BuiltinLoader = string;

interface LoaderThreadsafeContext {
	id: number;
	p: LoaderContextInternal;
}

interface LoaderContextInternal {
	// TODO: It's not a good way to do this, we should split the `source` into a separate type and avoid using `serde_json`, but it's a temporary solution.
	source: number[];
	resource: String;
	resourcePath: String;
	resourceQuery: String | null;
	resourceFragment: String | null;
}

interface LoaderContext
	extends Pick<
		LoaderContextInternal,
		"resource" | "resourcePath" | "resourceQuery" | "resourceFragment"
	> {
	source: {
		getCode(): string;
		getBuffer(): Buffer;
	};
}

interface LoaderResultInternal {
	content: number[];
}

interface LoaderResult {
	content: Buffer | string;
}

interface LoaderThreadsafeResult {
	id: number;
	p: LoaderResultInternal | null | undefined;
}

const toBuffer = (bufLike: string | Buffer): Buffer => {
	if (Buffer.isBuffer(bufLike)) {
		return bufLike;
	} else if (typeof bufLike === "string") {
		return Buffer.from(bufLike);
	}

	throw new Error("Buffer or string expected");
};

function createRawModuleRuleUses(uses: ModuleRuleUse[]): RawModuleRuleUse[] {
	return createRawModuleRuleUsesImpl([...uses].reverse());
}

function createRawModuleRuleUsesImpl(
	uses: ModuleRuleUse[]
): RawModuleRuleUse[] {
	const index = uses.findIndex(use => "builtinLoader" in use);
	if (index < 0) {
		return [composeJsUse(uses)];
	}

	const before = uses.slice(0, index);
	const after = uses.slice(index + 1);
	return [
		composeJsUse(before),
		createNativeUse(uses[index]),
		...createRawModuleRuleUsesImpl(after)
	];
}

function createNativeUse(use: ModuleRuleUse): RawModuleRuleUse {
	assert("builtinLoader" in use);

	if (use.builtinLoader === "sass-loader") {
		(use.options ??= {} as any).__exePath = require.resolve(
			`@tmp-sass-embedded/${process.platform}-${
				process.arch
			}/dart-sass-embedded/dart-sass-embedded${
				process.platform === "win32" ? ".bat" : ""
			}`
		);
	}

	return {
		builtinLoader: use.builtinLoader,
		options: JSON.stringify(use.options)
	};
}

function composeJsUse(uses: ModuleRuleUse[]): RawModuleRuleUse {
  async function loader(err: any, data: Buffer): Promise<Buffer> {
    if (err) {
      throw err;
    }

    const loaderThreadsafeContext: LoaderThreadsafeContext = JSON.parse(
      data.toString("utf-8")
    );

    const { p: payload, id } = loaderThreadsafeContext;

    const loaderContextInternal: LoaderContextInternal = {
      source: payload.source,
      resourcePath: payload.resourcePath,
      resourceQuery: payload.resourceQuery,
      resource: payload.resource,
      resourceFragment: payload.resourceFragment
    };

    let sourceBuffer = Buffer.from(loaderContextInternal.source);

    // Loader is executed from right to left
    for (const use of uses) {
      assert("loader" in use);
      const loaderContext = {
        ...loaderContextInternal,
        source: {
          getCode(): string {
            return sourceBuffer.toString("utf-8");
          },
          getBuffer(): Buffer {
            return sourceBuffer;
          }
        },
        getOptions() {
          return use.options;
        }
      };

      let loaderResult: LoaderResult;
      if (
        (loaderResult = await Promise.resolve().then(() =>
          use.loader.apply(loaderContext, [loaderContext])
        ))
      ) {
        const content = loaderResult.content;
        sourceBuffer = toBuffer(content);
      }
    }

    const loaderResultPayload: LoaderResultInternal = {
      content: [...sourceBuffer]
    };

    const loaderThreadsafeResult: LoaderThreadsafeResult = {
      id: id,
      p: loaderResultPayload
    };
    return Buffer.from(JSON.stringify(loaderThreadsafeResult), "utf-8");
  };
  loader.displayName = uses.map((item) => {
    assert('loader' in item);
    return item.loader.displayName || item.loader.name || "unknown-loader"
  }).join(" -> ");
	return {
		loader,
	};
}

class Rspack {
	#instance: ExternalObject<RspackInternal>;

	constructor(public options: RspackOptions) {
		const nativeConfig = Config.User2Native(options);
		this.#instance = binding.newRspack(nativeConfig);
	}

	async build() {
		const stats = await binding.build(this.#instance);
		return stats;
	}

	async rebuild() {
		const stats = await binding.rebuild(this.#instance);
		return stats;
	}
}

export { Rspack, createRawModuleRuleUses };
export type { ModuleRule };
export default Rspack;
