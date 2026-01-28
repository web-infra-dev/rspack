module.exports = function MyRuntimePlugin() {
	return {
		name: 'my-runtime-plugin',
		resolveShare: function(args) {
			const { shareScopeMap, scope, pkgName, version, GlobalFederation } = args;
			args.resolver = function () {
				shareScopeMap[scope][pkgName][version] = {
					lib: ()=>()=> 'This is react 0.2.1'
				};
				return {
          shared: shareScopeMap[scope][pkgName][version],
          useTreesShaking:false
        };
			};
			return args;
		}
	};
}
