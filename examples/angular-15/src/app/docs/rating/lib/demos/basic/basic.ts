import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-rating-basic',
  templateUrl: './basic.html'
})
export class DemoRatingBasicComponent {
  max = 10;
  rate = 7;
  isReadonly = true;
}
