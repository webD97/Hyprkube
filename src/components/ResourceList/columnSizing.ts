import { ColumnDefinition, DisplayableResource, PresentationComponent } from "../../hooks/useResourceWatch";

export type ColumnSizing = { headerChars: number; contentChars: number; rigid: boolean };

// Approximate the rendered width of a cell in characters. The table uses a
// monospace font, so a character count maps directly to column width.
function displayLength(component: PresentationComponent): number {
    switch (component.kind) {
        case "Text":
            return component.args.content.length;
        case "Hyperlink":
            return component.args.content.length + 2; // 🔗 + space
        case "RelativeTime":
            return 7; // e.g. "359d7h"
        case "ColoredBox":
            return 1;
        case "ColoredBoxes":
            return component.args.boxes.reduce((sum, group) => sum + group.length, 0)
                + Math.max(0, component.args.boxes.length - 1); // boxes + group separators
    }
}

// Per-column sizing inputs, derived from content over the full dataset (so sizing
// stays stable while the virtualized body scrolls). A column is `rigid` (sized to
// fit, never truncated) when all its cells are relative-time (Age). Keyed by
// column id (`${idx}_${title}`).
export function buildColumnInfo(
    columnDefinitions: ColumnDefinition[],
    data: [string, DisplayableResource][],
): Map<string, ColumnSizing> {
    const info = new Map<string, ColumnSizing>();
    columnDefinitions.forEach(({ title }, idx) => {
        let contentChars = 0;
        let allRelativeTime = true;
        let sawAny = false;
        for (const [, resource] of data) {
            const component = resource.columns[idx];
            if (!component) continue;
            sawAny = true;
            contentChars = Math.max(contentChars, displayLength(component));
            if (component.kind !== 'RelativeTime') allRelativeTime = false;
        }
        info.set(`${idx}_${title}`, {
            headerChars: title.length + 1, // room for the sort arrow
            contentChars,
            rigid: sawAny && allRelativeTime,
        });
    });
    return info;
}

// Resolve a pixel width for every column id. Each column has a base width — its
// header width for flexible columns, its full content width for rigid ones. Space
// left over after the bases is shared among flexible columns in proportion to
// their content. So rigid columns never truncate, every column at least fits its
// header, and the table fills the measured width; if the bases don't even fit, the
// surplus is clipped (overflow-x: hidden) rather than headers being shrunk.
// Columns without sizing info (the selection checkbox) take a fixed width.
export function computeColumnWidths(
    ids: string[],
    info: Map<string, ColumnSizing>,
    opts: { availableWidth: number; charWidth: number; paddingPx: number; checkboxPx: number },
): Map<string, string> {
    const { availableWidth, charWidth, paddingPx, checkboxPx } = opts;

    const cols = ids.map((id) => {
        const sizing = info.get(id);

        if (!sizing) return { id, basePx: checkboxPx, flexWeight: 0 };

        const contentChars = Math.max(sizing.contentChars, sizing.headerChars);

        return sizing.rigid
            ? { id, basePx: contentChars * charWidth + paddingPx, flexWeight: 0 }
            : { id, basePx: sizing.headerChars * charWidth + paddingPx, flexWeight: contentChars };
    });

    let baseTotal = 0;
    let flexTotal = 0;

    for (const c of cols) {
        baseTotal += c.basePx;
        flexTotal += c.flexWeight;
    }

    const leftover = availableWidth - baseTotal;

    const widths = new Map<string, string>();

    for (const c of cols) {
        const extra = leftover > 0 && flexTotal > 0 ? leftover * (c.flexWeight / flexTotal) : 0;

        widths.set(c.id, `${c.basePx + extra}px`);
    }

    return widths;
}
