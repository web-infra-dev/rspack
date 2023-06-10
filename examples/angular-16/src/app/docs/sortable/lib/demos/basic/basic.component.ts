import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'basic-demo',
  templateUrl: './basic.component.html'
})
export class DemoBasicComponent {
  itemStringsLeft = [
    'Windstorm',
    'Bombasto',
    'Magneta',
    'Tornado'
  ];

  itemStringsRight = ['Mr. O', 'Tomato'];
}
