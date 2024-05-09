# Mixed Debugging Between JavaScript and Rust

This discussion aims to illustrate the method for mixed debugging between JavaScript and Rust.

## Prerequisites

To illustrate this process, I'll use an example. Let's start by introduce the environment and example I have used.

- System: macos
- IDE: vscode
- Debugging target: `rspack build ${projectRoot}/basic`

Firstly, you need to build rspack in debug mode. To do this, execute the following commands in the project's root directory:

```bash
npm run build:binding:debug
npm run build:js
```

## Configure `launch.json` in vscode

It's necessary to configure two debug configurations within in `.vscode/launch.json.`

- attach for node:

```jsonc
{
  "name": "attach:node”,
  "request": "attach",  // refer: https://code.visualstudio.com/docs/editor/debugging#_launch-versus-attach-configurations
  "type": "node",
  // `9229` is the default port of message
  "port": 9229
}
```

- and launch for lldb

```jsonc
{
  "name": "launch:rust-from-node",
  "request": "launch”,
  "type": "lldb",    // it means we use `lldb` to launch the binary file of `node`
  "program": "node”,
  "args": [
    "--inspect",
    "--enable-source-maps",
    "${workspaceFolder}/packages/rspack-cli/bin/rspack",
    "build",
    "-c",
    "${workspaceFolder}/examples/basic/rspack.config.js",
  ],
   // `cwd` is just for repack find the correctly entry.
  "cwd": "${workspaceFolder}/examples/basic/"
}
```

Next, we can utilize [compounds](https://code.visualstudio.com/docs/editor/debugging#_compound-launch-configurations) to amalgamate the two commands:

```json
{
  "name": "mix-debug",
  "configurations": [
    "attach:node",
    "launch:rust-from-node"
  ]
}
```

Finally, your `﻿launch.json` should appear as follows:

```json
{
  "configurations": [
    {
      "name": "attach:node",
      "request": "attach",
      "type": "node",
      "port": 9229
    },
    {
      "name": "launch:rust-from-node",
      "request": "launch",
      "type": "lldb",
      "program": "node",
      "args": [
        "--inspect",
        "--enable-source-maps",
        "${workspaceFolder}/packages/rspack-cli/bin/rspack",
        "build",
        "-c",
        "${workspaceFolder}/examples/basic/rspack.config.js",
      ],
      "cwd": "${workspaceFolder}/examples/basic/"
    }
  ],
  "compounds": [
    {
      "name": "mix-debug",
      "configurations": [
        "attach:node",
        "launch:rust-from-node"
      ]
    }
  ]
}
```

## Debugging Attempt

Next, we can introduce some breakpoints and commence debugging. 

The result appears as follows:

<video width="640" height="480" controls>
  <source src="https://github.com/web-infra-dev/rspack/assets/30187863/106983f7-a59e-4d9e-9001-552f4441d88b" type="video/mp4">
  Your browser does not support the video tag.
</video>
