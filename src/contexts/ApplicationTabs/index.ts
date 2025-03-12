import React from "react";
import { MegaTabDefinition } from "../../components/MegaTabs";
import { TabContentRenderer, TabDefinition, TabIdentifier } from "../../hooks/useHeadlessTabs";

export type ApplicationTabsContextType = {
    applicationTabs: TabDefinition<MegaTabDefinition>[],
    activeApplicationTab: TabIdentifier,
    removeApplicationTab: (idx: TabIdentifier) => void,
    setActiveApplicationTab: (idx: TabIdentifier) => void,
    pushApplicationTab: (meta: MegaTabDefinition, render: TabContentRenderer) => void,
    replaceApplicationTab: (meta: MegaTabDefinition, render: TabContentRenderer) => void
}

const ApplicationTabsContext = React.createContext<ApplicationTabsContextType | null>(null);

export default ApplicationTabsContext;
