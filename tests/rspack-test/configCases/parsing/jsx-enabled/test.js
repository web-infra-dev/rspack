const fs = require("fs");
const path = require("path");

it("should keep jsx in output when parser jsx is enabled", () => {
  const bundle = fs.readFileSync(path.join(__dirname, "bundle0.jsx"), "utf-8");
  expect(bundle).toBe(`import { App, App1A, App1B, App1C, app1cProps } from "./App1";
import { App2, app2Props } from "./App2";

;// CONCATENATED MODULE: external "./App1"

;// CONCATENATED MODULE: external "./App2"

;// CONCATENATED MODULE: ./index.jsx



const DynamicComponent = ()=>{
    const Component = Math.random() > 0.5 ? App1A : App1C;
    return import("./App1").then((mod)=>{
        const Dynamic = mod[Component === App1A ? "App1A" : "App1C"];
        return <Dynamic/>;
    });
};
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
      <React.Fragment>
        <>
          <App1A/>
          <React.Suspense fallback={<div>Loading...</div>}>
            <DynamicComponent/>
          </React.Suspense>
        </>
      </React.Fragment>
      <App1C>
        {x ? <App1A>
            <App1B>
              <App2 props={app2Props}>
                <App1C props={app1cProps}/>
              </App2>
            </App1B>
          </App1A> : <App1B/>}
      </App1C>
      <App1C className="c"/>
      <App1C className="app1c"></App1C>
      <App1B/>
      <NamespaceComponents.Button label="Namespace button" {...{
        title: "extra",
        ["data-role"]: "primary"
    }} data-count={3} icon={<App1A/>} fragmentContent={<>
            <span>Nested</span>
            <span>Fragment</span>
          </>}/>
      <div className="wrapper">
        {[
        <section key="namespace-import" data-index="0">
            <App data-dynamic="registry" data-item="one"/>
            <foo:bar value="namespaced"/>
            <svg:path d="M0,0 L10,10" xlink:href="#one"/>
            <span>{"item-one".toUpperCase()}</span>
            { /* JSXEmptyExpr in action */ }
          </section>,
        <section key="legacy-widget" data-index="1">
            <App2 data-dynamic="registry" data-item="two" {...app2Props}/>
            app2
            <App2/>
            <foo:bar value="namespaced-two"/>
            <svg:path d="M10,10 L20,20" xlink:href="#two"/>
            <App1A/>
            { /* JSXEmptyExpr in action */ }
          </section>,
        <section key="external-app" data-index="2">
            <App1A data-dynamic="registry" data-item="fallback"/>
            <foo:bar value="namespaced-three"/>
            <svg:path d="M20,20 L30,30" xlink:href="#three"/>
            {(()=><NamespaceComponents.Button label="Inline child"/>)()}
            { /* JSXEmptyExpr in action */ }
          </section>
    ]}
        <group-container>{...spreadChildren}</group-container>
        <text-block dangerouslySetInnerHTML={{
        __html: "<strong>bold</strong>"
    }}/>
        <SectionWithSpread {...{
        "data-testid": "component-with-spread",
        role: "region"
    }}/>
      </div>
    </>;
}

export { Root as default };
`)
});

// TODO: There are some clear mangle errors, including but not limited to illegal component names like `t.App1`.
it ("should keep jsx in output when parser jsx is enabled (with minify)", () => {
  const bundle = fs.readFileSync(path.join(__dirname, "bundle1.jsx"), "utf-8");
  expect(bundle).toContain("<foo:bar value=");
  expect(bundle).toContain("<svg:path d=");
  expect(bundle).toContain("<group-container>");
  expect(bundle).toContain("<NamespaceComponents.Button label=\"Namespace button\"{...{");
  expect(bundle).toContain("<App data-dynamic=\"registry\"data-item=\"one\"/>");
  expect(bundle).toContain("<text-block dangerouslySetInnerHTML={{__html:\"<strong>bold</strong>\"}}/>");
  expect(bundle).toContain("<SectionWithSpread {...{\"data-testid\":\"component-with-spread\",role:\"region\"}}/>");
})
