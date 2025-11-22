const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

const independentShareDir = path.join(
	__dirname,
	"independent-packages"
);

const customPluginAssetPath = path.join(
	independentShareDir,
	"apply-plugin.json"
);

const uiLibShareContainerPath = path.join(
	independentShareDir,
	"ui_lib/1.0.0",
	"share-entry.js"
);

const uiLibDepShareContainerPath = path.join(
	independentShareDir,
	"ui_lib_dep/1.0.0",
	"share-entry.js"
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
		const shareModules = await uiLibDepShareContainerModule.get();
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
		const shareModules = await uiLibShareContainerModule.get();
 		expect(shareModules.Badge).toBe('Badge');
 		expect(shareModules.MessagePro).toBe('MessagePro');
});
