import { Paragraph } from "./Paragraph.svelte";
import {
  commands,
  events,
  type FetchTimelineOption,
  type ParagraphPositionIndex,
  type TimelineDay as RawTimelineDay,
} from "generated/tauri-commands";
import { Outline } from "./Outline.svelte";
import { fetchTimeline } from "$lib/commands";
import { addDays, isSameDay } from "date-fns";
import { getCurrent } from "@tauri-apps/api/webviewWindow";
import { unwrap } from "$lib/utils";

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
          (p) =>
            !p.isEmpty && !p.deleted && isSameDay(p.createdAt, this.dayStart),
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

  async reloadIndex() {
    const outlineIds = this.items.map(({ outline }) => outline.id);
    const paragraphIds = this.items
      .flatMap((item) => item.paragraphs)
      .map((p) => p.id);
    this.#paragraphPositionIndex = await commands
      .fetchParagraphPositionIndex(outlineIds, paragraphIds)
      .then(unwrap);
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
          for (const d of timeline.days) {
            d.reloadIndex();
          }
          return;
        }

        const targetDays = new Set(
          operation.targets.map((e) => e.currentValue.createdAt),
        );

        const latestDayStart = timeline.days[0]?.dayStart;
        const oldestDayStart =
          timeline.days[timeline.days.length - 1]?.dayStart;

        for (const createdAt of targetDays) {
          // when the timeline is empty
          if (!latestDayStart || !oldestDayStart) {
            const day = await fetchTimeline("latest");
            if (day) timeline.days.push(day);
            continue;
          }
          // when the updated paragraph is younger than the latest day in the timeline
          else if (addDays(latestDayStart, 1).getTime() <= createdAt) {
            const nextDay = await fetchTimeline({
              after: latestDayStart.getTime(),
            });

            if (nextDay && isSameDay(nextDay.dayStart, new Date(createdAt))) {
              timeline.days.unshift(nextDay);
            }
          }
          // when the updated paragraph is older than the oldest day in the timeline
          else if (createdAt < oldestDayStart.getTime()) {
            const prevDay = await fetchTimeline({
              before: oldestDayStart.getTime(),
            });

            if (prevDay && isSameDay(prevDay.dayStart, new Date(createdAt))) {
              timeline.days.push(prevDay);
            }
          }
          // when the updated paragraph's timestamp is between the oldest and latest day of the timeline
          else {
            for (const [i, day] of timeline.days.entries()) {
              const nextDayStart = timeline.days[i - 1]?.dayStart;
              const prevDayStart = timeline.days[i + 1]?.dayStart;

              if (isSameDay(new Date(createdAt), day.dayStart)) {
                await day.reload();
              } else if (
                prevDayStart !== undefined &&
                nextDayStart !== undefined &&
                prevDayStart.getTime() <= createdAt &&
                createdAt < nextDayStart.getTime()
              ) {
                const day = await fetchTimeline({ at: createdAt });
                if (day) timeline.days.splice(i + 1, 0, day);
              }
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

  async latest() {
    const day = await fetchTimeline("latest");
    if (day) {
      this.days.splice(0, this.days.length, day);
    }
  }

  cleanup = () => {
    this._cleanup?.();
  };
}
