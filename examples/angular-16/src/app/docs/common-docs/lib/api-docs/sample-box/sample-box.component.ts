import { Component, Input } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'ng-sample-box',
  templateUrl: './sample-box.component.html'
})
export class SampleBoxComponent {
  @Input() ts?: string;
  @Input() html?: string;
  @Input() spec?: string;
  @Input() style?: string;
}
