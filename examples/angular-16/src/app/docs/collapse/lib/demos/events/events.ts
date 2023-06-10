import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'collapse-demo-events',
  templateUrl: './events.html'
})
export class CollapseDemoEventsComponent {
  isCollapsed = false;
  message = 'expanded';

  collapsed(): void {
    this.message = 'collapsed';
  }

  collapses(): void {
    this.message = 'collapses';
  }

  expanded(): void {
    this.message = 'expanded';
  }

  expands(): void {
    this.message = 'expands';
  }
}
