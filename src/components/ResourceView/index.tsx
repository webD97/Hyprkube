import dayjs from "dayjs";
import LocalizedFormat from "dayjs/plugin/localizedFormat";
import RelativeTime from "dayjs/plugin/relativeTime";
import { DisplayableResource, ResourceViewData } from "../../hooks/useResourceWatch";
import EmojiHint from "../EmojiHint";

import { useMemo, useState } from "react";
import styles from './styles.module.css';

import {
    createColumnHelper,
    flexRender,
    getCoreRowModel,
    getSortedRowModel,
    SortingState,
    useReactTable,
    VisibilityState
} from '@tanstack/react-table';
import { CustomCell } from "./CustomCell";

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

function createColumns(titles: string[]) {
    const columnHelper = createColumnHelper<[string, DisplayableResource]>();
    return titles.map((title, idx) => {
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
}

const ResourceView: React.FC<ResourceViewProps> = (props) => {
    const {
        namespace,
        resourceNamePlural,
        columnTitles,
        resourceData = {},
        onResourceClicked = () => undefined,
        onDeleteClicked = () => undefined,
    } = props;

    const columns = useMemo(() => createColumns(columnTitles), [columnTitles]);
    const data = useMemo(() => Object.entries(resourceData), [resourceData]);
    const [sorting, setSorting] = useState<SortingState>([])

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
        defaultColumn: {
            cell: CustomCell
        },
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
                    ? <EmojiHint emoji="⏳">No {resourceNamePlural} {namespace ? `in namespace "${namespace}" yet` : 'in this cluster yet'}</EmojiHint>
                    : null
            }
        </div>
    );
}

export default ResourceView;
