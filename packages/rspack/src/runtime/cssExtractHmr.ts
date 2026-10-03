export function normalizeUrl(url: string): string {
  const urlString = url.trim();

  if (/^data:/i.test(urlString)) {
    return urlString;
  }

  const protocol =
    urlString.indexOf('//') !== -1 ? `${urlString.split('//')[0]}//` : '';
  const components = urlString
    .replace(new RegExp(protocol, 'i'), '')
    .split('/');
  const host = components[0].toLowerCase().replace(/\.$/, '');

  components[0] = '';

  const path = components
    .reduce((accumulator: string[], item) => {
      switch (item) {
        case '..':
          accumulator.pop();
          break;
        case '.':
          break;
        default:
          accumulator.push(item);
      }

      return accumulator;
    }, [])
    .join('/');

  return protocol + host + path;
}

type Option<T> = T | null | undefined;
type DebouncedFunction<T extends (...args: any[]) => any> = (
  ...args: Parameters<T>
) => void;

const srcByModuleId: Record<string, any> = Object.create(null);

const noDocument = typeof document === 'undefined';

const { forEach } = Array.prototype;

function debounce<T extends (...args: any[]) => any>(
  fn: T,
  time: number,
): DebouncedFunction<T> {
  let timeout: NodeJS.Timeout | number = 0;

  return function (this: any, ...args: Parameters<T>[]) {
    const self = this;

    const functionCall = function functionCall() {
      return fn.apply(self, args as Parameters<T>);
    };

    clearTimeout(timeout);

    timeout = setTimeout(functionCall, time);
  };
}

function noop() {}

function getCurrentScriptUrl(moduleId: string) {
  let src = srcByModuleId[moduleId];

  if (!src) {
    if (document.currentScript) {
      ({ src } = document.currentScript as HTMLScriptElement);
    } else {
      const scripts = document.getElementsByTagName('script');
      const lastScriptTag = scripts[scripts.length - 1];

      if (lastScriptTag) {
        ({ src } = lastScriptTag);
      }
    }

    srcByModuleId[moduleId] = src;
  }

  return (fileMap: string): Option<string[]> | null => {
    if (!src) {
      return null;
    }

    const splitResult = src.match(/([^\\/]+)\.js$/);
    const filename = splitResult && splitResult[1];

    if (!filename || !fileMap) {
      return [src.replace('.js', '.css')];
    }

    return fileMap.split(',').map((mapRule) => {
      const reg = new RegExp(`${filename}\\.js$`, 'g');

      return normalizeUrl(
        src.replace(reg, `${mapRule.replace(/{fileName}/g, filename)}.css`),
      );
    });
  };
}

const pendingCssUpdates = new WeakMap<
  HTMLLinkElement,
  { url: string | undefined }
>();
function updateCss(el: HTMLLinkElement & Record<string, any>, url?: string) {
  const normalizedUrl = url || el.getAttribute('href')?.split('?')[0];
  if (!isUrlRequest(el.href) || !normalizedUrl?.includes('.css') || el.disabled)
    return;
  const pending = pendingCssUpdates.get(el);
  if (pending) {
    pending.url = normalizedUrl;
    return;
  }
  if (el.isLoaded === false) return;
  const parent = el.parentNode;
  if (!parent) return;
  const state: { url: string | undefined } = { url: undefined };
  pendingCssUpdates.set(el, state);
  el.isLoaded = false;
  const previousHref = el.href;
  const newEl = el.cloneNode() as HTMLLinkElement & Record<string, any>;
  newEl.visited = true;
  newEl.isLoaded = false;
  const nextHref = `${normalizedUrl}?${Date.now()}`;
  let committed = false;
  let stopped = false;
  const cleanup = (retryQueued = true) => {
    if (stopped) return;
    stopped = true;
    observer.disconnect();
    newEl.remove();
    el.removeEventListener('load', finish);
    el.removeEventListener('error', failCommit);
    pendingCssUpdates.delete(el);
    el.isLoaded = true;
    el.visited = false;
    if (retryQueued && state.url && el.parentNode === parent)
      updateCss(el, state.url);
  };
  const finish = () => cleanup();
  const failCommit = () => {
    // Keep the already loaded replacement visible while recovering the document.
    window.location.reload();
  };
  const observer = new MutationObserver(() => {
    if (
      el.disabled ||
      el.parentNode !== parent ||
      el.href !== (committed ? newEl.href : previousHref)
    )
      cleanup(false);
  });
  observer.observe(parent, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: ['href', 'disabled'],
  });
  newEl.addEventListener(
    'load',
    () => {
      if (stopped) return;
      if (el.disabled || el.parentNode !== parent || el.href !== previousHref) {
        cleanup(false);
        return;
      }
      committed = true;
      el.addEventListener('load', finish);
      el.addEventListener('error', failCommit);
      el.setAttribute('href', nextHref);
    },
    { once: true },
  );
  newEl.addEventListener('error', () => cleanup(), { once: true });
  newEl.href = nextHref;
  parent.insertBefore(newEl, el.nextSibling);
}

function getReloadUrl(href: string, src: string[]): string {
  let ret = '';

  const normalizedHref = normalizeUrl(href);

  src.some((url) => {
    if (normalizedHref.indexOf(src as unknown as string) > -1) {
      ret = url;
    }
  });

  return ret;
}

function reloadStyle(src: Option<string[]>): boolean {
  if (!src) {
    return false;
  }

  const elements = document.querySelectorAll('link');
  let loaded = false;

  forEach.call(elements, (el) => {
    if (!el.href) {
      return;
    }

    const url = getReloadUrl(el.href, src);

    if (!isUrlRequest(url)) {
      return;
    }

    if (el.visited === true) {
      return;
    }

    if (url) {
      updateCss(el, url);

      loaded = true;
    }
  });

  return loaded;
}

function reloadAll() {
  const elements = document.querySelectorAll('link');

  forEach.call(elements, (el) => {
    if (el.visited === true) {
      return;
    }

    updateCss(el);
  });
}

function isUrlRequest(url: string): boolean {
  // An URL is not an request if

  // It is not http or https
  if (!/^[a-zA-Z][a-zA-Z\d+\-.]*:/.test(url)) {
    return false;
  }

  return true;
}

function cssReload(moduleId: string, options: Record<string, any>) {
  if (noDocument) {
    console.log('[HMR] No `window.document` found, CSS HMR disabled');

    return noop;
  }

  const getScriptSrc = getCurrentScriptUrl(moduleId);

  function update() {
    const src = getScriptSrc(options.filename);
    const reloaded = reloadStyle(src);

    if (options.locals) {
      console.log('[HMR] Detected local CSS Modules. Reload all CSS');

      reloadAll();

      return;
    }

    if (reloaded) {
      console.log('[HMR] CSS reload %s', src && src.join(' '));
    } else {
      console.log('[HMR] Reload all CSS');

      reloadAll();
    }
  }

  return debounce(update, 50);
}

export { cssReload };
