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
import { Menu } from "@tauri-apps/api/menu";
import { PhysicalPosition } from "@tauri-apps/api/dpi";
import { Gvk } from "../../model/k8s";

export interface ResourceViewProps {
    namespace?: string,
    resourceNamePlural?: string,
    columnTitles: string[],
    resourceData: ResourceViewData,
    gvk: Gvk,
    onResourceContextMenu: (gvk: Gvk, uid: string) => Promise<Menu>,
    onResourceClicked?: (gvk: Gvk, uid: string) => void
}

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
        gvk,
        resourceData = {},
        onResourceContextMenu,
        onResourceClicked = () => undefined
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
                            </tr>
                        ))}
                </thead>
                <tbody>
                    {
                        table.getRowModel().rows.map((row) => {
                            return (
                                <tr key={row.id}
                                    onClick={() => onResourceClicked(gvk, row.original[0])}
                                    onContextMenu={(e) => {
                                        e.preventDefault();

                                        onResourceContextMenu(gvk, row.original[0])
                                            .then(menu => menu.popup(new PhysicalPosition(e.screenX, e.screenY)))
                                            .catch(e => JSON.stringify(e));
                                    }}
                                >
                                    {
                                        row.getVisibleCells().map((cell) => {
                                            return (
                                                <td key={cell.id}>
                                                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                                                </td>
                                            )
                                        })}
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
