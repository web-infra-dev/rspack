import { ECompilerType } from "../../type";
import { BasicRunner } from "./basic";
import { JSDOM, VirtualConsole, ResourceLoader } from "jsdom";
import { IBasicRunnerOptions, TRunnerRequirer } from "../type";
import createFakeWorker from "../../helper/legacy/createFakeWorker";
import fs from "fs";
import path from "path";
import EventSource from "../../helper/legacy/EventSourceForNode";
import urlToRelativePath from "../../helper/legacy/urlToRelativePath";

export class JSDOMRunner<
	T extends ECompilerType = ECompilerType.Rspack
> extends BasicRunner<T> {
	private dom: JSDOM;
	private errors: Error[] = [];
	constructor(protected _webOptions: IBasicRunnerOptions<T>) {
		super({
			..._webOptions,
			runInNewContext: false
		});

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
				virtualConsole,
				beforeParse: window => {
					window.onerror = (message, ...remain) => {
						console.error(message, remain);
						this.errors.push(new Error(`${message}`));
					};
				}
			}
		);

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
	}

	protected createResourceLoader() {
		const urlToPath = (url: string) => {
			if (url.startsWith("https://test.cases/path/")) url = url.slice(24);
			return path.resolve(this._webOptions.dist, `./${url}`);
		};
		class CustomResourceLoader extends ResourceLoader {
			fetch(url: string, _: { element: HTMLScriptElement }) {
				try {
					return Promise.resolve(fs.readFileSync(urlToPath(url))) as any;
				} catch (err) {
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
		moduleScope["Worker"] = createFakeWorker({
			outputDirectory: this._options.dist
		});
		moduleScope["importScripts"] = (url: string) => {
			expect(url).toMatch(/^https:\/\/test\.cases\/path\//);
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

			const args = Object.keys(currentModuleScope);
			const argValues = args
				.map(arg => `window["${file!.path}"]["${arg}"]`)
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
                    script.src = "https://test.cases/path/${file.subPath}index.js";
                    return script;
                  }
                  return Reflect.get(target, prop, receiver);
                }
              });
            }
            return Reflect.get(target, prop, receiver);
          }
        });
        (function(window, self, globalThis, ${args.join(", ")}) {
          ${file.content}
        })($$g$$, $$g$$, $$g$$, ${argValues});
      `;

			this.preExecute(code, file);
			this.dom.window[file.path] = currentModuleScope;
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
