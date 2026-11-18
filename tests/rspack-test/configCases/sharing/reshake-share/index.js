import * as uiLib from 'ui-lib';
import * as uiLibDep from 'ui-lib-dep';

const bundlerName = globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK
	? "rspack"
	: "webpack";

const fs = require("fs");
const path = require("path");
__webpack_require__.p = 'PUBLIC_PATH';

const [[uiLibEntry, , uiLibGlobal]] = sharedFallbacks['ui-lib'];
const [[uiLibDepEntry, , uiLibDepGlobal]] = sharedFallbacks['ui-lib-dep'];
const uiLibShareContainerPath = path.join(__dirname, uiLibEntry);
const uiLibDepShareContainerPath = path.join(__dirname, uiLibDepEntry);

const customPluginAssetPath = path.join(
	uiLibDepShareContainerPath,
  '../..',
	"apply-plugin.json"
);


it("should build independent share file", () => {
	expect(uiLib.Badge).toBe("Badge");
	expect(uiLib.MessagePro).toBe("MessagePro");
	expect(uiLibDep.Message).toBe("Message");
	expect(fs.existsSync(uiLibShareContainerPath)).toBe(true);
	expect(fs.existsSync(uiLibDepShareContainerPath)).toBe(true);
	expect(fs.existsSync(customPluginAssetPath)).toBe(true);
});

it("secondary tree shaking shared container should only have specify usedExports", async () => {
    const uiLibDepShareContainerModule = eval("require")(uiLibDepShareContainerPath)[uiLibDepGlobal];
		await uiLibDepShareContainerModule.init({},{
		installInitialConsumes: async ()=>{
			return 'call init'
		}
		});
		const shareModulesGetter = await uiLibDepShareContainerModule.get();
		const shareModules = shareModulesGetter();
		expect(shareModules.Message).toBe('Message');
		expect(shareModules.Text).not.toBeDefined();
});


it("correct handle share dep while secondary tree shaking", async () => {
    const uiLibShareContainerModule = eval("require")(uiLibShareContainerPath)[uiLibGlobal];
		await uiLibShareContainerModule.init({},{
		installInitialConsumes: async ({webpackRequire})=>{
			webpackRequire.m[`${bundlerName}/sharing/consume/default/ui-lib-dep`] = (m)=>{
				m.exports = {
					Message: 'Message',
				}
			}
			return 'call init'
		}
		});
		const shareModulesGetter = await uiLibShareContainerModule.get();
		const shareModules = shareModulesGetter();
 		expect(shareModules.Badge).toBe('Badge');
 		expect(shareModules.MessagePro).toBe('MessagePro');
});
