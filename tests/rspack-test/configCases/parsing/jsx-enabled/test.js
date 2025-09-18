const fs = require("fs");
const path = require("path");

it("should keep jsx in output when parser jsx is enabled", () => {
  const bundle = fs.readFileSync(path.join(__dirname, "bundle0.mjs"), "utf-8");
  expect(bundle).toBe(`import { App2, app2Props } from "./App2";
import * as __WEBPACK_EXTERNAL_MODULE__App1_cd1629a5__ from "./App1";

;// CONCATENATED MODULE: external "./App1"

;// CONCATENATED MODULE: external "./App2"

;// CONCATENATED MODULE: ./index.jsx



const NamespaceComponents = {
    Button: ({ label, ...rest })=><button type="button" {...rest}>
      {label}
    </button>
};
const SectionWithSpread = (props)=><section {...props}/>;
const spreadChildren = [
    <span key="first">First</span>,
    <span key="second">Second</span>
];
function Root() {
    return <>
      <__WEBPACK_EXTERNAL_MODULE__App1_cd1629a5__.App1/>
      <__WEBPACK_EXTERNAL_MODULE__App1_cd1629a5__.App/>
      <NamespaceComponents.Button label="Namespace button" {...{
        title: 'extra',
        ['data-role']: 'primary'
    }} data-count={3} icon={<__WEBPACK_EXTERNAL_MODULE__App1_cd1629a5__.App1/>} fragmentContent={<>
            <span>Nested</span>
            <span>Fragment</span>
          </>}/>
      <div className="wrapper">
        {[
        <section key="namespace-import" data-index="0">
            <__WEBPACK_EXTERNAL_MODULE__App1_cd1629a5__.App data-dynamic="registry" data-item="one"/>
            <foo:bar value="namespaced"/>
            <svg:path d="M0,0 L10,10" xlink:href="#one"/>
            <span>{'item-one'.toUpperCase()}</span>
            { /* JSXEmptyExpr in action */ }
          </section>,
        <section key="legacy-widget" data-index="1">
            <App2 data-dynamic="registry" data-item="two" {...app2Props}/>app2<App2/>
            <foo:bar value="namespaced-two"/>
            <svg:path d="M10,10 L20,20" xlink:href="#two"/>
            <__WEBPACK_EXTERNAL_MODULE__App1_cd1629a5__.App1/>
            { /* JSXEmptyExpr in action */ }
          </section>,
        <section key="external-app" data-index="2">
            <__WEBPACK_EXTERNAL_MODULE__App1_cd1629a5__.App1 data-dynamic="registry" data-item="fallback"/>
            <foo:bar value="namespaced-three"/>
            <svg:path d="M20,20 L30,30" xlink:href="#three"/>
            {(()=><NamespaceComponents.Button label="Inline child"/>)()}
            { /* JSXEmptyExpr in action */ }
          </section>
    ]}
        <group-container>{...spreadChildren}</group-container>
        <text-block dangerouslySetInnerHTML={{
        __html: '<strong>bold</strong>'
    }}/>
        <SectionWithSpread {...{
        'data-testid': 'component-with-spread',
        role: 'region'
    }}/>
      </div>
    </>;
}
console.log(<Root/>);

export { Root as default };
`)
});

// TODO: There are some clear mangle errors, including but not limited to illegal component names like `t.App1`.
it ("should keep jsx in output when parser jsx is enabled (with minify)", () => {
  const bundle = fs.readFileSync(path.join(__dirname, "bundle1.mjs"), "utf-8");
  expect(bundle).toContain("<foo:bar value=");
  expect(bundle).toContain("<svg:path d=");
  expect(bundle).toContain("<group-container>");
  expect(bundle).toContain("<NamespaceComponents.Button label=\"Namespace button\"{...{");
  expect(bundle).toContain("<__WEBPACK_EXTERNAL_MODULE__App1_cd1629a5__.App data-dynamic=\"registry\"data-item=\"one\"/>");
  expect(bundle).toContain("<text-block dangerouslySetInnerHTML={{__html:\"<strong>bold</strong>\"}}/>");
  expect(bundle).toContain("<SectionWithSpread {...{\"data-testid\":\"component-with-spread\",role:\"region\"}}/>");
})
