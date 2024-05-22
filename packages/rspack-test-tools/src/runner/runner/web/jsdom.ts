import fs from "fs";
import { JSDOM, ResourceLoader, VirtualConsole } from "jsdom";
import path from "path";

import { escapeSep } from "../../../helper";
import createFakeWorker from "../../../helper/legacy/createFakeWorker";
import EventSource from "../../../helper/legacy/EventSourceForNode";
import urlToRelativePath from "../../../helper/legacy/urlToRelativePath";
import { ECompilerType } from "../../../type";
import { TRunnerRequirer } from "../../type";
import { IBasicRunnerOptions } from "../basic";
import { CommonJsRunner } from "../cjs";

export class JSDOMWebRunner<
	T extends ECompilerType = ECompilerType.Rspack
> extends CommonJsRunner<T> {
	private dom: JSDOM;
	constructor(protected _webOptions: IBasicRunnerOptions<T>) {
		super(_webOptions);

		const virtualConsole = new VirtualConsole();
		virtualConsole.sendTo(console);
		this.dom = new JSDOM(
			`
      <!doctype html>
      <html>
      <head></head>
      <body></body>
      </html>
    `,
			{
				url: "https://test.cases/path/index.html",
				resources: this.createResourceLoader(),
				runScripts: "dangerously",
				virtualConsole
			}
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

		const vmContext = this.dom.getInternalVMContext();
		vmContext["global"] = {};
	}

	run(file: string) {
		if (!file.endsWith(".js")) {
			const cssElement = this.dom.window.document.createElement("link");
			cssElement.href = file;
			cssElement.rel = "stylesheet";
			this.dom.window.document.head.appendChild(cssElement);
			return Promise.resolve();
		}
		return super.run(file);
	}

	getGlobal(name: string): unknown {
		return this.dom.window[name];
	}

	protected createResourceLoader() {
		const urlToPath = (url: string) => {
			if (url.startsWith("https://test.cases/path/")) url = url.slice(24);
			return path.resolve(this._webOptions.dist, `./${url}`).split("?")[0];
		};
		class CustomResourceLoader extends ResourceLoader {
			fetch(url: string, _: { element: HTMLScriptElement }) {
				try {
					return Promise.resolve(fs.readFileSync(urlToPath(url))) as any;
				} catch (err) {
					console.error(err);
					if ((err as { code: string }).code === "ENOENT") {
						return null;
					}
					throw err;
				}
			}
		}

		return new CustomResourceLoader();
	}

	protected createBaseModuleScope() {
		const moduleScope = super.createBaseModuleScope();
		moduleScope["EventSource"] = EventSource;
		moduleScope["Worker"] = createFakeWorker(this._options.env, {
			outputDirectory: this._options.dist
		});
		const urlToPath = (url: string) => {
			if (url.startsWith("https://test.cases/path/")) url = url.slice(24);
			return path.resolve(this._webOptions.dist, `./${url}`);
		};
		moduleScope["fetch"] = async (url: string) => {
			try {
				const buffer: Buffer = await new Promise((resolve, reject) =>
					fs.readFile(urlToPath(url), (err, b) =>
						err ? reject(err) : resolve(b)
					)
				);
				return {
					status: 200,
					ok: true,
					json: async () => JSON.parse(buffer.toString("utf-8"))
				};
			} catch (err) {
				if ((err as { code: string }).code === "ENOENT") {
					return {
						status: 404,
						ok: false
					};
				}
				throw err;
			}
		};
		moduleScope["URL"] = URL;
		moduleScope["importScripts"] = (url: string) => {
			this._options.env.expect(url).toMatch(/^https:\/\/test\.cases\/path\//);
			this.requirers.get("entry")!(this._options.dist, urlToRelativePath(url));
		};
		moduleScope["STATS"] = moduleScope.__STATS__;
		return moduleScope;
	}

	protected createJSDOMRequirer(): TRunnerRequirer {
		const requireCache = Object.create(null);

		return (currentDirectory, modulePath, context = {}) => {
			let file = context["file"] || this.getFile(modulePath, currentDirectory);
			if (!file) {
				return this.requirers.get("miss")!(currentDirectory, modulePath);
			}

			if (file.path in requireCache) {
				return requireCache[file.path].exports;
			}

			const m = {
				exports: {}
			};
			requireCache[file.path] = m;
			const currentModuleScope = this.createModuleScope(
				this.getRequire(),
				m,
				file
			);

			if (this._options.testConfig.moduleScope) {
				this._options.testConfig.moduleScope(currentModuleScope);
			}

			const scopeKey = escapeSep(file!.path);
			const args = Object.keys(currentModuleScope);
			const argValues = args
				.map(arg => `window["${scopeKey}"]["${arg}"]`)
				.join(", ");
			const code = `
        // hijack document.currentScript for auto public path
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
          }
        });
        (function(window, self, globalThis, console, ${args.join(", ")}) {
          ${file.content}
        })($$g$$, $$g$$, $$g$$, window["console"], ${argValues});
      `;

			this.preExecute(code, file);
			this.dom.window[scopeKey] = currentModuleScope;
			this.dom.window.eval(code);

			this.postExecute(m, file);
			return m.exports;
		};
	}

	protected createRunner() {
		super.createRunner();
		this.requirers.set("entry", this.createJSDOMRequirer());
	}
}
