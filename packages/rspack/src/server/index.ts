import * as binding from '@rspack/binding';
import type { ExternalObject, RspackInternal, RawModuleRule } from '@rspack/binding';

import * as Config from '../config';
import type { RspackOptions } from '../config';

interface ModuleRule {
  test?: string;
  uses?: ((this: LoaderContext, loaderContext: LoaderContext) => Promise<LoaderResult | void> | LoaderResult | void)[];
  type?: RawModuleRule['type'];
}

interface LoaderRunnerContext {
  loaders: ModuleRule['uses'];
}

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
  extends Pick<LoaderContextInternal, 'resource' | 'resourcePath' | 'resourceQuery' | 'resourceFragment'> {
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
  } else if (typeof bufLike === 'string') {
    return Buffer.from(bufLike);
  }

  throw new Error('Buffer or string expected');
};

function createRspackModuleRuleAdapter(context: LoaderRunnerContext): (err: any, data: Buffer) => Promise<Buffer> {
  const { loaders } = context;

  return async function (err: any, data: Buffer): Promise<Buffer> {
    if (err) {
      throw err;
    }

    const loaderThreadsafeContext: LoaderThreadsafeContext = JSON.parse(data.toString('utf-8'));

    const { p: payload, id } = loaderThreadsafeContext;

    const loaderContextInternal: LoaderContextInternal = {
      source: payload.source,
      resourcePath: payload.resourcePath,
      resourceQuery: payload.resourceQuery,
      resource: payload.resource,
      resourceFragment: payload.resourceFragment,
    };

    let sourceBuffer = Buffer.from(loaderContextInternal.source);

    // Loader is executed from right to left
    for (const loader of ([...loaders] || []).reverse()) {
      const loaderContext = {
        ...loaderContextInternal,
        source: {
          getCode(): string {
            return sourceBuffer.toString('utf-8');
          },
          getBuffer(): Buffer {
            return sourceBuffer;
          },
        },
      };

      let loaderResult: LoaderResult;
      if ((loaderResult = await Promise.resolve().then(() => loader.apply(loaderContext, [loaderContext])))) {
        const content = loaderResult.content;
        sourceBuffer = toBuffer(content);
      }
    }

    const loaderResultPayload: LoaderResultInternal = {
      content: [...sourceBuffer],
    };

    const loaderThreadsafeResult: LoaderThreadsafeResult = {
      id: id,
      p: loaderResultPayload,
    };
    return Buffer.from(JSON.stringify(loaderThreadsafeResult), 'utf-8');
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

export { Rspack, createRspackModuleRuleAdapter };
export type { ModuleRule, LoaderRunnerContext };
export default Rspack;
