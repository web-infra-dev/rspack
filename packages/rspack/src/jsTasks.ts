import { registerJsTaskRunner, type JsLoaderTask } from '@rspack/binding';
import type { Compiler } from './Compiler';
import { runLoaders } from './loader-runner';

let initialized = false;

export function initializeMainTaskRunner(): void {
  if (initialized) return;
  registerJsTaskRunner((compiler, task) => {
    void executeLoader(compiler as Compiler, task as JsLoaderTask);
  });
  initialized = true;
}

async function executeLoader(
  compiler: Compiler,
  task: JsLoaderTask,
): Promise<void> {
  try {
    task.complete(await runLoaders(compiler, task.takeContext()));
  } catch (error) {
    task.fail(
      error instanceof Error ? (error.stack ?? error.message) : String(error),
    );
  }
}
