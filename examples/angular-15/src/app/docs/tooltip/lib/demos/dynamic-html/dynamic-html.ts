import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-tooltip-dynamic-html',
  templateUrl: './dynamic-html.html'
})
export class DemoTooltipDynamicHtmlComponent {
  html = `<span class="btn-block btn-danger well-sm">Never trust not sanitized HTML!!!</span>`;
}
