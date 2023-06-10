import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-select-on-enter',
  templateUrl: './select-on-enter.html'
})
export class DemoRatingSelectOnEnterComponent {
  max = 10;
  rate = 7;
  isReadonly = false;

  confirmSelection(event: KeyboardEvent) {
    if (event.keyCode === 13 || event.key === 'Enter') {
      this.isReadonly = true;
    }
  }

  resetStars() {
    this.rate = 0;
    this.isReadonly = false;
  }
}
