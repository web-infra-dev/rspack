module.exports = function(params) {
  return {
    name: 'basic-plugin',
    version: '1.0.0',
		getParams(){
			return params
		},
    resolveShare: function(args) {
				const { shareScopeMap, scope, pkgName, version, GlobalFederation } = args;
        args.resolver = function () {
          shareScopeMap[scope][pkgName][version] = {
						lib: ()=>()=> 'This is react 0.2.1'
					};
          return shareScopeMap[scope][pkgName][version];
        };
        return args;
    }
  };
};
