import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { Script } from 'node:vm';
import { JSDOM, ResourceLoader, VirtualConsole } from 'jsdom';
import { escapeSep } from '../../helper';
import EventSource from '../../helper/legacy/EventSourceForNode';
import urlToRelativePath from '../../helper/legacy/urlToRelativePath';
import type { TRunnerFile, TRunnerRequirer } from '../../type';
import { type INodeRunnerOptions, NodeRunner } from '../node';

export interface IWebRunnerOptions extends INodeRunnerOptions {
  location: string;
}

// Compatibility code to suppress iconv-lite warnings
require('iconv-lite').skipDecodeWarning = true;

const FAKE_HOSTS = [
  'https://example.com/public/path',
  'https://example.com',
  'https://test.cases/path/',
  'https://test.cases/server/',
  'https://test.cases',
];

const FAKE_TEST_ROOT_HOST = 'https://test.cases/root/';

export class WebRunner extends NodeRunner {
  private dom: JSDOM;
  constructor(protected _webOptions: IWebRunnerOptions) {
    super(_webOptions);

    const virtualConsole = new VirtualConsole({});
    virtualConsole.sendTo(console, {
      omitJSDOMErrors: true,
    });
    this.dom = new JSDOM(
      `
      <!doctype html>
      <html>
      <head></head>
      <body></body>
      </html>
    `,
      {
        url: this._webOptions.location,
        resources: this.createResourceLoader(),
        runScripts: 'dangerously',
        virtualConsole,
      },
    );

    this.dom.window.console = console;
    // compat with FakeDocument
    this.dom.window.eval(`
      Object.defineProperty(document.head, "_children", {
        get: function() {
          return Array.from(document.head.children).map(function(ele) {
            var type = ele.tagName.toLowerCase();
            return new Proxy(ele, {
              get(target, prop, receiver) {
                if (prop === "_type") {
                  return target.tagName.toLowerCase();
                }
                if (prop === "_href") {
                  return Reflect.get(target, "href", receiver);
                }
                return Reflect.get(target, prop, receiver);
              },
            });
          });
        }
      });
    `);

    if (this._options.compilerOptions.node !== false) {
      const vmContext = this.dom.getInternalVMContext();
      vmContext.global = {};
    }
  }

  run(file: string) {
    if (!file.endsWith('.js') && !file.endsWith('.mjs')) {
      this.log(`css: ${file}`);
      const cssElement = this.dom.window.document.createElement('link');
      cssElement.href = file;
      cssElement.rel = 'stylesheet';
      this.dom.window.document.head.appendChild(cssElement);
      return Promise.resolve();
    }
    return super.run(file);
  }

  getGlobal(name: string): unknown {
    return this.globalContext![name];
  }

  protected log(message: string) {
    this._options.logs?.push(`[WebRunner] ${message}`);
  }

  protected createResourceLoader() {
    const that = this;
    class CustomResourceLoader extends ResourceLoader {
      fetch(url: string, options: { element: HTMLScriptElement }) {
        if (that._options.testConfig.resourceLoader) {
          that.log(`resource custom loader: start ${url}`);
          const content = that._options.testConfig.resourceLoader(
            url,
            options.element,
          );
          if (content !== undefined) {
            that.log(`resource custom loader: accepted`);
            return Promise.resolve(content) as any;
          } else {
            that.log(`resource custom loader: not found`);
          }
        }

        const filePath = that.urlToPath(url);
        that.log(`resource loader: ${url} -> ${filePath}`);
        let finalCode: string | Buffer | void;

        if (path.extname(filePath) === '.js') {
          const currentDirectory = path.dirname(filePath);
          const file = that.getFile(filePath, currentDirectory);
          if (!file) {
            throw new Error(`File not found: ${filePath}`);
          }

          const [_m, code] = that.getModuleContent(file);
          finalCode = code;
        } else {
          finalCode = fs.readFileSync(filePath);
        }

        try {
          that.dom.window['__LINK_SHEET__'] ??= {};
          that.dom.window['__LINK_SHEET__'][url] = finalCode!.toString();
          return Promise.resolve(finalCode!) as any;
        } catch (err) {
          console.error(err);
          if ((err as { code: string }).code === 'ENOENT') {
            return null;
          }
          throw err;
        }
      }
    }

    return new CustomResourceLoader();
  }

  private urlToPath(url: string) {
    if (url.startsWith('file://')) {
      return fileURLToPath(url);
    }
    if (url.startsWith(FAKE_TEST_ROOT_HOST)) {
      return path.resolve(
        __TEST_PATH__,
        `./${url.slice(FAKE_TEST_ROOT_HOST.length)}`,
      );
    }
    let dist = url;
    for (const host of FAKE_HOSTS) {
      if (url.startsWith(host)) {
        dist = url.slice(host.length);
        break;
      }
    }
    return path.resolve(this._webOptions.dist, `./${dist}`).split('?')[0];
  }

  protected createBaseModuleScope() {
    const moduleScope = super.createBaseModuleScope();
    moduleScope.EventSource = EventSource;
    moduleScope.fetch = async (url: string) => {
      try {
        const filePath = this.urlToPath(url);
        this.log(`fetch: ${url} -> ${filePath}`);
        const buffer: Buffer = await new Promise((resolve, reject) =>
          fs.readFile(filePath, (err, b) => (err ? reject(err) : resolve(b))),
        );
        return {
          status: 200,
          ok: true,
          json: async () => JSON.parse(buffer.toString('utf-8')),
        };
      } catch (err) {
        if ((err as { code: string }).code === 'ENOENT') {
          return {
            status: 404,
            ok: false,
          };
        }
        throw err;
      }
    };
    moduleScope.URL = URL;
    moduleScope.importScripts = (url: string) => {
      const path = urlToRelativePath(url);
      this.log(`importScripts: ${url} -> ${path}`);
      this.requirers.get('entry')!(this._options.dist, path);
    };
    moduleScope.getComputedStyle = (element: HTMLElement) => {
      const computedStyle = this.dom.window.getComputedStyle(element);
      const getPropertyValue =
        computedStyle.getPropertyValue.bind(computedStyle);
      return {
        ...computedStyle,
        getPropertyValue(v: any) {
          return getPropertyValue(v);
        },
      };
    };
    moduleScope.window = this.dom.window;
    moduleScope.document = this.dom.window.document;
    moduleScope.getLinkSheet = (link: HTMLLinkElement) => {
      return this.dom.window['__LINK_SHEET__'][link.href];
    };
    return moduleScope;
  }

  protected getModuleContent(file: TRunnerFile): [
    {
      exports: Record<string, unknown>;
    },
    string,
    number,
  ] {
    const m = {
      exports: {},
    };
    const currentModuleScope = this.createModuleScope(
      this.getRequire(),
      m,
      file,
    );

    if (this._options.testConfig.moduleScope) {
      this._options.testConfig.moduleScope(
        currentModuleScope,
        this._options.stats,
        this._options.compilerOptions,
      );
    }

    if (file.content.includes('__STATS__')) {
      currentModuleScope.__STATS__ = this._options.stats?.();
    }
    if (file.content.includes('__STATS_I__')) {
      const statsIndex = this._options.stats?.()?.__index__;
      if (typeof statsIndex === 'number') {
        currentModuleScope.__STATS_I__ = statsIndex;
      }
    }

    const proxyCode = `// hijack document.currentScript for auto public path
			var $$g$$ = new Proxy(window, {
				get(target, prop, receiver) {
					if (prop === "document") {
						return new Proxy(window.document, {
							get(target, prop, receiver) {
								if (prop === "currentScript") {
									var script = target.createElement("script");
									script.src = "https://test.cases/path/${escapeSep(file.subPath)}index.js";
									return script;
								}
								return Reflect.get(target, prop, receiver);
							}
						});
					}
					return Reflect.get(target, prop, receiver);
				},
			});
			var $$self$$ = new Proxy(window, {
				get(target, prop, receiver) {
				  if (prop === "__HMR_UPDATED_RUNTIME__") {
					  return window["__GLOBAL_SHARED__"]["__HMR_UPDATED_RUNTIME__"];
					}
					return Reflect.get(target, prop, receiver);
				},
				set(target, prop, value, receiver) {
					if (prop === "__HMR_UPDATED_RUNTIME__") {
						window["__GLOBAL_SHARED__"]["__HMR_UPDATED_RUNTIME__"] = value;
					}
					return Reflect.set(target, prop, value, receiver);
				}
			});`;

    const proxyLines = proxyCode.split('\n');

    const locatedError = createLocatedError(
      this._options.errors || [],
      proxyLines.length + 1,
    );
    const originIt = currentModuleScope.it;
    currentModuleScope.it = (
      description: string,
      fn: (...args: any[]) => Promise<void>,
    ) => {
      return originIt(description, async (...args: any[]) => {
        try {
          return await fn(...args);
        } catch (e) {
          throw locatedError(e as Error, file);
        }
      });
    };

    const scopeKey = escapeSep(file!.path);
    const args = Object.keys(currentModuleScope).filter(
      (arg) => !['window', 'self', 'globalThis', 'console'].includes(arg),
    );
    const argValues = args
      .map((arg) => `window["${scopeKey}"]["${arg}"]`)
      .join(', ');
    this.dom.window[scopeKey] = currentModuleScope;
    this.dom.window['__GLOBAL_SHARED__'] = this.globalContext;
    this.dom.window['__LOCATED_ERROR__'] = locatedError;
    this.dom.window['__FILE__'] = file;

    return [
      m,
      `${proxyCode}
			(function(window, self, globalThis, console, ${args.join(', ')}) { try {
				${file.content}
			} catch (e) {
				throw __LOCATED_ERROR__(e, window["__FILE__"]);
			}})($$g$$, $$self$$, $$g$$, window["console"], ${argValues});`,
      proxyLines.length + 1,
    ];
  }

  protected createJSDOMRequirer(): TRunnerRequirer {
    return (currentDirectory, modulePath, context = {}) => {
      const file = context.file || this.getFile(modulePath, currentDirectory);
      this.log(`jsdom: ${modulePath} -> ${file?.path}`);
      if (!file) {
        return this.requirers.get('miss')!(currentDirectory, modulePath);
      }

      if (file.path in this.requireCache) {
        return this.requireCache[file.path].exports;
      }

      const [m, code, lineOffset] = this.getModuleContent(file);

      this.preExecute(code, file);

      try {
        const script = new Script(code);
        const vmContext = this.dom.getInternalVMContext();
        script.runInContext(vmContext, {
          filename: file.path,
          lineOffset: -lineOffset,
        });
      } catch (e) {
        const error = new Error(
          `Parse script '${file.path}' failed:\n${(e as Error).message}`,
        );
        error.stack = `${error.message}\n${(e as Error).stack}`;
        this._options.errors?.push(error);
        throw error;
      }

      this.postExecute(m, file);

      this.requireCache[file.path] = m;
      return m.exports;
    };
  }

  protected createRunner() {
    super.createRunner();
    this.requirers.set('cjs', this.createJSDOMRequirer());
  }
}

export const createLocatedError = (
  collectedErrors: Error[],
  offset: number,
) => {
  return (e: Error, file: TRunnerFile) => {
    const match = (e.stack || e.message).match(/<anonymous>:(\d+)/);
    if (match) {
      const [, line] = match;
      const realLine = Number(line) - offset;
      const codeLines = file.content.split('\n');
      const lineContents = [
        ...codeLines
          .slice(Math.max(0, realLine - 3), Math.max(0, realLine - 1))
          .map((line) => `│  ${line}`),
        `│> ${codeLines[realLine - 1]}`,
        ...codeLines.slice(realLine, realLine + 2).map((line) => `│  ${line}`),
      ];
      const message = `Error in JSDOM when running file '${file.path}' at line ${realLine}: ${e.message}\n${lineContents.join('\n')}`;
      const finalError = new Error(message);
      finalError.stack = `${message}\n${e.stack}`;
      collectedErrors.push(finalError);
      return finalError;
    } else {
      return e;
    }
  };
};
