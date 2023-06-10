import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-tooltip-styling-local',
  templateUrl: './styling-local.html',
  styles: [
    `
      :host .tooltip-inner {
        background-color: #009688;
        color: #fff;
      }
      :host .tooltip.top .tooltip-arrow:before,
      :host .tooltip.top .tooltip-arrow {
        border-top-color: #009688;
      }
    `
  ]
})
export class DemoTooltipStylingLocalComponent {}
