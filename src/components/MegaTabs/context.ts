import React from "react";
import { MegaTabDefinition } from ".";

export type MegaTabContextType<T> = {
    setMeta: (updater: (meta: T) => T) => T
};

export const MegaTabContext = React.createContext<MegaTabContextType<MegaTabDefinition> | null>(null);
