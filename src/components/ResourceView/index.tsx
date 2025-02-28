import { DisplayableResource, ResourceViewData } from "../../hooks/useResourceWatch";
import EmojiHint from "../EmojiHint";

import { useEffect, useMemo, useState } from "react";
import styles from './styles.module.css';

import {
    ColumnDef,
    createColumnHelper,
    flexRender,
    getCoreRowModel,
    getSortedRowModel,
    RowSelectionState,
    SortingState,
    useReactTable,
    VisibilityState
} from '@tanstack/react-table';
import { PhysicalPosition } from "@tauri-apps/api/dpi";
import { Menu } from "@tauri-apps/api/menu";
import { Gvk } from "../../model/k8s";
import Checkbox from "../Checkbox";
import { CustomCell } from "./CustomCell";

type _TData = [string, DisplayableResource];

export interface ResourceViewProps {
    namespace?: string,
    resourceNamePlural?: string,
    columnTitles: string[],
    resourceData: ResourceViewData,
    gvk: Gvk,
    onResourceContextMenu: (gvk: Gvk, uid: string) => Promise<Menu>,
    onResourceClicked?: (gvk: Gvk, uid: string) => void,
    onSelectionChanged?: (rows: _TData[]) => void
}

function createColumns(titles: string[]) {
    const columnHelper = createColumnHelper<_TData>();
    const dataColumns = titles.map((title, idx) => {
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

    const selectionColumn: ColumnDef<_TData> = {
        id: '_selection',
        header({ table }) {
            return <Checkbox
                checked={table.getIsAllPageRowsSelected()}
                onChange={table.getToggleAllRowsSelectedHandler()}
            />
        },
        cell({ row }) {
            return <Checkbox
                checked={row.getIsSelected()}
                disabled={!row.getCanSelect()}
                onChange={row.getToggleSelectedHandler()}
            />
        },
    };

    return [selectionColumn, ...dataColumns];
}

const ResourceView: React.FC<ResourceViewProps> = (props) => {
    const {
        namespace,
        resourceNamePlural,
        columnTitles,
        gvk,
        resourceData = {},
        onResourceContextMenu,
        onResourceClicked = () => undefined,
        onSelectionChanged = () => undefined
    } = props;

    const columns = useMemo(() => createColumns(columnTitles), [columnTitles]);
    const data = useMemo(() => Object.entries(resourceData), [resourceData]);
    const [sorting, setSorting] = useState<SortingState>([]);
    const [rowSelection, setRowSelection] = useState<RowSelectionState>({});

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
        getRowId([resourceUid]) {
            return resourceUid;
        },
        onSortingChange: setSorting,
        onRowSelectionChange: (x) => {
            setRowSelection(x);
        },
        state: {
            sorting,
            columnVisibility,
            rowSelection
        }
    });

    // Notify parent if selection changes
    useEffect(() => {
        onSelectionChanged(table.getSelectedRowModel().rows.map(row => row.original));
    }, [table, rowSelection, onSelectionChanged, gvk]);

    // Remove deleted resources from selection
    useEffect(() => {
        const newSelection: RowSelectionState = {};

        Object.keys(rowSelection)
            .filter(selectionUid => data.map(([dataUid]) => dataUid).includes(selectionUid))
            .forEach(uid => newSelection[uid] = true);

        if (Object.keys(newSelection).length == Object.keys(rowSelection).length) return;

        setRowSelection(newSelection);
    }, [data, rowSelection]);

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
