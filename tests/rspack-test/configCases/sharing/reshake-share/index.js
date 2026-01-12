import shared_0 from 'ui-lib';
import shared_1 from 'ui-lib-dep';

const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");
__webpack_require__.p = 'PUBLIC_PATH';

const treeShakingSharedDir = path.join(
	__dirname,
	"independent-packages"
);

const uiLibShareContainerPath = path.join(
	treeShakingSharedDir,
	"ui_lib/1.0.0",
	"share-entry.js"
);

const uiLibDepShareContainerPath = path.join(
	treeShakingSharedDir,
	"ui_lib_dep/1.0.0",
	"share-entry.js"
);

const customPluginAssetPath = path.join(
	uiLibDepShareContainerPath,
  '../..',
	"apply-plugin.json"
);


it("should build independent share file", () => {
	expect(fs.existsSync(uiLibShareContainerPath)).toBe(true);
	expect(fs.existsSync(uiLibDepShareContainerPath)).toBe(true);
	expect(fs.existsSync(customPluginAssetPath)).toBe(true);
});

it("reshake share container should only have specify usedExports", async () => {
    const uiLibDepShareContainerModule = __non_webpack_require__(uiLibDepShareContainerPath).reshake_share_ui_lib_dep_100;
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


it("correct handle share dep while reshake", async () => {
    const uiLibShareContainerModule = __non_webpack_require__(uiLibShareContainerPath).reshake_share_ui_lib_100;
		await uiLibShareContainerModule.init({},{
		installInitialConsumes: async ({webpackRequire})=>{
			webpackRequire.m['webpack/sharing/consume/default/ui-lib-dep'] = (m)=>{
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
