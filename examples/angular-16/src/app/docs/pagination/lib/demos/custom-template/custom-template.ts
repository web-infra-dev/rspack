import { Component } from '@angular/core';

import { PagesModel } from 'ngx-bootstrap/pagination';

type Roman = {
  [key: string]: number;
};

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-pagination-custom-template',
  templateUrl: './custom-template.html'
})
export class DemoPaginationCustomTemplateComponent {

  convertToRoman(pageInfo: PagesModel): string {

    const roman: Roman = {
      M: 1000,
      CM: 900,
      D: 500,
      CD: 400,
      C: 100,
      XC: 90,
      L: 50,
      XL: 40,
      X: 10,
      IX: 9,
      V: 5,
      IV: 4,
      I: 1
    };

    let pageNumber = pageInfo.number;

    return Object.keys(roman).reduce((acc, symbol) => {
      const numeralSystem = Math.floor(pageNumber / roman[symbol as keyof Roman]);
      pageNumber -= numeralSystem * roman[symbol];

      return acc + symbol.repeat(numeralSystem);
    }, '');
  }
}
