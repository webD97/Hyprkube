import { ColumnDefinition, DisplayableResource, PresentationComponent, ResourcePresentationData } from "../../hooks/useResourceWatch";
import EmojiHint from "../EmojiHint";

import { useVirtualizer } from "@tanstack/react-virtual";
import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import styles from './styles.module.css';

import {
    ColumnDef,
    ColumnFiltersState,
    createColumnHelper,
    flexRender,
    getCoreRowModel,
    getFilteredRowModel,
    getSortedRowModel,
    RowSelectionState,
    SortingState,
    useReactTable,
    VisibilityState
} from '@tanstack/react-table';
import { Checkbox, Input, Space } from "antd";
import React from "react";
import { createPortal } from "react-dom";
import { KubeContextSource } from "../../hooks/useContextDiscovery";
import { Gvk } from "../../model/k8s";
import ResourceContextMenu from "../ResourceContextMenu";
import { buildColumnInfo, computeColumnWidths } from "./columnSizing";
import { CustomCell } from "./CustomCell";

type _TData = [string, DisplayableResource];

export interface ResourcePresentationProps {
    contextSource: KubeContextSource,
    namespace?: string,
    resourceNamePlural?: string,
    columnDefinitions: ColumnDefinition[],
    resourceData: ResourcePresentationData,
    gvk: Gvk,
    onResourceClicked?: (gvk: Gvk, uid: string) => void,
    onSelectionChanged?: (rows: _TData[]) => void,
    searchbarPortal: React.RefObject<HTMLDivElement | null>,
}

function createColumns(columnDefinitions: ColumnDefinition[]) {
    const columnHelper = createColumnHelper<_TData>();
    const dataColumns = columnDefinitions.map(({ title, filterable }, idx) => {
        return columnHelper.accessor(row => row[1].columns[idx], {
            id: `${idx}_${title}`,
            header: () => title,
            sortingFn: (rowA, rowB, columnId) => {
                const valueA = rowA.getValue<PresentationComponent>(columnId).sortableValue;
                const valueB = rowB.getValue<PresentationComponent>(columnId).sortableValue;

                return valueA.localeCompare(valueB, undefined, { numeric: true });
            },
            filterFn: (row, columnId, filterValue) => {
                return row.getValue<PresentationComponent>(columnId).sortableValue.includes(filterValue as string);
            },
            enableColumnFilter: filterable,
            enableSorting: true, // TODO: View in backend should decide this
        });
    });

    const selectionColumn: ColumnDef<_TData> = {
        id: '_selection',
        header({ table }) {
            return <Checkbox
                checked={table.getIsAllPageRowsSelected()}
                onChange={table.getToggleAllRowsSelectedHandler()}
                onClick={(e) => e.stopPropagation()}
            />
        },
        cell({ row }) {
            return <Checkbox
                checked={row.getIsSelected()}
                disabled={!row.getCanSelect()}
                onChange={row.getToggleSelectedHandler()}
                onClick={(e) => e.stopPropagation()}
            />
        },
    };

    return [selectionColumn, ...dataColumns];
}

const ResourceList: React.FC<ResourcePresentationProps> = (props) => {
    const {
        contextSource,
        namespace,
        resourceNamePlural,
        columnDefinitions,
        gvk,
        resourceData = {},
        onResourceClicked = () => undefined,
        onSelectionChanged = () => undefined,
        searchbarPortal
    } = props;

    const columns = useMemo(() => createColumns(columnDefinitions), [columnDefinitions]);
    const data = useMemo(() => Object.entries(resourceData), [resourceData]);

    // Stable per-column sizing inputs derived from the full dataset (see columnSizing).
    const columnInfo = useMemo(() => buildColumnInfo(columnDefinitions, data), [columnDefinitions, data]);
    const [sorting, setSorting] = useState<SortingState>([]);
    const [rowSelection, setRowSelection] = useState<RowSelectionState>({});
    const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([]);

    const columnVisibility: VisibilityState = useMemo(() => {
        if (namespace === "") {
            return {};
        }

        return {
            [columnDefinitions.findIndex(c => c.title === "Namespace")]: false
        };
    }, [namespace, columnDefinitions]);

    const table = useReactTable({
        columns,
        data,
        getCoreRowModel: getCoreRowModel(),
        getSortedRowModel: getSortedRowModel(),
        getFilteredRowModel: getFilteredRowModel(),
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
        onColumnFiltersChange: setColumnFilters,
        state: {
            sorting,
            columnVisibility,
            rowSelection,
            columnFilters
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

    const tableContainerRef = useRef<HTMLDivElement>(null);

    // Width of one monospace character (≈ the `ch` unit), measured once.
    const charWidth = useMemo(() => {
        const ctx = document.createElement('canvas').getContext('2d');
        if (!ctx) return 8.4;

        ctx.font = '14px monospace';

        return ctx.measureText('0').width || 8.4;
    }, []);

    // Content width available to the table (excludes the vertical scrollbar).
    const [availableWidth, setAvailableWidth] = useState(0);

    useLayoutEffect(() => {
        const el = tableContainerRef.current;
        if (!el) return;
        const update = () => setAvailableWidth(el.clientWidth);
        update();
        const observer = new ResizeObserver(update);
        observer.observe(el);

        return () => observer.disconnect();
    }, []);

    const { rows } = table.getRowModel();
    const rowVirtualizer = useVirtualizer({
        count: rows.length,
        estimateSize: () => 31, // ~2.22em row height at 14px base font
        getScrollElement: () => tableContainerRef.current,
        getItemKey: (index) => rows[index].id,
        overscan: 12,
    });

    const virtualRows = rowVirtualizer.getVirtualItems();
    const paddingTop = virtualRows.length > 0 ? virtualRows[0].start : 0;
    const paddingBottom = virtualRows.length > 0
        ? rowVirtualizer.getTotalSize() - virtualRows[virtualRows.length - 1].end
        : 0;
    const visibleLeafColumns = table.getVisibleLeafColumns();
    const visibleColumnCount = visibleLeafColumns.length;
    const columnWidths = computeColumnWidths(
        visibleLeafColumns.map((column) => column.id),
        columnInfo,
        {
            availableWidth,
            charWidth,
            paddingPx: 14, // cells' `0 0.5em` at the 14px base font
            checkboxPx: Math.ceil(charWidth * 3),
        },
    );

    return (
        <div ref={tableContainerRef} className={styles.container}>
            {
                !searchbarPortal.current
                    ? null
                    : createPortal(
                        <></>,
                        searchbarPortal.current
                    )
            }
            {
                Object.keys(resourceData).length == 0
                    ? (
                        <EmojiHint emoji="⏳">
                            No {resourceNamePlural} {namespace ? `in namespace "${namespace}" yet` : 'in this cluster yet'}.
                        </EmojiHint>
                    )
                    : (
                        <table>
                            <colgroup>
                                {
                                    visibleLeafColumns.map((column) => (
                                        <col
                                            key={column.id}
                                            style={{ width: columnWidths.get(column.id) }}
                                        />
                                    ))
                                }
                            </colgroup>
                            <thead>
                                {
                                    table.getHeaderGroups().map((headerGroup) => (
                                        <React.Fragment key={headerGroup.id}>
                                            <tr>
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
                                            <tr>
                                                {
                                                    headerGroup.headers.map((header) => (
                                                        <th key={header.id} colSpan={header.colSpan}>
                                                            {
                                                                header.column.getCanFilter() && (
                                                                    <Space.Compact>
                                                                        <Input type="search" size="small"
                                                                            value={columnFilters.find(({ id }) => id === header.column.id)?.value as string ?? ''}
                                                                            onChange={(e) => table.setColumnFilters((currentFilters) => {
                                                                                const columnId = columns.find(({ id }) => id === header.column.id)?.id;
                                                                                if (!columnId) return currentFilters;

                                                                                return [
                                                                                    ...currentFilters.filter(({ id }) => id !== columnId),
                                                                                    { id: columnId, value: e.target.value }
                                                                                ];
                                                                            })}
                                                                        />
                                                                    </Space.Compact>
                                                                )
                                                            }
                                                        </th>
                                                    ))
                                                }
                                            </tr>
                                        </React.Fragment>
                                    ))}
                            </thead>
                            <tbody>
                                {
                                    paddingTop > 0 && (
                                        <tr aria-hidden>
                                            <td colSpan={visibleColumnCount} style={{ height: paddingTop, padding: 0, border: 0 }} />
                                        </tr>
                                    )
                                }
                                {
                                    virtualRows.map((virtualRow) => {
                                        const row = rows[virtualRow.index];
                                        const { namespace, name } = row.original[1];

                                        return (
                                            <ResourceContextMenu
                                                key={row.id}
                                                contextSource={contextSource}
                                                {...{ namespace, name, gvk }}
                                            >
                                                <tr
                                                    data-index={virtualRow.index}
                                                    ref={rowVirtualizer.measureElement}
                                                >
                                                    {
                                                        row.getVisibleCells().map((cell, idx) => (
                                                            <td key={cell.id} onClick={idx === 0 ? undefined : () => onResourceClicked(gvk, row.original[0])}>
                                                                {flexRender(cell.column.columnDef.cell, cell.getContext())}
                                                            </td>
                                                        ))
                                                    }
                                                </tr>
                                            </ResourceContextMenu>
                                        )
                                    })}
                                {
                                    paddingBottom > 0 && (
                                        <tr aria-hidden>
                                            <td colSpan={visibleColumnCount} style={{ height: paddingBottom, padding: 0, border: 0 }} />
                                        </tr>
                                    )
                                }
                            </tbody>
                        </table>
                    )
            }
        </div>
    );
}

export default ResourceList;
