import { ReactElement, useState } from "react";
import { TabProps } from ".";

type Tab = ReactElement<TabProps>;

export function useTabs(): [Tab[], number, (tab: Tab) => void, (idx: number) => void, (idx: number) => void] {
    const [tabs, setTabs] = useState<Tab[]>([]);
    const [activeTab, setActiveTab] = useState(0);

    const pushTab = (tab: ReactElement<TabProps>) => {
        setTabs(tabs => [...tabs, tab]);
    }

    const removeTab = (remove_idx: number) => {
        setTabs(tabs => tabs.filter((_, idx) => idx !== remove_idx));
    }

    return [tabs, activeTab, pushTab, removeTab, setActiveTab];
};
