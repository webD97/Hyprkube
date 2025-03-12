import { ReactElement, useState } from "react";

export type TabDefinition<T> = {
    meta: T,
    render: () => ReactElement
}

export type TabIdentifier = number;

export type TabContentRenderer = () => ReactElement;

export function useHeadlessTabs<T>(initialTabs: TabDefinition<T>[] = []): [
    TabDefinition<T>[],
    TabIdentifier,
    (meta: T, render: TabContentRenderer) => void,
    (idx: TabIdentifier) => void,
    (idx: TabIdentifier) => void,
    (meta: T, render: TabContentRenderer) => void
] {
    const [tabs, setTabs] = useState<TabDefinition<T>[]>(initialTabs);
    const [activeTab, setActiveTab] = useState<TabIdentifier>(0);

    const pushTab = (meta: T, render: TabContentRenderer) => {
        setTabs(tabs => [...tabs, { meta, render }]);
        setActiveTab(tabs.length);
    }

    const removeTab = (remove_idx: TabIdentifier) => {
        setTabs(tabs => tabs.filter((_, idx) => idx !== remove_idx));

        if (remove_idx < activeTab) {
            setActiveTab(activeTab => activeTab - 1);
        }
        else if (remove_idx === activeTab && activeTab === tabs.length - 1) {
            // Do not change the activeTab because it will be correct on next render
            // But ensure we're not going out of bounds
            setActiveTab(activeTab => activeTab - 1);
        }
    }

    const replaceActiveTab = (meta: T, render: TabContentRenderer) => {
        if (tabs.length === 0) {
            pushTab(meta, render);
            return;
        }

        setTabs(tabs => tabs.map((tab, idx) => {
            if (idx !== activeTab) return tab;
            return ({ meta, render })
        }))
    }

    return [tabs, activeTab, pushTab, removeTab, setActiveTab, replaceActiveTab];
};