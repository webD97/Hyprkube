import React from "react";
import { useHeadlessTabs } from "../../hooks/useHeadlessTabs";

export type MegaTabDefinition = {
    title: string,
    icon: React.ReactNode,
    subtitle?: string,
    keepAlive?: boolean,
    immortal?: boolean
};

export type MegaTabsContextType = ReturnType<typeof useHeadlessTabs<MegaTabDefinition>>;

/**
 * Context for the application-level tabs
 */
const MegaTabsContext = React.createContext<MegaTabsContextType | null>(null);

export default MegaTabsContext;
