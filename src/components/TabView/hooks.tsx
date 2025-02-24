import { ReactElement, useState } from "react";
import { TabProps } from ".";

export type TabElement = ReactElement<TabProps>;

export function useTabs(): [TabElement[], number, (tab: TabElement) => void, (idx: number) => void, (idx: number) => void] {
    const [tabs, setTabs] = useState<TabElement[]>([]);
    const [activeTab, setActiveTab] = useState(0);

    const pushTab = (tab: ReactElement<TabProps>) => {
        setTabs(tabs => [...tabs, tab]);
    }

    const removeTab = (remove_idx: number) => {
        setTabs(tabs => tabs.filter((_, idx) => idx !== remove_idx));
    }

    return [tabs, activeTab, pushTab, removeTab, setActiveTab];
};
