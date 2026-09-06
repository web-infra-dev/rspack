<picture>
  <img alt="Rspack Banner" src="https://assets.rspack.rs/rspack/rspack-banner.png">
</picture>

# Rspack

<p>
  <a href="https://discord.gg/79ZZ66GH9E"><img src="https://img.shields.io/badge/chat-discord-blue?style=flat-square&logo=discord&colorA=564341&colorB=EDED91" alt="discord channel" /></a>
  <a href="https://www.npmjs.com/package/@rspack/core?activeTab=readme"><img src="https://img.shields.io/npm/v/@rspack/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a>
  <a href="https://crates.io/crates/rspack_core"><img src="https://img.shields.io/crates/v/rspack_core?style=flat-square&colorA=564341&colorB=EDED91" alt="crates version" /></a>
  <a href="https://npmcharts.com/compare/@rspack/core?minimal=true"><img src="https://img.shields.io/npm/dm/@rspack/core.svg?style=flat-square&colorA=564341&colorB=EDED91" alt="downloads" /></a>
  <a href="https://nodejs.org/en/about/previous-releases"><img src="https://img.shields.io/node/v/@rspack/core.svg?style=flat-square&colorA=564341&colorB=EDED91" alt="node version"></a>
  <a href="https://github.com/web-infra-dev/rspack/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square&colorA=564341&colorB=EDED91" alt="license" /></a>
  <a href="https://codspeed.io/web-infra-dev/rspack"><img src="https://img.shields.io/endpoint?url=https%3A%2F%2Fcodspeed.io%2Fbadge.json&style=flat-square&colorA=564341&colorB=EDED91" alt="codspeed" /></a>
</p>

English | [简体中文](./README.zh-CN.md)

Rspack is a fast Rust-based bundler for the web. It modernizes the webpack API to enable seamless replacement of webpack while delivering lightning-fast build speeds.

## ✨ Features

- 🚀 **Fast Startup**: Based on Rust, the build speed is extremely fast, bringing you the ultimate development experience.
- ⚡ **Lightning HMR**: With a built-in incremental compilation mechanism, HMR is extremely fast and fully capable of developing large-scale projects.
- 📦 **Webpack Compatible**: Compatible with plugins and loaders in the webpack ecosystem, seamlessly integrating excellent libraries built by the community.
- 🎨 **Module Federation**: Provide first-class support for Module Federation to facilitate the development of large-scale web applications.
- 🛠️ **Production Optimization**: Various optimization strategies are built in by default, such as tree shaking, minification, etc.
- 🎯 **Framework Agnostic**: Not bound to any frontend framework, ensuring enough flexibility.

Read [Introduction](https://rspack.rs/guide/start/introduction) for details.

## 🦀 Rstack

Rspack is part of Rstack, the fast, unified JavaScript toolchain for developers and agents.

| Name                                                  | Description              | Version                                                                                                                                                                          |
| ----------------------------------------------------- | ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [Rspack](https://github.com/web-infra-dev/rspack)     | Bundler                  | <a href="https://npmjs.com/package/@rspack/core"><img src="https://img.shields.io/npm/v/@rspack/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a>     |
| [Rsbuild](https://github.com/web-infra-dev/rsbuild)   | Build tool               | <a href="https://npmjs.com/package/@rsbuild/core"><img src="https://img.shields.io/npm/v/@rsbuild/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a>   |
| [Rslib](https://github.com/web-infra-dev/rslib)       | Library development tool | <a href="https://npmjs.com/package/@rslib/core"><img src="https://img.shields.io/npm/v/@rslib/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a>       |
| [Rspress](https://github.com/web-infra-dev/rspress)   | Static site generator    | <a href="https://npmjs.com/package/@rspress/core"><img src="https://img.shields.io/npm/v/@rspress/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a>   |
| [Rsdoctor](https://github.com/web-infra-dev/rsdoctor) | Build analyzer           | <a href="https://npmjs.com/package/@rsdoctor/core"><img src="https://img.shields.io/npm/v/@rsdoctor/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a> |
| [Rstest](https://github.com/web-infra-dev/rstest)     | Testing framework        | <a href="https://npmjs.com/package/@rstest/core"><img src="https://img.shields.io/npm/v/@rstest/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a>     |
| [Rslint](https://github.com/web-infra-dev/rslint)     | Linter                   | <a href="https://npmjs.com/package/@rslint/core"><img src="https://img.shields.io/npm/v/@rslint/core?style=flat-square&colorA=564341&colorB=EDED91" alt="npm version" /></a>     |

## Getting started

<p>
  <a target="_blank" href="https://stackblitz.com/fork/github/rstackjs/rspack-stackblitz-example">
    <img
      alt="Open in StackBlitz"
      src="https://developer.stackblitz.com/img/open_in_stackblitz.svg"
    />
  </a>
</p>

See [Quick start](https://rspack.rs/guide/start/quick-start).

## Contribution

Please read the [contributing guide](./CONTRIBUTING.md) and let's build Rspack together.

### Code of conduct

This repo has adopted the ByteDance Open Source Code of Conduct. Please check [Code of conduct](./CODE_OF_CONDUCT.md) for more details.

## Community

Come chat with us on [Discord](https://discord.gg/79ZZ66GH9E)! Rspack team and Rspack users are active there, and we're always looking for contributions.

## Links

| Name                                                                                      | Description                                                                   |
| ----------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| [awesome-rstack](https://github.com/rstackjs/awesome-rstack)                              | A curated list of awesome things related to Rstack                            |
| [agent-skills](https://github.com/rstackjs/agent-skills)                                  | A collection of Agent Skills for Rstack                                       |
| [Rspack 2.x docs](https://rspack.rs/)                                                     | Documentation for Rspack 2.x                                                  |
| [Rspack 1.x docs](https://v1.rspack.rs/)                                                  | Documentation for Rspack 1.x                                                  |
| [Rspack 0.x docs](https://v0.rspack.rs/)                                                  | Documentation for Rspack 0.x version                                          |
| [rspack-dev-server](https://github.com/rstackjs/rspack-dev-server)                        | Dev server for Rspack                                                         |
| [rstack-examples](https://github.com/rstackjs/rstack-examples)                            | Examples showcasing Rstack                                                    |
| [rspack-sources](https://github.com/web-infra-dev/rspack/tree/main/crates/rspack_sources) | Rust port of [webpack-sources](https://www.npmjs.com/package/webpack-sources) |
| [rstack-design-resources](https://github.com/rstackjs/rstack-design-resources)            | Design resources for Rstack                                                   |

## Contributors

<a href="https://github.com/web-infra-dev/rspack/graphs/contributors"><img src="https://opencollective.com/rspack/contributors.svg?width=890&button=false" /></a>

## Benchmarks

- See [build-tools-performance](https://github.com/rstackjs/build-tools-performance) for comparisons between Rspack and other tools.
- See [Rspack Benchmarks](https://ecosystem-benchmark.rspack.rs/) for Rspack's performance trends over time.

## Credits

Thanks to:

- [The webpack team and community](https://webpack.js.org/) for creating a great bundler and ecosystem from which we draw a lot of inspiration.
- [@sokra](https://github.com/sokra) for the great work on the [webpack](https://github.com/webpack/webpack) project.
- [@ScriptedAlchemy](https://github.com/ScriptedAlchemy) for creating Module Federation and helping Rspack connect with the community.
- The [SWC](https://github.com/swc-project/swc) project created by [@kdy1](https://github.com/kdy1), which powers Rspack's code parsing, transformation and minification.
- The [esbuild](https://github.com/evanw/esbuild) project created by [@evanw](https://github.com/evanw), which inspired the concurrent architecture of Rspack.
- The [NAPI-RS](https://github.com/napi-rs/napi-rs) project created by [@Brooooooklyn](https://github.com/Brooooooklyn), which powers Rspack's node-binding implementation.
- The [Parcel](https://github.com/parcel-bundler/parcel) project created by [@devongovett](https://github.com/devongovett) which is the pioneer of rust bundler and inspired Rspack's incremental rebuild design.
- The [Vite](https://github.com/vitejs/vite) project created by [Evan You](https://github.com/yyx990803) which inspired Rspack's compatibility design of webpack's ecosystem.
- The `rolldown-legacy` project created by old Rolldown team, It's the predecessor of the [rolldown](https://github.com/rolldown) project, which explores the possibility of making a performant bundler in Rust with Rollup-compatible API. It inspires the design principles of Rspack.
- The [html-webpack-plugin](https://github.com/jantimon/html-webpack-plugin) project created by [@jantimon](https://github.com/jantimon), `@rspack/html-plugin` is a fork of [html-webpack-plugin](https://github.com/jantimon/html-webpack-plugin) to avoid some webpack API usage not supported in Rspack.
- The [Turbopack](https://github.com/vercel/turbo) project which inspired the AST path logic of Rspack.
- The [react-refresh-webpack-plugin](https://github.com/pmmmwh/react-refresh-webpack-plugin) created by [@pmmmwh](https://github.com/pmmmwh), which inspires implement [react refresh rspack plugin](https://github.com/rstackjs/rspack-plugin-react-refresh).
- The [prefresh](https://github.com/preactjs/prefresh) created by [@Jovi De Croock](https://github.com/JoviDeCroock), which inspires implement [preact refresh rspack plugin](https://github.com/rstackjs/rspack-plugin-preact-refresh).
- The [mini-css-extract-plugin](https://github.com/webpack/mini-css-extract-plugin) project created by [@sokra](https://github.com/sokra) which inspired implement css extract plugin.
- The [copy-webpack-plugin](https://github.com/webpack/copy-webpack-plugin) project created by [@kevlened](https://github.com/kevlened) which inspired implement copy rspack plugin.
- The [webpack-subresource-integrity](https://github.com/waysact/webpack-subresource-integrity) project created by [@jscheid](https://github.com/jscheid), which inspires implement subresource integrity rspack plugin.
- The [circular-dependency-plugin](https://github.com/aackerman/circular-dependency-plugin) project created by [@aackerman](https://github.com/aackerman), which inspres implement circular dependency rspack plugin.
- The [tracing-chrome](https://github.com/thoren-d/tracing-chrome) project created by [thoren-d](https://github.com/thoren-d), which inspires the implementation of Rspack tracing.

## License

Rspack is [MIT licensed](https://github.com/web-infra-dev/rspack/blob/main/LICENSE).


## 🌐 Web Resources & Interactive Index
- [CATEGORY CARE](https://quizverses.pages.dev/category-care.html)
- [BUBBLE SHOOTER WILD WEST](https://frskillcrafts.pages.dev/bubble-shooter-wild-west.html)
- [CATEGORY BUSINESS137](https://themindzone.pages.dev/category-business137.html)
- [BRAWL BROS SQUAD](https://thelearnquester.web.app/brawl-bros-squad.html)
- [BEST FRIENDS PUZZLE](https://thelearnquesters.pages.dev/best-friends-puzzle.html)
- [STICKMAN PRISON AND LOVE](https://thequizzone.pages.dev/stickman-prison-and-love.html)
- [LOVE CATS ROPE](https://studyquests.github.io/love-cats-rope.html)
- [FREE HOOPS](https://thequizzone.pages.dev/free-hoops.html)
- [CATEGORY CASUAL](https://quizverses.github.io/category-casual.html)
- [CATEGORY ADVENTURE](https://studyquests.pages.dev/category-adventure.html)
- [HAMSTER COMBO IDLE](https://theskillquest.pages.dev/hamster-combo-idle.html)
- [CATEGORY STORY45](https://thelearnquesters.pages.dev/category-story45.html)
- [MY TINY MARKET](https://thelearnquesters.pages.dev/my-tiny-market.html)
- [SKIBIDI SURVIVOR RUSH](https://studyquests.github.io/skibidi-survivor-rush.html)
- [TIKTOK TRENDS COLORED DENIM](https://thelearnquesters.pages.dev/tiktok-trends-colored-denim.html)
- [LOLLIPOP STACK RUN](https://studyquests.github.io/lollipop-stack-run.html)
- [I8 CITY DRIVER](https://quizverses.github.io/i8-city-driver.html)
- [DR PARKING](https://quizverses.pages.dev/dr-parking.html)
- [THREAD MATCH 2](https://thequizzone.pages.dev/thread-match-2.html)
- [CATEGORY MAKEUP51](https://studyquests.github.io/category-makeup51.html)
- [WATERPARK SORT](https://thelearnquesters.pages.dev/waterpark-sort.html)
- [JELLY BELLY MAKE THE ELEPHANT](https://quizverses.github.io/jelly-belly-make-the-elephant.html)
- [SOLITAIRE STORY TRIPEAKS 5](https://thequizzone.pages.dev/solitaire-story-tripeaks-5.html)
- [WAR STATE IO CONQUER BATTLES](https://thelearnquesters.pages.dev/war-state-io-conquer-battles.html)
- [BUBBLE IT JAM](https://thequizzone.pages.dev/bubble-it-jam.html)
- [BLOCK SNIPER](https://quizverses.github.io/block-sniper.html)
- [COLOR BLOCK JAM 2](https://thequizzone.pages.dev/color-block-jam-2.html)
- [CATEGORY CASUAL 2](https://studyquests.github.io/category-casual-2.html)
- [DIGITAL AQUA](https://thequizzone.pages.dev/digital-aqua.html)
- [SWORDSMAN ADVENTURE](https://thequizzone.pages.dev/swordsman-adventure.html)
- [FRUIT MAHJONG 3D](https://thequizzone.pages.dev/fruit-mahjong-3d.html)
- [DARTS MASTERCARTOON 3D](https://thequizzone.pages.dev/darts-mastercartoon-3d.html)
- [MERGE BALLS SHOOTER 2048 CONNECT FRUITS](https://thelearnquesters.pages.dev/merge-balls-shooter-2048-connect-fruits.html)
- [CATEGORY BIKE](https://studyquests.pages.dev/category-bike.html)
- [PUZZLE SOLITAIRE PICTURE MATCH](https://thelearnquesters.pages.dev/puzzle-solitaire-picture-match.html)
- [VEX HYPER DASH](https://studyquests.github.io/vex-hyper-dash.html)
- [CATEGORY CAR](https://quizverses.pages.dev/category-car.html)
- [CATEGORY BOARDGAMES](https://studyquests.github.io/category-boardgames.html)
- [WORD JAM ASSOCIATION PUZZLE](https://quizverses.github.io/word-jam-association-puzzle.html)
- [BOMB HEAD HOT POTATO](https://studyquests.github.io/bomb-head-hot-potato.html)
- [NEW YEAR MAKEUP TRENDS](https://studyquests.github.io/new-year-makeup-trends.html)
- [GO CHICKEN GO](https://thequizzone.pages.dev/go-chicken-go.html)
- [GT DRIFT MOST WANTED](https://studyquests.github.io/gt-drift-most-wanted.html)
- [TRIANGLE WAY](https://thequizzone.pages.dev/triangle-way.html)
- [POOPY ESCAPE THE PRISON](https://thelearnquesters.pages.dev/poopy-escape-the-prison.html)
- [BLACK PINK STPATRICKS DAY CONCERT](https://quizverses.github.io/black-pink-stpatricks-day-concert.html)
- [CATEGORY QUIZ](https://thequizzone.pages.dev/category-quiz.html)
- [FOOD SORT 3D](https://thelearnquesters.pages.dev/food-sort-3d.html)
- [FASHION PRINCESS DRESS UP](https://thelearnquesters.pages.dev/fashion-princess-dress-up.html)
- [SPIN SHOT SIEGE](https://quizverses-9d2f2.web.app/spin-shot-siege.html)
- [SPA EMPIRE](https://thelearnquesters.pages.dev/spa-empire.html)
- [BATZO RUNNER](https://quizverses-9d2f2.web.app/batzo-runner.html)
- [SEAFARING MEMORY CHALLENGE](https://thequizzone.pages.dev/seafaring-memory-challenge.html)
- [CATEGORY THINKY](https://quizverses.pages.dev/category-thinky.html)
- [TUNG TUNG SAHUR OBBY CHALLENGE](https://thequizzone.pages.dev/tung-tung-sahur-obby-challenge.html)
- [MAGIC KINGDOM HEX MATCH](https://studyquests.github.io/magic-kingdom-hex-match.html)
- [FISHING BEAR](https://quizverses-9d2f2.web.app/fishing-bear.html)
- [CATEGORY CUTE](https://studyquests.github.io/category-cute.html)
- [MAHJONG CLASSIC](https://thequizzone.pages.dev/mahjong-classic.html)
- [HERO FIGHT CLASH](https://thelearnquesters.pages.dev/hero-fight-clash.html)
- [CATEGORY FASHION105](https://studyquests.github.io/category-fashion105.html)
- [STICK WAR SAGA](https://thequizzone.pages.dev/stick-war-saga.html)
- [THUMBPINBALL](https://thelearnquesters.pages.dev/thumbpinball.html)
- [SPACE SHIFT](https://thelearnquesters.pages.dev/space-shift.html)
- [HOME PIN 1](https://thequizzone.pages.dev/home-pin-1.html)
- [MERMAIDCORE MAKEUP](https://thelearnquesters.pages.dev/mermaidcore-makeup.html)
- [DREAM PET HOTEL](https://studyquests.pages.dev/dream-pet-hotel.html)
- [CATEGORY SOLITAIRE27](https://thequizzone.pages.dev/category-solitaire27.html)
- [TRIAL XTREME](https://thequizzone.pages.dev/trial-xtreme.html)
- [SPRUNKI 3D SHOOTER](https://studyquests.pages.dev/sprunki-3d-shooter.html)
- [BARBIECORE AESTHETICS](https://thequizzone.pages.dev/barbiecore-aesthetics.html)
- [BIRD SORT CHALLENGES](https://quizverses-9d2f2.web.app/bird-sort-challenges.html)
- [CATEGORY HERO72](https://quizverses.pages.dev/category-hero72.html)
- [FALLING BLOCKS HALLOWEEN CHALLENGE](https://thelearnquesters.pages.dev/falling-blocks-halloween-challenge.html)
- [CATEGORY BALL175](https://studyquests.github.io/category-ball175.html)
- [CATEGORY TOWER DEFENSE118](https://thequizzone.pages.dev/category-tower-defense118.html)
- [HAPPY BLOCKS](https://quizverses.github.io/happy-blocks.html)
- [MY HOSPITAL LEARN CARE](https://thelearnquesters.pages.dev/my-hospital-learn-care.html)
- [BUTTERFLY KYODAI RAINBOW](https://studyquests.pages.dev/butterfly-kyodai-rainbow.html)
- [NONOGRAM MASTER](https://thelearnquesters.pages.dev/nonogram-master.html)
- [CATEGORY BLOCK94](https://quizverses-9d2f2.web.app/category-block94.html)
- [SCARY BABY YELLOW GAME](https://studyquests.pages.dev/scary-baby-yellow-game.html)
- [CATEGORY CASUAL](https://studyquests.github.io/category-casual.html)
- [CATEGORY PUZZLE 8](https://thequizzone.pages.dev/category-puzzle-8.html)
- [CATCH THE GOOSE](https://thequizzone.pages.dev/catch-the-goose.html)
- [BUBBLE SHOOTER REMASTERED](https://studyquests.github.io/bubble-shooter-remastered.html)
- [CATEGORY DEEP IMMERSIVE24](https://quizverses.pages.dev/category-deep-immersive24.html)
- [CATEGORY MINECRAFT 2](https://thequizzone.pages.dev/category-minecraft-2.html)
- [SUPERHERO PHONE SIMULATOR](https://thelearnquesters.pages.dev/superhero-phone-simulator.html)
- [ROPE COLOR SORT 3D](https://thelearnquesters.pages.dev/rope-color-sort-3d.html)
- [SISYPHUS SIMULATOR](https://studyquests.github.io/sisyphus-simulator.html)
- [CHECKERS DRAUGHTS MULTIPLAYER](https://studyquests.pages.dev/checkers-draughts-multiplayer.html)
- [KEY QUEST](https://quizverses.github.io/key-quest.html)
- [SUPER SWING](https://thelearnquesters.pages.dev/super-swing.html)
- [CATEGORY TOWER DEFENSE 2](https://thequizzone.pages.dev/category-tower-defense-2.html)
- [TWO STUNT SUPERCARS](https://studyquests.github.io/two-stunt-supercars.html)
- [TUNNEL ROAD](https://studyquests.pages.dev/tunnel-road.html)
- [CATEGORY WEBGAME](https://thequizzone.pages.dev/category-webgame.html)
- [INDEX21](https://quizverses.pages.dev/index21.html)
- [AVATAR LIFE MY TOWN](https://quizverses.pages.dev/avatar-life-my-town.html)
- [CATEGORY CASUAL971](https://studyquests.github.io/category-casual971.html)
- [WINTER SOLITAIRE TRIPEAKS](https://thequizzone.pages.dev/winter-solitaire-tripeaks.html)
- [CLUB TYCOON IDLE CLICKER](https://thequizzone.pages.dev/club-tycoon-idle-clicker.html)
- [ATOMIC MERGE 2048](https://thequizzone.pages.dev/atomic-merge-2048.html)
- [SOLITAIRE STORY TRIPEAKS 6](https://quizverses-9d2f2.web.app/solitaire-story-tripeaks-6.html)
- [CATEGORY SURVIVAL366](https://thequizzone.pages.dev/category-survival366.html)
- [CATEGORY TOWER DEFENSE](https://thequizzone.pages.dev/category-tower-defense.html)
- [ARCHERY MASTER BOW AND ARROW](https://quizverses-9d2f2.web.app/archery-master-bow-and-arrow.html)
- [BR BR PATAPIM OBBY CHALLENGE](https://themindzone.pages.dev/br-br-patapim-obby-challenge.html)
- [MAZE ESCAPE CRAFT MAN](https://themindzone.pages.dev/maze-escape-craft-man.html)
- [FORTRESS OF THE SINISTER](https://thequizzone.pages.dev/fortress-of-the-sinister.html)
- [SCHOOL SIMULATOR MY SCHOOL](https://themindzone.pages.dev/school-simulator-my-school.html)
- [STUMBLE GUYS](https://quizverses-9d2f2.web.app/stumble-guys.html)
- [CAR SIMULATOR 3D CAR GAME 3D](https://themindzone.pages.dev/car-simulator-3d-car-game-3d.html)
- [BLOCK UP](https://themindzone.pages.dev/block-up.html)
- [CATEGORY FOOD](https://quizverses.pages.dev/category-food.html)
- [NAIL QUEEN](https://quizverses.github.io/nail-queen.html)
- [WARCALL IO](https://quizverses.github.io/warcall-io.html)
- [CATEGORY MAHJONG 2](https://quizverses.pages.dev/category-mahjong-2.html)
- [MERGE BLOCKS 2048 STYLE](https://studyquests.pages.dev/merge-blocks-2048-style.html)
- [RED STICKMAN VS CRAFTMANS](https://thequizzone.pages.dev/red-stickman-vs-craftmans.html)
- [FRAY FIGHT](https://themindzone.pages.dev/fray-fight.html)
- [CONQ](https://quizverses.github.io/conq.html)
- [PERFECT SHOT](https://quizverses.github.io/perfect-shot.html)
- [MONEY FACTORY EARN A BILLION](https://studyquests.github.io/money-factory-earn-a-billion.html)
- [CATEGORY CAN T STOP PLAYING215](https://studyquests.github.io/category-can-t-stop-playing215.html)
- [PORTAL HOP](https://thelearnquesters.pages.dev/portal-hop.html)
- [SUGAR HEROES](https://studyquests.pages.dev/sugar-heroes.html)
- [FOXY ECO SORT](https://thelearnquesters.pages.dev/foxy-eco-sort.html)
- [INDEX4](https://studyquesthub.web.app/index4.html)
