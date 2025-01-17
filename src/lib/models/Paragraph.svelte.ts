import {
  base64URLToUint8Array,
  uint8ArrayToBase64URL,
  uuidv7,
} from "$lib/utils";
import { generateKeyBetween } from "fractional-indexing-jittered";
import {
  type Paragraph as RawParagraph,
  type Quote,
  type Links,
  type Path,
  commands,
  events,
} from "../../generated/tauri-commands";
import { Outline } from "./Outline.svelte";
import * as Y from "yjs";
import { ReversedLinkIndex, WeakRefMap } from "./utils";

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
  static reversedLinkIndex = new ReversedLinkIndex(this.buffer);

  readonly id: string;
  fractionalIndex = $state<string>() as string;
  doc = $state<string>() as string;
  quote = $state<Quote | null>(null);
  links = $state<Links>() as Links;
  readonly createdAt: Readonly<Date>;
  updatedAt = $state<Readonly<Date>>() as Readonly<Date>;
  private _outlineId: string;
  private _outlineRef = $state<WeakRef<Outline> | undefined>(undefined);
  private _path = $state<Path | undefined>(undefined);
  private _ydoc: Y.Doc | undefined;
  private _pendingYUpdates: Uint8Array[] = [];

  static inject(commands: Commands) {
    this.#commands = commands;
  }

  private constructor(data: RawParagraph, outline: Outline) {
    this.id = data.id;
    this.fractionalIndex = data.fractionalIndex;
    this.doc = data.doc;
    this.quote = data.quote;
    this.links = data.links;
    this.createdAt = new Date(data.createdAt);
    this.updatedAt = new Date(data.updatedAt);
    this._outlineId = data.outlineId;
    this._outlineRef = new WeakRef(outline);
  }

  static from(data: RawParagraph, outline: Outline) {
    const paragraph = this.buffer.get(data.id);

    if (paragraph) {
      paragraph.fractionalIndex = data.fractionalIndex;
      paragraph.doc = data.doc;
      paragraph.quote = data.quote;
      paragraph.links = data.links;
      paragraph._outlineId = data.outlineId;
      paragraph._outlineRef = new WeakRef(outline);
      paragraph.#initEffect();
      return paragraph;
    } else {
      const paragraph = new Paragraph(data, outline);
      this.buffer.set(paragraph.id, paragraph);
      paragraph.#initEffect();
      return paragraph;
    }
  }

  static new(outline: Outline, fractionalIndex?: string): Paragraph {
    const paragraph = Paragraph.from(
      {
        id: uuidv7(),
        outlineId: outline.id,
        fractionalIndex: fractionalIndex ?? generateKeyBetween(null, null),
        doc: "",
        links: {},
        quote: null,
        createdAt: new Date().getUTCMilliseconds(),
        updatedAt: new Date().getUTCMilliseconds(),
      },
      outline,
    );

    paragraph._ydoc = new Y.Doc();
    paragraph._ydoc.on("updateV2", (u) => {
      paragraph._pendingYUpdates.push(u);
      void Paragraph.#commands.insertPendingYUpdate(
        paragraph.id,
        uint8ArrayToBase64URL(u),
      );
    });

    const yFractionalIndex = paragraph._ydoc.getText("fractionalIndex");
    yFractionalIndex.insert(0, outline.fractionalIndex);

    return paragraph;
  }

  #initEffect() {
    $effect(() => {
      Paragraph.reversedLinkIndex.set(this.id, this.links);
    });
  }

  get outlineId() {
    return this._outlineId;
  }

  get outline(): Outline | null {
    return this._outlineRef?.deref() ?? null;
  }

  set outline(outline: Outline | null) {
    this._outlineRef = outline ? new WeakRef(outline) : undefined;
  }

  get path(): Promise<Path> {
    if (this._path) {
      return Promise.resolve(this._path);
    } else {
      return Paragraph.#commands.fetchPath(this._outlineId).then((path) => {
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
        void Paragraph.#commands.insertPendingYUpdate(
          this.id,
          uint8ArrayToBase64URL(u),
        );
      });
    }

    const updates = await Paragraph.#commands.fetchYUpdatesByDocId(this.id);

    for (const u of updates) {
      Y.applyUpdateV2(this._ydoc, base64URLToUint8Array(u));
    }

    return this._ydoc;
  }

  async save() {
    await Paragraph.#commands.upsertParagraph(
      this.toJSON(),
      this._pendingYUpdates.map((u) => uint8ArrayToBase64URL(u)),
    );
  }

  async moveTo(target: Outline, index: number | "last") {
    const ydoc = await this.ydoc();

    if (this._outlineId !== target.id) {
      this.outline?._removeParagraph(this);

      this._outlineId = target.id;
      this._outlineRef = new WeakRef(target);

      const yParentId = ydoc.getText("outlineId");
      yParentId.delete(0, yParentId.length);
      if (this._outlineId) yParentId.insert(0, this._outlineId);
    }

    const prev =
      target.paragraphs[index === "last" ? target.paragraphs.length - 1 : index]
        ?.fractionalIndex ?? null;
    const next =
      index === "last"
        ? null
        : (target.paragraphs[index]?.fractionalIndex ?? null);
    this.fractionalIndex = generateKeyBetween(prev, next);

    const yFractionalIndex = ydoc.getText("fractionalIndex");
    yFractionalIndex.delete(0, yFractionalIndex.length);
    yFractionalIndex.insert(0, this.fractionalIndex);

    target._insertParagraph(this);
  }

  toJSON(): RawParagraph {
    return {
      id: this.id,
      outlineId: this._outlineId,
      fractionalIndex: this.fractionalIndex,
      doc: this.doc,
      links: this.links,
      quote: this.quote,
      createdAt: this.createdAt.getUTCMilliseconds(),
      updatedAt: this.updatedAt.getUTCMilliseconds(),
    };
  }

  static async init() {
    await events.paragraphChange.listen((e) => {
      const payload = e.payload;

      const operation = payload.operation;

      if ("insert" in operation) {
        for (const { currentValue } of operation.insert.targets) {
          const outline = Outline.buffer.get(currentValue.outlineId);
          outline?._insertParagraph(Paragraph.from(currentValue, outline));
        }
      } else if ("update" in operation) {
        for (const { currentValue, relatedYUpdates } of operation.update
          .targets) {
          const paragraph = this.buffer.get(currentValue.id);

          if (paragraph) {
            paragraph.fractionalIndex = currentValue.fractionalIndex;
            paragraph.doc = currentValue.doc;
            paragraph.quote = currentValue.quote;
            paragraph.links = currentValue.links;

            if (currentValue.outlineId !== paragraph._outlineId) {
              paragraph.outline?._removeParagraph(paragraph);
              paragraph._outlineId = currentValue.outlineId;
              const outline = Outline.buffer.get(currentValue.outlineId);
              if (outline) paragraph._outlineRef = new WeakRef(outline);
            } else {
              paragraph.outline?.sortParagraphs();
            }

            if (paragraph._ydoc) {
              for (const u of relatedYUpdates) {
                Y.applyUpdateV2(paragraph._ydoc, base64URLToUint8Array(u));
              }
            }
          } else {
            const outline = Outline.buffer.get(currentValue.outlineId);
            if (outline)
              outline._insertParagraph(Paragraph.from(currentValue, outline));
          }
        }
      } else if ("delete" in operation) {
        const deletedParagraphs = operation.delete.target_ids
          .map((id) => Paragraph.buffer.get(id))
          .filter((o) => o !== undefined);

        for (const paragraph of deletedParagraphs) {
          paragraph.outline?._removeParagraph(paragraph);
        }
      }
    });
  }
}
