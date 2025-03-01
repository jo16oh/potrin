import {
  base64URLToUint8Array,
  insertToFractionalIndexedArray,
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
} from "../../generated/tauri-commands";
import { Paragraph } from "./Paragraph.svelte";
import * as Y from "yjs";
import { generateKeyBetween } from "fractional-indexing-jittered";
import { DescendantsIndex, ReversedLinkIndex, WeakRefMap } from "./utils";
import type { JSONContent } from "@tiptap/core";
import type { AnyYMapValue } from "./utils";
import { getOutlineSchema } from "$lib/components/editor/schema";
import { yXmlFragmentToProseMirrorRootNode } from "y-prosemirror";
import type { Schema } from "@tiptap/pm/model";

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
      outline._fractionalIndex = data.fractionalIndex;
      outline._doc = JSON.parse(data.doc);
      outline.text = data.text;
      outline.links = data.links;
      outline._hidden = data.hidden;
      outline._collapsed = data.collapsed;
      outline._deleted = data.deleted;
      outline._updatedAt = new Date(data.updatedAt);
      outline._parentId = data.parentId;
      if (parent) outline.parentRef = new WeakRef(parent);
      if (data.path) outline.path = data.path;
      return outline;
    } else {
      const outline = new Outline(data, parent);
      this.buffer.set(data.id, outline);
      return outline;
    }
  }

  static async new(parent?: Outline): Promise<Outline> {
    const id = uuidv7();

    const linkToThis = { id: id, text: "", hidden: false };
    const path = parent
      ? parent._path
        ? [...parent._path, linkToThis]
        : [...(await commands.fetchPath(parent.id).then(unwrap)), linkToThis]
      : [linkToThis];

    const outline = Outline.from(
      {
        id: id,
        parentId: parent ? parent.id : null,
        fractionalIndex: generateKeyBetween(null, null),
        doc: '{ "type": "doc", "content": [] }',
        text: "",
        links: {},
        path: path,
        hidden: false,
        collapsed: false,
        deleted: false,
        createdAt: new Date().getTime(),
        updatedAt: new Date().getTime(),
      },
      parent,
    );

    Outline.YDocManager.init(outline);

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
        paragraphsMap.set(e.outlineId, [e]);
      }
    }

    if (!root) throw new Error("root outline not found");
    return this.createTree(root, childrenMap, paragraphsMap);
  }

  private static createTree(
    parentData: RawOutline,
    childrenMap: Map<string, RawOutline[]>,
    paragraphsMap: Map<string, RawParagraph[]>,
    parentRef?: Outline,
  ): Outline {
    const parent = Outline.from(parentData, parentRef);

    parent._children =
      childrenMap
        .get(parentData.id)
        ?.map((c) => this.createTree(c, childrenMap, paragraphsMap, parent))
        .sort(byFractionalIndex) ?? [];

    parent._paragraphs =
      paragraphsMap
        .get(parentData.id)
        ?.map((c) => Paragraph.from(c, parent))
        .sort(byFractionalIndex) ?? [];

    return parent;
  }

  static #updateLinks(id_to: string, path: Path) {
    const backlinks = Outline.reversedLinkIndex.get(id_to);

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

  // static async init() {
  //   await events.outlineChange.listen((e) => {
  //     const payload = e.payload;
  //
  //     const operation = payload.operation;
  //
  //     if ("insert" in operation) {
  //       for (const { currentValue } of operation.insert.targets) {
  //         if (!currentValue.parentId) continue;
  //         const parent = this.buffer.get(currentValue.parentId);
  //         if (parent) parent.insertChild(Outline.from(currentValue, parent));
  //         this.#updateLinks(currentValue.id, currentValue.path);
  //       }
  //     } else if ("update" in operation) {
  //       for (const { currentValue, relatedYUpdates } of operation.update
  //         .targets) {
  //         const outline = this.buffer.get(currentValue.id);
  //
  //         if (outline) {
  //           outline.#fractionalIndex = currentValue.fractionalIndex;
  //           outline.#doc = currentValue.doc;
  //           outline.#text = currentValue.text;
  //           outline.links = currentValue.links;
  //           outline.path = currentValue.path;
  //           outline.#hidden = currentValue.hidden;
  //           outline.#collapsed = currentValue.collapsed;
  //           outline.#deleted = currentValue.deleted;
  //
  //           if (currentValue.parentId !== outline.#parentId) {
  //             const parent = currentValue.parentId
  //               ? this.buffer.get(currentValue.parentId)
  //               : undefined;
  //
  //             if (parent) {
  //               outline.parent?.removeChild(outline);
  //               outline.#parentId = currentValue.parentId;
  //               outline.parentRef = new WeakRef(parent);
  //               parent.insertChild(outline);
  //             } else {
  //               outline.parent?.removeChild(outline);
  //               outline.#parentId = currentValue.parentId;
  //               outline.parentRef = undefined;
  //             }
  //           } else {
  //             outline.parent?.sortChildren();
  //           }
  //
  //           if (outline.#ydoc) {
  //             for (const u of relatedYUpdates) {
  //               Y.applyUpdateV2(outline.#ydoc, base64URLToUint8Array(u));
  //             }
  //           }
  //         } else if (currentValue.parentId) {
  //           const parent = this.buffer.get(currentValue.parentId);
  //           parent?.insertChild(Outline.from(currentValue, parent));
  //         }
  //
  //         Outline.#updatePath(
  //           currentValue.id,
  //           currentValue.text,
  //           currentValue.path.length - 1,
  //         );
  //
  //         this.#updateLinks(currentValue.id, currentValue.path);
  //       }
  //     } else if ("delete" in operation) {
  //       const deletedOutlines = operation.delete.target_ids
  //         .map((id) => Outline.buffer.get(id))
  //         .filter((o) => o !== undefined);
  //
  //       for (const o of deletedOutlines) {
  //         o.parent?.removeChild(o);
  //       }
  //     }
  //   });
  // }

  readonly id: string;
  readonly createdAt: Readonly<Date>;
  private _fractionalIndex: string;
  private _doc = $state<JSONContent>();
  private readonly _text: string = $state("");
  private _updatedAt = $state<Readonly<Date>>() as Readonly<Date>;
  private _children = $state.raw<Outline[]>() as Outline[];
  private _paragraphs = $state.raw<Paragraph[]>([]);
  private _parentId = $state<string | null>(null);
  readonly _parentRef = $state.raw<WeakRef<Outline> | undefined>(); // allow update only through setter
  readonly _path = $state<Path | null>(null); // allow update only through setter
  readonly _links = $state<Readonly<Links>>() as Links; // allow update only through setter
  private _hidden = $state() as boolean;
  private _collapsed = $state() as boolean;
  private _deleted = $state() as boolean;
  private _yDocManager: InstanceType<typeof Outline.YDocManager> | undefined;
  readonly #conflictChecker: ConflictChecker;

  private constructor(data: RawOutline, parent?: Outline) {
    this.id = data.id;
    this._fractionalIndex = data.fractionalIndex;
    this._doc = JSON.parse(data.doc);
    // setting path before setting text because the setter of text depends on path
    this.path = data.path;
    this.text = data.text;
    this.createdAt = new Date(data.createdAt);
    this._updatedAt = new Date(data.updatedAt);
    this.links = data.links;
    this._hidden = data.hidden;
    this._collapsed = data.collapsed;
    this._deleted = data.deleted;
    this._parentId = data.parentId;
    this.parentRef = parent ? new WeakRef(parent) : undefined;
    this.#conflictChecker = ConflictChecker.get(this.id);
  }

  get fractionalIndex() {
    return this._fractionalIndex;
  }

  get doc() {
    return this._doc;
  }

  get text() {
    return this._text;
  }

  get updatedAt() {
    return this._updatedAt;
  }

  get links(): Readonly<Links> {
    return this._links;
  }

  get hidden() {
    return this._hidden;
  }

  get collapsed() {
    return this._collapsed;
  }

  get deleted() {
    return this._deleted;
  }

  get parentId() {
    return this._parentId;
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

  get conflict(): boolean {
    if (this._text.length === 0) {
      return false;
    } else if (this.parent) {
      return this.parent.#conflictChecker.check(this._text);
    } else if (this.parentId) {
      const checker = ConflictChecker.get(this.parentId);
      return checker.check(this._text);
    } else {
      const checker = ConflictChecker.get(ConflictChecker.root);
      return checker.check(this._text);
    }
  }

  get path(): Promise<Path> {
    if (this._path) {
      return Promise.resolve(this._path);
    } else {
      return Outline.#commands.fetchPath(this.id).then((r) => {
        const path = unwrap(r);
        this.path = path;
        return path;
      });
    }
  }

  set text(value: string) {
    // @ts-expect-error allow update only through this setter
    this._text = value;

    (async () => {
      const path = await this.path;
      path.at(-1)!.text = value;
      Outline.#updatePath(this.id, this._text, path.length - 1);
      Outline.#updateLinks(this.id, path);
    })();
  }

  private set parentRef(value: WeakRef<Outline> | undefined) {
    // @ts-expect-error allow update only through this setter
    this._parentRef = value;
    this.#registarTextToConflictChecker();
  }

  private set path(value: Path | null) {
    // @ts-expect-error allow update only through this setter
    this._path = value;

    if (value) {
      Outline.descendantsIndex.set(this.id, value);
    }
  }

  private set links(value: Links) {
    // @ts-expect-error allow update only through this setter
    this._links = value;
    Outline.reversedLinkIndex.set(this.id, value);
  }

  updatePath(text: string, depth: number) {
    if (!this._path) return;

    const link = this._path[depth];
    if (link) link.text = text;
  }

  #registarTextToConflictChecker() {
    if (this.parent) {
      this.parent.#conflictChecker.set(this.id, this._text);
    } else if (this.parentId) {
      const checker = ConflictChecker.get(this.parentId);
      checker.set(this.id, this._text);
      if (this._text) {
        Outline.#commands
          .fetchConflictingOutlineIds(this.id, this.parentId, this._text)
          .then(unwrap)
          .then((r) => {
            for (const [id, text] of r) {
              checker.set(id, text);
            }
            checker.reconcile(r.map((o) => o[0]));
          });
      }
    } else {
      const checker = ConflictChecker.get(ConflictChecker.root);
      checker.set(this.id, this._text);
      if (this._text) {
        void Outline.#commands
          .fetchConflictingOutlineIds(this.id, this.parentId, this._text)
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

  async yDocManager() {
    if (!this._yDocManager) {
      this._yDocManager = new Outline.YDocManager(this);

      const updates = await Outline.#commands
        .fetchYUpdatesByDocId(this.id)
        .then(unwrap);

      for (const u of updates) {
        Y.applyUpdateV2(
          this._yDocManager.yDoc,
          base64URLToUint8Array(u),
          "database",
        );
      }
    }

    return this._yDocManager;
  }

  async save() {
    const yDocManager = this._yDocManager;
    if (yDocManager) {
      const length = yDocManager.pendingYUpdates.length;
      if (length) {
        const updates = yDocManager.pendingYUpdates.map(uint8ArrayToBase64URL);
        await Outline.#commands
          .upsertOutline(this.toJSON(), updates)
          .then(unwrap)
          .then(() => updates.splice(0, length));
      }
    }
  }

  sortChildren() {
    this._children = [...this._children.sort(byFractionalIndex)];
  }

  sortParagraphs() {
    this._paragraphs = [...this._paragraphs.sort(byFractionalIndex)];
  }

  // async moveTo(target: Outline | "root", index: number | "last") {
  //   const ydoc = await this.ydoc();
  //
  //   if (target !== "root" && this.parentId !== target.id) {
  //     this.parent?.removeChild(this);
  //
  //     this._parentId = target.id;
  //     this.parentRef = new WeakRef(target);
  //
  //     const yParentId = ydoc.getText("parentId");
  //     yParentId.delete(0, yParentId.length);
  //     if (this.parentId) yParentId.insert(0, this.parentId);
  //   } else if (target === "root") {
  //     this.parent?.removeChild(this);
  //
  //     this._parentId = null;
  //     this.parentRef = undefined;
  //
  //     const yParentId = ydoc.getText("parentId");
  //     yParentId.delete(0, yParentId.length);
  //
  //     return;
  //   }
  //
  //   const prev =
  //     target._children[index === "last" ? target._children.length - 1 : index]
  //       ?.fractionalIndex ?? null;
  //   const next =
  //     index === "last"
  //       ? null
  //       : (target._children[index]?.fractionalIndex ?? null);
  //   this._fractionalIndex = generateKeyBetween(prev, next);
  //
  //   const yFractionalIndex = ydoc.getText("fractionalIndex");
  //   yFractionalIndex.delete(0, yFractionalIndex.length);
  //   yFractionalIndex.insert(0, this._fractionalIndex);
  //
  //   target.insertChild(this);
  // }

  insertChild(child: Outline) {
    this._children = [...insertToFractionalIndexedArray(this._children, child)];
    this.#conflictChecker.reconcile(this._children.map((c) => c.id));
  }

  insertParagraph(paragraph: Paragraph) {
    this._paragraphs = [
      ...insertToFractionalIndexedArray(this._paragraphs, paragraph),
    ];
  }

  removeChild(child: Outline) {
    const idx = this._children.findIndex((c) => c.id === child.id);
    this._children = this._children.toSpliced(idx, 1);
    this.#conflictChecker.reconcile(this._children.map((c) => c.id));
  }

  removeParagraph(paragraph: Paragraph) {
    const idx = this._paragraphs.findIndex((c) => c.id === paragraph.id);
    this._paragraphs = this._paragraphs.toSpliced(idx, 1);
  }

  flatten(): Outline[] {
    return [this, ...this._children.map((c) => c.flatten()).flat()];
  }

  toJSON(): RawOutline {
    return {
      id: this.id,
      parentId: this.parentId,
      fractionalIndex: this._fractionalIndex,
      doc: JSON.stringify(this._doc),
      text: this._text,
      path: this._path ? this._path : null,
      links: this.links,
      hidden: this._hidden,
      collapsed: this._collapsed,
      deleted: this._deleted,
      createdAt: this.createdAt.getTime(),
      updatedAt: this._updatedAt.getTime(),
    };
  }

  private static YDocManager = class {
    readonly yDoc: Y.Doc;
    readonly yMap: Y.Map<AnyYMapValue>;
    readonly pendingYUpdates: Uint8Array[] = [];
    readonly #outlineRef: WeakRef<Outline>;
    readonly #schema: Schema;

    static init(outline: Outline) {
      const yDocManager = new Outline.YDocManager(outline);
      outline._yDocManager = yDocManager;

      yDocManager.yDoc.transact(() => {
        yDocManager.yMap.set("parentId", outline._parentId);
        yDocManager.yMap.set("fractionalIndex", outline._fractionalIndex);
        yDocManager.yMap.set("doc", new Y.XmlFragment());
        yDocManager.yMap.set("text", outline._text);
        yDocManager.yMap.set("links", new Y.Map());
        yDocManager.yMap.set("hidden", outline._hidden);
        yDocManager.yMap.set("collapsed", outline._collapsed);
        yDocManager.yMap.set("deleted", outline._deleted);
      });
    }

    constructor(outline: Outline) {
      this.yDoc = new Y.Doc();
      this.yMap = this.yDoc.getMap<AnyYMapValue>("potrin");
      this.#outlineRef = new WeakRef(outline);
      this.#schema = getOutlineSchema();

      this.yDoc.on("updateV2", async (u, origin) => {
        if (origin === "database") return;
        if (!this.outline) return;

        this.pendingYUpdates.push(u);
        Outline.#commands
          .insertPendingYUpdate(this.outline.id, uint8ArrayToBase64URL(u))
          .then(unwrap);

        const node = yXmlFragmentToProseMirrorRootNode(this.doc, this.#schema);

        this.outline._doc = node.toJSON();
        this.outline.text = node.textContent;
      });
    }

    get outline() {
      return this.#outlineRef.deref();
    }

    get doc(): Y.XmlFragment {
      return this.yMap.get("doc") as Y.XmlFragment;
    }

    set parentId(value: string | null) {
      this.yMap.set("parentId", value);
      if (this.outline) {
        this.outline._parentId = value;
      }
    }

    set fractionalIndex(value: string) {
      this.yMap.set("fractionalIndex", value);
      if (this.outline) {
        this.outline._fractionalIndex = value;
      }
    }

    set text(value: string) {
      this.yMap.set("text", value);
      if (this.outline) {
        this.outline.text = value;
      }
    }

    set links(value: Links) {
      this.yMap.set("links", value);
      if (this.outline) {
        this.outline.links = value;
      }
    }

    set hidden(value: boolean) {
      this.yMap.set("hidden", value);
      if (this.outline) {
        this.outline._hidden = value;
      }
    }

    set collapsed(value: boolean) {
      this.yMap.set("collapsed", value);
      if (this.outline) {
        this.outline._collapsed = value;
      }
    }

    set deleted(value: boolean) {
      this.yMap.set("deleted", value);
      if (this.outline) {
        this.outline._deleted = value;
      }
    }
  };
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
