import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import { useContext, useEffect, useMemo, useState } from 'react';
import listClusterProfiles, { ClusterProfile } from '../../api/listClusterProfiles';
import EmojiHint from '../../components/EmojiHint';
import GvkList from '../../components/GvkList';
import { MegaTabContext } from '../../components/MegaTabs/context';
import TabView from '../../components/TabView';
import { useTabs } from '../../components/TabView/hooks';
import ResourceListInspector from '../../containers/ResourceListInspector';
import ApplicationTabsContext from '../../contexts/ApplicationTabs';
import { useClusterDiscovery } from '../../hooks/useClusterDiscovery';
import { KubeContextSource } from '../../hooks/useContextDiscovery';
import useHiddenGvks from '../../hooks/useHiddenGvks';
import usePinnedGvks from '../../hooks/usePinnedGvks';
import { Gvk } from '../../model/k8s';
import { capitalizeFirstLetter } from '../../utils/strings';
import { createMenuForNormalGvks, createMenuForPinnedGvks } from './menus';
import classes from './styles.module.css';

export interface ClusterViewProps {
    contextSource: KubeContextSource,
    preSelectedGvk?: Gvk
}

const ClusterView: React.FC<ClusterViewProps> = ({ contextSource, preSelectedGvk }) => {
    const [clusterProfiles, setClusterProfiles] = useState<ClusterProfile[]>([]);
    const [activeGvk, setActiveGvk] = useState<Gvk | undefined>(preSelectedGvk);
    const pinnedGvks = usePinnedGvks(clusterProfiles?.[0]?.[0]);
    const hiddenGvks = useHiddenGvks(clusterProfiles?.[0]?.[0]);
    const { discovery } = useClusterDiscovery(contextSource.source, contextSource.context);
    const [bottomTabs, activeBottomTab, pushBottomTab, removeBottomTab, setActiveBottomTab] = useTabs();
    const { pushApplicationTab } = useContext(ApplicationTabsContext)!;
    const tabContext = useContext(MegaTabContext);

    useEffect(() => {
        if (!activeGvk) {
            tabContext?.setMeta(meta => ({ ...meta, subtitle: undefined }));
        } else {
            tabContext?.setMeta(meta => ({ ...meta, subtitle: makeTabSubtitle(activeGvk) }));
        }
    }, [activeGvk, tabContext]);

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

    if (!clusterProfiles[0]?.[0]) {
        return null;
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
                                    setActiveGvk(gvk);
                                }}
                                onResourceAuxClicked={(gvk) => {
                                    pushApplicationTab(
                                        { title: capitalizeFirstLetter(contextSource.context), icon: '🌍', keepAlive: true },
                                        () => <ClusterView contextSource={contextSource} preSelectedGvk={gvk} />
                                    );
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

function makeTabSubtitle(gvk: Gvk) {
    return `${gvk.kind}`;
}

export default ClusterView;
