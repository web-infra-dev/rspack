import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { JSDOM, ResourceLoader, VirtualConsole } from "jsdom";
import { escapeSep } from "../../helper";
import EventSource from "../../helper/legacy/EventSourceForNode";
import urlToRelativePath from "../../helper/legacy/urlToRelativePath";
import type { ECompilerType, TRunnerFile, TRunnerRequirer } from "../../type";
import { type INodeRunnerOptions, NodeRunner } from "../node";

export interface IWebRunnerOptions<
	T extends ECompilerType = ECompilerType.Rspack
> extends INodeRunnerOptions<T> {
	location: string;
}

// Compatibility code to suppress iconv-lite warnings
require("iconv-lite").skipDecodeWarning = true;

const FAKE_HOSTS = [
	"https://example.com/public/path",
	"https://example.com",
	"https://test.cases/path/",
	"https://test.cases/server/",
	"https://test.cases"
];

const FAKE_TEST_ROOT_HOST = "https://test.cases/root/";

export class WebRunner<
	T extends ECompilerType = ECompilerType.Rspack
> extends NodeRunner<T> {
	private dom: JSDOM;
	constructor(protected _webOptions: IWebRunnerOptions<T>) {
		super(_webOptions);

		const virtualConsole = new VirtualConsole({});
		virtualConsole.sendTo(console, {
			omitJSDOMErrors: true
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

		if (this._options.compilerOptions.node !== false) {
			const vmContext = this.dom.getInternalVMContext();
			vmContext.global = {};
		}
	}

	run(file: string) {
		if (!file.endsWith(".js") && !file.endsWith(".mjs")) {
			const cssElement = this.dom.window.document.createElement("link");
			cssElement.href = file;
			cssElement.rel = "stylesheet";
			this.dom.window.document.head.appendChild(cssElement);
			return Promise.resolve();
		}
		return super.run(file);
	}

	getGlobal(name: string): unknown {
		return this.globalContext![name];
	}

	protected createResourceLoader() {
		const that = this;
		class CustomResourceLoader extends ResourceLoader {
			fetch(url: string, _: { element: HTMLScriptElement }) {
				const filePath = that.urlToPath(url);
				let finalCode: string | Buffer | void;
				if (path.extname(filePath) === ".js") {
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
					that.dom.window["__LINK_SHEET__"] ??= {};
					that.dom.window["__LINK_SHEET__"][url] = finalCode!.toString();
					return Promise.resolve(finalCode!) as any;
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

	private urlToPath(url: string) {
		if (url.startsWith("file://")) {
			return fileURLToPath(url);
		}
		if (url.startsWith(FAKE_TEST_ROOT_HOST)) {
			return path.resolve(
				__TEST_PATH__,
				`./${url.slice(FAKE_TEST_ROOT_HOST.length)}`
			);
		}
		let dist = url;
		for (const host of FAKE_HOSTS) {
			if (url.startsWith(host)) {
				dist = url.slice(host.length);
				break;
			}
		}
		return path.resolve(this._webOptions.dist, `./${dist}`).split("?")[0];
	}

	protected createBaseModuleScope() {
		const moduleScope = super.createBaseModuleScope();
		moduleScope.EventSource = EventSource;
		moduleScope.fetch = async (url: string) => {
			try {
				const buffer: Buffer = await new Promise((resolve, reject) =>
					fs.readFile(this.urlToPath(url), (err, b) =>
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
		moduleScope.URL = URL;
		moduleScope.importScripts = (url: string) => {
			this._options.env.expect(url).toMatch(/^https:\/\/test\.cases\/path\//);
			this.requirers.get("entry")!(this._options.dist, urlToRelativePath(url));
		};
		moduleScope.getComputedStyle = (element: HTMLElement) => {
			const computedStyle = this.dom.window.getComputedStyle(element);
			const getPropertyValue =
				computedStyle.getPropertyValue.bind(computedStyle);
			return {
				...computedStyle,
				getPropertyValue(v: any) {
					return getPropertyValue(v);
				}
			};
		};
		moduleScope.window = this.dom.window;
		moduleScope.document = this.dom.window.document;
		moduleScope.getLinkSheet = (link: HTMLLinkElement) => {
			return this.dom.window["__LINK_SHEET__"][link.href];
		};
		return moduleScope;
	}

	protected getModuleContent(file: TRunnerFile): [
		{
			exports: Record<string, unknown>;
		},
		string
	] {
		const m = {
			exports: {}
		};
		const currentModuleScope = this.createModuleScope(
			this.getRequire(),
			m,
			file
		);

		if (this._options.testConfig.moduleScope) {
			this._options.testConfig.moduleScope(
				currentModuleScope,
				this._options.stats,
				this._options.compilerOptions
			);
		}

		if (file.content.includes("__STATS__")) {
			currentModuleScope.__STATS__ = this._options.stats?.();
		}
		if (file.content.includes("__STATS_I__")) {
			const statsIndex = this._options.stats?.()?.__index__;
			if (typeof statsIndex === "number") {
				currentModuleScope.__STATS_I__ = statsIndex;
			}
		}

		const scopeKey = escapeSep(file!.path);
		const args = Object.keys(currentModuleScope).filter(
			arg => !["window", "self", "globalThis", "console"].includes(arg)
		);
		const argValues = args
			.map(arg => `window["${scopeKey}"]["${arg}"]`)
			.join(", ");
		this.dom.window[scopeKey] = currentModuleScope;
		this.dom.window["__GLOBAL_SHARED__"] = this.globalContext;
		return [
			m,
			`
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
			});
			(function(window, self, globalThis, console, ${args.join(", ")}) {
				${file.content}
			})($$g$$, $$self$$, $$g$$, window["console"], ${argValues});
		`
		];
	}

	protected createJSDOMRequirer(): TRunnerRequirer {
		return (currentDirectory, modulePath, context = {}) => {
			const file = context.file || this.getFile(modulePath, currentDirectory);
			if (!file) {
				return this.requirers.get("miss")!(currentDirectory, modulePath);
			}

			if (file.path in this.requireCache) {
				return this.requireCache[file.path].exports;
			}

			const [m, code] = this.getModuleContent(file);

			this.preExecute(code, file);
			this.dom.window.eval(code);
			this.postExecute(m, file);

			this.requireCache[file.path] = m;
			return m.exports;
		};
	}

	protected createRunner() {
		super.createRunner();
		this.requirers.set("cjs", this.createJSDOMRequirer());
	}
}
