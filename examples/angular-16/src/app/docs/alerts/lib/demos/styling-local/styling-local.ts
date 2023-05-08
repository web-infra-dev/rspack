import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-alert-styling-local',
  templateUrl: './styling-local.html',
  styles: [
    `
  :host .alert-md-local {
    background-color: #009688;
    border-color: #00695C;
    color: #fff;
  }
  `
  ]
})
export class DemoAlertStylingLocalComponent {}
