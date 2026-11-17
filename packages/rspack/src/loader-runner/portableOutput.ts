import type { AsyncLocalStorage } from 'node:async_hooks';
import { createRequire } from 'node:module';
import path from 'node:path';

interface Invocation {
  owner: PortableOutputState;
  open: boolean;
  paths?: string[];
}

// Async scopes distinguish detached work from the next invocation of the same context.
const require = createRequire(import.meta.url);
const invocation: AsyncLocalStorage<Invocation> | undefined = IS_BROWSER
  ? undefined
  : new (require('node:async_hooks').AsyncLocalStorage)();

export function closePortableOutputInvocation(): void {
  const current = invocation?.getStore();
  if (current) current.open = false;
}

export class PortableOutputState {
  private paths: Set<string> | undefined;
  private certified = true;

  readonly declare = (declaration?: { paths?: readonly string[] }): void => {
    const current = invocation?.getStore();
    if (!current?.open || current.owner !== this) return;
    const paths = declaration?.paths ?? [];
    if (
      !Array.isArray(paths) ||
      paths.some((p) => typeof p !== 'string' || !path.isAbsolute(p))
    ) {
      throw new TypeError(
        'portableOutput paths must be an array of absolute paths',
      );
    }
    (current.paths ??= []).push(...paths);
  };

  /**
   * Opens one loader invocation and returns the function that closes it. Entering the async
   * scope here, rather than running the loader inside a callback, keeps loader error stacks free
   * of extra frames.
   */
  begin(): () => void {
    if (!invocation) {
      // Browsers cannot isolate detached async work. Preserve execution without authority.
      this.certified = false;
      return () => {};
    }
    const current: Invocation = { owner: this, open: true };
    invocation.enterWith(current);
    return () => {
      current.open = false;
      this.merge(current.paths);
    };
  }

  merge(paths: readonly string[] | null | undefined): void {
    if (paths == null) {
      this.certified = false;
    } else if (this.certified && paths.length) {
      this.paths ??= new Set();
      for (const value of paths) this.paths.add(value);
    }
  }

  result(): string[] | undefined {
    return this.certified ? Array.from(this.paths ?? []).sort() : undefined;
  }
}
