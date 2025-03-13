import { ReactElement, useEffect, useState } from "react";

export type TabDefinition<T> = {
    meta: T,
    setMeta: (updater: (meta: T) => T) => T,
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

    function makeTab(meta: T, render: TabContentRenderer): TabDefinition<T> {
        return ({
            meta,
            render,
            setMeta(updater) {
                const newMeta = updater(meta);

                setTabs(tabs => tabs.map(tab => {
                    if (tab.render !== render) return tab;

                    return ({ ...tab, meta: newMeta });
                }));

                return newMeta;
            }
        });
    }

    const pushTab = (meta: T, render: TabContentRenderer) => {
        setTabs(tabs => [...tabs, makeTab(meta, render)]);
        setActiveTab(tabs.length);
    };

    const removeTab = (remove_idx: TabIdentifier) => {
        setTabs(tabs => tabs.filter((_, idx) => idx !== remove_idx));

        if (remove_idx < activeTab) {
            setActiveTab(activeTab => activeTab - 1);
        }
    };

    const replaceActiveTab = (meta: T, render: TabContentRenderer) => {
        if (tabs.length === 0) {
            pushTab(meta, render);
            return;
        }

        setTabs(tabs => tabs.map((tab, idx) => {
            if (idx !== activeTab) return tab;
            return makeTab(meta, render)
        }))
    };

    useEffect(() => {
        if (tabs.length > 0 && activeTab > tabs.length - 1) {
            setActiveTab(tabs.length - 1);
        }
    }, [activeTab, tabs.length]);

    console.log(activeTab)

    return [tabs, activeTab, pushTab, removeTab, setActiveTab, replaceActiveTab];
};