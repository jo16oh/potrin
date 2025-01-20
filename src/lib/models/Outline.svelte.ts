import {
  base64URLToUint8Array,
  insertToFractionalIndexArray,
  uint8ArrayToBase64URL,
  byFractionalIndex,
  uuidv7,
  unwrap,
} from "$lib/utils";
import { SvelteSet } from "svelte/reactivity";
import {
  type Links,
  type Outline as RawOutline,
  type Paragraph as RawParagraph,
  type Path,
  commands,
  events,
} from "../../generated/tauri-commands";
import { Paragraph } from "./Paragraph.svelte";
import * as Y from "yjs";
import { generateKeyBetween } from "fractional-indexing-jittered";
import { DescendantsIndex, ReversedLinkIndex, WeakRefMap } from "./utils";

export type { RawOutline };

type Commands = Pick<
  typeof commands,
  | "fetchPath"
  | "fetchConflictingOutlineIds"
  | "fetchYUpdatesByDocId"
  | "insertPendingYUpdate"
  | "upsertOutline"
>;

export class Outline {
  static #commands: Commands = commands;
  static readonly buffer = new WeakRefMap<string, Outline>();
  static readonly reversedLinkIndex = new ReversedLinkIndex(this.buffer);
  static readonly descendantsIndex = new DescendantsIndex(
    Outline.buffer,
    Paragraph.buffer,
  );

  static inject(commands: Commands) {
    this.#commands = commands;
  }

  static from(data: RawOutline, parent?: Outline) {
    const outline = this.buffer.get(data.id);

    if (outline) {
      outline.#fractionalIndex = data.fractionalIndex;
      outline.#doc = data.doc;
      outline.links = data.links;
      outline.#text = data.text;
      outline.#updatedAt = new Date(data.updatedAt);
      outline.#parentId = data.parentId;
      if (parent) outline.parentRef = new WeakRef(parent);
      if (data.path) outline.path = data.path;
      return outline;
    } else {
      const outline = new Outline(data, parent);
      this.buffer.set(data.id, outline);
      return outline;
    }
  }

  static new(): Outline {
    const outline = Outline.from({
      id: uuidv7(),
      parentId: null,
      fractionalIndex: generateKeyBetween(null, null),
      doc: "",
      text: "",
      links: {},
      path: null,
      createdAt: new Date().getUTCMilliseconds(),
      updatedAt: new Date().getUTCMilliseconds(),
    });

    outline.#ydoc = new Y.Doc();
    outline.#ydoc.on("updateV2", (u) => {
      outline.#pendingYUpdates.push(u);
      void Outline.#commands.insertPendingYUpdate(
        outline.id,
        uint8ArrayToBase64URL(u),
      );
    });

    const fractionalIndex = outline.#ydoc.getText("fractionalIndex");
    fractionalIndex.insert(0, outline.#fractionalIndex);

    return outline;
  }

  static tree(
    outlines: RawOutline[],
    paragraphs: RawParagraph[],
    rootId: string,
  ): Outline {
    let root: RawOutline | undefined;
    const childrenMap = new Map<string, RawOutline[]>();
    const paragraphsMap = new Map<string, RawParagraph[]>();

    for (const e of outlines) {
      if (e.id === rootId) root = e;
      if (!e.parentId) continue;

      const children = childrenMap.get(e.parentId);

      if (children) {
        children.push(e);
      } else {
        childrenMap.set(e.parentId, [e]);
      }
    }

    for (const e of paragraphs) {
      const paragraphs = paragraphsMap.get(e.outlineId);
      if (paragraphs) {
        paragraphs.push(e);
      } else {
        paragraphsMap.set(e.id, []);
      }
    }

    if (!root) throw new Error("root outline not found");
    return this.createTree(root, childrenMap, paragraphsMap);
  }

  private static createTree(
    root: RawOutline,
    childrenMap: Map<string, RawOutline[]>,
    paragraphsMap: Map<string, RawParagraph[]>,
    parent_ref?: Outline,
  ): Outline {
    const parent = Outline.from(root, parent_ref);

    parent.#children =
      childrenMap
        .get(root.id)
        ?.map((c) => this.createTree(c, childrenMap, paragraphsMap, parent))
        .sort(byFractionalIndex) ?? [];

    parent.#paragraphs =
      paragraphsMap
        .get(root.id)
        ?.map((c) => Paragraph.from(c, parent))
        .sort(byFractionalIndex) ?? [];

    return parent;
  }

  static #updateLinks(id_to: string, path: Path) {
    const backlinks = [
      ...Outline.reversedLinkIndex.get(id_to),
      ...Paragraph.reversedLinkIndex.get(id_to),
    ];

    for (const o of backlinks) {
      o.links = {
        ...o.links,
        [id_to]: path,
      };
    }
  }

  static #updatePath(id: string, text: string, depth: number) {
    const descendants = Outline.descendantsIndex.get(id);

    for (const o of descendants) {
      o.updatePath(text, depth);
    }
  }

  static async init() {
    await events.outlineChange.listen((e) => {
      const payload = e.payload;

      const operation = payload.operation;

      if ("insert" in operation) {
        for (const { currentValue } of operation.insert.targets) {
          if (!currentValue.parentId) continue;
          const parent = this.buffer.get(currentValue.parentId);
          if (parent) parent.insertChild(Outline.from(currentValue, parent));
          this.#updateLinks(currentValue.id, currentValue.path);
        }
      } else if ("update" in operation) {
        for (const { currentValue, relatedYUpdates } of operation.update
          .targets) {
          const outline = this.buffer.get(currentValue.id);

          if (outline) {
            outline.#fractionalIndex = currentValue.fractionalIndex;
            outline.#doc = currentValue.doc;
            outline.links = currentValue.links;
            outline.path = currentValue.path;

            if (outline.#text !== currentValue.text) {
              outline.#text = currentValue.text;
              Outline.#updatePath(
                currentValue.id,
                currentValue.text,
                currentValue.path.length - 1,
              );
            }

            if (currentValue.parentId !== outline.#parentId) {
              const parent = currentValue.parentId
                ? this.buffer.get(currentValue.parentId)
                : undefined;

              if (parent) {
                outline.parent?.removeChild(outline);
                outline.#parentId = currentValue.parentId;
                outline.parentRef = new WeakRef(parent);
                parent.insertChild(outline);
              } else {
                outline.parent?.removeChild(outline);
                outline.#parentId = currentValue.parentId;
                outline.parentRef = undefined;
              }
            } else {
              outline.parent?.sortChildren();
            }

            if (outline.#ydoc) {
              for (const u of relatedYUpdates) {
                Y.applyUpdateV2(outline.#ydoc, base64URLToUint8Array(u));
              }
            }
          } else if (currentValue.parentId) {
            const parent = this.buffer.get(currentValue.parentId);
            parent?.insertChild(Outline.from(currentValue, parent));
          }

          this.#updateLinks(currentValue.id, currentValue.path);
        }
      } else if ("delete" in operation) {
        const deletedOutlines = operation.delete.target_ids
          .map((id) => Outline.buffer.get(id))
          .filter((o) => o !== undefined);

        for (const o of deletedOutlines) {
          o.parent?.removeChild(o);
        }
      }
    });
  }

  readonly id: string;
  readonly createdAt: Readonly<Date>;
  #fractionalIndex: string;
  #doc = $state<string>() as string;
  #text: string;
  #updatedAt = $state<Readonly<Date>>() as Readonly<Date>;
  #children = $state.raw<Outline[]>() as Outline[];
  #paragraphs = $state.raw<Paragraph[]>() as Paragraph[];
  #parentId = $state<string | null>(null);
  readonly #parentRef = $state.raw<WeakRef<Outline> | undefined>(); // allow update only through setter
  readonly #path = $state<Path | null>(null); // allow update only through setter
  readonly #links = $state<Readonly<Links>>() as Links; // allow update only through setter
  #ydoc: Y.Doc | undefined;
  #pendingYUpdates: Uint8Array[] = [];
  readonly #conflictChecker: ConflictChecker;

  private constructor(data: RawOutline, parent?: Outline) {
    this.id = data.id;
    this.#fractionalIndex = data.fractionalIndex;
    this.#doc = data.doc;
    this.#text = data.text;
    this.createdAt = new Date(data.createdAt);
    this.#updatedAt = new Date(data.updatedAt);
    this.path = data.path;
    this.links = data.links;
    this.#parentId = data.parentId;
    this.parentRef = parent ? new WeakRef(parent) : undefined;
    this.#conflictChecker = ConflictChecker.get(this.id);
  }

  get fractionalIndex() {
    return this.#fractionalIndex;
  }

  get doc() {
    return this.#doc;
  }

  get text() {
    return this.#text;
  }

  get updatedAt() {
    return this.#updatedAt;
  }

  get links(): Readonly<Links> {
    return this.#links;
  }

  get parentId() {
    return this.#parentId;
  }

  get parent(): Outline | null {
    return this.#parentRef?.deref() ?? null;
  }

  get children(): Readonly<Outline[]> {
    return this.#children;
  }

  get paragraphs(): Readonly<Paragraph[]> {
    return this.#paragraphs;
  }

  get conflict(): boolean {
    if (this.#text.length === 0) {
      return false;
    } else if (this.parent) {
      return this.parent.#conflictChecker.check(this.#text);
    } else if (this.parentId) {
      const checker = ConflictChecker.get(this.parentId);
      return checker.check(this.#text);
    } else {
      const checker = ConflictChecker.get(ConflictChecker.root);
      return checker.check(this.#text);
    }
  }

  get path(): Promise<Path> {
    if (this.#path) {
      return Promise.resolve(this.#path);
    } else {
      return Outline.#commands.fetchPath(this.id).then((r) => {
        const path = unwrap(r);
        this.path = path;
        return path;
      });
    }
  }

  private set parentRef(value: WeakRef<Outline> | undefined) {
    // @ts-expect-error allow update only through this setter
    this.#parentRef = value;
    this.#registarTextToConflictChecker();
  }

  private set path(value: Path | null) {
    // @ts-expect-error allow update only through this setter
    this.#path = value;

    if (value) {
      Outline.descendantsIndex.set(this.id, value);
    }
  }

  private set links(value: Links) {
    // @ts-expect-error allow update only through this setter
    this.#links = value;
    Outline.reversedLinkIndex.set(this.id, value);
  }

  updatePath(text: string, depth: number) {
    if (!this.#path) return;

    const link = this.#path[depth];
    if (link) link.text = text;
  }

  #registarTextToConflictChecker() {
    if (this.parent) {
      this.parent.#conflictChecker.set(this.id, this.#text);
    } else if (this.parentId) {
      const checker = ConflictChecker.get(this.parentId);
      checker.set(this.id, this.#text);
      if (this.#text) {
        void Outline.#commands
          .fetchConflictingOutlineIds(this.id, this.parentId, this.#text)
          .then((r) => {
            const result = unwrap(r);
            for (const [id, text] of result) {
              checker.set(id, text);
            }
            checker.reconcile(result.map((o) => o[0]));
          });
      }
    } else {
      const checker = ConflictChecker.get(ConflictChecker.root);
      checker.set(this.id, this.#text);
      if (this.#text) {
        void Outline.#commands
          .fetchConflictingOutlineIds(this.id, this.parentId, this.#text)
          .then((r) => {
            const result = unwrap(r);
            for (const [id, text] of result) {
              checker.set(id, text);
            }
            checker.reconcile(result.map((o) => o[0]));
          });
      }
    }
  }

  async ydoc() {
    if (!this.#ydoc) {
      this.#ydoc = new Y.Doc();
      this.#ydoc.on("updateV2", (u) => {
        this.#pendingYUpdates.push(u);
        void Outline.#commands.insertPendingYUpdate(
          this.id,
          uint8ArrayToBase64URL(u),
        );
      });
    }

    const updates = await Outline.#commands.fetchYUpdatesByDocId(this.id);

    for (const u of unwrap(updates)) {
      Y.applyUpdateV2(this.#ydoc, base64URLToUint8Array(u));
    }

    return this.#ydoc;
  }

  async save() {
    const updates = this.#pendingYUpdates.map((u) => uint8ArrayToBase64URL(u));
    this.#pendingYUpdates.length = 0;
    await Outline.#commands.upsertOutline(this.toJSON(), updates);
  }

  sortChildren() {
    this.#children = [...this.#children.sort(byFractionalIndex)];
  }

  sortParagraphs() {
    this.#paragraphs = [...this.#paragraphs.sort(byFractionalIndex)];
  }

  async moveTo(target: Outline | "root", index: number | "last") {
    const ydoc = await this.ydoc();

    if (target !== "root" && this.parentId !== target.id) {
      this.parent?.removeChild(this);

      this.#parentId = target.id;
      this.parentRef = new WeakRef(target);

      const yParentId = ydoc.getText("parentId");
      yParentId.delete(0, yParentId.length);
      if (this.parentId) yParentId.insert(0, this.parentId);
    } else if (target === "root") {
      this.parent?.removeChild(this);

      this.#parentId = null;
      this.parentRef = undefined;

      const yParentId = ydoc.getText("parentId");
      yParentId.delete(0, yParentId.length);

      return;
    }

    const prev =
      target.#children[index === "last" ? target.#children.length - 1 : index]
        ?.fractionalIndex ?? null;
    const next =
      index === "last"
        ? null
        : (target.#children[index]?.fractionalIndex ?? null);
    this.#fractionalIndex = generateKeyBetween(prev, next);

    const yFractionalIndex = ydoc.getText("fractionalIndex");
    yFractionalIndex.delete(0, yFractionalIndex.length);
    yFractionalIndex.insert(0, this.#fractionalIndex);

    target.insertChild(this);
  }

  insertChild(child: Outline) {
    this.#children = [...insertToFractionalIndexArray(this.#children, child)];
    this.#conflictChecker.reconcile(this.#children.map((c) => c.id));
  }

  insertParagraph(paragraph: Paragraph) {
    this.#paragraphs = [
      ...insertToFractionalIndexArray(this.#paragraphs, paragraph),
    ];
  }

  removeChild(child: Outline) {
    const idx = this.#children.findIndex((c) => c.id === child.id);
    this.#paragraphs = [...this.#paragraphs.splice(idx, 1)];
    this.#conflictChecker.reconcile(this.#children.map((c) => c.id));
  }

  removeParagraph(paragraph: Paragraph) {
    const idx = this.#paragraphs.findIndex((c) => c.id === paragraph.id);
    this.#paragraphs = [...this.#paragraphs.splice(idx, 1)];
  }

  flatten(): Outline[] {
    return [this, ...this.#children.map((c) => c.flatten()).flat()];
  }

  toJSON(): RawOutline {
    return {
      id: this.id,
      parentId: this.parentId,
      fractionalIndex: this.#fractionalIndex,
      doc: this.#doc,
      text: this.#text,
      path: this.#path ? this.#path : null,
      links: this.links,
      createdAt: this.createdAt.getUTCMilliseconds(),
      updatedAt: this.#updatedAt.getUTCMilliseconds(),
    };
  }
}

class ConflictChecker {
  static readonly root = Symbol();
  static readonly buffer = new WeakRefMap<string | symbol, ConflictChecker>();
  readonly #textToIdsMap = new Map<string, SvelteSet<string>>();
  readonly #prevTextMap = new Map<string, string>();

  static get(key: string | symbol): ConflictChecker {
    const checker = this.buffer.get(key);
    if (checker) {
      return checker;
    } else {
      const checker = new ConflictChecker();
      this.buffer.set(key, checker);
      return checker;
    }
  }

  set(id: string, text: string | null) {
    // delete prev text
    const prevText = this.#prevTextMap.get(id);
    if (prevText) {
      const idSet = this.#textToIdsMap.get(prevText);
      if (!idSet) return;
      idSet.delete(id);
      if (idSet.size === 0) this.#textToIdsMap.delete(prevText);
    }

    // set new text
    if (text && 0 < text.length) {
      const idSet = this.#textToIdsMap.get(text);

      if (idSet) {
        idSet.add(id);
      } else {
        this.#textToIdsMap.set(text, new SvelteSet([id]));
      }

      this.#prevTextMap.set(id, text);
    }
  }

  check(text: string): boolean {
    const duplications = this.#textToIdsMap.get(text);
    if (!duplications) return false;

    if (duplications.size > 1) {
      return true;
    } else {
      return false;
    }
  }

  reconcile(childrenIds: string[]) {
    const prev = new Set(this.#prevTextMap.keys());
    const current = new Set(childrenIds);

    for (const id of prev.difference(current)) {
      const text = this.#prevTextMap.get(id);
      if (!text) continue;
      this.#prevTextMap.delete(id);
      this.#textToIdsMap.get(text)?.delete(id);
    }
  }
}
