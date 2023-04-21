var policy;
__webpack_require__.tt = function () {
    // Create Trusted Type policy if Trusted Types are available and the policy doesn't exist yet.
    if (policy === undefined) {
        policy = {
            createScript: function (script) {
                return script;
            },
            createScriptURL: function (url) {
                return url;
            }
        };
    }

    if (typeof trustedTypes !== "undefined" && trustedTypes.createPolicy) {
        policy = trustedTypes.createPolicy('$policyName$', policy);
    }
    return policy;
}