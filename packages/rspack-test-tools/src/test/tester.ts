import fs from 'fs-extra';
import type {
  ITestContext,
  ITestEnv,
  ITester,
  ITesterConfig,
  ITestProcessor,
} from '../type';
import { TestContext } from './context';
import { generateDebugReport } from './debug';

export class Tester implements ITester {
  private context: ITestContext;
  private steps: ITestProcessor[] = [];
  step = 0;
  total = 0;

  constructor(private config: ITesterConfig) {
    this.context = config.createContext
      ? config.createContext(config)
      : new TestContext(config);
    this.steps = config.steps || [];
    this.step = 0;
    this.total = config.steps?.length || 0;
    if (config.contextValue) {
      for (const [key, value] of Array.from(
        Object.entries(config.contextValue),
      )) {
        this.context.setValue(key, value);
      }
    }
  }
  getContext(): ITestContext {
    return this.context;
  }
  async prepare() {
    fs.mkdirSync(this.context.getDist(), { recursive: true });
    const tempDir = this.context.getTemp();
    if (tempDir) {
      fs.mkdirSync(tempDir, { recursive: true });
    }
    for (const i of this.steps) {
      if (typeof i.beforeAll === 'function') {
        await i.beforeAll(this.context);
      }
    }
  }
  async compile() {
    const currentStep = this.steps[this.step];
    if (!currentStep) return;

    await this.runStepMethods(currentStep, [
      'before',
      'config',
      'compiler',
      'build',
    ]);
  }
  async check(env: ITestEnv) {
    const currentStep = this.steps[this.step];
    if (!currentStep) return;

    await this.runCheckStepMethods(
      currentStep,
      env,
      this.context.hasError() ? ['check'] : ['run', 'check'],
    );
  }

  async after() {
    const currentStep = this.steps[this.step];
    if (!currentStep) return;
    await this.runStepMethods(currentStep, ['after'], true);
  }

  next() {
    if (this.context.hasError()) {
      return false;
    }
    if (this.steps[this.step + 1]) {
      this.step++;
      return true;
    }
    return false;
  }

  async resume() {
    for (const i of this.steps) {
      if (typeof i.afterAll === 'function') {
        await i.afterAll(this.context);
      }
    }
    try {
      await this.context.closeCompiler();
    } catch (e: any) {
      console.warn(
        `Error occured while closing compilers of '${this.config.name}':\n${e.stack}`,
      );
    }
    if (process.env.WASM) {
      Tester.logWasmMemory(this.config.name);
    }
    if (__DEBUG__) {
      try {
        generateDebugReport(this.context);
      } catch (e) {
        console.warn(`Generate debug report failed: ${(e as Error).message}`);
      }
    }
  }

  private static wasmBinding: any;
  private static logWasmMemory(testName: string) {
    try {
      if (!Tester.wasmBinding) {
        Tester.wasmBinding = require('@rspack/binding');
      }
      const binding = Tester.wasmBinding;
      const parts: string[] = [];
      if (binding.__sharedMemory) {
        const wasmMB = binding.__sharedMemory.buffer.byteLength / 1048576;
        parts.push(`wasm_linear=${wasmMB.toFixed(0)}MB`);
      }
      if (binding.wasmAllocStats) {
        const [alloc, dealloc, peak] = binding.wasmAllocStats();
        const liveMB = Number(alloc - dealloc) / 1048576;
        const peakMB = Number(peak) / 1048576;
        parts.push(`rust_live=${liveMB.toFixed(0)}MB`);
        parts.push(`rust_peak=${peakMB.toFixed(0)}MB`);
      }
      if (parts.length > 0) {
        console.log(`[WASM-MEM] ${testName} | ${parts.join(' | ')}`);
      }
    } catch (_) {
      // ignore if binding not available
    }
  }

  private async runStepMethods(
    step: ITestProcessor,
    methods: Array<'before' | 'config' | 'compiler' | 'build' | 'after'>,
    force = false,
  ) {
    for (const i of methods) {
      if (!force && this.context.hasError()) return;
      if (typeof step[i] === 'function') {
        try {
          await step[i]!(this.context);
        } catch (e) {
          this.context.emitError(e as Error);
        }
      }
    }
  }

  private async runCheckStepMethods(
    step: ITestProcessor,
    env: ITestEnv,
    methods: Array<'run' | 'check'>,
  ) {
    try {
      for (const i of methods) {
        if (typeof step[i] === 'function') {
          await step[i]!(env, this.context);
        }
      }
    } catch (e) {
      const errors = this.context.getError();
      console.error(
        new Error([...errors, e].map((e) => (e as Error).message).join('\n')),
      );
      throw e;
    }
  }
}
