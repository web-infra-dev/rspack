import { NodeBuilder } from '../../core/NodeBuilder';
import { Node } from '../../core/Node';

export class RawNode extends Node {
  constructor(value: Node);

  value: Node;
  nodeType: string;

  generate(builder: NodeBuilder): string;
  copy(source: RawNode): this;
}