import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-popover-events',
  templateUrl: './events.html'
})
export class DemoPopoverEventsComponent {
  message?: string;

  onShown(): void {
    this.message = 'shown';
  }

  onHidden(): void {
    this.message = 'hidden';
  }
}
