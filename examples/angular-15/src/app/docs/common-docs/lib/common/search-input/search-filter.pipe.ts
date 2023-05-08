import { Pipe, PipeTransform } from '@angular/core';
import { Routes, Route } from '@angular/router';

@Pipe({ name: 'SearchFilter' })
export class SearchFilterPipe implements PipeTransform {
  transform(value: Routes, text: any): any {
    if (!text) {
      return value;
    }

    const items = value;
    const newItems: any = [];

    items.forEach(function(item: any): void {
      if (!item.children?.length && item.data?.[0]?.toLowerCase().indexOf(text.toLowerCase()) !== -1) {
        newItems.push(item);
      }

      if (item.children?.length) {
        item.children.forEach((childItem: Route) => {
          if (childItem.data?.[0]?.toLowerCase().indexOf(text.toLowerCase()) !== -1) {
            newItems.push(childItem);
          }
        });
      }
    });
    return newItems;
  }
}
