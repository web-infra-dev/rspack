# Getting Started

```sh
yarn install
pnpm run start
```

Open [http://localhost:3001](http://localhost:3001).

The demo is annotated so navigate through the demos and apps available.

Included apps:

- App #1 (ReactJS - acts as the app shell plus is an aggregation with other remotes): [http://localhost:3001](http://localhost:3001)
- App #2 (ReactJS - plus is an aggregation with other remotes): [http://localhost:3002](http://localhost:3002)
- App #3 (ReactJS): [http://localhost:3003](http://localhost:3003)
- App #4 (SvelteJS): [http://localhost:3004](http://localhost:3004)
- App #5 (LitElement): [http://localhost:3005](http://localhost:3005)
  <img src="https://ssl.google-analytics.com/collect?v=1&t=event&ec=email&ea=open&t=event&tid=UA-120967034-1&z=1589682154&cid=ae045149-9d17-0367-bbb0-11c41d92b411&dt=ModuleFederationExamples&dp=/email/ComprehensiveDemo">

# Running Playwright E2E Tests

To run the Playwright test suite locally in headless mode, execute `pnpm test:e2e` from this workspace. The tests automatically start the demo and verify each application.

For an interactive UI to debug or explore tests, run `pnpm test:e2e:ui`.

In CI scenarios run `pnpm e2e:ci`. This command builds the applications, installs the required Playwright browsers and runs the tests with a concise reporter.
