import binding from '@rspack/binding';

import type { Compiler } from '../Compiler';
import { runLoaders } from './index';

const compilers = new Map<number, WeakRef<Compiler>>();
const collectedCompilers = new FinalizationRegistry<number>((id) => {
  compilers.delete(id);
});
let channel: binding.JsLoaderChannel | undefined;
let activeBuilds = 0;
let receiving = false;

export function getLoaderChannel(): binding.JsLoaderChannel {
  return (channel ??= new binding.JsLoaderChannel());
}

export function registerLoaderCompiler(id: number, compiler: Compiler): void {
  compilers.set(id, new WeakRef(compiler));
  collectedCompilers.register(compiler, id, compiler);
}

export function unregisterLoaderCompiler(id: number, compiler: Compiler): void {
  compilers.delete(id);
  collectedCompilers.unregister(compiler);
}

async function runTask(task: binding.JsLoaderTask): Promise<void> {
  try {
    const compiler = compilers.get(task.compilerId)?.deref();
    if (!compiler) {
      throw new Error(
        'The Compiler has been closed or garbage collected by JavaScript.',
      );
    }
    task.reply(await runLoaders(compiler, task.takeContext()));
  } catch (error) {
    task.fail(
      error instanceof Error ? error.stack || error.message : String(error),
    );
  }
}

async function receiveTasks(): Promise<void> {
  try {
    while (activeBuilds > 0) {
      const task = await getLoaderChannel().receive();
      if (task) {
        // Keep receiving while a loader awaits importModule, which can enqueue more loaders.
        void runTask(task);
      }
    }
  } finally {
    receiving = false;
  }
}

export function runWithLoaderTasks(
  run: (callback: (error: Error | null) => void) => void,
  callback: (error: Error | null) => void,
): void {
  activeBuilds++;
  if (!receiving) {
    receiving = true;
    void receiveTasks();
  }
  let active = true;
  const finish = () => {
    if (active) {
      active = false;
      if (--activeBuilds === 0) {
        // Resolve the outstanding receive so an idle compiler does not keep Node alive.
        getLoaderChannel().wake();
      }
    }
  };
  try {
    run((error) => {
      finish();
      callback(error);
    });
  } catch (error) {
    finish();
    throw error;
  }
}
