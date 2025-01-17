import {
  base64URLToUint8Array,
  insertToFractionalIndexArray,
  uint8ArrayToBase64URL,
  byFractionalIndex,
  uuidv7,
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
import { ReversedLinkIndex, WeakRefMap } from "./utils";

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

  readonly id: string;
  fractionalIndex: string;
  doc = $state<string>() as string;
  links = $state<Links>() as Links;
  text: string;
  readonly createdAt: Readonly<Date>;
  updatedAt = $state<Readonly<Date>>() as Readonly<Date>;
  private _children = $state.raw<Outline[]>() as Outline[];
  private _paragraphs = $state.raw<Paragraph[]>() as Paragraph[];
  private _parentId = $state<string | null>(null);
  private _parentRef = $state.raw<WeakRef<Outline> | undefined>();
  private _path = $state<Path | undefined>();
  private _ydoc: Y.Doc | undefined;
  private _pendingYUpdates: Uint8Array[] = [];
  private readonly _conflictChecker: ConflictChecker;

  static inject(commands: Commands) {
    this.#commands = commands;
  }

  private constructor(data: RawOutline, parent?: Outline) {
    this.id = data.id;
    this.fractionalIndex = data.fractionalIndex;
    this.doc = data.doc;
    this.links = data.links;
    this.text = data.text;
    this.createdAt = new Date(data.createdAt);
    this.updatedAt = new Date(data.updatedAt);
    this._parentId = data.parentId;
    this._parentRef = parent ? new WeakRef(parent) : undefined;
    this._conflictChecker = ConflictChecker.get(this.id);
  }

  static from(data: RawOutline, parent?: Outline) {
    const outline = this.buffer.get(data.id);

    if (outline) {
      outline.fractionalIndex = data.fractionalIndex;
      outline.doc = data.doc;
      outline.links = data.links;
      outline.text = data.text;
      outline.updatedAt = new Date(data.updatedAt);
      outline._parentId = data.parentId;
      if (parent) outline._parentRef = new WeakRef(parent);
      outline.#initEffect();
      return outline;
    } else {
      const outline = new Outline(data, parent);
      this.buffer.set(data.id, outline);
      outline.#initEffect();
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

    outline._ydoc = new Y.Doc();
    outline._ydoc.on("updateV2", (u) => {
      outline._pendingYUpdates.push(u);
      void Outline.#commands.insertPendingYUpdate(
        outline.id,
        uint8ArrayToBase64URL(u),
      );
    });

    const fractionalIndex = outline._ydoc.getText("fractionalIndex");
    fractionalIndex.insert(0, outline.fractionalIndex);

    return outline;
  }

  #initEffect() {
    $effect(() => {
      if (this.parent) {
        this.parent._conflictChecker.set(this.id, this.text);
      } else if (this._parentId) {
        const checker = ConflictChecker.get(this._parentId);
        checker.set(this.id, this.text);
        if (this.text) {
          void Outline.#commands
            .fetchConflictingOutlineIds(this.id, this._parentId, this.text)
            .then((r) => {
              for (const [id, text] of r) {
                checker.set(id, text);
              }
              checker.reconcile(r.map((o) => o[0]));
            });
        }
      } else {
        const checker = ConflictChecker.get(ConflictChecker.root);
        checker.set(this.id, this.text);
        if (this.text) {
          void Outline.#commands
            .fetchConflictingOutlineIds(this.id, this._parentId, this.text)
            .then((r) => {
              for (const [id, text] of r) {
                checker.set(id, text);
              }
              checker.reconcile(r.map((o) => o[0]));
            });
        }
      }
    });

    $effect(() => {
      this._conflictChecker.reconcile(this._children.map((c) => c.id));
    });

    $effect(() => {
      Outline.reversedLinkIndex.set(this.id, this.links);
    });
  }

  static tree(outlines: RawOutline[], paragraphs: RawParagraph[]): Outline[] {
    const roots: RawOutline[] = [];
    const childrenMap = new Map<string, RawOutline[]>();
    const paragraphsMap = new Map<string, RawParagraph[]>();

    for (const e of outlines) {
      if (!e.parentId) {
        roots.push(e);
        continue;
      }

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

    return roots.map((e) => this.createTree(e, childrenMap, paragraphsMap));
  }

  private static createTree(
    root: RawOutline,
    childrenMap: Map<string, RawOutline[]>,
    paragraphsMap: Map<string, RawParagraph[]>,
    parent_ref?: Outline,
  ): Outline {
    const parent = Outline.from(root, parent_ref);

    parent._children =
      childrenMap
        .get(root.id)
        ?.map((c) => this.createTree(c, childrenMap, paragraphsMap, parent))
        .sort(byFractionalIndex) ?? [];

    parent._paragraphs =
      paragraphsMap
        .get(root.id)
        ?.map((c) => Paragraph.from(c, parent))
        .sort(byFractionalIndex) ?? [];

    return parent;
  }

  get parent(): Outline | null {
    return this._parentRef?.deref() ?? null;
  }

  get children(): Readonly<Outline[]> {
    return this._children;
  }

  get paragraphs(): Readonly<Paragraph[]> {
    return this._paragraphs;
  }

  get parentId() {
    return this._parentId;
  }

  get conflict(): boolean {
    if (this.text.length === 0) {
      return false;
    } else if (this.parent) {
      return this.parent._conflictChecker.check(this.text);
    } else if (this._parentId) {
      const checker = ConflictChecker.get(this._parentId);
      return checker.check(this.text);
    } else {
      const checker = ConflictChecker.get(ConflictChecker.root);
      return checker.check(this.text);
    }
  }

  get path(): Promise<Path> {
    if (this._path) {
      return Promise.resolve(this._path);
    } else {
      return Outline.#commands.fetchPath(this.id).then((path) => {
        this._path = path;
        return path;
      });
    }
  }

  async ydoc() {
    if (!this._ydoc) {
      this._ydoc = new Y.Doc();
      this._ydoc.on("updateV2", (u) => {
        this._pendingYUpdates.push(u);
        void Outline.#commands.insertPendingYUpdate(
          this.id,
          uint8ArrayToBase64URL(u),
        );
      });
    }

    const updates = await Outline.#commands.fetchYUpdatesByDocId(this.id);

    for (const u of updates) {
      Y.applyUpdateV2(this._ydoc, base64URLToUint8Array(u));
    }

    return this._ydoc;
  }

  async save() {
    const updates = this._pendingYUpdates.map((u) => uint8ArrayToBase64URL(u));
    this._pendingYUpdates.length = 0;
    await Outline.#commands.upsertOutline(this.toJSON(), updates);
  }

  sortChildren() {
    this._children = [...this._children.sort(byFractionalIndex)];
  }

  sortParagraphs() {
    this._paragraphs = [...this._paragraphs.sort(byFractionalIndex)];
  }

  async moveTo(target: Outline | "root", index: number | "last") {
    const ydoc = await this.ydoc();

    if (target !== "root" && this._parentId !== target.id) {
      this.parent?._removeChild(this);

      this._parentId = target.id;
      this._parentRef = new WeakRef(target);

      const yParentId = ydoc.getText("parentId");
      yParentId.delete(0, yParentId.length);
      if (this._parentId) yParentId.insert(0, this._parentId);
    } else if (target === "root") {
      this.parent?._removeChild(this);

      this._parentId = null;
      this._parentRef = undefined;

      const yParentId = ydoc.getText("parentId");
      yParentId.delete(0, yParentId.length);

      return;
    }

    const prev =
      target._children[index === "last" ? target._children.length - 1 : index]
        ?.fractionalIndex ?? null;
    const next =
      index === "last"
        ? null
        : (target._children[index]?.fractionalIndex ?? null);
    this.fractionalIndex = generateKeyBetween(prev, next);

    const yFractionalIndex = ydoc.getText("fractionalIndex");
    yFractionalIndex.delete(0, yFractionalIndex.length);
    yFractionalIndex.insert(0, this.fractionalIndex);

    target._insertChild(this);
  }

  _insertChild(child: Outline) {
    this._children = [...insertToFractionalIndexArray(this._children, child)];
  }

  _insertParagraph(paragraph: Paragraph) {
    this._paragraphs = [
      ...insertToFractionalIndexArray(this._paragraphs, paragraph),
    ];
  }

  _removeChild(child: Outline) {
    const idx = this._children.findIndex((c) => c.id === child.id);
    this._paragraphs = [...this._paragraphs.splice(idx, 1)];
  }

  _removeParagraph(paragraph: Paragraph) {
    const idx = this._paragraphs.findIndex((c) => c.id === paragraph.id);
    this._paragraphs = [...this._paragraphs.splice(idx, 1)];
  }

  flatten(): Outline[] {
    return [this, ...this._children.map((c) => c.flatten()).flat()];
  }

  toJSON(): RawOutline {
    return {
      id: this.id,
      parentId: this.parentId,
      fractionalIndex: this.fractionalIndex,
      doc: this.doc,
      text: this.text,
      path: this._path ? this._path : null,
      links: this.links,
      createdAt: this.createdAt.getUTCMilliseconds(),
      updatedAt: this.updatedAt.getUTCMilliseconds(),
    };
  }

  static #updateLinks(id_to: string, path: Path) {
    const backlinks = [
      ...Outline.reversedLinkIndex.get(id_to),
      ...Paragraph.reversedLinkIndex.get(id_to),
    ];
    for (const o of backlinks) {
      o.links[id_to] = path;
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
          if (parent) parent._insertChild(Outline.from(currentValue, parent));
          this.#updateLinks(currentValue.id, currentValue.path);
        }
      } else if ("update" in operation) {
        for (const { currentValue, relatedYUpdates } of operation.update
          .targets) {
          const outline = this.buffer.get(currentValue.id);

          if (outline) {
            outline.fractionalIndex = currentValue.fractionalIndex;
            outline.doc = currentValue.doc;
            outline.text = currentValue.text;
            outline.links = currentValue.links;
            outline._path = currentValue.path;

            if (currentValue.parentId !== outline._parentId) {
              const parent = currentValue.parentId
                ? this.buffer.get(currentValue.parentId)
                : undefined;

              if (parent) {
                outline.parent?._removeChild(outline);
                outline._parentId = currentValue.parentId;
                outline._parentRef = new WeakRef(parent);
                parent._insertChild(outline);
              } else {
                outline.parent?._removeChild(outline);
                outline._parentId = currentValue.parentId;
                outline._parentRef = undefined;
              }
            } else {
              outline.parent?.sortChildren();
            }

            if (outline._ydoc) {
              for (const u of relatedYUpdates) {
                Y.applyUpdateV2(outline._ydoc, base64URLToUint8Array(u));
              }
            }
          } else if (currentValue.parentId) {
            const parent = this.buffer.get(currentValue.parentId);
            parent?._insertChild(Outline.from(currentValue, parent));
          }

          this.#updateLinks(currentValue.id, currentValue.path);
        }
      } else if ("delete" in operation) {
        const deletedOutlines = operation.delete.target_ids
          .map((id) => Outline.buffer.get(id))
          .filter((o) => o !== undefined);

        for (const o of deletedOutlines) {
          o.parent?._removeChild(o);
        }
      }
    });
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
