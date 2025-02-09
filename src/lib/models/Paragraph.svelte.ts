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
import type { JSONContent } from "@tiptap/core";

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
      paragraph.#fractionalIndex = data.fractionalIndex;
      paragraph.doc = JSON.parse(data.doc);
      paragraph.#updatedAt = new Date(data.updatedAt);
      paragraph.#hidden = data.hidden;
      paragraph.#deleted = data.deleted;
      paragraph.#outlineId = data.outlineId;
      paragraph.#outlineRef = new WeakRef(outline);
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

    paragraph.#ydoc = new Y.Doc();
    paragraph.#ydoc.on("updateV2", (u) => {
      paragraph.#pendingYUpdates.push(u);
      Paragraph.#commands
        .insertPendingYUpdate(paragraph.id, uint8ArrayToBase64URL(u))
        .then(unwrap);
    });

    const yFractionalIndex = paragraph.#ydoc.getText("fractionalIndex");
    yFractionalIndex.insert(0, outline.fractionalIndex);

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
  #fractionalIndex = $state<string>() as string;
  #doc = $state<JSONContent>() as JSONContent;
  #updatedAt = $state<Readonly<Date>>() as Readonly<Date>;
  #hidden = $state() as boolean;
  #deleted = $state() as boolean;
  #outlineId: string;
  #outlineRef = $state<WeakRef<Outline> | undefined>(undefined);
  readonly #path = $state<Path | undefined>(undefined); //allow update only through setter
  readonly #quote = $state<Quote | null>(null); //allow update only through setter
  readonly #links = $state<Readonly<Links>>() as Links; //allow update only through setter
  #ydoc: Y.Doc | undefined;
  #pendingYUpdates: Uint8Array[] = [];

  private constructor(data: RawParagraph, outline: Outline) {
    this.id = data.id;
    this.createdAt = new Date(data.createdAt);
    this.#fractionalIndex = data.fractionalIndex;
    this.doc = JSON.parse(data.doc);
    this.#hidden = data.hidden;
    this.#deleted = data.deleted;
    this.#updatedAt = new Date(data.updatedAt);
    this.#outlineId = data.outlineId;
    this.#outlineRef = new WeakRef(outline);
    this.quote = data.quote;
    this.links = data.links;
  }

  get fractionalIndex() {
    return this.#fractionalIndex;
  }

  get doc(): JSONContent {
    return this.#doc;
  }

  get quote() {
    return this.#quote;
  }

  get updatedAt() {
    return this.#updatedAt;
  }

  get links() {
    return this.#links;
  }

  get hidden() {
    return this.#hidden;
  }

  get deleted() {
    return this.#deleted;
  }

  get outlineId() {
    return this.#outlineId;
  }

  get outline(): Outline | null {
    return this.#outlineRef?.deref() ?? null;
  }

  get path(): Promise<Path> {
    if (this.#path) {
      return Promise.resolve(this.#path);
    } else {
      return Paragraph.#commands.fetchPath(this.#outlineId).then((r) => {
        const path = unwrap(r);
        this.path = path;
        return path;
      });
    }
  }

  set doc(value: JSONContent) {
    this.#doc = value;

    Paragraph.#updateQuote(this.id, value);
  }

  private set links(value: Links) {
    // @ts-expect-error allow update only thorugh setter
    this.#links = value;
    Outline.reversedLinkIndex.set(this.id, this.#links);
  }

  private set path(value: Path | null) {
    // @ts-expect-error allow update only through this setter
    this.#path = value;

    if (value) {
      Outline.descendantsIndex.set(this.id, value);
    }
  }

  private set quote(value: Quote | null) {
    // @ts-expect-error allow update only through this setter
    this.#quote = value;
    Paragraph.#reversedQuoteIndex.set(this.id, value);
  }

  updatePath(text: string, depth: number) {
    if (!this.#path) return;

    const link = this.#path[depth];
    if (link) link.text = text;
  }

  async ydoc() {
    if (!this.#ydoc) {
      this.#ydoc = new Y.Doc();
      this.#ydoc.on("updateV2", (u) => {
        this.#pendingYUpdates.push(u);
        void Paragraph.#commands.insertPendingYUpdate(
          this.id,
          uint8ArrayToBase64URL(u),
        );
      });
    }

    const updates = await Paragraph.#commands.fetchYUpdatesByDocId(this.id);

    for (const u of unwrap(updates)) {
      Y.applyUpdateV2(this.#ydoc, base64URLToUint8Array(u));
    }

    return this.#ydoc;
  }

  async save() {
    if (this.#pendingYUpdates.length) {
      await this.outline?.save();
      await Paragraph.#commands.upsertParagraph(
        this.toJSON(),
        this.#pendingYUpdates.map((u) => uint8ArrayToBase64URL(u)),
      );
    }
  }

  async moveTo(target: Outline, index: number | "last") {
    const ydoc = await this.ydoc();

    if (this.#outlineId !== target.id) {
      this.outline?.removeParagraph(this);

      this.#outlineId = target.id;
      this.#outlineRef = new WeakRef(target);

      const yParentId = ydoc.getText("outlineId");
      yParentId.delete(0, yParentId.length);
      if (this.#outlineId) yParentId.insert(0, this.#outlineId);
    }

    const prev =
      target.paragraphs[index === "last" ? target.paragraphs.length - 1 : index]
        ?.fractionalIndex ?? null;
    const next =
      index === "last"
        ? null
        : (target.paragraphs[index]?.fractionalIndex ?? null);
    this.#fractionalIndex = generateKeyBetween(prev, next);

    const yFractionalIndex = ydoc.getText("fractionalIndex");
    yFractionalIndex.delete(0, yFractionalIndex.length);
    yFractionalIndex.insert(0, this.#fractionalIndex);

    target.insertParagraph(this);
  }

  toJSON(): RawParagraph {
    return {
      id: this.id,
      outlineId: this.#outlineId,
      fractionalIndex: this.#fractionalIndex,
      doc: JSON.stringify(this.#doc),
      links: this.#links,
      hidden: this.#hidden,
      deleted: this.#deleted,
      quote: this.#quote,
      createdAt: this.createdAt.getUTCMilliseconds(),
      updatedAt: this.#updatedAt.getUTCMilliseconds(),
    };
  }
}
