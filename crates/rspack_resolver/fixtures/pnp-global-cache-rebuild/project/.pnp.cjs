const RAW_RUNTIME_STATE = '{\
  "dependencyTreeRoots": [{ "name": "project", "reference": "workspace:." }],\
  "enableTopLevelFallback": false,\
  "ignorePatternData": null,\
  "fallbackExclusionList": [],\
  "fallbackPool": [],\
  "packageRegistryData": [\
    [null, [[null, {\
      "packageLocation": "./",\
      "packageDependencies": [["a", "npm:1.0.0"]]\
    }]]],\
    ["project", [["workspace:.", {\
      "packageLocation": "./",\
      "packageDependencies": [["a", "npm:1.0.0"]]\
    }]]],\
    ["a", [["npm:1.0.0", {\
      "packageLocation": "../cache/a.zip/node_modules/a/",\
      "packageDependencies": [["b", "npm:1.0.0"]]\
    }]]],\
    ["b", [["npm:1.0.0", {\
      "packageLocation": "../cache/b.zip/node_modules/b/",\
      "packageDependencies": []\
    }]]]\
  ]\
}';
