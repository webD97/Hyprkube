import React from "react";
import { MegaTabDefinition } from "../../contexts/MegaTabs";
import { TabIdentifier } from "../../hooks/useHeadlessTabs";

export type MegaTabContextType<T> = {
    tabIdentifier: TabIdentifier,
    setMeta: (idx: TabIdentifier, updater: (meta: T) => T) => void
};

/**
 * Context for the tab where the calling component is displayed
 */
export const MegaTabContext = React.createContext<MegaTabContextType<MegaTabDefinition> | null>(null);
