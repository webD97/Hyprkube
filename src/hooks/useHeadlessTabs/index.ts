import { ReactElement, useCallback, useRef, useState } from "react";

export type TabDefinition<T extends object> = {
    meta: T,
    render: () => ReactElement
}

export type TabIdentifier = ReturnType<typeof crypto.randomUUID>;
export type TabMetaUpdateFunction<T> = (meta: T) => T;
export type TabContentRenderer = () => ReactElement;
export type TabState<T extends object> = Record<TabIdentifier, TabDefinition<T>>;

export type PushTabFn<T extends object> = (meta: T, render: TabContentRenderer) => TabIdentifier;
export type CloseTabFn = (idx: TabIdentifier) => void;
export type SwitchTabFn = (idx: TabIdentifier | number) => void;
export type ReplaceTabFn<T extends object> = (meta: T, render: TabContentRenderer) => void;
export type UpdateTabMetaFn<T extends object> = (idx: TabIdentifier, meta_updater: TabMetaUpdateFunction<T>) => void;

/**
 * Create the data structures and function to build a tab navigation
 * 
 * @param initialTabs
 */
export function useHeadlessTabs<T extends object>(initialTabs: TabDefinition<T>[] = []): {
    tabState: TabState<T>,
    activeTab: TabIdentifier | null,
    pushTab: PushTabFn<T>,
    closeTab: CloseTabFn,
    switchTab: SwitchTabFn,
    replaceActiveTab: ReplaceTabFn<T>,
    updateTabMeta: UpdateTabMetaFn<T>
} {
    const [tabs, setTabs] = useState<TabState<T>>(
        Object.fromEntries(
            initialTabs.map(tab => ([crypto.randomUUID(), tab]))
        )
    );

    const [activeTab, setActiveTab] = useState<TabIdentifier | null>(
        (Object.keys(tabs) as TabIdentifier[])[0] || null
    );

    // Keeps track of the order in which the user switched between tabs
    const tabHistory = useRef<TabIdentifier[]>([]);

    const updateTabMeta = useCallback((tab_index: TabIdentifier, meta_updater: TabMetaUpdateFunction<T>) => {
        setTabs(tabs => {
            const new_tabs = { ...tabs };
            const current_tab = tabs[tab_index];
            const current_meta = current_tab.meta;
            new_tabs[tab_index] = { ...current_tab, meta: meta_updater(current_meta) };

            return new_tabs;
        });
    }, []);

    const pushTab = useCallback((meta: T, render: TabContentRenderer) => {
        const newTabId = crypto.randomUUID();

        setTabs(tabs => {
            const newTabs = { ...tabs };
            newTabs[newTabId] = { meta, render };

            return newTabs;
        });

        return newTabId;
    }, []);

    const replaceActiveTab = useCallback((meta: T, render: TabContentRenderer) => {
        if (Object.keys(tabs).length === 0) {
            pushTab(meta, render);
            return;
        }

        if (activeTab === null) return;

        setTabs(tabs => {
            const newTabs = { ...tabs };
            newTabs[activeTab] = { meta, render };

            return newTabs;
        })
    }, [activeTab, pushTab, setTabs, tabs]);

    const switchTab = useCallback((idx: TabIdentifier | number) => {
        if (activeTab) {
            tabHistory.current.push(activeTab);
        }

        if (typeof (idx) === "number") {
            const newTab = (Object.keys(tabs) as TabIdentifier[])[idx];
            setActiveTab(newTab);
            return;
        }

        setActiveTab(idx);
    }, [activeTab, tabs]);

    const closeTab = useCallback((remove_idx: TabIdentifier) => {
        setTabs(tabs => {
            const newTabs = { ...tabs };
            delete newTabs[remove_idx];

            return newTabs;
        });

        // If the currently open tab is closed, remove it from the history and jump to the previously opened tab
        if (remove_idx === activeTab) {
            if (tabHistory.current.length > 0) {
                tabHistory.current = tabHistory.current.filter(id => id !== remove_idx);

                setActiveTab(tabHistory.current[tabHistory.current.length - 1]);
            }
            else {
                setActiveTab(tabHistory.current[0] || null);
            }
        }
    }, [activeTab]);

    return {
        tabState: tabs,
        activeTab,
        pushTab,
        closeTab,
        switchTab,
        replaceActiveTab,
        updateTabMeta
    };
};