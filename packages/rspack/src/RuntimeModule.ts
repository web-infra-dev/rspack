import type {
  JsAddingRuntimeModule,
  JsRuntimeModule,
  JsSource,
} from '@rspack/binding';
import type { Chunk } from './Chunk';
import type { ChunkGraph } from './ChunkGraph';
import type { Compilation } from './Compilation';

export enum RuntimeModuleStage {
  NORMAL = 0,
  BASIC = 5,
  ATTACH = 10,
  TRIGGER = 20,
}

export class RuntimeModule {
  static STAGE_NORMAL = RuntimeModuleStage.NORMAL;
  static STAGE_BASIC = RuntimeModuleStage.BASIC;
  static STAGE_ATTACH = RuntimeModuleStage.ATTACH;
  static STAGE_TRIGGER = RuntimeModuleStage.TRIGGER;

  static __to_binding(module: RuntimeModule): JsAddingRuntimeModule {
    return {
      name: module.name,
      stage: module.stage,
      generator: module.generate.bind(module),
      fullHash: module.fullHash,
      dependentHash: module.dependentHash,
      isolate: module.shouldIsolate(),
    };
  }

  private _name: string;
  private _stage: RuntimeModuleStage;
  public fullHash = false;
  public dependentHash = false;
  protected chunk: Chunk | null = null;
  protected compilation: Compilation | null = null;
  protected chunkGraph: ChunkGraph | null = null;
  constructor(name: string, stage = RuntimeModuleStage.NORMAL) {
    this._name = name;
    this._stage = stage;
  }

  attach(compilation: Compilation, chunk: Chunk, chunkGraph: ChunkGraph) {
    this.compilation = compilation;
    this.chunk = chunk;
    this.chunkGraph = chunkGraph;
  }

  get source(): JsSource | undefined {
    return undefined;
  }

  get name(): string {
    return this._name;
  }

  get stage(): RuntimeModuleStage {
    return this._stage;
  }

  identifier() {
    return `webpack/runtime/${this._name}`;
  }

  readableIdentifier() {
    return `webpack/runtime/${this._name}`;
  }

  shouldIsolate(): boolean {
    return true;
  }

  generate(): string {
    throw new Error(
      `Should implement "generate" method of runtime module "${this.name}"`,
    );
  }
}

export function createRenderedRuntimeModule(
  module: JsRuntimeModule,
): RuntimeModule {
  const RuntimeModuleClass = {
    [module.constructorName]: class extends RuntimeModule {
      private _source: JsSource | undefined;
      constructor() {
        super(module.name, module.stage);
        this._source = module.source;
      }

      /**
       * @deprecated use `module.constructor.name` instead
       */
      get constructorName() {
        return module.constructorName;
      }

      /**
       * @deprecated use `module.identifier()` instead
       */
      get moduleIdentifier() {
        return module.moduleIdentifier;
      }

      get source(): JsSource | undefined {
        return this._source;
      }

      identifier(): string {
        return module.moduleIdentifier;
      }

      readableIdentifier(): string {
        return module.moduleIdentifier;
      }

      shouldIsolate(): boolean {
        return module.isolate;
      }

      generate(): string {
        return this._source?.source.toString('utf-8') || '';
      }
    },
  }[module.constructorName];
  return new RuntimeModuleClass();
}
