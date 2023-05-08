import { Component } from '@angular/core';

interface IItemObject {
  id: number;
  name: string;
}

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'complex-datamodel-demo',
  templateUrl: './complex-datamodel.component.html'
})
export class ComplexDatamodelDemoComponent {
  itemObjectsLeft: IItemObject[] = [
    { id: 1, name: 'Windstorm' },
    { id: 2, name: 'Bombasto' },
    { id: 3, name: 'Magneta' }
  ];

  itemObjectsRight: IItemObject[] = [
    { id: 4, name: 'Tornado' },
    { id: 5, name: 'Mr. O' },
    { id: 6, name: 'Tomato' }
  ];
}
