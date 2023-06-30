__webpack_require__.hmd = function(module) {
    module = Object.create(module);
    if (!module.children) module.children = [];
    Object.defineProperty(module, 'exports', {
        enumerable: true,
        set: function() {
            throw new Error('ES Modules may not assign module.exports or exports.*, Use ESM export syntax, instead: ' + module.id);
        }
    });
    return module;
}