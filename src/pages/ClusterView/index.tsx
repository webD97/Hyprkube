import { use, useCallback, useState } from 'react';
import { ErrorBoundary } from 'react-error-boundary';
import { Panel, PanelGroup, PanelResizeHandle } from 'react-resizable-panels';
import EmojiHint from '../../components/EmojiHint';
import RotatingSpinner from '../../components/RotatingSpinner';
import TabView from '../../components/TabView';
import { useTabs } from '../../components/TabView/hooks';
import ResourceListInspector from '../../containers/ResourceListInspector';
import { MegaTabContext } from '../../contexts/MegaTab';
import MegaTabsContext from '../../contexts/MegaTabs';
import { DiscoveryResult, useClusterDiscovery } from '../../hooks/useClusterDiscovery';
import useClusterProfiles from '../../hooks/useClusterProfiles';
import { KubeContextSource } from '../../hooks/useContextDiscovery';
import { Gvk } from '../../model/k8s';
import { capitalizeFirstLetter } from '../../utils/strings';
import Sidebar from './Sidebar';
import classes from './styles.module.css';

export interface ClusterViewProps {
    contextSource: KubeContextSource,
    preSelectedGvk?: Gvk,
    preSelectedNamespace?: string
}

const ClusterView: React.FC<ClusterViewProps> = ({ contextSource, preSelectedGvk, preSelectedNamespace }) => {
    const clusterProfiles = useClusterProfiles();
    const [activeGvk, setActiveGvk] = useState<Gvk | undefined>(preSelectedGvk);
    const [bottomTabs, activeBottomTab, pushBottomTab, removeBottomTab, setActiveBottomTab] = useTabs();
    const [currentNamespace, setCurrentNamespace] = useState(preSelectedNamespace || 'default');

    const { pushTab, switchTab } = use(MegaTabsContext)!;
    const { setMeta, tabIdentifier } = use(MegaTabContext)!;

    const onClusterDiscoveryStarted = useCallback(() => {
        setMeta(tabIdentifier, (meta) => ({ ...meta, icon: <RotatingSpinner reverse /> }));
    }, [setMeta, tabIdentifier]);

    const onClusterDiscoveryFinished = useCallback(() => {
        setMeta(tabIdentifier, (meta) => ({ ...meta, icon: '🌍' }));
    }, [setMeta, tabIdentifier]);

    const { discovery } = useClusterDiscovery(contextSource.source, contextSource.context, {
        onStart: onClusterDiscoveryStarted,
        onFinished: onClusterDiscoveryFinished,
    });

    function updateTabMeta(gvk: Gvk, namespace: string) {
        setMeta(tabIdentifier, meta => ({ ...meta, subtitle: makeTabSubtitle(discovery, gvk, namespace) }));
    }

    function handleGvkChange(nextGvk: Gvk) {
        setActiveGvk(nextGvk);
        updateTabMeta(nextGvk, currentNamespace);
    }

    function handleNamespaceChange(nextNamespace: string) {
        setCurrentNamespace(nextNamespace);
        updateTabMeta(activeGvk!, nextNamespace);
    }

    function handleGvkClick(gvk: Gvk, target: '_self' | '_blank') {
        if (target === '_self') {
            handleGvkChange(gvk);
        } else if (target === '_blank') {
            switchTab(
                pushTab(
                    { icon: '🌍', title: capitalizeFirstLetter(contextSource.context), subtitle: makeTabSubtitle(discovery, gvk, currentNamespace), keepAlive: true },
                    () => <ClusterView preSelectedNamespace={currentNamespace} contextSource={contextSource} preSelectedGvk={gvk} />
                )
            );
        }
    }

    if (!clusterProfiles[0]?.[0]) {
        return null;
    }

    return (
        <div className={classes.container}>
            <PanelGroup direction='horizontal'>
                <Panel minSize={12.5} maxSize={30} defaultSize={15}>
                    <Sidebar
                        clusterProfile={clusterProfiles[0][0]}
                        discovery={discovery}
                        onGvkClicked={handleGvkClick}
                    />
                </Panel>
                <PanelResizeHandle />
                <Panel>
                    <PanelGroup direction='vertical'>
                        <Panel id="mainArea" minSize={20} maxSize={80}>
                            <section className={classes.mainArea}>
                                <ErrorBoundary
                                    fallbackRender={(context) => (
                                        <div role="alert">
                                            <p>Something went wrong:</p>
                                            <pre style={{ color: "red" }}>{JSON.stringify(context, undefined, 2)}</pre>
                                        </div>
                                    )}
                                >
                                    {
                                        !activeGvk
                                            ? <EmojiHint emoji="👈">Select a resource to get started.</EmojiHint>
                                            : (
                                                <ResourceListInspector
                                                    gvk={activeGvk}
                                                    preSelectedNamespace={preSelectedNamespace || 'default'}
                                                    onNamespaceChanged={handleNamespaceChange}
                                                    contextSource={contextSource}
                                                    clusterProfile={clusterProfiles[0][0]}
                                                    pushBottomTab={pushBottomTab}
                                                />
                                            )
                                    }</ErrorBoundary>
                            </section>
                        </Panel>
                        {
                            (bottomTabs.length > 0) && (
                                <>
                                    <PanelResizeHandle />
                                    <Panel id="bottomTabs" defaultSize={65}>
                                        <section className={classes.bottomPanel}>
                                            <TabView
                                                activeTab={activeBottomTab}
                                                onCloseClicked={(idx) => removeBottomTab(idx)}
                                                setActiveTab={setActiveBottomTab}
                                            >
                                                {bottomTabs}
                                            </TabView>
                                        </section>
                                    </Panel>
                                </>
                            )
                        }
                    </PanelGroup>
                </Panel>
            </PanelGroup>
        </div>
    )
}

function makeTabSubtitle(discovery: DiscoveryResult, gvk: Gvk, namespace: string) {
    if (findResourceScope(discovery, gvk) === 'namespaced') {
        return `${gvk.kind}${namespace && ` (${namespace})`}`;
    }
    return `${gvk.kind}`;
}

function findResourceScope(discovery: DiscoveryResult, gvk: Gvk) {
    return discovery.gvks[gvk.group]?.kinds.find(resource => resource.kind === gvk.kind)?.scope || 'namespaced';
}

export default ClusterView;
