import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-tooltip-custom-content',
  templateUrl: './custom-content.html'
})
export class DemoTooltipCustomContentComponent {
  content = 'Vivamus sagittis lacus vel augue laoreet rutrum faucibus.';
}
