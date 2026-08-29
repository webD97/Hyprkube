import { Activity } from "react";
import { KubeContextSource } from "../../hooks/useContextDiscovery";
import { TabIdentifier, useHeadlessTabs } from "../../hooks/useHeadlessTabs";
import KubeconfigEditor from "./KubeconfigEditor";
import styles from './styles.module.css';

export interface ClusterSettingsProps {
    contextSource: KubeContextSource,
    onDirty?: () => void,
    onDirtyCleared?: () => void,
}

export default function ClusterSettings({
    contextSource,
    onDirty,
    onDirtyCleared
}: ClusterSettingsProps) {
    const tabs = useHeadlessTabs([
        { meta: { title: '📝 Kubeconfig' }, render: () => <KubeconfigEditor contextSource={contextSource} onDirty={onDirty} onDirtyCleared={onDirtyCleared} /> },
    ]);

    return (
        <div className={styles.clusterSettingsContainer}>
            <nav>
                <ul>
                    {
                        Object.entries(tabs.tabState).map(([id, tab]) => (
                            <li key={id}
                                onClick={() => tabs.switchTab(id as TabIdentifier)}
                                className={tabs.activeTab === id ? styles.active : ''}
                            >
                                {tab.meta.title}
                            </li>
                        ))
                    }
                </ul>
            </nav>
            <main>{
                Object.entries(tabs.tabState).map(([id, tab]) => (
                    <Activity key={id} mode={id === tabs.activeTab ? 'visible' : 'hidden'}>
                        {tab.render()}
                    </Activity>
                ))
            }</main>
        </div >
    );
}