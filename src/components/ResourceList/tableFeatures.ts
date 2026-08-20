import {
    columnFilteringFeature,
    columnVisibilityFeature,
    createFilteredRowModel,
    createSortedRowModel,
    rowSelectionFeature,
    rowSortingFeature,
    tableFeatures,
} from '@tanstack/react-table';

// Module scope keeps the object identity stable (required — `features` feeds the
// table options) and lets `typeof features` be shared by every module that types
// columns or cells.
export const features = tableFeatures({
    columnFilteringFeature,
    columnVisibilityFeature,
    rowSelectionFeature,
    rowSortingFeature,
    filteredRowModel: createFilteredRowModel(),
    sortedRowModel: createSortedRowModel(),
});

export type ResourceTableFeatures = typeof features;
