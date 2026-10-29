import { cloneElement, ReactElement, useState } from "react";
import { TabProps } from ".";

export type TabElement = ReactElement<TabProps>;

export function useTabs(): [TabElement[], number, (tab: TabElement) => void, (idx: number) => void, (idx: number) => void, (tab: TabElement) => void] {
    const [tabs, setTabs] = useState<TabElement[]>([]);
    const [activeTab, setActiveTab] = useState(0);

    const pushTab = (tab: ReactElement<TabProps>) => {
        // Tabs need a stable key so closing one doesn't remount the others
        setTabs(tabs => [...tabs, cloneElement(tab, { key: crypto.randomUUID() })]);
        setActiveTab(tabs.length);
    }

    const removeTab = (remove_idx: number) => {
        setTabs(tabs => tabs.filter((_, idx) => idx !== remove_idx));
    }

    const replaceActiveTab = (newTab: ReactElement<TabProps>) => {
        if (tabs.length === 0) {
            pushTab(newTab);
            return;
        }

        setTabs(tabs => tabs.map((tab, idx) => {
            if (idx !== activeTab) return tab;
            return cloneElement(newTab, { key: crypto.randomUUID() });
        }))
    }

    return [tabs, activeTab, pushTab, removeTab, setActiveTab, replaceActiveTab];
};
