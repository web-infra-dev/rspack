The following tab components are being imported remotely from "bravo-app".

Notice that your browser's route is `/routing/<foo|bar>` depending on which tab is active.

If you open [http://localhost:3002](http://localhost:3002) you will see the same tab components at the root level.

The "Bar" tab also lazily renders the styled-component `Button` from the [UI Library](http://localhost:3003) demo only when rendered.
