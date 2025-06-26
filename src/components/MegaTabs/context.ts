import React from "react";
import { MegaTabDefinition } from ".";
import { TabIdentifier } from "../../hooks/useHeadlessTabs";

export type MegaTabContextType<T> = {
    tabIdentifier: TabIdentifier,
    setMeta: (idx: TabIdentifier, updater: (meta: T) => T) => void
};

export const MegaTabContext = React.createContext<MegaTabContextType<MegaTabDefinition> | null>(null);
