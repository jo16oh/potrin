import {
  base64URLToUint8Array,
  uint8ArrayToBase64URL,
  unwrap,
  uuidv7,
} from "$lib/utils";
import { generateKeyBetween } from "fractional-indexing-jittered";
import {
  type Paragraph as RawParagraph,
  type Quote,
  type Links,
  type Path,
  commands,
} from "../../generated/tauri-commands";
import { Outline } from "./Outline.svelte";
import * as Y from "yjs";
import { ReversedQuoteIndex, WeakRefMap } from "./utils";
import { type JSONContent } from "@tiptap/core";
import type { AnyYMapValue } from "./utilTypes";
import { getParagraphSchema } from "$lib/components/editor/schema";
import { yXmlFragmentToProseMirrorRootNode } from "y-prosemirror";
import type { Schema } from "@tiptap/pm/model";

export type { RawParagraph };

type Commands = Pick<
  typeof commands,
  | "fetchPath"
  | "fetchConflictingOutlineIds"
  | "fetchYUpdatesByDocId"
  | "insertPendingYUpdate"
  | "upsertParagraph"
>;

export class Paragraph {
  static #commands: Commands = commands;
  static buffer: WeakRefMap<string, Paragraph> = new WeakRefMap();
  static #reversedQuoteIndex = new ReversedQuoteIndex(this.buffer);

  static inject(commands: Commands) {
    this.#commands = commands;
  }

  static from(data: RawParagraph, outline: Outline) {
    const paragraph = this.buffer.get(data.id);

    if (paragraph) {
      paragraph._fractionalIndex = data.fractionalIndex;
      paragraph.doc = JSON.parse(data.doc);
      paragraph._updatedAt = new Date(data.updatedAt);
      paragraph._hidden = data.hidden;
      paragraph._deleted = data.deleted;
      paragraph._outlineId = data.outlineId;
      paragraph._outlineRef = new WeakRef(outline);
      paragraph.quote = data.quote;
      paragraph.links = data.links;
      return paragraph;
    } else {
      const paragraph = new Paragraph(data, outline);
      this.buffer.set(paragraph.id, paragraph);
      return paragraph;
    }
  }

  static new(outline: Outline, fractionalIndex?: string): Paragraph {
    const paragraph = Paragraph.from(
      {
        id: uuidv7(),
        outlineId: outline.id,
        fractionalIndex: fractionalIndex ?? generateKeyBetween(null, null),
        doc: '{ "type": "doc", "content": [] }',
        links: {},
        hidden: false,
        deleted: false,
        quote: null,
        createdAt: new Date().getUTCMilliseconds(),
        updatedAt: new Date().getUTCMilliseconds(),
      },
      outline,
    );

    Paragraph.YDocManager.init(paragraph);

    return paragraph;
  }

  static #updateQuote(id_to: string, doc: JSONContent) {
    for (const p of this.#reversedQuoteIndex.get(id_to)) {
      if (p.quote) {
        p.quote.latestDoc = JSON.stringify(doc);
      }
    }
  }

  // static async init() {
  //   await events.paragraphChange.listen((e) => {
  //     const payload = e.payload;
  //
  //     const operation = payload.operation;
  //
  //     if ("insert" in operation) {
  //       for (const { currentValue } of operation.insert.targets) {
  //         const outline = Outline.buffer.get(currentValue.outlineId);
  //         outline?.insertParagraph(Paragraph.from(currentValue, outline));
  //       }
  //     } else if ("update" in operation) {
  //       for (const { currentValue, relatedYUpdates } of operation.update
  //         .targets) {
  //         const paragraph = this.buffer.get(currentValue.id);
  //
  //         if (paragraph) {
  //           paragraph.#fractionalIndex = currentValue.fractionalIndex;
  //           paragraph.#doc = currentValue.doc;
  //           paragraph.#hidden = currentValue.hidden;
  //           paragraph.#deleted = currentValue.deleted;
  //           paragraph.quote = currentValue.quote;
  //           paragraph.links = currentValue.links;
  //
  //           if (currentValue.outlineId !== paragraph.#outlineId) {
  //             paragraph.outline?.removeParagraph(paragraph);
  //             paragraph.#outlineId = currentValue.outlineId;
  //             const outline = Outline.buffer.get(currentValue.outlineId);
  //             if (outline) paragraph.#outlineRef = new WeakRef(outline);
  //           } else {
  //             paragraph.outline?.sortParagraphs();
  //           }
  //
  //           if (paragraph.#ydoc) {
  //             for (const u of relatedYUpdates) {
  //               Y.applyUpdateV2(paragraph.#ydoc, base64URLToUint8Array(u));
  //             }
  //           }
  //         } else {
  //           const outline = Outline.buffer.get(currentValue.outlineId);
  //           if (outline)
  //             outline.insertParagraph(Paragraph.from(currentValue, outline));
  //         }
  //
  //         this.#updateQuote(currentValue.id, currentValue.doc);
  //       }
  //     } else if ("delete" in operation) {
  //       const deletedParagraphs = operation.delete.target_ids
  //         .map((id) => Paragraph.buffer.get(id))
  //         .filter((o) => o !== undefined);
  //
  //       for (const paragraph of deletedParagraphs) {
  //         paragraph.outline?.removeParagraph(paragraph);
  //       }
  //     }
  //   });
  // }

  readonly id: string;
  readonly createdAt: Readonly<Date>;
  private _fractionalIndex = $state<string>() as string;
  private readonly _doc = $state<JSONContent>() as JSONContent;
  private _updatedAt = $state<Readonly<Date>>() as Readonly<Date>;
  private _hidden = $state() as boolean;
  private _deleted = $state() as boolean;
  private _outlineId: string;
  private _outlineRef = $state<WeakRef<Outline> | undefined>();
  private readonly _path = $state<Path | undefined>(); //allow update only through setter
  private readonly _quote = $state<Quote | null>(null); //allow update only through setter
  private readonly _links = $state<Readonly<Links>>() as Links; //allow update only through setter
  private _yDocManager: InstanceType<typeof Paragraph.YDocManager> | undefined;

  private constructor(data: RawParagraph, outline: Outline) {
    this.id = data.id;
    this.createdAt = new Date(data.createdAt);
    this._fractionalIndex = data.fractionalIndex;
    this.doc = JSON.parse(data.doc);
    this._hidden = data.hidden;
    this._deleted = data.deleted;
    this._updatedAt = new Date(data.updatedAt);
    this._outlineId = data.outlineId;
    this._outlineRef = new WeakRef(outline);
    this.quote = data.quote;
    this.links = data.links;
  }

  get fractionalIndex() {
    return this._fractionalIndex;
  }

  get doc(): JSONContent {
    return this._doc;
  }

  get quote() {
    return this._quote;
  }

  get updatedAt() {
    return this._updatedAt;
  }

  get links() {
    return this._links;
  }

  get hidden() {
    return this._hidden;
  }

  get deleted() {
    return this._deleted;
  }

  get outlineId() {
    return this._outlineId;
  }

  get outline(): Outline | null {
    return this._outlineRef?.deref() ?? null;
  }

  get path(): Promise<Path> {
    if (this._path) {
      return Promise.resolve(this._path);
    } else {
      return Paragraph.#commands.fetchPath(this._outlineId).then((r) => {
        const path = unwrap(r);
        this.path = path;
        return path;
      });
    }
  }

  private set doc(value: JSONContent) {
    // @ts-expect-error allow update only thorugh setter
    this._doc = value;
    Paragraph.#updateQuote(this.id, value);
  }

  private set links(value: Links) {
    // @ts-expect-error allow update only thorugh setter
    this._links = value;
    Outline.reversedLinkIndex.set(this.id, this._links);
  }

  private set path(value: Path | null) {
    // @ts-expect-error allow update only through this setter
    this._path = value;

    if (value) {
      Outline.descendantsIndex.set(this.id, value);
    }
  }

  private set quote(value: Quote | null) {
    // @ts-expect-error allow update only through this setter
    this._quote = value;
    Paragraph.#reversedQuoteIndex.set(this.id, value);
  }

  updatePath(text: string, depth: number) {
    if (!this._path) return;

    const link = this._path[depth];
    if (link) link.text = text;
  }

  async yDocManager() {
    if (!this._yDocManager) {
      this._yDocManager = new Paragraph.YDocManager(this);

      const updates = await Paragraph.#commands
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
        await this.outline?.save();
        const updates = yDocManager.pendingYUpdates.map(uint8ArrayToBase64URL);
        await Paragraph.#commands
          .upsertParagraph(this.toJSON(), updates)
          .then(unwrap)
          .then(() => yDocManager.pendingYUpdates.splice(0, length));
      }
    }
  }

  // async moveTo(target: Outline, index: number | "last") {
  //   const ydoc = await this.ydoc();
  //
  //   if (this._outlineId !== target.id) {
  //     this.outline?.removeParagraph(this);
  //
  //     this._outlineId = target.id;
  //     this._outlineRef = new WeakRef(target);
  //
  //     const yParentId = ydoc.getText("outlineId");
  //     yParentId.delete(0, yParentId.length);
  //     if (this._outlineId) yParentId.insert(0, this._outlineId);
  //   }
  //
  //   const prev =
  //     target.paragraphs[index === "last" ? target.paragraphs.length - 1 : index]
  //       ?.fractionalIndex ?? null;
  //   const next =
  //     index === "last"
  //       ? null
  //       : (target.paragraphs[index]?.fractionalIndex ?? null);
  //   this._fractionalIndex = generateKeyBetween(prev, next);
  //
  //   const yFractionalIndex = ydoc.getText("fractionalIndex");
  //   yFractionalIndex.delete(0, yFractionalIndex.length);
  //   yFractionalIndex.insert(0, this._fractionalIndex);
  //
  //   target.insertParagraph(this);
  // }

  toJSON(): RawParagraph {
    return {
      id: this.id,
      outlineId: this._outlineId,
      fractionalIndex: this._fractionalIndex,
      doc: JSON.stringify(this._doc),
      links: this._links,
      hidden: this._hidden,
      deleted: this._deleted,
      quote: this._quote,
      createdAt: this.createdAt.getUTCMilliseconds(),
      updatedAt: this._updatedAt.getUTCMilliseconds(),
    };
  }

  private static YDocManager = class {
    readonly yDoc: Y.Doc;
    readonly yMap: Y.Map<AnyYMapValue>;
    readonly pendingYUpdates: Uint8Array[] = [];
    readonly #paragraphRef: WeakRef<Paragraph>;
    readonly #schema: Schema;

    static init(paragraph: Paragraph) {
      const yDocManager = new Paragraph.YDocManager(paragraph);
      paragraph._yDocManager = yDocManager;

      yDocManager.yDoc.transact(() => {
        yDocManager.yMap.set("outlineId", paragraph._outlineId);
        yDocManager.yMap.set("fractionalIndex", paragraph._fractionalIndex);
        yDocManager.yMap.set("doc", new Y.XmlFragment());
        yDocManager.yMap.set("links", new Y.Map());
        yDocManager.yMap.set("quote", paragraph._quote);
        yDocManager.yMap.set("hidden", paragraph._hidden);
        yDocManager.yMap.set("deleted", paragraph._deleted);
      });
    }

    constructor(paragraph: Paragraph) {
      this.yDoc = new Y.Doc();
      this.yMap = this.yDoc.getMap<AnyYMapValue>("potrin");
      this.#paragraphRef = new WeakRef(paragraph);
      this.#schema = getParagraphSchema();

      this.yDoc.on("updateV2", async (u, origin) => {
        if (origin === "database") return;
        if (!this.paragraph) return;

        this.pendingYUpdates.push(u);
        Paragraph.#commands
          .insertPendingYUpdate(this.paragraph.id, uint8ArrayToBase64URL(u))
          .then(unwrap);

        this.paragraph.doc = yXmlFragmentToProseMirrorRootNode(
          this.doc,
          this.#schema,
        ).toJSON();
      });
    }

    get paragraph() {
      return this.#paragraphRef.deref();
    }

    get doc(): Y.XmlFragment {
      return this.yMap.get("doc") as Y.XmlFragment;
    }

    set outlineId(value: string) {
      this.yMap.set("outlineId", value);
      if (this.paragraph) {
        this.paragraph._outlineId = value;
      }
    }

    set fractionalIndex(value: string) {
      this.yMap.set("fractionalIndex", value);
      if (this.paragraph) {
        this.paragraph._fractionalIndex = value;
      }
    }

    set links(value: Links) {
      this.yMap.set("links", value);
      if (this.paragraph) {
        this.paragraph.links = value;
      }
    }

    set quote(value: Quote | null) {
      this.yMap.set("quote", value);
      if (this.paragraph) {
        this.paragraph.quote = value;
      }
    }

    set hidden(value: boolean) {
      this.yMap.set("hidden", value);
      if (this.paragraph) {
        this.paragraph._hidden = value;
      }
    }

    set deleted(value: boolean) {
      this.yMap.set("deleted", value);
      if (this.paragraph) {
        this.paragraph._deleted = value;
      }
    }
  };
}
