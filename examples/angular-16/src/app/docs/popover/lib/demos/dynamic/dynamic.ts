import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-popover-dynamic',
  templateUrl: './dynamic.html'
})
export class DemoPopoverDynamicComponent {
  title = 'Welcome word';
  content = 'Vivamus sagittis lacus vel augue laoreet rutrum faucibus.';
}
