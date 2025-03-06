import { Paragraph } from "./Paragraph.svelte";
import {
  events,
  type FetchTimelineOption,
  type ParagraphPositionIndex,
  type TimelineDay as RawTimelineDay,
} from "../../generated/tauri-commands";
import { Outline } from "./Outline.svelte";
import { fetchTimeline } from "$lib/commands";
import { isSameDay } from "date-fns";
import { getCurrent } from "@tauri-apps/api/webviewWindow";

export class TimelineDay {
  readonly dayStart: Date;
  #paragraphPositionIndex = $state<ParagraphPositionIndex>()!;
  #outlines = $state<Outline[]>([]);

  constructor(data: RawTimelineDay) {
    this.dayStart = new Date(data.dayStart);
    this.#paragraphPositionIndex = data.paragraphPositionIndex;

    const { outlines: rawOutlines, paragraphs: rawParagraphs } = data;
    const outlines = rawOutlines.map((o) => Outline.from(o));
    const outlineMap = new Map(outlines.map((o) => [o.id, o]));
    const paragraphs = rawParagraphs
      .map((p) => {
        const outline = outlineMap.get(p.outlineId);
        const paragraph = outline ? Paragraph.from(p, outline) : undefined;
        if (outline && paragraph) outline.insertParagraph(paragraph);
        return paragraph;
      })
      .filter((p) => p !== undefined);

    // Sort the outlines along with the paragraphs
    // because the paragraphs are already sorted by createdAt by the sql.
    this.#outlines = Array.from(new Set(paragraphs.map((p) => p.outlineId)))
      .map((id) => outlineMap.get(id))
      .filter((o) => o !== undefined);
  }

  get paragraphPositionIndex() {
    return this.#paragraphPositionIndex;
  }

  get items(): { outline: Outline; paragraphs: Paragraph[] }[] {
    const items = this.#outlines
      .map((o) => {
        const paragraphs = o.paragraphs.filter(
          (p) => !p.isEmpty && isSameDay(p.createdAt, this.dayStart),
        );
        return paragraphs.length
          ? { outline: o, paragraphs: paragraphs }
          : undefined;
      })
      .filter((o) => o !== undefined);

    items.sort(
      (a, b) =>
        Math.max(
          b.outline.createdAt.getTime(),
          ...b.paragraphs.map((p) => p.createdAt.getTime()),
        ) -
        Math.max(
          a.outline.createdAt.getTime(),
          ...a.paragraphs.map((p) => p.createdAt.getTime()),
        ),
    );

    return items;
  }

  async reload() {
    const tl = await fetchTimeline({ at: this.dayStart.getTime() });
    if (tl) {
      this.#outlines = tl.#outlines;
      this.#paragraphPositionIndex = tl.#paragraphPositionIndex;
    }
  }
}

export class Timeline {
  readonly days = $state<TimelineDay[]>([]);
  private _cleanup: (() => void) | undefined;

  private constructor(...days: TimelineDay[]) {
    this.days.push(...days);
  }

  static async init(option: FetchTimelineOption) {
    const timeline = await fetchTimeline(option).then((data) =>
      data ? new Timeline(data) : new Timeline(),
    );

    timeline._cleanup = await events
      .paragraphChange(getCurrent())
      .listen(async (e) => {
        const operation = e.payload.operation;

        if (operation.kind === "delete") {
          // remove from day
          return;
        }

        for (const { currentValue: p } of operation.targets) {
          for (const [i, day] of timeline.days.entries()) {
            const nextDay = timeline.days[i + 1];
            const prevDay = timeline.days[i - 1];

            if (isSameDay(new Date(p.createdAt), day.dayStart)) {
              await day.reload();
            } else if (
              nextDay !== undefined &&
              prevDay !== undefined &&
              nextDay.dayStart.getTime() < p.createdAt &&
              p.createdAt < prevDay.dayStart.getTime()
            ) {
              const day = await fetchTimeline({ at: p.createdAt });
              if (day) timeline.days.splice(i + 1, 0, day);
            }
          }
        }
      });

    return timeline;
  }

  async loadTop() {
    const option = this.days[0]
      ? { after: this.days[0].dayStart.getTime() }
      : "latest";

    const day = await fetchTimeline(option);

    return day ? () => this.days.unshift(day) : null;
  }

  async loadBottom() {
    const lastIndex = this.days.length - 1;
    const option = this.days[lastIndex]
      ? { before: this.days[lastIndex].dayStart.getTime() }
      : "latest";

    const day = await fetchTimeline(option);

    return day ? () => this.days.push(day) : null;
  }

  cleanup = () => {
    this._cleanup?.();
  };
}
