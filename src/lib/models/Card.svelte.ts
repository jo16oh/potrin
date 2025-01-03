import {
  base64URLToUint8Array,
  uint8ArrayToBase64URL,
  uuidv7,
} from "$lib/utils";
import { generateKeyBetween } from "fractional-indexing-jittered";
import {
  type Card as RawCard,
  type Quote,
  type Links,
  type Path,
  commands,
  events,
} from "../../generated/tauri-commands";
import { Outline } from "./Outline.svelte";
import * as Y from "yjs";
import { ReversedLinkIndex, WeakRefMap } from "./utils";

export type { RawCard };

type Commands = Pick<
  typeof commands,
  | "fetchPath"
  | "fetchConflictingOutlineIds"
  | "fetchYUpdatesByDocId"
  | "insertPendingYUpdate"
  | "upsertCard"
>;

export class Card {
  static #commands: Commands = commands;
  static buffer: WeakRefMap<string, Card> = new WeakRefMap();
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

  private constructor(data: RawCard, outline: Outline) {
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

  static from(data: RawCard, outline: Outline) {
    const card = this.buffer.get(data.id);

    if (card) {
      card.fractionalIndex = data.fractionalIndex;
      card.doc = data.doc;
      card.quote = data.quote;
      card.links = data.links;
      card._outlineId = data.outlineId;
      card._outlineRef = new WeakRef(outline);
      card.#initEffect();
      return card;
    } else {
      const card = new Card(data, outline);
      this.buffer.set(card.id, card);
      card.#initEffect();
      return card;
    }
  }

  static new(outline: Outline, fractionalIndex?: string): Card {
    const card = Card.from(
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

    card._ydoc = new Y.Doc();
    card._ydoc.on("updateV2", (u) => {
      card._pendingYUpdates.push(u);
      void Card.#commands.insertPendingYUpdate(
        card.id,
        uint8ArrayToBase64URL(u),
      );
    });

    const yFractionalIndex = card._ydoc.getText("fractionalIndex");
    yFractionalIndex.insert(0, outline.fractionalIndex);

    return card;
  }

  #initEffect() {
    $effect(() => {
      Card.reversedLinkIndex.set(this.id, this.links);
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
      return Card.#commands.fetchPath(this._outlineId).then((path) => {
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
        void Card.#commands.insertPendingYUpdate(
          this.id,
          uint8ArrayToBase64URL(u),
        );
      });
    }

    const updates = await Card.#commands.fetchYUpdatesByDocId(this.id);

    for (const u of updates) {
      Y.applyUpdateV2(this._ydoc, base64URLToUint8Array(u));
    }

    return this._ydoc;
  }

  async save() {
    await Card.#commands.upsertCard(
      this.toJSON(),
      this._pendingYUpdates.map((u) => uint8ArrayToBase64URL(u)),
    );
  }

  async moveTo(target: Outline, index: number | "last") {
    const ydoc = await this.ydoc();

    if (this._outlineId !== target.id) {
      this.outline?._removeCard(this);

      this._outlineId = target.id;
      this._outlineRef = new WeakRef(target);

      const yParentId = ydoc.getText("outlineId");
      yParentId.delete(0, yParentId.length);
      if (this._outlineId) yParentId.insert(0, this._outlineId);
    }

    const prev =
      target.cards[index === "last" ? target.cards.length - 1 : index]
        ?.fractionalIndex ?? null;
    const next =
      index === "last" ? null : (target.cards[index]?.fractionalIndex ?? null);
    this.fractionalIndex = generateKeyBetween(prev, next);

    const yFractionalIndex = ydoc.getText("fractionalIndex");
    yFractionalIndex.delete(0, yFractionalIndex.length);
    yFractionalIndex.insert(0, this.fractionalIndex);

    target._insertCard(this);
  }

  toJSON(): RawCard {
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
    await events.cardChange.listen((e) => {
      const payload = e.payload;

      const operation = payload.operation;

      if ("insert" in operation) {
        for (const { currentValue } of operation.insert.targets) {
          const outline = Outline.buffer.get(currentValue.outlineId);
          outline?._insertCard(Card.from(currentValue, outline));
        }
      } else if ("update" in operation) {
        for (const { currentValue, relatedYUpdates } of operation.update
          .targets) {
          const card = this.buffer.get(currentValue.id);

          if (card) {
            card.fractionalIndex = currentValue.fractionalIndex;
            card.doc = currentValue.doc;
            card.quote = currentValue.quote;
            card.links = currentValue.links;

            if (currentValue.outlineId !== card._outlineId) {
              card.outline?._removeCard(card);
              card._outlineId = currentValue.outlineId;
              const outline = Outline.buffer.get(currentValue.outlineId);
              if (outline) card._outlineRef = new WeakRef(outline);
            } else {
              card.outline?.sortCards();
            }

            if (card._ydoc) {
              for (const u of relatedYUpdates) {
                Y.applyUpdateV2(card._ydoc, base64URLToUint8Array(u));
              }
            }
          } else {
            const outline = Outline.buffer.get(currentValue.outlineId);
            if (outline) outline._insertCard(Card.from(currentValue, outline));
          }
        }
      } else if ("delete" in operation) {
        const deletedCards = operation.delete.target_ids
          .map((id) => Card.buffer.get(id))
          .filter((o) => o !== undefined);

        for (const card of deletedCards) {
          card.outline?._removeCard(card);
        }
      }
    });
  }
}
