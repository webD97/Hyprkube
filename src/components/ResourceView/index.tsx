import dayjs from "dayjs";
import LocalizedFormat from "dayjs/plugin/localizedFormat";
import RelativeTime from "dayjs/plugin/relativeTime";
import { DisplayableResource, ResourceField, ResourceViewData } from "../../hooks/useResourceWatch";
import EmojiHint from "../EmojiHint";

import { useMemo, useState } from "react";
import styles from './styles.module.css';

import {
    ColumnDef,
    createColumnHelper,
    flexRender,
    getCoreRowModel,
    getSortedRowModel,
    SortingState,
    useReactTable,
    VisibilityState
} from '@tanstack/react-table';

export interface ResourceViewProps {
    namespace?: string,
    resourceNamePlural?: string,
    columnTitles: string[],
    resourceData: ResourceViewData,
    onResourceClicked?: (uid: string) => void,
    onDeleteClicked?: (uid: string) => void,
}

dayjs.extend(RelativeTime);
dayjs.extend(LocalizedFormat);

const ResourceView: React.FC<ResourceViewProps> = (props) => {
    const {
        namespace,
        resourceNamePlural,
        columnTitles,
        resourceData = {},
        onResourceClicked = () => undefined,
        onDeleteClicked = () => undefined,
    } = props;

    const columns = useMemo(() => {
        const columnHelper = createColumnHelper<[string, DisplayableResource]>();

        return columnTitles.map((title, idx) => {
            return columnHelper.accessor(row => row[1].columns[idx], {
                id: idx.toString(),
                header: () => title,
                sortingFn: (rowA, rowB, columnId) => {
                    const [, payloadA] = rowA.original;
                    const [, payloadB] = rowB.original;

                    const valueA = payloadA.columns[parseInt(columnId)].sortableValue;
                    const valueB = payloadB.columns[parseInt(columnId)].sortableValue;

                    return valueA.localeCompare(valueB, undefined, { numeric: true });
                }
            });
        });
    }, [columnTitles]);

    const data: [string, DisplayableResource][] = useMemo(() => {
        return Object.entries(resourceData);
    }, [resourceData]);

    const [sorting, setSorting] = useState<SortingState>([])

    const defaultColumn: Partial<ColumnDef<[string, DisplayableResource]>> = {
        cell(props) {
            return (props.getValue() as ResourceField).components.map((inner, idx) => {
                if ("PlainString" in inner) {
                    return <span key={idx}>{inner.PlainString}</span>;
                }
                else if ("ColoredString" in inner) {
                    const { string, color } = inner.ColoredString;
                    return <span key={idx} style={{ color }}>{string}</span>;
                }
                else if ("ColoredBox" in inner) {
                    const { color } = inner.ColoredBox;
                    return <span key={idx} style={{ color }}>■{"\u00A0"}</span>;
                }
                else if ("Hyperlink" in inner) {
                    const { url, display_text } = inner.Hyperlink;
                    return <a key={idx} style={{ cursor: "pointer" }} onClick={() => open(url)} title={url}>🔗&nbsp;{display_text}</a>;
                }
                else if ("RelativeTime" in inner) {
                    const { iso } = inner.RelativeTime;
                    const date = dayjs(iso);
                    return <span key={idx} title={date.format("LLL")}>{dayjs().to(date, true)}</span>;
                }

                return <span key={idx}>(Unhandled)</span>;
            });
        }
    };

    const columnVisibility: VisibilityState = useMemo(() => {
        if (namespace === "") {
            return {};
        }

        return {
            [columnTitles.indexOf("Namespace")]: false
        };
    }, [namespace, columnTitles]);

    const table = useReactTable({
        columns,
        data,
        getCoreRowModel: getCoreRowModel(),
        getSortedRowModel: getSortedRowModel(),
        defaultColumn,
        onSortingChange: setSorting,
        state: {
            sorting,
            columnVisibility
        }
    });

    return (
        <div className={styles.container}>
            <table>
                <thead>
                    {
                        table.getHeaderGroups().map((headerGroup) => (
                            <tr key={headerGroup.id}>
                                {
                                    headerGroup.headers.map((header) => (
                                        <th key={header.id} colSpan={header.colSpan}
                                            onClick={header.column.getToggleSortingHandler()}
                                            className={header.column.getCanSort() ? styles.sortable : undefined}
                                        >
                                            {flexRender(header.column.columnDef.header, header.getContext())}
                                            {{ asc: ' ⬆️', desc: ' ⬇️' }[header.column.getIsSorted() as string] ?? null}
                                        </th>
                                    ))}
                                <th className={styles.resourceQuickActions}>Actions</th>
                            </tr>
                        ))}
                </thead>
                <tbody>
                    {
                        table.getRowModel().rows.map((row) => {
                            return (
                                <tr key={row.id} onClick={() => {
                                    onResourceClicked(row.original[0])
                                }}>
                                    {
                                        row.getVisibleCells().map((cell) => {
                                            return (
                                                <td key={cell.id}>
                                                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                                                </td>
                                            )
                                        })}
                                    <td className={styles.resourceQuickActions}>
                                        <button onClick={(e) => {
                                            e.stopPropagation();
                                            onDeleteClicked(row.original[0]);
                                        }}>Delete</button>
                                    </td>
                                </tr>
                            )
                        })}
                </tbody>
            </table>
            {
                Object.keys(resourceData).length == 0
                    ? <EmojiHint emoji="⏳">No {resourceNamePlural} in namespace "{namespace}" yet</EmojiHint>
                    : null
            }
        </div>
    );
}

export default ResourceView;
