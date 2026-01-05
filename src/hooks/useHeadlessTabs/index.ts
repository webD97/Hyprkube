import { ReactElement, useCallback, useState } from "react";

export type TabDefinition<T extends object> = {
    meta: T,
    render: () => ReactElement
}

export type TabIdentifier = number;

export type TabMetaUpdateFunction<T> = (meta: T) => T;

export type TabContentRenderer = () => ReactElement;

export function useHeadlessTabs<T extends object>(initialTabs: TabDefinition<T>[] = []): [
    TabDefinition<T>[],
    TabIdentifier,
    (meta: T, render: TabContentRenderer) => void,
    (idx: TabIdentifier) => void,
    (idx: TabIdentifier) => void,
    (meta: T, render: TabContentRenderer) => void,
    (idx: TabIdentifier, meta_updater: TabMetaUpdateFunction<T>) => void
] {
    const [tabs, setTabs] = useState<TabDefinition<T>[]>(initialTabs);
    const [activeTab, setActiveTab] = useState<TabIdentifier>(0);

    const setTabMeta = useCallback((tab_index: TabIdentifier, meta_updater: TabMetaUpdateFunction<T>) => {
        setTabs(tabs => {
            const new_tabs = [...tabs];
            const current_meta = tabs[tab_index].meta;
            new_tabs[tab_index].meta = meta_updater(current_meta);

            return new_tabs;
        });
    }, []);

    const pushTab = useCallback((meta: T, render: TabContentRenderer) => {
        setTabs(tabs => [...tabs, { meta, render }]);
        setActiveTab(tabs.length);
    }, [tabs]);

    const removeTab = useCallback((remove_idx: TabIdentifier) => {
        setTabs(tabs => tabs.filter((_, idx) => idx !== remove_idx));

        if (remove_idx < activeTab) {
            setActiveTab(activeTab => activeTab - 1);
        }
    }, [activeTab]);

    const replaceActiveTab = useCallback((meta: T, render: TabContentRenderer) => {
        if (tabs.length === 0) {
            pushTab(meta, render);
            return;
        }

        setTabs(tabs => tabs.map((tab, idx) => {
            if (idx !== activeTab) return tab;
            return ({ meta, render })
        }))
    }, [activeTab, pushTab, setTabs, tabs]);

    if (tabs.length > 0 && activeTab > tabs.length - 1) {
        setActiveTab(tabs.length - 1);
    }

    return [tabs, activeTab, pushTab, removeTab, setActiveTab, replaceActiveTab, setTabMeta];
};