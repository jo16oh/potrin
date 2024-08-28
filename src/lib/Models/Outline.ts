export type RawOutline = {
  id: string;
  parent: string | null;
  text: string;
};

export class Outline {
  private readonly underlying: RawOutline;
  private parentRef: WeakRef<Outline> | undefined;
  children: Outline[] = [];

  constructor(underlying: RawOutline, parent?: Outline) {
    this.underlying = underlying;
    this.parentRef = parent ? new WeakRef(parent) : undefined;
  }

  get id(): string {
    return this.underlying.id;
  }

  get text(): string {
    return this.underlying.text;
  }

  set text(text: string) {
    this.underlying.text = text;
  }

  get parent(): Outline | undefined {
    return this.parentRef ? this.parentRef.deref() : undefined;
  }

  set parent(parent: Outline) {
    this.parentRef = new WeakRef(parent);
  }

  static treeFromArray(data: RawOutline[]): Outline[] {
    const roots: RawOutline[] = [];
    const childrenMap = new Map<string, RawOutline[]>();

    for (const e of data) {
      childrenMap.set(e.id, []);
    }

    for (const e of data) {
      if (!e.parent) {
        roots.push(e);
        continue;
      }
      childrenMap.get(e.parent)?.push(e);
    }

    return roots.map((e) => createTree(e));

    function createTree(root: RawOutline, parent_ref?: Outline): Outline {
      const parent = new Outline(root, parent_ref);
      const children = childrenMap
        .get(root.id)
        ?.map((c) => createTree(c, parent));
      if (children) parent.children = children;

      return parent;
    }
  }
}
