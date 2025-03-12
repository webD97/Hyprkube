import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';

import { useEffect, useMemo, useState } from 'react';

import listClusterProfiles, { ClusterProfile } from '../../api/listClusterProfiles';
import EmojiHint from '../../components/EmojiHint';
import GvkList from '../../components/GvkList';
import TabView, { Tab } from '../../components/TabView';
import { useTabs } from '../../components/TabView/hooks';
import ResourceListInspector from '../../containers/ResourceListInspector';
import { DiscoveryResult, useClusterDiscovery } from '../../hooks/useClusterDiscovery';
import { KubeContextSource } from '../../hooks/useContextDiscovery';
import useHiddenGvks from '../../hooks/useHiddenGvks';
import usePinnedGvks from '../../hooks/usePinnedGvks';
import { Gvk } from '../../model/k8s';
import { createMenuForNormalGvks, createMenuForPinnedGvks } from './menus';
import classes from './styles.module.css';

export interface ClusterViewProps {
    contextSource: KubeContextSource
}

const ClusterView: React.FC<ClusterViewProps> = (props) => {
    const { contextSource } = props;

    const [clusterProfiles, setClusterProfiles] = useState<ClusterProfile[]>([]);
    const pinnedGvks = usePinnedGvks(clusterProfiles?.[0]?.[0]);
    const hiddenGvks = useHiddenGvks(clusterProfiles?.[0]?.[0]);

    const { discovery } = useClusterDiscovery(contextSource.source, contextSource.context);

    const [bottomTabs, activeBottomTab, pushBottomTab, removeBottomTab, setActiveBottomTab] = useTabs();
    const [resourceTabs, activeResourceTab, pushResourceTab, removeResourceTab, setActiveResourceTab, replaceActiveResourceTab] = useTabs();

    useEffect(() => {
        listClusterProfiles()
            .then(profiles => {
                setClusterProfiles(profiles);
            })
            .catch(e => alert(JSON.stringify(e)));
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

    function makeTab(gvk: Gvk) {
        return (
            <Tab
                title={'📄 ' + findResourcePlural(discovery, gvk)}
            >
                {() => (
                    <ResourceListInspector
                        gvk={gvk}
                        contextSource={contextSource}
                        clusterProfile={clusterProfiles[0][0]}
                        pushBottomTab={pushBottomTab}
                    />
                )}
            </Tab>
        )
            ;
    }

    return (
        <div className={classes.container}>
            <nav>
                <h2>Pinned resources</h2>
                {
                    sortedPinnedGvks.length == 0
                        ? null
                        : (
                            <GvkList flat withGroupNames
                                gvks={sortedPinnedGvks}
                                onResourceClicked={(gvk) => {
                                    replaceActiveResourceTab(
                                        makeTab(gvk)
                                    )
                                }}
                                onResourceAuxClicked={(gvk) => {
                                    pushResourceTab(
                                        makeTab(gvk)
                                    )
                                }}
                                onGvkContextMenu={(gvk) => createMenuForPinnedGvks({
                                    clusterProfile: clusterProfiles[0][0],
                                    gvk
                                })}
                            />
                        )
                }

                <h2>Builtin resources</h2>
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
                                        replaceActiveResourceTab(
                                            makeTab(gvk)
                                        )
                                    }}
                                    onResourceAuxClicked={(gvk) => {
                                        pushResourceTab(
                                            makeTab(gvk)
                                        )
                                    }}
                                    onGvkContextMenu={(gvk) => createMenuForNormalGvks({
                                        clusterProfile: clusterProfiles[0][0],
                                        gvk
                                    })}
                                />
                            );
                        })
                }

                <h2>Custom resources</h2>
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
                                        replaceActiveResourceTab(
                                            makeTab(gvk)
                                        )
                                    }}
                                    onResourceAuxClicked={(gvk) => {
                                        pushResourceTab(
                                            makeTab(gvk)
                                        )
                                    }}
                                    onGvkContextMenu={(gvk) => createMenuForNormalGvks({
                                        clusterProfile: clusterProfiles[0][0],
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
                    resourceTabs.length < 1
                        ? <EmojiHint emoji="👈">Select a resource to get started.</EmojiHint>
                        : null
                }
                <TabView eager
                    activeTab={activeResourceTab}
                    onCloseClicked={(idx) => removeResourceTab(idx)}
                    setActiveTab={setActiveResourceTab}
                >
                    {resourceTabs}
                </TabView>
            </section>
        </div>
    )
}

function findResourcePlural(discovery: DiscoveryResult, gvk: Gvk): string {
    return discovery.gvks[gvk.group]?.kinds.find(resource => resource.kind === gvk.kind)?.plural || gvk.kind + 's';
}

export default ClusterView;
