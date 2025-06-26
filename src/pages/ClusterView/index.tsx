import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import { useCallback, useContext, useEffect, useMemo, useState } from 'react';
import listClusterProfiles, { ClusterProfile } from '../../api/listClusterProfiles';
import EmojiHint from '../../components/EmojiHint';
import GvkList from '../../components/GvkList';
import { MegaTabContext } from '../../components/MegaTabs/context';
import TabView from '../../components/TabView';
import { useTabs } from '../../components/TabView/hooks';
import ResourceListInspector from '../../containers/ResourceListInspector';
import ApplicationTabsContext from '../../contexts/ApplicationTabs';
import { DiscoveryResult, useClusterDiscovery } from '../../hooks/useClusterDiscovery';
import { KubeContextSource } from '../../hooks/useContextDiscovery';
import useHiddenGvks from '../../hooks/useHiddenGvks';
import usePinnedGvks from '../../hooks/usePinnedGvks';
import { Gvk } from '../../model/k8s';
import { capitalizeFirstLetter } from '../../utils/strings';
import { createMenuForNormalGvks, createMenuForPinnedGvks } from './menus';
import classes from './styles.module.css';

export interface ClusterViewProps {
    contextSource: KubeContextSource,
    preSelectedGvk?: Gvk,
    preSelectedNamespace?: string
}

const ClusterView: React.FC<ClusterViewProps> = ({ contextSource, preSelectedGvk, preSelectedNamespace }) => {
    const [clusterProfiles, setClusterProfiles] = useState<ClusterProfile[]>([]);
    const [activeGvk, setActiveGvk] = useState<Gvk | undefined>(preSelectedGvk);
    const pinnedGvks = usePinnedGvks(clusterProfiles?.[0]?.[0]);
    const hiddenGvks = useHiddenGvks(clusterProfiles?.[0]?.[0]);
    const { discovery } = useClusterDiscovery(contextSource.source, contextSource.context);
    const [bottomTabs, activeBottomTab, pushBottomTab, removeBottomTab, setActiveBottomTab] = useTabs();
    const { pushApplicationTab } = useContext(ApplicationTabsContext)!;
    const { setMeta, tabIdentifier } = useContext(MegaTabContext)!;
    const [currentNamespace, setCurrentNamespace] = useState(preSelectedNamespace || 'default');

    useEffect(() => {
        if (!activeGvk) {
            setMeta(tabIdentifier, meta => ({ ...meta, subtitle: undefined }));
        } else {
            setMeta(tabIdentifier, meta => ({ ...meta, subtitle: makeTabSubtitle(discovery, activeGvk, currentNamespace) }));
        }
    }, [activeGvk, currentNamespace, discovery, setMeta, tabIdentifier]);

    useEffect(() => {
        listClusterProfiles()
            .then(profiles => {
                setClusterProfiles(profiles);
            })
            .catch(e => alert(JSON.stringify(e)));
    }, []);

    const onNamespaceChanged = useCallback((namespace: string) => {
        setCurrentNamespace(namespace);
    }, []);

    dayjs.extend(relativeTime);

    const sortedPinnedGvks = useMemo(() => {
        return pinnedGvks.sort((a, b) => {
            if (a.kind < b.kind) {
                return -1;
            }
            if (a.kind > b.kind) {
                return 1;
            }

            return 0;
        });
    }, [pinnedGvks]);

    if (!clusterProfiles[0]?.[0]) {
        return null;
    }

    return (
        <div className={classes.container}>
            <nav>
                <h2 className={classes.resourceSectionTitle}>Pinned resources</h2>
                {
                    sortedPinnedGvks.length == 0
                        ? null
                        : (
                            <GvkList flat withGroupNames
                                gvks={sortedPinnedGvks}
                                onResourceClicked={(gvk) => {
                                    setActiveGvk(gvk);
                                }}
                                onResourceAuxClicked={(gvk) => {
                                    pushApplicationTab(
                                        { title: capitalizeFirstLetter(contextSource.context), icon: '🌍', keepAlive: true },
                                        () => <ClusterView preSelectedNamespace={currentNamespace} contextSource={contextSource} preSelectedGvk={gvk} />
                                    );
                                }}
                                onGvkContextMenu={(gvk) => createMenuForPinnedGvks({
                                    clusterProfile: clusterProfiles[0][0],
                                    openInNewTab: (gvk) => {
                                        pushApplicationTab(
                                            { title: capitalizeFirstLetter(contextSource.context), icon: '🌍', keepAlive: true },
                                            () => <ClusterView contextSource={contextSource} preSelectedGvk={gvk} />
                                        )
                                    },
                                    gvk
                                })}
                            />
                        )
                }

                <h2 className={classes.resourceSectionTitle}>Builtin resources</h2>
                {
                    Object.values(discovery?.gvks || [])
                        .filter((group) => !group.isCrd)
                        .sort((groupA, groupB) => groupA.name.localeCompare(groupB.name))
                        .map(({ name: groupName, kinds }, idx) => {
                            const gvks = kinds
                                .map(({ kind, version }) => ({ group: groupName, version, kind }))
                                .filter(gvk => (
                                    !hiddenGvks.some(current => (
                                        current.group === gvk.group &&
                                        current.version === gvk.version &&
                                        current.kind === gvk.kind
                                    ))
                                ));

                            // Don't show groups where everything is hidden
                            if (gvks.length === 0) return;

                            return (
                                <GvkList key={idx}
                                    gvks={gvks}
                                    onResourceClicked={(gvk) => {
                                        setActiveGvk(gvk);
                                    }}
                                    onResourceAuxClicked={(gvk) => {
                                        pushApplicationTab(
                                            { title: capitalizeFirstLetter(contextSource.context), icon: '🌍', keepAlive: true },
                                            () => <ClusterView contextSource={contextSource} preSelectedGvk={gvk} />
                                        );
                                    }}
                                    onGvkContextMenu={(gvk) => createMenuForNormalGvks({
                                        clusterProfile: clusterProfiles[0][0],
                                        openInNewTab: (gvk) => {
                                            pushApplicationTab(
                                                { title: capitalizeFirstLetter(contextSource.context), icon: '🌍', keepAlive: true },
                                                () => <ClusterView contextSource={contextSource} preSelectedGvk={gvk} />
                                            )
                                        },
                                        gvk
                                    })}
                                />
                            );
                        })
                }

                <h2 className={classes.resourceSectionTitle}>Custom resources</h2>
                {
                    Object.values(discovery?.gvks || [])
                        .filter((group) => group.isCrd)
                        .sort((groupA, groupB) => groupA.name.localeCompare(groupB.name))
                        .map(({ name: groupName, kinds }, idx) => {
                            const gvks = kinds
                                .map(({ kind, version }) => ({ group: groupName, version, kind }))
                                .filter(gvk => (
                                    !hiddenGvks.some(current => (
                                        current.group === gvk.group &&
                                        current.version === gvk.version &&
                                        current.kind === gvk.kind
                                    ))
                                ));

                            // Don't show groups where everything is hidden
                            if (gvks.length === 0) return;

                            return (
                                <GvkList key={idx}
                                    gvks={gvks}
                                    onResourceClicked={(gvk) => {
                                        setActiveGvk(gvk);
                                    }}
                                    onResourceAuxClicked={(gvk) => {
                                        pushApplicationTab(
                                            { title: capitalizeFirstLetter(contextSource.context), icon: '🌍', keepAlive: true },
                                            () => <ClusterView contextSource={contextSource} preSelectedGvk={gvk} />
                                        );
                                    }}
                                    onGvkContextMenu={(gvk) => createMenuForNormalGvks({
                                        clusterProfile: clusterProfiles[0][0],
                                        openInNewTab: (gvk) => {
                                            pushApplicationTab(
                                                { title: capitalizeFirstLetter(contextSource.context), icon: '🌍', keepAlive: true },
                                                () => <ClusterView contextSource={contextSource} preSelectedGvk={gvk} />
                                            )
                                        },
                                        gvk
                                    })}
                                />
                            );
                        })
                }
            </nav>
            <section className={classes.bottomPanel}>
                <TabView
                    activeTab={activeBottomTab}
                    onCloseClicked={(idx) => removeBottomTab(idx)}
                    setActiveTab={setActiveBottomTab}
                >
                    {bottomTabs}
                </TabView>
            </section>
            <section className={classes.mainArea}>
                {
                    !activeGvk
                        ? <EmojiHint emoji="👈">Select a resource to get started.</EmojiHint>
                        : (
                            <ResourceListInspector
                                gvk={activeGvk}
                                preSelectedNamespace={preSelectedNamespace || 'default'}
                                onNamespaceChanged={onNamespaceChanged}
                                contextSource={contextSource}
                                clusterProfile={clusterProfiles[0][0]}
                                pushBottomTab={pushBottomTab}
                            />
                        )
                }
            </section>
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
